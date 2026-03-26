# Primitive Batch Plan: PB-30 -- Combat Damage Triggers

**Generated**: 2026-03-25
**Primitive**: Combat damage trigger infrastructure extensions -- subtype/token filters, "one or more" batch trigger, equipped/enchanted creature triggers, damaged player reference, damage amount reference
**CR Rules**: CR 510.3a, CR 603.2, CR 603.2c, CR 603.2g, CR 603.10
**Cards affected**: 42 (38 existing fixes + 4 already-working cards with approximation TODOs)
**Dependencies**: PB-23 (controller-filtered creature triggers -- DONE)
**Deferred items from prior PBs**: None directly; PB-37 deferred items (Heartstone, Training Grounds, etc.) are unrelated

## Primitive Specification

PB-30 extends the combat damage trigger infrastructure that PB-23 introduced. Two trigger
conditions already exist:
- `WhenDealsCombatDamageToPlayer` (self-referential: "whenever ~ deals combat damage to a player")
- `WheneverCreatureYouControlDealsCombatDamageToPlayer` (any creature you control)

The existing triggers lack:
1. **Subtype filter** on `WheneverCreatureYouControlDealsCombatDamageToPlayer` -- needed for "whenever a Ninja/Vampire/Dragon you control deals combat damage"
2. **Token filter** -- needed for "whenever a creature token you control deals combat damage" (Curiosity Crafter)
3. **Keyword filter** -- needed for "whenever a creature you control with toxic deals combat damage" (Necrogen Rotpriest)
4. **"One or more" batch trigger** -- fires once per damaged player per combat damage step, not per creature (Professional Face-Breaker, Grim Hireling, Nature's Will, etc.)
5. **Equipped/enchanted creature combat damage trigger** -- fires when the creature this equipment/aura is attached to deals combat damage (Sword cycle, Umezawa's Jitte, Curiosity, etc.)
6. **Damaged player reference** (`PlayerTarget::DamagedPlayer`) -- "that player" in trigger effects (Sword of Feast and Famine: "that player discards a card")
7. **Combat damage amount reference** (`EffectAmount::CombatDamageDealt`) -- "that much damage" / "that many tokens" (Balefire Dragon, Lathril, Old Gnawbone)
8. **Any creature deals combat damage to your opponents** (Edric) -- global trigger with opponent filter

This batch adds these capabilities to the TriggerCondition enum, the TriggerEvent enum,
the event dispatch in abilities.rs, the enrichment in replay_harness.rs, and the data
propagation through PendingTrigger/EffectContext.

## CR Rule Text

**CR 510.3a**: "Any abilities that triggered on damage being dealt or while state-based
actions are performed afterward are put onto the stack before the active player gets
priority; the order in which they triggered doesn't matter."

**CR 603.2**: "Whenever a game event or game state matches a triggered ability's trigger
event, that ability automatically triggers."

**CR 603.2c**: "An ability triggers only once each time its trigger event occurs. However,
it can trigger repeatedly if one event contains multiple occurrences."

**CR 603.2g**: "An ability triggers only if its trigger event actually occurs. An event
that's prevented or replaced won't trigger anything."

**CR 603.10**: "Normally, objects that exist immediately after an event are checked to see
if the event matched any trigger conditions." (Combat damage triggers are NOT look-back
triggers -- the creature must be on the battlefield after damage is dealt.)

## Engine Changes

### Change 1: Add `filter` field to `WheneverCreatureYouControlDealsCombatDamageToPlayer`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add an optional `TargetFilter` field to the existing variant:
```rust
WheneverCreatureYouControlDealsCombatDamageToPlayer {
    /// Optional filter on the damage-dealing creature (subtype, token, keyword, etc.).
    /// None = any creature you control. Some(filter) = creature must match filter.
    #[serde(default)]
    filter: Option<TargetFilter>,
},
```
**Pattern**: Follow `WheneverCreatureEntersBattlefield { filter: Option<TargetFilter> }` at line ~1848
**Impact**: This is a breaking change to the variant shape. All existing usages (Ohran Frostfang, Enduring Curiosity, Toski, Bident of Thassa, Coastal Piracy, Reconnaissance Mission) must be updated to `{ filter: None }`.

### Change 2: Add `is_token` field to `TargetFilter`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add a boolean field to `TargetFilter` struct:
```rust
/// Must be a token. Default: false (no restriction).
#[serde(default)]
pub is_token: bool,
```
**Line**: After `legendary: bool` field at ~line 1786
**Pattern**: Follow the `legendary: bool` field pattern
**Needed by**: Curiosity Crafter ("creature token you control")

### Change 3: Add `has_keyword` field to `TargetFilter`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add an optional keyword field:
```rust
/// Must have this keyword ability. None = no restriction.
/// Used for "creature with toxic" (Necrogen Rotpriest).
#[serde(default)]
pub has_keyword: Option<KeywordAbility>,
```
**Line**: After `has_keywords` field at ~line 1746
**Note**: `has_keywords` is `OrdSet<KeywordAbility>` (must have ALL). This new field is for a single required keyword. Alternatively, could reuse `has_keywords` if callers set it to a single-element set. Decision: reuse `has_keywords` -- Necrogen Rotpriest can set `has_keywords: OrdSet::unit(KeywordAbility::Toxic(0))`. However, Toxic has a parameter, so matching needs to be "has any Toxic variant". This is complex. Instead, add a `has_keyword_name` or just use the `has_keywords` existing field. **Decision**: Skip this for PB-30 -- Necrogen Rotpriest needs "creature with toxic" which requires matching any `Toxic(N)` variant. Defer this to PB-37 as a keyword-family filter. The Necrogen Rotpriest TODO will remain.

### Change 4: Add `WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer` TriggerCondition

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant after `WheneverCreatureYouControlDealsCombatDamageToPlayer`:
```rust
/// "Whenever one or more creatures you control deal combat damage to a player."
///
/// CR 510.3a / CR 603.2c: Unlike the per-creature variant, this fires ONCE per
/// damaged player per combat damage step, regardless of how many creatures dealt
/// damage to that player. The batch grouping is done in the event dispatch.
///
/// Optional filter restricts which creatures count (e.g., "non-Human creatures").
WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer {
    #[serde(default)]
    filter: Option<TargetFilter>,
},
```
**Hash discriminant**: 36 (next after `WhenYouCastThisSpell` = 35)

### Change 5: Add `WhenEquippedCreatureDealsCombatDamageToPlayer` TriggerCondition

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant:
```rust
/// "Whenever equipped creature deals combat damage to a player."
///
/// CR 510.3a: Fires when the creature this Equipment is attached to deals > 0
/// combat damage to a player. The trigger source is the Equipment, not the creature.
/// The Equipment must be on the battlefield and attached to the dealing creature.
WhenEquippedCreatureDealsCombatDamageToPlayer,
```
**Hash discriminant**: 37

### Change 6: Add `WhenEnchantedCreatureDealsDamageToPlayer` TriggerCondition

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant:
```rust
/// "Whenever enchanted creature deals damage to a player" / "...deals combat damage..."
///
/// CR 510.3a: Fires when the creature this Aura is attached to deals > 0 damage
/// to a player. Covers both "deals damage" (any, including noncombat) and
/// "deals combat damage" variants. Use `combat_only: bool` to distinguish.
WhenEnchantedCreatureDealsDamageToPlayer {
    /// If true, only combat damage triggers this. If false, any damage.
    #[serde(default)]
    combat_only: bool,
},
```
**Hash discriminant**: 38
**Needed by**: Curiosity, Ophidian Eye ("deals damage" -- any), Sigil of Sleep ("deals damage"), Breath of Fury ("deals combat damage")

### Change 7: Add `WhenAnyCreatureDealsCombatDamageToOpponent` TriggerCondition

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant:
```rust
/// "Whenever a creature deals combat damage to one of your opponents."
///
/// CR 510.3a / CR 603.2: Fires on ALL battlefield permanents when ANY creature
/// (not just yours) deals combat damage to an opponent of the trigger source's
/// controller. Used by Edric, Spymaster of Trest.
WhenAnyCreatureDealsCombatDamageToOpponent,
```
**Hash discriminant**: 39

### Change 8: Add `DamagedPlayer` variant to `PlayerTarget`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to `PlayerTarget` enum:
```rust
/// The player who was dealt combat damage in the triggering event.
/// Resolved from PendingTrigger data (damaged_player) at effect execution time.
/// Used by "that player discards a card" (Sword of Feast and Famine), "goad each
/// creature that player controls" (Marisi), etc.
DamagedPlayer,
```
**Line**: After `TriggeringPlayer` variant

### Change 9: Add `CombatDamageDealt` variant to `EffectAmount`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to `EffectAmount` enum:
```rust
/// The amount of combat damage dealt in the triggering event.
/// Resolved from EffectContext.combat_damage_amount at execution time.
/// Used by "deals that much damage" (Balefire Dragon), "create that many tokens"
/// (Lathril, Old Gnawbone).
CombatDamageDealt,
```

### Change 10: Add new TriggerEvent variants

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add 4 new variants to `TriggerEvent` enum:
```rust
/// "Whenever one or more creatures you control deal combat damage to a player."
/// Fires once per damaged player per combat damage step (batch grouping).
AnyCreatureYouControlBatchCombatDamage,

/// "Whenever equipped creature deals combat damage to a player."
/// Fires on Equipment permanents when their attached creature deals combat damage.
EquippedCreatureDealsCombatDamageToPlayer,

/// "Whenever enchanted creature deals damage to a player."
/// Fires on Aura permanents when their attached creature deals damage.
EnchantedCreatureDealsDamageToPlayer,

/// "Whenever a creature deals combat damage to one of your opponents."
/// Fires globally for any creature dealing combat damage to an opponent.
AnyCreatureDealsCombatDamageToOpponent,
```
**Hash discriminants**: 40, 41, 42, 43

### Change 11: Add `combat_damage_amount` and `damaged_player` to EffectContext

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add two fields to `EffectContext` struct:
```rust
/// Amount of combat damage dealt in the triggering event.
/// Set from PendingTrigger data for combat damage triggers.
/// Read by EffectAmount::CombatDamageDealt.
pub combat_damage_amount: u32,

/// The player who was dealt combat damage in the triggering event.
/// Set from PendingTrigger data for combat damage triggers.
/// Read by PlayerTarget::DamagedPlayer.
pub damaged_player: Option<PlayerId>,
```

### Change 12: Add `damaged_player` and `combat_damage_amount` to PendingTrigger

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add two fields to `PendingTrigger` struct:
```rust
/// CR 510.3a: The player dealt combat damage (for combat damage triggers).
/// Used at flush/resolution time to populate EffectContext.damaged_player.
#[serde(default)]
pub damaged_player: Option<PlayerId>,

/// CR 510.3a: The amount of combat damage dealt (for damage-amount-dependent effects).
/// Used at resolution time to populate EffectContext.combat_damage_amount.
#[serde(default)]
pub combat_damage_amount: u32,
```

### Change 13: Wire combat damage triggers in abilities.rs

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `GameEvent::CombatDamageDealt { assignments }` handler (~line 4233):

**13a**: For `SelfDealsCombatDamageToPlayer` triggers, populate `damaged_player` and `combat_damage_amount` on the PendingTrigger. Currently `collect_triggers_for_event` is called which creates PendingTriggers without this data. After `collect_triggers_for_event`, iterate the newly-added triggers and set the fields:
```rust
let pre_len = triggers.len();
collect_triggers_for_event(state, &mut triggers, TriggerEvent::SelfDealsCombatDamageToPlayer, Some(assignment.source), None);
// Populate combat damage data on newly-added triggers
let CombatDamageTarget::Player(damaged_pid) = &assignment.target else { unreachable!() };
for t in &mut triggers[pre_len..] {
    t.damaged_player = Some(*damaged_pid);
    t.combat_damage_amount = assignment.amount;
}
```

**13b**: For `AnyCreatureYouControlDealsCombatDamageToPlayer`, same pattern -- populate `damaged_player` and `combat_damage_amount`.

**13c**: Add batch trigger dispatch: After iterating per-creature assignments, group by damaged player and fire `AnyCreatureYouControlBatchCombatDamage` once per damaged player:
```rust
// "One or more creatures you control deal combat damage to a player" batch trigger
let mut damaged_players_by_controller: HashMap<(PlayerId, PlayerId), u32> = HashMap::new();
for assignment in assignments {
    if assignment.amount == 0 { continue; }
    let CombatDamageTarget::Player(damaged_pid) = &assignment.target else { continue; };
    if let Some(obj) = state.objects.get(&assignment.source) {
        if obj.zone == ZoneId::Battlefield && obj.is_phased_in() {
            *damaged_players_by_controller.entry((obj.controller, *damaged_pid)).or_default() += assignment.amount;
        }
    }
}
for ((controller, damaged_pid), total_amount) in &damaged_players_by_controller {
    let pre_len = triggers.len();
    collect_triggers_for_event(state, &mut triggers, TriggerEvent::AnyCreatureYouControlBatchCombatDamage, None, None);
    // Filter: only triggers from permanents controlled by `controller`
    // and populate damaged_player + amount
    triggers[pre_len..].retain_mut(|t| {
        if t.controller != *controller { return false; }
        t.damaged_player = Some(*damaged_pid);
        t.combat_damage_amount = *total_amount;
        true
    });
}
```
**Note**: The filter on the `TriggeredAbilityDef` for subtype/token/non-Human matching needs to be checked against the DEALING creatures at trigger time, not the source permanent. For "one or more" triggers, the filter check should verify that at least one of the dealing creatures matches. This requires passing the set of dealing creature ObjectIds. **Simplification**: For the batch trigger, store the filter on the TriggerCondition and check it in the dispatch loop against the dealing creatures. This is complex. **Alternative**: Add a `combat_damage_filter` field to `TriggeredAbilityDef` and check it at collection time. **Decision**: Use a simpler approach -- the TargetFilter on the TriggerCondition variant will be checked by the enrichment code (`enrich_spec_from_def`) and stored. At trigger time, iterate the assignments to see if ANY qualifying creature dealt damage to this player. This happens inside the batch loop above.

**13d**: Add equipped/enchanted creature trigger dispatch: For each assignment where damage > 0 and target is a player, find Equipment/Aura permanents attached to the dealing creature and fire triggers:
```rust
// Equipment triggers: "whenever equipped creature deals combat damage to a player"
for assignment in assignments {
    if assignment.amount == 0 { continue; }
    let CombatDamageTarget::Player(damaged_pid) = &assignment.target else { continue; };
    if let Some(creature) = state.objects.get(&assignment.source) {
        if creature.zone != ZoneId::Battlefield { continue; }
        for &attachment_id in &creature.attachments {
            let pre_len = triggers.len();
            collect_triggers_for_event(state, &mut triggers,
                TriggerEvent::EquippedCreatureDealsCombatDamageToPlayer,
                Some(attachment_id), None);
            for t in &mut triggers[pre_len..] {
                t.damaged_player = Some(*damaged_pid);
                t.combat_damage_amount = assignment.amount;
            }
            // Also check for enchanted creature triggers
            let pre_len2 = triggers.len();
            collect_triggers_for_event(state, &mut triggers,
                TriggerEvent::EnchantedCreatureDealsDamageToPlayer,
                Some(attachment_id), None);
            for t in &mut triggers[pre_len2..] {
                t.damaged_player = Some(*damaged_pid);
                t.combat_damage_amount = assignment.amount;
            }
        }
    }
}
```

**13e**: Add "any creature deals combat damage to opponent" (Edric) trigger dispatch:
```rust
// Edric pattern: "whenever a creature deals combat damage to one of your opponents"
for assignment in assignments {
    if assignment.amount == 0 { continue; }
    let CombatDamageTarget::Player(damaged_pid) = &assignment.target else { continue; };
    if state.objects.get(&assignment.source).is_none_or(|o| o.zone != ZoneId::Battlefield) {
        continue;
    }
    collect_triggers_for_event(state, &mut triggers,
        TriggerEvent::AnyCreatureDealsCombatDamageToOpponent,
        None, Some(assignment.source));
}
```
In `collect_triggers_for_event`, add controller filtering: only fire if the damaged player is an opponent of the trigger source's controller.

### Change 14: Update `collect_triggers_for_event` controller filtering

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Extend the existing controller-filtering block (~line 5253) to handle new event types:
- `AnyCreatureYouControlBatchCombatDamage`: controller match between trigger source and controller who declared attackers
- `AnyCreatureDealsCombatDamageToOpponent`: verify damaged player is an opponent of trigger source's controller. The damaged player must be passed via `entering_object` mechanism or a new parameter. **Decision**: Use `entering_object` to carry the dealing creature's ObjectId, then check that the damaged player (from PendingTrigger) is an opponent. Actually, the opponent check happens at a different level -- the Edric trigger fires on ALL permanents, but the effect says "its controller may draw a card" (meaning the creature's controller, not Edric's). For Edric, the trigger fires globally; the filtering is that the damaged player must be an opponent of Edric's controller. This check can happen during `collect_triggers_for_event` by comparing the `damaged_player` against the trigger source's controller.

**Revised approach**: For `AnyCreatureDealsCombatDamageToOpponent`, the calling code in Change 13e will handle the opponent check BEFORE calling `collect_triggers_for_event`, by verifying `damaged_pid != trigger source controller`. Actually, since `collect_triggers_for_event` iterates ALL battlefield objects, the check must happen per-object. Add it to the filtering block:
```rust
TriggerEvent::AnyCreatureDealsCombatDamageToOpponent => {
    // The damaged player (carried in entering_object's controller check)
    // must be an opponent of the trigger source's controller.
    // This is checked after triggers are collected by retaining only
    // triggers where damaged_player != trigger.controller.
}
```
**Simpler**: After `collect_triggers_for_event` returns, filter triggers where `t.damaged_player != Some(t.controller)`.

### Change 15: Update `collect_triggers_for_event` filter matching for subtype/token on combat damage

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `collect_triggers_for_event`, for `AnyCreatureYouControlDealsCombatDamageToPlayer`, add subtype/token filter checking. The `entering_object` parameter carries the dealing creature's ObjectId. After the controller check, also check the TriggeredAbilityDef's filter (if any) against the dealing creature.

This requires adding a `combat_filter` field to `TriggeredAbilityDef`, or reusing `etb_filter`. **Decision**: Add a new `combat_damage_filter: Option<TargetFilter>` field to `TriggeredAbilityDef`:
```rust
/// Optional filter on the creature that dealt combat damage.
/// Used by "whenever a [Ninja/token/non-Human] creature you control deals combat damage".
#[serde(default)]
pub combat_damage_filter: Option<TargetFilter>,
```

### Change 16: Wire TriggerCondition -> TriggerEvent in `enrich_spec_from_def`

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Update the existing `WheneverCreatureYouControlDealsCombatDamageToPlayer` enrichment block (~line 2375) to handle the new `{ filter }` field, and add new blocks for each new TriggerCondition variant:

**16a**: Update existing block for the filter field:
```rust
TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer { filter } => {
    spec = spec.with_triggered_ability(TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer,
        combat_damage_filter: filter.clone(),
        // ...
    });
}
```

**16b**: Add block for `WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter }`:
```rust
TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter } => {
    spec = spec.with_triggered_ability(TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlBatchCombatDamage,
        combat_damage_filter: filter.clone(),
        // ...
    });
}
```

**16c**: Add block for `WhenEquippedCreatureDealsCombatDamageToPlayer`:
```rust
TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer => {
    spec = spec.with_triggered_ability(TriggeredAbilityDef {
        trigger_on: TriggerEvent::EquippedCreatureDealsCombatDamageToPlayer,
        // ...
    });
}
```

**16d**: Add block for `WhenEnchantedCreatureDealsDamageToPlayer { combat_only }`:
```rust
TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer { combat_only } => {
    spec = spec.with_triggered_ability(TriggeredAbilityDef {
        trigger_on: TriggerEvent::EnchantedCreatureDealsDamageToPlayer,
        // combat_only stored in trigger data or checked at trigger time
        // ...
    });
}
```

**16e**: Add block for `WhenAnyCreatureDealsCombatDamageToOpponent`:
```rust
TriggerCondition::WhenAnyCreatureDealsCombatDamageToOpponent => {
    spec = spec.with_triggered_ability(TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureDealsCombatDamageToOpponent,
        // ...
    });
}
```

### Change 17: Wire `PlayerTarget::DamagedPlayer` resolution

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In the `resolve_player_target` function (or wherever PlayerTarget variants are matched), add:
```rust
PlayerTarget::DamagedPlayer => {
    ctx.damaged_player.expect("DamagedPlayer used outside combat damage trigger context")
}
```

### Change 18: Wire `EffectAmount::CombatDamageDealt` resolution

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In the `resolve_amount` function (or wherever EffectAmount variants are matched), add:
```rust
EffectAmount::CombatDamageDealt => ctx.combat_damage_amount as i32,
```

### Change 19: Wire data from PendingTrigger to EffectContext at resolution

**File**: `crates/engine/src/rules/resolution.rs` (or `abilities.rs` flush_pending_triggers)
**Action**: When a `PendingTriggerKind::Normal` trigger is resolved and its `damaged_player` or `combat_damage_amount` are set, propagate them to the EffectContext:
```rust
ctx.damaged_player = trigger.damaged_player;
ctx.combat_damage_amount = trigger.combat_damage_amount;
```
**Pattern**: Follow how `triggering_player` is propagated from PendingTrigger to EffectContext.

### Change 20: Exhaustive match updates

Files requiring new match arms for the new TriggerCondition/TriggerEvent/PlayerTarget/EffectAmount variants:

| File | Match expression | Line (approx) | Action |
|------|-----------------|------|--------|
| `state/hash.rs` | `TriggerCondition::*` | L3941-4037 | Hash new variants (discriminants 36-39) |
| `state/hash.rs` | `TriggerEvent::*` | L1756-1827 | Hash new variants (discriminants 40-43) |
| `state/hash.rs` | `PlayerTarget::*` | (find) | Hash `DamagedPlayer` |
| `state/hash.rs` | `EffectAmount::*` | (find) | Hash `CombatDamageDealt` |
| `state/hash.rs` | `TargetFilter` | (find) | Hash `is_token` field |
| `testing/replay_harness.rs` | `TriggerCondition::*` enrichment | L2066-2394 | Add enrichment blocks for 5 new variants |
| `testing/replay_harness.rs` | `TargetFilter` matching | (find) | Add `is_token` check |
| `effects/mod.rs` | `PlayerTarget::*` | (find) | Add `DamagedPlayer` arm |
| `effects/mod.rs` | `EffectAmount::*` | (find) | Add `CombatDamageDealt` arm |
| `rules/abilities.rs` | `GameEvent::CombatDamageDealt` | L4233 | Wire 4 new trigger dispatch loops |
| `rules/abilities.rs` | `collect_triggers_for_event` | L5253 | Controller/filter check for new events |
| `state/game_object.rs` | `TriggeredAbilityDef` | L508 | Add `combat_damage_filter` field |
| `state/stubs.rs` | `PendingTrigger` | L192 | Add `damaged_player`, `combat_damage_amount` |
| `effects/mod.rs` | `EffectContext` | L48 | Add `combat_damage_amount`, `damaged_player` |
| `cards/card_definition.rs` | `TargetFilter` | L1738 | Add `is_token` |

### Change 21: Wire `is_token` check in TargetFilter matching

**File**: `crates/engine/src/effects/mod.rs` (or wherever `TargetFilter` matching occurs)
**Action**: In the filter matching function, add:
```rust
if filter.is_token && !obj.is_token { return false; }
```
**Pattern**: Follow how `legendary` field is checked.

### Change 22: Update existing card defs using `WheneverCreatureYouControlDealsCombatDamageToPlayer`

**Files**: All 6 existing card defs that use this variant must be updated to `{ filter: None }`:
- `ohran_frostfang.rs`
- `enduring_curiosity.rs`
- `toski_bearer_of_secrets.rs`
- `bident_of_thassa.rs`
- `coastal_piracy.rs` (verify)
- `reconnaissance_mission.rs` (verify)

## Card Definition Fixes

Cards are grouped by which trigger pattern they need. Some cards have ADDITIONAL gaps
beyond the combat damage trigger (marked as "partial fix" -- the trigger TODO is removed
but other TODOs may remain).

### Group A: "Whenever ~ deals combat damage to a player" + damage amount/player reference
These cards already have `WhenDealsCombatDamageToPlayer` available but need `EffectAmount::CombatDamageDealt` and/or `PlayerTarget::DamagedPlayer`.

#### balefire_dragon.rs
**Oracle text**: "Flying. Whenever Balefire Dragon deals combat damage to a player, it deals that much damage to each creature that player controls."
**Current state**: Only Flying implemented; trigger omitted entirely.
**Fix**: Add triggered ability with `WhenDealsCombatDamageToPlayer`, effect = `DealDamage` to each creature `DamagedPlayer` controls, amount = `CombatDamageDealt`. Requires `ForEach` over creatures of damaged player.

#### lathril_blade_of_the_elves.rs
**Oracle text**: "Menace. Whenever Lathril deals combat damage to a player, create that many 1/1 green Elf Warrior creature tokens."
**Current state**: Only Menace implemented.
**Fix**: Add triggered ability with `WhenDealsCombatDamageToPlayer`, effect = `CreateToken` count = `CombatDamageDealt`.

#### dragonlord_ojutai.rs
**Oracle text**: "Flying. Dragonlord Ojutai has hexproof as long as it's untapped. Whenever Dragonlord Ojutai deals combat damage to a player, look at the top three cards..."
**Current state**: Flying + conditional hexproof implemented. Combat damage trigger TODO.
**Fix**: Partial fix -- combat damage trigger expressible with `WhenDealsCombatDamageToPlayer`. The "look at top 3, put 1 in hand" effect is a separate DSL gap (no `LookAtTopCards` effect). **Deferred to PB-37**.

#### hellkite_tyrant.rs
**Oracle text**: "Flying, trample. Whenever Hellkite Tyrant deals combat damage to a player, gain control of all artifacts that player controls."
**Current state**: Only Flying + Trample.
**Fix**: Partial fix -- trigger expressible. "Gain control of all artifacts that player controls" is a separate DSL gap (no `GainControlAll` effect). **Deferred to PB-32/PB-37**.

#### dokuchi_silencer.rs
**Oracle text**: "Ninjutsu {B}{B}. Whenever Dokuchi Silencer deals combat damage to a player, you may discard a creature card. When you do, destroy target creature or planeswalker that player controls."
**Current state**: Ninjutsu implemented. Trigger TODO.
**Fix**: Partial fix -- the trigger fires with `WhenDealsCombatDamageToPlayer`, but the reflexive "when you do" pattern (discard -> destroy) is a separate DSL gap. **Deferred**.

#### biting_palm_ninja.rs
**Oracle text**: "Ninjutsu {2}{B}. Menace counter ETB. Whenever Biting-Palm Ninja deals combat damage to a player, you may remove a menace counter. When you do, that player reveals their hand..."
**Current state**: Ninjutsu + menace counter ETB. Trigger TODO.
**Fix**: Partial fix -- trigger fires but the reflexive "when you do" + remove counter + reveal hand chain is a separate gap. **Deferred**.

#### steel_hellkite.rs
**Oracle text**: "{X}: Destroy each nonland permanent with mana value X whose controller was dealt combat damage by Steel Hellkite this turn."
**Current state**: Flying + pump implemented. Activated ability TODO.
**Fix**: NOT a combat damage trigger -- it's an activated ability that references combat damage tracking. **Exclude from PB-30**.

### Group B: "Whenever a creature you control deals combat damage to a player" (per-creature, now with filter)
These use `WheneverCreatureYouControlDealsCombatDamageToPlayer` but need damage/player data.

#### old_gnawbone.rs
**Oracle text**: "Flying. Whenever a creature you control deals combat damage to a player, create that many Treasure tokens."
**Current state**: Only Flying.
**Fix**: Add triggered ability with `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: None }`, effect = `CreateToken` with `treasure_token_spec()`, count = `CombatDamageDealt`.

#### the_indomitable.rs
**Oracle text**: "Trample. Whenever a creature you control deals combat damage to a player, draw a card. Crew 3."
**Current state**: Trample + Crew.
**Fix**: Add triggered ability with `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: None }`, effect = `DrawCards(1)`.

#### marisi_breaker_of_the_coil.rs
**Oracle text**: "...Whenever a creature you control deals combat damage to a player, goad each creature that player controls."
**Current state**: Empty (both abilities have TODOs).
**Fix**: Partial fix -- add the combat damage trigger with `DamagedPlayer` + Goad each creature. The "opponents can't cast spells during combat" static is a separate gap.

#### saskia_the_unyielding.rs
**Oracle text**: "...As Saskia enters, choose a player. Whenever a creature you control deals combat damage to a player, it deals that much damage to the chosen player."
**Current state**: Only Vigilance + Haste.
**Fix**: Partial fix only -- the "choose a player" ETB replacement + persistent chosen player state is a separate gap. **Deferred to PB-37**.

### Group C: "Whenever a [Subtype] you control deals combat damage to a player" (per-creature with subtype filter)

#### ingenious_infiltrator.rs
**Oracle text**: "Ninjutsu {U}{B}. Whenever a Ninja you control deals combat damage to a player, draw a card."
**Current state**: Using `WhenDealsCombatDamageToPlayer` (self only) as approximation.
**Fix**: Change to `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: Some(TargetFilter { has_subtype: Some("Ninja"), .. }) }`.

#### rakish_heir.rs
**Oracle text**: "Whenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it."
**Current state**: Empty abilities with TODO.
**Fix**: Add triggered ability with subtype filter for Vampire. The "put a +1/+1 counter on it" refers to the dealing creature -- needs `EffectTarget` for the triggering creature. **Note**: This requires knowing WHICH creature triggered. For per-creature triggers, the dealing creature's ObjectId should be available as `entering_object` in the PendingTrigger. Add `EffectTarget::TriggeringCreature` or use existing `EffectTarget::Source` (but source is the Rakish Heir, not the dealing creature). **Decision**: Add `EffectTarget::EnteringObject` or reuse `entering_object_id` on PendingTrigger for the dealing creature. When the trigger resolves, `entering_object_id` is the dealing creature's ObjectId, mapped to `EffectTarget::TriggeringCreature`.

#### stensia_masquerade.rs
**Oracle text**: "Attacking creatures you control have first strike. Whenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it."
**Current state**: First strike grant implemented. Vampire trigger TODO.
**Fix**: Same as rakish_heir -- needs `TriggeringCreature` target for the counter.

#### necrogen_rotpriest.rs
**Oracle text**: "Toxic 2. Whenever a creature you control with toxic deals combat damage to a player, that player gets an additional poison counter."
**Current state**: Only Toxic(2).
**Fix**: Needs keyword filter (any Toxic variant). **Deferred** -- keyword-family matching is complex.

#### alela_cunning_conqueror.rs
**Oracle text**: Complex. "Whenever a Faerie you control deals combat damage to a player, you may create a 1/1 Faerie token..."
**Current state**: Approximated with self-only triggers.
**Fix**: Change second trigger to `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: Faerie }`.

#### prosperous_thief.rs
**Oracle text**: "Ninjutsu {U}. Whenever a Ninja you control deals combat damage to a player, create a Treasure token."
**Current state**: Ninjutsu implemented. Trigger TODO.
**Fix**: Add `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: Some(TargetFilter { has_subtype: Some("Ninja"), .. }) }`, effect = CreateToken(treasure).

### Group D: "Whenever one or more creatures you control deal combat damage to a player" (batch trigger)

#### professional_face_breaker.rs
**Oracle text**: "Menace. Whenever one or more creatures you control deal combat damage to a player, create a Treasure token."
**Current state**: Only Menace.
**Fix**: Add `WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter: None }`, effect = CreateToken(treasure).

#### grim_hireling.rs
**Oracle text**: "Whenever one or more creatures you control deal combat damage to a player, create two Treasure tokens."
**Current state**: Empty.
**Fix**: Partial fix -- add batch trigger with CreateToken(treasure, 2). The activated ability is a separate gap.

#### natures_will.rs
**Oracle text**: "Whenever one or more creatures you control deal combat damage to a player, tap all lands that player controls and untap all lands you control."
**Current state**: Empty.
**Fix**: Add batch trigger with DamagedPlayer. Effect needs "tap all lands target player controls" + "untap all lands you control" -- these are expressible as `ForEach` + `TapPermanent`/`UntapPermanent` with filters. Partial fix if ForEach over opponent lands is not expressible.

#### contaminant_grafter.rs
**Oracle text**: "Trample, toxic 1. Whenever one or more creatures you control deal combat damage to one or more players, proliferate."
**Current state**: Trample + Toxic.
**Fix**: Add batch trigger with Proliferate effect.

#### keeper_of_fables.rs
**Oracle text**: "Whenever one or more non-Human creatures you control deal combat damage to a player, draw a card."
**Current state**: Empty.
**Fix**: Add batch trigger with filter `non_creature: false` + exclude subtype "Human". **Note**: TargetFilter has no `exclude_subtype` field. **Decision**: Add `exclude_subtypes: Vec<SubType>` to TargetFilter, or handle "non-Human" as a negative filter. **Simpler**: For PB-30, this card may need to remain deferred if negative subtype filtering is not available. **Decision**: Defer this card -- negative subtype filter is a separate gap.

### Group E: "Whenever equipped creature deals combat damage to a player" (equipment trigger)

#### sword_of_feast_and_famine.rs
**Oracle text**: "Equipped creature gets +2/+2 and has protection from black and from green. Whenever equipped creature deals combat damage to a player, that player discards a card and you untap all lands you control. Equip {2}"
**Fix**: Add `WhenEquippedCreatureDealsCombatDamageToPlayer` trigger. Effect: `Sequence([DiscardCards(DamagedPlayer, 1), UntapAllLands(Controller)])`. The "untap all lands you control" may need `ForEach` over lands.

#### sword_of_fire_and_ice.rs
**Oracle text**: "...Whenever equipped creature deals combat damage to a player, Sword of Fire and Ice deals 2 damage to any target and you draw a card."
**Fix**: Add trigger. Effect: `Sequence([DealDamage(DeclaredTarget{0}, Fixed(2)), DrawCards(Controller, 1)])`.

#### sword_of_body_and_mind.rs
**Oracle text**: "...Whenever equipped creature deals combat damage to a player, you create a 2/2 green Wolf creature token and that player mills ten cards."
**Fix**: Add trigger. Effect: `Sequence([CreateToken(wolf_2_2), MillCards(DamagedPlayer, 10)])`.

#### sword_of_light_and_shadow.rs
**Oracle text**: "...Whenever equipped creature deals combat damage to a player, you gain 3 life and you may return up to one target creature card from your graveyard to your hand."
**Fix**: Partial fix -- trigger + gain life expressible. MoveZone from graveyard to hand with target may need verification.

#### sword_of_sinew_and_steel.rs
**Oracle text**: "...Whenever equipped creature deals combat damage to a player, destroy target planeswalker and target artifact."
**Fix**: Add trigger with targets and Destroy effects.

#### sword_of_truth_and_justice.rs
**Oracle text**: "...Whenever equipped creature deals combat damage to a player, put a +1/+1 counter on a creature you control, then proliferate."
**Fix**: Add trigger. Effect: Sequence([AddCounter, Proliferate]).

#### sword_of_war_and_peace.rs
**Oracle text**: "...Whenever equipped creature deals combat damage to a player, Sword of War and Peace deals damage to that player equal to the number of cards in your hand and you gain 1 life for each card in their hand."
**Fix**: Partial fix -- needs `EffectAmount::CardCount` for hand sizes + `DamagedPlayer`.

#### umezawas_jitte.rs
**Oracle text**: "Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte."
**Fix**: Note -- this says "deals combat damage" not "to a player" -- includes creature combat damage. This is a different trigger: `WhenEquippedCreatureDealsCombatDamage` (any target). **Decision**: For PB-30, approximate as `WhenEquippedCreatureDealsCombatDamageToPlayer` (slight inaccuracy -- misses combat damage to creatures). Add TODO for the "to any target" variant.

#### mask_of_memory.rs
**Oracle text**: "Whenever equipped creature deals combat damage to a player, you may draw two cards. If you do, discard a card. Equip {1}"
**Fix**: Add trigger. Effect: DrawCards(2) then Discard(1). The "may" is a choice; approximate without.

#### quietus_spike.rs
**Oracle text**: "Equipped creature has deathtouch. Whenever equipped creature deals combat damage to a player, that player loses half their life, rounded up."
**Fix**: Partial fix -- trigger expressible. "Loses half their life rounded up" is a separate gap (no HalfLife effect). Deathtouch grant is a separate static grant gap.

#### the_reaver_cleaver.rs
**Oracle text**: "Equipped creature gets +1/+1 and has trample. Whenever equipped creature deals combat damage to a player or battle, create that many Treasure tokens. Equip {3}"
**Fix**: Add trigger with `CombatDamageDealt` for Treasure count.

### Group F: "Whenever enchanted creature deals damage to a player" (aura trigger)

#### curiosity.rs
**Oracle text**: "Enchant creature. Whenever enchanted creature deals damage to an opponent, you may draw a card."
**Fix**: Add `WhenEnchantedCreatureDealsDamageToPlayer { combat_only: false }` trigger. Effect: DrawCards(1). Note: "deals damage" includes noncombat damage.

#### ophidian_eye.rs
**Oracle text**: "Flash. Enchant creature. Whenever enchanted creature deals damage to an opponent, you may draw a card."
**Fix**: Same pattern as Curiosity.

#### sigil_of_sleep.rs
**Oracle text**: "Enchant creature. Whenever enchanted creature deals damage to a player, return target creature that player controls to its owner's hand."
**Fix**: Add trigger with `DamagedPlayer` target resolution. Needs targeting.

#### breath_of_fury.rs
**Oracle text**: "Enchant creature you control. When enchanted creature deals combat damage to a player, sacrifice it and attach..."
**Fix**: Partial fix -- trigger expressible but the sacrifice+re-attach+extra combat chain has multiple gaps. **Deferred**.

### Group G: "Whenever a creature deals combat damage to one of your opponents" (global-opponent)

#### edric_spymaster_of_trest.rs
**Oracle text**: "Whenever a creature deals combat damage to one of your opponents, its controller may draw a card."
**Fix**: Add `WhenAnyCreatureDealsCombatDamageToOpponent` trigger. Effect: `DrawCards(ControllerOf(TriggeringCreature), 1)`. Needs both `TriggeringCreature` target and "its controller" player target.

#### strixhaven_stadium.rs
**Oracle text**: Complex. Two combat damage triggers.
**Fix**: Partial fix -- the "creature you control deals combat damage to an opponent" trigger is expressible but the point counter tracking + lose-game condition has multiple gaps. **Deferred**.

### Group H: Cards excluded from PB-30 (not combat damage triggers)

These 6 cards appeared in the 48-file grep but their TODOs are NOT about combat damage trigger infrastructure:

- **crystal_barricade.rs**: Damage prevention, not trigger
- **hope_of_ghirapur.rs**: Activated ability referencing combat damage tracking, not trigger
- **niv_mizzet_visionary.rs**: Noncombat damage trigger, not combat
- **kor_haven.rs**: Damage prevention activated ability
- **spike_weaver.rs**: Damage prevention activated ability
- **galadhrim_ambush.rs**: Damage prevention instant
- **kaito_shizuki.rs**: Planeswalker emblem with combat damage trigger -- deferred (PW emblem gap)
- **kaito_dancing_shadow.rs**: Complex PW + combat damage -- deferred
- **ancient_bronze_dragon.rs**: d20 roll + reflexive trigger -- deferred (dice gap)

### Group I: Cards already using triggers correctly with remaining non-trigger TODOs

- **bident_of_thassa.rs**: Combat damage trigger already wired (PB-23). Remaining TODO is "forced attack" activated ability -- not PB-30.
- **toski_bearer_of_secrets.rs**: Trigger wired. Remaining TODO is "must attack" -- not PB-30.

## New EffectTarget Variant Needed

### `EffectTarget::TriggeringCreature`

Several cards reference "that creature" or "it" in combat damage triggers, meaning the creature that dealt the damage (not the trigger source). Examples: Rakish Heir ("put a +1/+1 counter on it"), Edric ("its controller may draw a card").

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add to `EffectTarget` enum:
```rust
/// The creature that triggered this ability (e.g., the creature that dealt combat damage).
/// Resolved from PendingTrigger::entering_object_id at effect execution time.
TriggeringCreature,
```

The `entering_object_id` field on PendingTrigger already carries the relevant creature ObjectId for per-creature triggers (`AnyCreatureYouControlDealsCombatDamageToPlayer` passes `entering_object = Some(assignment.source)`). Wire this to EffectContext and add resolution.

## Summary of Fixable Cards (this batch)

Cards fully fixable by PB-30 engine changes:

| Card | Pattern | Trigger Type |
|------|---------|------|
| old_gnawbone | B | per-creature + damage amount |
| the_indomitable | B | per-creature (draw) |
| professional_face_breaker | D | batch (treasure) |
| contaminant_grafter | D | batch (proliferate) |
| ingenious_infiltrator | C | per-creature subtype (Ninja) |
| prosperous_thief | C | per-creature subtype (Ninja) |
| rakish_heir | C | per-creature subtype (Vampire) + counter on it |
| stensia_masquerade | C | per-creature subtype (Vampire) + counter on it |
| alela_cunning_conqueror | C | per-creature subtype (Faerie) |
| curiosity | F | enchanted creature damage |
| ophidian_eye | F | enchanted creature damage |
| mask_of_memory | E | equipped creature damage |
| sword_of_fire_and_ice | E | equipped creature damage |
| sword_of_body_and_mind | E | equipped creature damage |
| sword_of_sinew_and_steel | E | equipped creature damage |
| sword_of_truth_and_justice | E | equipped creature damage |
| the_reaver_cleaver | E | equipped creature damage |
| lathril_blade_of_the_elves | A | self damage + amount |

Cards partially fixable (trigger added, other gaps remain):

| Card | Pattern | Remaining Gap |
|------|---------|------|
| balefire_dragon | A | ForEach over damaged player's creatures |
| marisi_breaker_of_the_coil | B | CantCast during combat static |
| sword_of_feast_and_famine | E | UntapAllLands effect |
| sword_of_light_and_shadow | E | May return creature from GY |
| sword_of_war_and_peace | E | CardCount hand size |
| grim_hireling | D | X-cost sac activated ability |
| natures_will | D | Tap/untap all lands of player |
| sigil_of_sleep | F | Bounce target of DamagedPlayer |

Cards deferred (trigger gap is not the only/primary blocker):

| Card | Reason |
|------|--------|
| dragonlord_ojutai | LookAtTopCards effect gap |
| hellkite_tyrant | GainControlAll effect gap |
| dokuchi_silencer | Reflexive "when you do" gap |
| biting_palm_ninja | Reflexive trigger + counter removal chain |
| saskia_the_unyielding | ChoosePlayer ETB gap |
| keeper_of_fables | Negative subtype filter (non-Human) |
| necrogen_rotpriest | Keyword-family filter (any Toxic) |
| quietus_spike | HalfLife effect gap + Deathtouch grant |
| breath_of_fury | Aura re-attachment gap |
| edric_spymaster_of_trest | TriggeringCreature controller target |
| strixhaven_stadium | Point counter tracking gap |
| steel_hellkite | Not a trigger (activated ability) |
| kaito_shizuki | PW emblem gap |
| kaito_dancing_shadow | PW complexity |
| ancient_bronze_dragon | d20 roll gap |
| umezawas_jitte | "Deals combat damage" (any target, not just player) |

**Estimated fixable count**: ~26 cards (18 fully fixable + 8 partial fixes with trigger added)

## Unit Tests

**File**: `crates/engine/tests/combat_damage_triggers.rs`
**Tests to write**:
- `test_self_combat_damage_trigger_draws_card` -- CR 510.3a: Scroll Thief deals combat damage, draws a card
- `test_self_combat_damage_trigger_damaged_player` -- Balefire Dragon: verify DamagedPlayer resolution
- `test_self_combat_damage_trigger_damage_amount` -- Lathril: verify CombatDamageDealt resolution
- `test_per_creature_you_control_combat_damage_trigger` -- Ohran Frostfang: multiple creatures, each triggers separately
- `test_per_creature_subtype_filter_combat_damage` -- Ingenious Infiltrator: Ninja filter, non-Ninja creature should NOT trigger
- `test_batch_one_or_more_trigger_fires_once` -- Professional Face-Breaker: 3 creatures deal damage, trigger fires once
- `test_batch_trigger_per_damaged_player` -- 2 creatures damage different players, trigger fires twice
- `test_equipped_creature_combat_damage_trigger` -- Sword of Fire and Ice: equipped creature deals damage, trigger fires on equipment
- `test_enchanted_creature_damage_trigger` -- Curiosity: enchanted creature deals ANY damage, draws card
- `test_combat_damage_trigger_prevented_does_not_fire` -- CR 603.2g: prevented damage = no trigger
- `test_combat_damage_trigger_first_strike_double_strike` -- CR 510.4: double strike creature triggers twice
- `test_equipped_creature_unequipped_no_trigger` -- Equipment not attached = no trigger
**Pattern**: Follow tests for combat damage in `tests/combat_tests.rs` and trigger tests in `tests/trigger_tests.rs`

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (or explicitly deferred with reason)
- [ ] Existing 6 card defs using old variant shape updated to `{ filter: None }`
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in fixable card defs

## Risks & Edge Cases

1. **Breaking variant shape change**: Converting `WheneverCreatureYouControlDealsCombatDamageToPlayer` from unit variant to struct variant with `filter` field will break all 6 existing usages. Must update them all atomically.

2. **Batch trigger grouping complexity**: The "one or more" batch trigger requires grouping assignments by (controller, damaged_player) before dispatching. The filter check (e.g., "non-Human creatures") must verify that at least one of the dealing creatures in the batch matches. This requires iterating the assignments again within the trigger dispatch.

3. **Equipment/Aura attachment traversal**: The trigger dispatch for equipped/enchanted creature triggers requires iterating `creature.attachments` for each assignment. If a creature has multiple Equipment/Auras with combat damage triggers, each fires independently. Performance should be fine since attachment counts are small.

4. **First strike + double strike**: A creature with double strike deals combat damage twice (first strike step + regular step). Each step fires a separate `CombatDamageDealt` event. Per-creature triggers fire per step. Batch triggers should also fire per step (once per step per damaged player). Verify this is correct per CR 510.4.

5. **`entering_object_id` reuse**: PendingTrigger's `entering_object_id` is currently used for ETB filter checks. Reusing it for "the dealing creature" in combat damage triggers requires that the resolution code distinguishes between ETB and combat damage contexts. The `triggering_event` field (TriggerEvent enum) provides this disambiguation.

6. **"Deals damage" vs "deals combat damage"**: Curiosity and Ophidian Eye say "deals damage" (not "combat damage"). The `WhenEnchantedCreatureDealsDamageToPlayer` trigger with `combat_only: false` must also fire on non-combat damage events (`GameEvent::DamageDealt`). This requires wiring in the non-combat damage path too, which is more complex. **Consider**: For PB-30, implement the combat-only variant first and note the non-combat variant as a follow-up. Curiosity/Ophidian Eye TODOs would remain partially fixed.

7. **`DamagedPlayer` availability**: The `damaged_player` field on PendingTrigger is only populated for triggers originating from `CombatDamageDealt` event handling. If `DamagedPlayer` is used in effects outside this context, it will be None and cause a panic. Add a runtime guard or make it return an error gracefully.

8. **Hash discriminant chain**: The last TriggerCondition discriminant is 35 (WhenYouCastThisSpell). New variants start at 36. The last TriggerEvent discriminant is 39 (ControllerGainsLife). New variants start at 40. Verify no gaps in the chain.
