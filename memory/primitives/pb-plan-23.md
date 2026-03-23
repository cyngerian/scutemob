# Primitive Batch Plan: PB-23 -- Controller-Filtered Creature Triggers

**Generated**: 2026-03-23
**Primitive**: Add controller filtering to creature death/ETB triggers; add new trigger variants for creature-attacks and creature-deals-combat-damage
**CR Rules**: CR 603.2, CR 603.10a (look-back triggers), CR 508.1m (attackers declared), CR 510.3a (combat damage triggers)
**Cards affected**: ~145 (44 existing WheneverCreatureDies + 33 WheneverCreatureEntersBattlefield + ~56 attack triggers + ~12 combat damage triggers)
**Dependencies**: None (first DSL gap closure batch)
**Deferred items from prior PBs**: PB-12 TODO comment at abilities.rs:5766 (wire AnyCreatureDies + trigger doubling for death triggers)

---

## Primitive Specification

The current DSL has two structural problems:

1. **`WheneverCreatureDies`** fires on ALL creature deaths globally. Cards like "Whenever a creature you control dies" (Grave Pact, Pitiless Plunderer) and "Whenever a creature an opponent controls dies" (Massacre Wurm) need controller filtering. Additionally, `WheneverCreatureDies` is **not wired at all** in `enrich_spec_from_def` -- it exists as a `TriggerCondition` variant but never gets converted to a runtime `TriggerEvent`, so these triggers silently never fire.

2. **No trigger variants exist** for "Whenever a creature you control attacks" or "Whenever a creature you control deals combat damage to a player." These are extremely common trigger patterns in Commander staples.

### What this batch adds:

**A. Controller field on `WheneverCreatureDies`:**
- Add `controller: Option<TargetController>` field (None = any creature, You = your creatures, Opponent = opponent's creatures)
- Add a new `TriggerEvent::AnyCreatureDies` variant
- Wire `WheneverCreatureDies` in `enrich_spec_from_def` (currently missing!)
- Wire `AnyCreatureDies` dispatch in `check_triggers` for `CreatureDied` events

**B. Controller field on `WheneverCreatureEntersBattlefield`:**
- Already has `filter: Option<TargetFilter>` which includes `controller: TargetController` -- this is already wired via `ETBTriggerFilter.controller_you` in `enrich_spec_from_def`. No engine change needed, just verify card defs use the filter correctly.

**C. New `TriggerCondition::WheneverCreatureYouControlAttacks`:**
- New CardDef trigger condition
- New `TriggerEvent::AnyCreatureYouControlAttacks`
- Wire in `enrich_spec_from_def`
- Wire in `check_triggers` under `AttackersDeclared`

**D. New `TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer`:**
- New CardDef trigger condition
- New `TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer`
- Wire in `enrich_spec_from_def`
- Wire in `check_triggers` under `CombatDamageDealt`

---

## CR Rule Text

**CR 603.2**: "Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers."

**CR 603.2c**: "An ability triggers only once each time its trigger event occurs. However, it can trigger repeatedly if one event contains multiple occurrences."

**CR 603.10a**: "Some zone-change triggers look back in time. These are leaves-the-battlefield abilities [...]" -- Applies to death triggers. The creature's characteristics are checked from the graveyard object (preserved by `move_object_to_zone`).

**CR 508.1m**: "Any abilities that trigger on attackers being declared trigger." -- Attack triggers fire after the declare attackers step.

**CR 510.3a**: Combat damage triggers fire after combat damage is dealt. The creature must still be on the battlefield (NOT a look-back trigger).

---

## Engine Changes

### Change 1: Add `controller` field to `TriggerCondition::WheneverCreatureDies`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Change variant from:
```rust
WheneverCreatureDies,
```
to:
```rust
WheneverCreatureDies { controller: Option<TargetController> },
```
**Line**: ~1768
**CR**: 603.2 -- controller filtering on death triggers

### Change 2: Add new TriggerCondition variants

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add after `WhenSelfBecomesTapped` (line ~1864):
```rust
/// "Whenever a creature you control attacks" -- fires once per attacking creature.
/// CR 508.1m / CR 603.2
WheneverCreatureYouControlAttacks,
/// "Whenever a creature you control deals combat damage to a player."
/// CR 510.3a / CR 603.2
WheneverCreatureYouControlDealsCombatDamageToPlayer,
```

### Change 3: Add new TriggerEvent variants

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add before the step/phase-based section (line ~376):
```rust
/// CR 603.10a: Fires on ALL battlefield permanents when ANY creature dies.
/// "Whenever a creature [you control / an opponent controls] dies" -- controller
/// filter applied at trigger-collection time via ETBTriggerFilter-like mechanism.
AnyCreatureDies,
/// CR 508.1m: Fires on ALL battlefield permanents when ANY creature attacks.
/// "Whenever a creature you control attacks" -- controller filter applied at
/// trigger-collection time (source.controller == attacker.controller).
AnyCreatureYouControlAttacks,
/// CR 510.3a: Fires on ALL battlefield permanents when ANY creature deals
/// combat damage to a player. Controller filter at trigger-collection time.
AnyCreatureYouControlDealsCombatDamageToPlayer,
```

### Change 4: Add `DeathTriggerFilter` struct

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add near `ETBTriggerFilter` (line ~428):
```rust
/// Filter applied to death triggers to restrict which dying creatures cause
/// the trigger to fire. Analogous to ETBTriggerFilter for ETB triggers.
/// CR 603.2 / CR 603.10a
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeathTriggerFilter {
    /// If true, the dying creature must have been controlled by the trigger source's controller.
    /// "Whenever a creature you control dies"
    pub controller_you: bool,
    /// If true, the dying creature must have been controlled by an opponent of the trigger source's controller.
    /// "Whenever a creature an opponent controls dies"
    pub controller_opponent: bool,
    /// If true, the dying creature must NOT be the trigger source itself ("another creature").
    pub exclude_self: bool,
    /// If true, the dying creature must be a nontoken.
    pub nontoken_only: bool,
}
```

### Change 5: Add `death_filter` field to `TriggeredAbilityDef`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add after `etb_filter` field (line ~453):
```rust
/// Optional death filter for "whenever [another] [creature] [you control] dies"
/// triggers. When present, the trigger only fires if the dying creature
/// matches all specified criteria. CR 603.2 / CR 603.10a
#[serde(default)]
pub death_filter: Option<DeathTriggerFilter>,
```

### Change 6: Wire `WheneverCreatureDies` in `enrich_spec_from_def`

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add a new block after the `WheneverCreatureEntersBattlefield` wiring (line ~2250). Pattern follows the ETB wiring exactly:
```rust
// CR 603.10a: Convert "Whenever [another] creature [you control] dies" triggers
// into runtime TriggeredAbilityDef entries so check_triggers can dispatch them
// via AnyCreatureDies events.
for ability in &def.abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WheneverCreatureDies { controller },
        effect,
        ..
    } = ability
    {
        let death_filter = DeathTriggerFilter {
            controller_you: matches!(controller, Some(TargetController::You)),
            controller_opponent: matches!(controller, Some(TargetController::Opponent)),
            exclude_self: false, // "Whenever a creature dies" includes self; most cards add this
            nontoken_only: false,
        };
        spec = spec.with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::AnyCreatureDies,
            intervening_if: None,
            description: "Whenever a creature dies (CR 603.10a)".to_string(),
            effect: Some(effect.clone()),
            etb_filter: None,
            death_filter: Some(death_filter),
            targets: vec![],
        });
    }
}
```

Similarly wire `WheneverCreatureYouControlAttacks` and `WheneverCreatureYouControlDealsCombatDamageToPlayer`.

### Change 7: Wire `AnyCreatureDies` dispatch in `check_triggers`

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `GameEvent::CreatureDied` handler (line ~3543), add AFTER the existing SelfDies handling:
```rust
// AnyCreatureDies: fires on ALL battlefield permanents when any creature dies.
// CR 603.10a: death triggers look back in time, but the OBSERVING permanents
// (e.g., Blood Artist, Zulaport Cutthroat) must be on the battlefield NOW.
// The DYING creature's characteristics are available via new_grave_id.
collect_triggers_for_event(
    state,
    &mut triggers,
    TriggerEvent::AnyCreatureDies,
    None,             // Check all battlefield permanents
    Some(*new_grave_id), // Pass the dead creature (now in graveyard) for filter checks
);
// Apply death_filter to the newly collected triggers.
// The death_filter checks controller_you against the dying creature's pre-death controller.
// ... (filter logic using death_controller, entering_object_id for controller comparison)
```

**Key detail**: The `collect_triggers_for_event` function currently only supports `etb_filter` for the `entering_object` parameter. We need to extend it (or add post-filtering) to also check `death_filter` against the dying creature. The simplest approach: reuse the `entering_object_id` field on `PendingTrigger` to carry the dead creature's graveyard ObjectId, and add death_filter checking in `collect_triggers_for_event` analogous to etb_filter.

Specifically, in `collect_triggers_for_event` (line ~4508), add a parallel check:
```rust
if let Some(ref death_filter) = trigger_def.death_filter {
    if let Some(dying_id) = entering_object {
        if let Some(dying_obj) = state.objects.get(&dying_id) {
            // controller_you: dying creature must share controller with trigger source
            if death_filter.controller_you && dying_obj.controller != obj.controller {
                continue;
            }
            // controller_opponent: dying creature must be controlled by opponent
            if death_filter.controller_opponent && dying_obj.controller == obj.controller {
                continue;
            }
            // exclude_self: dying creature must not be the trigger source
            if death_filter.exclude_self && dying_id == obj_id {
                continue;
            }
            // nontoken_only: dying creature must not be a token
            if death_filter.nontoken_only && dying_obj.is_token {
                continue;
            }
        } else {
            continue;
        }
    } else {
        continue;
    }
}
```

**Important**: For death triggers, the dying creature's controller should be `death_controller` from the event (pre-death, captures steal effects correctly), not the graveyard object's controller (which was reset to owner by `move_object_to_zone`). We need to pass `death_controller` through. The cleanest approach: add a `dying_creature_controller: Option<PlayerId>` field to `PendingTrigger`, or use the existing `triggering_player` field.

### Change 8: Wire `AnyCreatureYouControlAttacks` in `check_triggers`

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `GameEvent::AttackersDeclared` handler (line ~3041), after the per-attacker SelfAttacks loop, add:
```rust
// AnyCreatureYouControlAttacks: fires on ALL battlefield permanents
// for each creature that attacks, filtered by controller match.
for (attacker_id, attack_target) in attackers {
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::AnyCreatureYouControlAttacks,
        None,               // Check all battlefield permanents
        Some(*attacker_id), // The attacking creature
    );
    // Filter: only keep triggers where the attacking creature's controller
    // matches the trigger source's controller. Done inside collect_triggers_for_event
    // by checking entering_object controller == source controller.
}
```

The controller filtering can use `etb_filter` with `controller_you: true` (since the entering_object_id parameter is reused for the attacking creature). OR add a dedicated filter in `collect_triggers_for_event` that checks the "entering_object" controller against the trigger source controller when the event is `AnyCreatureYouControlAttacks`.

### Change 9: Wire `AnyCreatureYouControlDealsCombatDamageToPlayer` in `check_triggers`

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `GameEvent::CombatDamageDealt` handler (line ~3902), after each player-damage assignment:
```rust
// AnyCreatureYouControlDealsCombatDamageToPlayer: fires on ALL battlefield
// permanents when any creature controlled by their controller deals combat
// damage to a player. CR 510.3a
if matches!(assignment.target, CombatDamageTarget::Player(_)) && assignment.amount > 0 {
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer,
        None,
        Some(assignment.source), // The creature that dealt damage
    );
}
```

### Change 10: Exhaustive match updates

Files requiring new match arms for changed/new variants:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | `TriggerCondition::` match | L3841 | Change `WheneverCreatureDies` arm to destructure `{ controller }` (disc 7, hash controller); add new variants disc 28-29 |
| `crates/engine/src/state/hash.rs` | `TriggerEvent::` match | L1702 | Add `AnyCreatureDies` disc 28, `AnyCreatureYouControlAttacks` disc 29, `AnyCreatureYouControlDealsCombatDamageToPlayer` disc 30 |
| `crates/engine/src/state/hash.rs` | `DeathTriggerFilter` | N/A | Add new `HashInto for DeathTriggerFilter` impl (after `ETBTriggerFilter` at L1807) |
| `crates/engine/src/state/hash.rs` | `TriggeredAbilityDef` | N/A | Add `death_filter` field to the existing hash impl |
| `crates/engine/src/state/mod.rs` | pub use exports | L34 | Add `DeathTriggerFilter` to the `game_object::` export list |
| `crates/engine/src/lib.rs` | pub use exports | L36 | Add `DeathTriggerFilter` to the re-export list |
| `crates/engine/src/cards/helpers.rs` | pub use exports | L19 | Add `DeathTriggerFilter` to the `game_object::` export list |
| `crates/engine/src/testing/replay_harness.rs` | use imports | L10 | Add `DeathTriggerFilter` to imports |
| `crates/engine/src/rules/replacement.rs` | `queue_carddef_etb_triggers` match | L1148 | No change needed (only matches `WhenEntersBattlefield` and `TributeNotPaid`) |
| `crates/engine/src/rules/resolution.rs` | Any `TriggerCondition` matches | N/A | Check if any exist; may need updating |

**Note on TriggerCondition exhaustive match**: `TriggerCondition` is NOT matched exhaustively in engine code -- it's only used in pattern matching (if-let) in card defs and `enrich_spec_from_def`. The hash.rs impl IS exhaustive. The `queue_carddef_etb_triggers` function only matches `WhenEntersBattlefield` and `TributeNotPaid` via if-let chains, not exhaustive match. So the main exhaustive match site is **hash.rs only**.

---

## Card Definition Fixes

### Category A: WheneverCreatureDies cards (44 files)

These cards already use `TriggerCondition::WheneverCreatureDies` but:
1. The trigger NEVER FIRES (not wired in `enrich_spec_from_def`)
2. Need to add `{ controller: Some(TargetController::You) }` or `{ controller: None }` as appropriate

**Cards needing `controller: None` (any creature dies):**
- `blood_artist.rs` -- "whenever Blood Artist or another creature dies"
- `zulaport_cutthroat.rs` -- "whenever Zulaport Cutthroat or another creature dies"
- `poison_tip_archer.rs` -- "whenever another creature dies"
- `cruel_celebrant.rs` -- "whenever another creature or planeswalker you control dies" (approximate as creature-only for now)
- `falkenrath_noble.rs` -- "whenever ~ or another creature dies"
- `fecundity.rs` -- "whenever a creature dies"
- `blade_of_the_bloodchief.rs` -- "whenever a creature dies"

**Cards needing `controller: Some(TargetController::You)` (creature you control dies):**
- `grave_pact.rs` -- "whenever a creature you control dies"
- `pitiless_plunderer.rs` -- "whenever another creature you control dies"
- `grim_haruspex.rs` -- "whenever another nontoken creature you control dies"
- `dark_prophecy.rs` -- "whenever a creature you control dies"
- `moldervine_reclamation.rs` -- "whenever a creature you control dies"
- `midnight_reaper.rs` -- "whenever a nontoken creature you control dies"
- `pawn_of_ulamog.rs` -- "whenever another nontoken creature you control dies"
- `sifter_of_skulls.rs` -- "whenever another nontoken creature you control dies"
- `bastion_of_remembrance.rs` -- "whenever a creature you control dies"
- `elenda_the_dusk_rose.rs` -- "whenever another creature dies" (any) + "when Elenda dies" (self)
- `morbid_opportunist.rs` -- "whenever one or more other creatures die" (any, once per turn)
- `skemfar_avenger.rs` -- "whenever another Elf or Berserker you control dies"
- `marionette_apprentice.rs` -- "whenever another creature you control dies"
- `crossway_troublemakers.rs` -- various death triggers
- `elas_il_kor_sadistic_pilgrim.rs` -- "whenever another creature you control dies"
- `vindictive_vampire.rs` -- "whenever another creature you control dies"
- `the_meathook_massacre.rs` -- two triggers (you control dies / opponent's creature dies)
- `spiteful_banditry.rs` -- "whenever a creature you control dies"
- `nadiers_nightblade.rs` -- "whenever another creature you control leaves the battlefield"
- `prowess_of_the_fair.rs` -- "whenever another nontoken Elf creature you control dies" (approximate)
- `cordial_vampire.rs` -- "whenever another creature dies" (any)
- `yahenni_undying_partisan.rs` -- "whenever another creature you control dies" (approximate -- actually opponent's creature)
- `boggart_shenanigans.rs` -- "whenever another Goblin you control dies"
- `teysa_orzhov_scion.rs` -- "whenever another black creature you control dies"
- `patron_of_the_vein.rs` -- "whenever a creature an opponent controls dies"
- `skullclamp.rs` -- "whenever equipped creature dies"
- `liliana_dreadhorde_general.rs` -- "whenever a creature you control dies"
- `miara_thorn_of_the_glade.rs` -- "whenever another Elf you control dies"
- `agent_venom.rs` -- death trigger
- `black_market.rs` -- "whenever a creature dies"
- `dreadhound.rs` -- "whenever a creature dies"
- `luminous_broodmoth.rs` -- "whenever a creature you control without flying dies"
- `vein_ripper.rs` -- "whenever an opponent sacrifices a nontoken permanent" (not quite creature-dies)
- `vengeful_bloodwitch.rs` -- death trigger
- `massacre_wurm.rs` -- "whenever a creature an opponent controls dies"
- `elderfang_venom.rs` -- death trigger
- `syr_konrad_the_grim.rs` -- multiple triggers including "whenever a creature dies"

**Full list**: All 44 files from grep results. Each needs:
1. Change `WheneverCreatureDies` to `WheneverCreatureDies { controller: <correct value> }`
2. Verify the effect is correct for the oracle text

### Category B: WheneverCreatureEntersBattlefield cards (33 files)

These already work via `enrich_spec_from_def` -> `ETBTriggerFilter`. **Most are already functional.** Check each to verify the filter has `controller: TargetController::You` where oracle says "you control."

Cards to verify (spot-check a few; most should be correct already since the ETB wiring exists):
- `impact_tremors.rs` -- "whenever a creature you control enters" (should have filter with controller: You)
- `prosperous_innkeeper.rs` -- "whenever another creature you control enters"
- `purphoros_god_of_the_forge.rs` -- "whenever another creature you control enters"
- `terror_of_the_peaks.rs` -- "whenever another creature you control enters"
- etc.

### Category C: Attack trigger cards (~56 files) -- need `WheneverCreatureYouControlAttacks`

These are cards with TODO mentioning "creature you control attacks" or similar:
- `shared_animosity.rs` -- "whenever a creature you control attacks"
- `hellrider.rs` -- "whenever a creature you control attacks"
- `beastmaster_ascension.rs` -- "whenever a creature you control attacks"
- `druids_repository.rs` -- "whenever a creature you control attacks"
- `mardu_ascendancy.rs` -- "whenever a nontoken creature you control attacks"
- `marisi_breaker_of_the_coil.rs` -- "whenever a creature you control deals combat damage to a player" (category D)
- `kolaghan_the_storms_fury.rs` -- "whenever a Dragon you control attacks" (subtype filter needed)
- `utvara_hellkite.rs` -- "whenever a Dragon you control attacks" (subtype filter needed)
- `grand_warlord_radha.rs` -- "whenever one or more creatures you control attack"

Many of the 56 attack-trigger TODOs will need `WheneverCreatureYouControlAttacks` + an effect that may itself be a DSL gap (e.g., dynamic buff amounts). Those with simple effects (create token, add counter, gain life) can be fully implemented. Those with complex effects keep a reduced TODO.

### Category D: Combat damage trigger cards (~12 files)

Cards needing `WheneverCreatureYouControlDealsCombatDamageToPlayer`:
- `coastal_piracy.rs` / `reconnaissance_mission.rs` / `bident_of_thassa.rs` -- "whenever a creature you control deals combat damage to a player, draw a card"
- `toski_bearer_of_secrets.rs` -- "whenever a creature you control deals combat damage to a player, draw a card"
- `ohran_frostfang.rs` -- "whenever a creature you control deals combat damage to a player, draw a card"
- `curiosity_crafter.rs` -- "whenever a creature token you control deals combat damage to a player, draw a card"
- `marisi_breaker_of_the_coil.rs` -- "whenever a creature you control deals combat damage to a player, goad each creature that player controls"
- `old_gnawbone.rs` -- "whenever a creature you control deals combat damage to a player, create treasure tokens"
- `enduring_curiosity.rs` -- similar
- `ingenious_infiltrator.rs` -- "whenever a Ninja you control deals combat damage to a player"
- `kindred_discovery.rs` -- "whenever a creature you chose attacks or deals combat damage to a player"

---

## New Card Definitions

No new card definitions needed -- all affected cards already exist as authored stubs with TODOs.

---

## Unit Tests

**File**: `crates/engine/tests/creature_triggers.rs` (new file)

**Tests to write**:

1. `test_whenever_creature_you_control_dies_fires_on_your_creature` -- CR 603.10a
   - Setup: Player A has Zulaport Cutthroat + another creature. Kill the creature. Verify trigger fires.
2. `test_whenever_creature_you_control_dies_ignores_opponent_creature` -- CR 603.10a
   - Setup: Player A has card with "your creature dies" trigger. Kill Player B's creature. Verify no trigger.
3. `test_whenever_any_creature_dies_fires_on_any` -- CR 603.10a
   - Setup: Player A has Blood Artist (any creature dies). Kill Player B's creature. Verify trigger fires.
4. `test_whenever_creature_opponent_controls_dies` -- CR 603.10a
   - Setup: Player A has Massacre Wurm ("opponent's creature dies"). Kill Player B's creature. Verify trigger fires. Kill Player A's creature. Verify no trigger.
5. `test_death_trigger_controller_uses_pre_death_controller` -- CR 603.10a + CR 400.7
   - Verify stolen creature dying triggers "your creature dies" for the stealing player, not the owner.
6. `test_whenever_creature_you_control_attacks_fires` -- CR 508.1m
   - Setup: Player has "whenever a creature you control attacks" enchantment + attacking creature. Verify trigger fires.
7. `test_whenever_creature_you_control_attacks_ignores_opponent` -- CR 508.1m
   - Verify the trigger does NOT fire when an opponent's creature attacks.
8. `test_whenever_creature_you_control_attacks_fires_per_creature` -- CR 603.2c
   - Setup: Attack with 3 creatures. Verify trigger fires 3 times.
9. `test_whenever_creature_you_control_deals_combat_damage_to_player` -- CR 510.3a
   - Setup: Coastal Piracy + attacking creature that deals damage to player. Verify draw trigger.
10. `test_combat_damage_trigger_ignores_creature_damage` -- CR 510.3a
    - Verify the trigger does NOT fire when combat damage is dealt to a creature (blocker).
11. `test_death_trigger_multiplayer_apnap` -- CR 603.3b
    - Multiple players' death triggers should be ordered APNAP.

**Pattern**: Follow tests in `crates/engine/tests/trigger_doubling.rs` and `crates/engine/tests/alliance.rs` (if it exists) for trigger setup patterns.

---

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved
- [ ] `WheneverCreatureDies { controller }` added to `TriggerCondition`
- [ ] `WheneverCreatureYouControlAttacks` added to `TriggerCondition`
- [ ] `WheneverCreatureYouControlDealsCombatDamageToPlayer` added to `TriggerCondition`
- [ ] `AnyCreatureDies` added to `TriggerEvent`
- [ ] `AnyCreatureYouControlAttacks` added to `TriggerEvent`
- [ ] `AnyCreatureYouControlDealsCombatDamageToPlayer` added to `TriggerEvent`
- [ ] `DeathTriggerFilter` struct created
- [ ] `death_filter` field on `TriggeredAbilityDef`
- [ ] `enrich_spec_from_def` wires all three new trigger types
- [ ] `check_triggers` dispatches all three new `TriggerEvent` variants
- [ ] Hash impl updated (hash.rs) for all new types/variants
- [ ] State exports updated (mod.rs, lib.rs, helpers.rs)
- [ ] All 44 WheneverCreatureDies card defs updated with `{ controller: ... }`
- [ ] Attack-trigger card defs updated where effect is expressible
- [ ] Combat-damage card defs updated where effect is expressible
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)

---

## Implementation Order (for runner)

### Session 1: Engine infrastructure
1. Add `DeathTriggerFilter` struct to `game_object.rs`
2. Add `death_filter` field to `TriggeredAbilityDef`
3. Change `WheneverCreatureDies` to `WheneverCreatureDies { controller: Option<TargetController> }`
4. Add `WheneverCreatureYouControlAttacks` and `WheneverCreatureYouControlDealsCombatDamageToPlayer` to `TriggerCondition`
5. Add `AnyCreatureDies`, `AnyCreatureYouControlAttacks`, `AnyCreatureYouControlDealsCombatDamageToPlayer` to `TriggerEvent`
6. Update hash.rs (all new types, changed variant, new TriggerEvent discriminants)
7. Update exports (mod.rs, lib.rs, helpers.rs, replay_harness.rs imports)
8. Wire `enrich_spec_from_def` for all three triggers
9. Wire `check_triggers` `CreatureDied` -> `AnyCreatureDies` dispatch
10. Wire `check_triggers` `AttackersDeclared` -> `AnyCreatureYouControlAttacks` dispatch
11. Wire `check_triggers` `CombatDamageDealt` -> `AnyCreatureYouControlDealsCombatDamageToPlayer` dispatch
12. Extend `collect_triggers_for_event` to check `death_filter`
13. `cargo check`, `cargo build --workspace`

### Session 2: Card def backfill (death triggers, ~44 cards)
1. Update all 44 `WheneverCreatureDies` card defs to `WheneverCreatureDies { controller: ... }`
2. Verify each card's oracle text via MCP to set correct controller value
3. `cargo build --workspace`

### Session 3: Card def backfill (attack + combat damage triggers, ~30-40 cards)
1. Update attack-trigger card defs to use `WheneverCreatureYouControlAttacks`
2. Update combat-damage card defs to use `WheneverCreatureYouControlDealsCombatDamageToPlayer`
3. Cards with complex effects that remain DSL gaps keep a reduced TODO (e.g., "TODO: dynamic buff amount")
4. `cargo build --workspace`

### Session 4: Tests + verification
1. Write `creature_triggers.rs` test file
2. Run full test suite
3. Clippy clean
4. Final workspace build

---

## Risks & Edge Cases

- **Pre-death controller for death triggers**: When a creature is stolen (Mind Control) and then dies, "whenever a creature you control dies" should trigger for the stealing player. The `CreatureDied` event carries `controller` (pre-death), but `move_object_to_zone` resets controller to owner. The `collect_triggers_for_event` call must use the event's `death_controller`, not the graveyard object's controller. **This requires passing death_controller into the filter check.**

- **Death trigger + entering_object_id parameter overloading**: The `collect_triggers_for_event` function uses `entering_object` for ETB filters. For death triggers, we reuse this parameter to pass the dying creature's graveyard ObjectId. This is semantically different but mechanically identical (we just need the dying object for filter checks). Alternative: add a dedicated `dying_object` parameter, but that changes the function signature for all callers.

- **"Whenever a creature you control attacks" fires per-creature**: Unlike "whenever you attack" (which fires once per combat), this fires once PER attacking creature. The implementation must loop over each attacker in `AttackersDeclared.attackers` and call `collect_triggers_for_event` for each, similar to how `SelfAttacks` is dispatched.

- **Nontoken filter**: Some cards say "whenever another nontoken creature you control dies" (Grim Haruspex, Midnight Reaper). The `DeathTriggerFilter.nontoken_only` field handles this. The `is_token` field on `GameObject` must be checked in the filter logic.

- **Trigger doubling**: PB-12 deferred a TODO about doubling death triggers (Panharmonicon-like effects for creature dies). This batch wires the base triggers; doubling support is separate and remains deferred.

- **Card defs with complex effects**: Many attack-trigger cards have effects that are themselves DSL gaps (e.g., Shared Animosity's "count sharing creature types" buff). These cards will keep reduced TODOs after PB-23 -- the trigger condition is now expressible but the effect remains a gap. This is expected and acceptable.

- **WheneverCreatureEntersBattlefield already works**: This trigger IS wired in `enrich_spec_from_def` via `ETBTriggerFilter`. No engine changes needed for ETB triggers. The 33 card defs using it should already work. Spot-check a few to verify.
