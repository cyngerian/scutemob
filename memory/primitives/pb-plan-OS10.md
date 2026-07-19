# Primitive Batch Plan: PB-OS10 — singleton cleanup pair (inter-target distinctness + Jitte any-recipient combat trigger)

**Generated**: 2026-07-19
**Primitive(s)**:
1. `TargetRequirement::TargetPermanentDistinctFrom(usize)` — inter-target distinctness ("another target permanent"), CR 601.2c.
2. `TriggerCondition::WhenEquippedCreatureDealsCombatDamage` + runtime `TriggerEvent::EquippedCreatureDealsCombatDamage` — equipped-creature-deals-combat-damage trigger for ANY recipient (player, creature, planeswalker), CR 510.3a / 603.2c.
**CR Rules**: 601.2c (target announcement / "another" distinctness), 510.3a (combat damage triggers), 603.2c (once-per-event trigger), 400.7 (object identity — n/a here, cost does not move source), 602.2 / 700.2a/700.2d (modal activated ability).
**Cards affected**: 2 (`hidden_strings` — stays `known_wrong`; `umezawas_jitte` — `known_wrong` → **Complete** if execution-verified).
**Dependencies**: PB-XS (`exclude_self` pattern this mirrors), PB-EF7 (`AbilityDefinition::Activated::modes` / `ModeSelection` — required for the Jitte modal conversion), PB-AC4 (`mode_targets` per-mode target slices). All present.
**Deferred items from prior PBs**: none pulled into this batch. (OOS-XS-1 and OOS-EF7-1 were both filed/deferred and are the seeds of *this* batch.)

**TODO sweep** (roster-recall gate): grepped `crates/card-defs/src/defs/` for TODO/known_wrong notes naming these two primitives:
- `TargetPermanentDistinctFrom` / "another target" / "distinctness": **only `hidden_strings.rs`** self-identifies (its `known_wrong` note: "'another target permanent' distinctness is not enforced"). 0 other cards.
- `WhenEquippedCreatureDealsCombatDamage` (any-recipient): **only `umezawas_jitte.rs`** self-identifies. Note: `quietus_spike.rs` and `glimmer_lens.rs` reference `WhenEquippedCreatureDealsCombatDamageToPlayer` but their oracle text is genuinely "...to a player" — they are NOT any-recipient cards and are correctly on the existing `...ToPlayer` variant. Do **not** repoint them.
- Result: **TODO sweep confirms the 2-card roster; 0 additional cards.**

---

## Primitive Specification

### Primitive 1 — inter-target distinctness (OOS-XS-1)

CR 601.2c (verified via MCP this session): *"The same target can't be chosen multiple times for any one instance of the word 'target'. However, if the spell uses the word 'target' in multiple places, the same object or player can be chosen once for each instance of the word 'target'..."* So the engine **default** (two `TargetPermanent` slots → the same permanent may fill both) is already correct. The word **"another"** on Hidden Strings' second target is what forbids reuse. Therefore a *blanket* duplicate-rejection pass would be **wrong** (it would break legitimate same-target reuse on other cards); distinctness must be **opt-in per slot**.

**Decision: OPTION (a)** — a new `TargetRequirement` variant `TargetPermanentDistinctFrom(usize)`, where the `usize` is the **requirement-slot index** this slot must differ from. This mirrors the PB-XS `exclude_self` pattern (opt-in, per-requirement, wire-closure enum extension) and is lower-blast-radius than option (b): option (b) (a post-bind pass) still needs a per-slot DSL marker to know *which* slots must be distinct (blanket rejection is CR-wrong per above), so it collapses into option (a) plus extra plumbing. Option (a) reuses the existing best-fit slot map that `validate_targets_inner` already computes.

For all **type/legality** checks the new variant behaves exactly like `TargetRequirement::TargetPermanent` (any battlefield permanent). The distinctness constraint is enforced in a single post-slot-assignment pass in the target validator.

### Primitive 2 — any-recipient equipped-creature combat-damage trigger (OOS-EF7-1)

Only `WhenEquippedCreatureDealsCombatDamageToPlayer` / `TriggerEvent::EquippedCreatureDealsCombatDamageToPlayer` exist; they fire solely for `CombatDamageTarget::Player` assignments (abilities.rs ~5363). Umezawa's Jitte's oracle is *"Whenever equipped creature deals combat damage"* — ANY recipient (a blocking/attacking creature or a planeswalker, not just a player). Add a distinct variant pair that fires for **every** `CombatDamageTarget` (Player/Creature/Planeswalker), **once per equipped source creature per combat-damage step** (CR 603.2c — a creature dealing damage to two blockers is one dealing event → one trigger; double strike is two separate steps → two triggers, handled by this collector being invoked once per step).

---

## CR Rule Text (authoritative, from MCP this session)

**601.2c** — "The player announces their choice of an appropriate object or player for each target the spell requires. ... The same target can't be chosen multiple times for any one instance of the word 'target' on the spell. However, if the spell uses the word 'target' in multiple places, the same object or player can be chosen once for each instance of the word 'target' (as long as it fits the targeting criteria). ... The chosen objects and/or players each become a target of that spell."

(510.3a / 603.2c are cited inline in the existing equipped-creature trigger code and hash comments; not re-pasted.)

---

## Engine Changes

Numbered in dependency order. The runner should do all DSL-enum additions first (card-types), then engine dispatch, then the card defs, then the single batched wire bump, then tests.

### Change 1 — new `TargetRequirement` variant (DSL enum)

**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: Add a unit-tuple variant to `pub enum TargetRequirement` (enum starts L2942). Place it adjacent to `TargetSpellWithSingleTarget` (the current last variant region) or next to `TargetPermanent`:
```rust
/// CR 601.2c ("another target"): a battlefield permanent that must be a DIFFERENT
/// object than the target bound to requirement slot `usize`. Type-legality is
/// identical to `TargetPermanent`; only inter-target distinctness is added.
/// Mirrors the `exclude_self` opt-in pattern (PB-XS). The index is the
/// *requirement-slot* index (position in the ability's `targets`/`mode_targets`
/// list), not a declaration-order index.
TargetPermanentDistinctFrom(usize),
```
**Pattern**: Follow `TargetSpellWithSingleTarget` (PB-EF11) as the "new unit-ish variant" precedent.

### Change 2 — new `TriggerCondition` variant (DSL enum)

**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: Add, immediately after `WhenEquippedCreatureDealsCombatDamageToPlayer` (L3492):
```rust
/// "Whenever equipped creature deals combat damage" (ANY recipient — player,
/// creature, or planeswalker). CR 510.3a / 603.2c. Distinct from
/// `WhenEquippedCreatureDealsCombatDamageToPlayer`, which fires only on damage to
/// a player. Fires once per equipped source creature per combat-damage step. The
/// trigger source is the Equipment. Used by Umezawa's Jitte.
WhenEquippedCreatureDealsCombatDamage,
```

### Change 3 — new `TriggerEvent` variant (runtime enum)

**File**: `crates/card-types/src/state/game_object.rs`
**Action**: Add to `pub enum TriggerEvent` (enum starts L413), immediately after `EquippedCreatureDealsCombatDamageToPlayer` (L554):
```rust
/// CR 510.3a / 603.2c: fires on an Equipment when its equipped creature deals
/// combat damage to ANY recipient (player, creature, or planeswalker). Fired once
/// per equipped source creature per combat-damage step from `rules/abilities.rs`.
EquippedCreatureDealsCombatDamage,
```

### Change 4 — TriggerCondition → TriggerEvent conversion (runtime enrichment)

**File**: `crates/engine/src/testing/replay_harness.rs` (`enrich_spec_from_def`, L3333)
**Action**: Add a new conversion loop mirroring the existing `WhenEquippedCreatureDealsCombatDamageToPlayer` loop (L2959–2985), immediately after it:
```rust
// CR 510.3a: Convert "Whenever equipped creature deals combat damage" (any recipient).
for ability in abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamage,
        effect, targets, ..
    } = ability {
        triggered_abilities.push(TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            trigger_on: TriggerEvent::EquippedCreatureDealsCombatDamage,
            intervening_if: None,
            description: "Whenever equipped creature deals combat damage (CR 510.3a)".to_string(),
            effect: Some(effect.clone()),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: targets.clone(),
        });
    }
}
```
**CR**: 510.3a. (Not an exhaustive match — an if-let loop; no exhaustiveness break.)

### Change 5 — fire the new trigger in combat-damage collection

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add a new firing block immediately AFTER the existing equipped/enchanted `...ToPlayer` loop (which ends ~L5414), and before the `AnyCreatureDealsCombatDamageToOpponent` block (L5415). Unlike the `...ToPlayer` loop, iterate ALL recipients and **dedupe by source creature** so it fires once per equipped creature per step (CR 603.2c):
```rust
// CR 510.3a / 603.2c: EquippedCreatureDealsCombatDamage (any recipient).
// Fires once per equipped SOURCE creature per combat-damage step, regardless of
// how many recipients it damaged (trample/multi-block = one dealing event; double
// strike = two steps = two invocations of this collector).
let mut damaged_sources: Vec<ObjectId> = Vec::new();
for assignment in assignments {
    if assignment.amount == 0 { continue; } // CR 603.2g
    if !damaged_sources.contains(&assignment.source) {
        damaged_sources.push(assignment.source);
    }
}
for source_creature in damaged_sources {
    // CR 603.10: combat-damage triggers do NOT look back — source must still be on bf.
    let on_bf = state.objects.get(&source_creature)
        .map(|o| o.zone == ZoneId::Battlefield).unwrap_or(false);
    if !on_bf { continue; }
    let attachments: Vec<ObjectId> = state.expect_object(source_creature)
        .map(|o| o.attachments.iter().copied().collect()).unwrap_or_default();
    // total damage this source dealt this step (for cards that read the amount;
    // Jitte ignores it but populate for parity with the ...ToPlayer path).
    let total: u32 = assignments.iter()
        .filter(|a| a.source == source_creature).map(|a| a.amount).sum();
    for attachment_id in attachments {
        let pre_len = triggers.len();
        collect_triggers_for_event(
            state, &mut triggers,
            TriggerEvent::EquippedCreatureDealsCombatDamage,
            Some(attachment_id), None,
        );
        for t in &mut triggers[pre_len..] {
            t.entering_object_id = Some(source_creature);
            t.combat_damage_amount = total;
            // damaged_player intentionally left None — recipient may be a creature/pw.
        }
    }
}
```
**CR**: 510.3a / 603.2c / 603.2g. Only Equipment attachments participate (this is the equipped-creature trigger; the Aura analogue for any-recipient is out of scope). `collect_triggers_for_event` already filters to attachments whose `trigger_on == EquippedCreatureDealsCombatDamage`, so non-Jitte attachments are ignored.

**Verification note for the runner**: confirm `collect_triggers_for_event` matches the Equipment attachment's `trigger_on` field (it does for the `...ToPlayer` sibling). If an Equipment is attached but its trigger is the `...ToPlayer` variant it must NOT fire here (distinct discriminant) — a decoy test covers this.

### Change 6 — inter-target distinctness enforcement in target validation

**File**: `crates/engine/src/rules/casting.rs`

**6a. Type-legality arm (EXHAUSTIVE match, no catch-all — L6244 `let valid = match req`)**: add
```rust
TargetRequirement::TargetPermanentDistinctFrom(_) => on_battlefield,
```
(identical to the `TargetPermanent` arm at L6246). This is REQUIRED for compilation — the match at L6244–6434 has no `_` arm (confirmed: ends at L6434 with `UpToN`).

**6b. Distinctness helper** — add a free function near `validate_object_satisfies_requirement`:
```rust
/// CR 601.2c ("another target"): enforce that every `TargetPermanentDistinctFrom(k)`
/// slot is filled by a different object than slot `k`. `slot_object[i]` is the
/// ObjectId bound to requirement slot `i` (None if that slot took a player/no target).
fn enforce_inter_target_distinctness(
    requirements: &[TargetRequirement],
    slot_object: &[Option<ObjectId>],
) -> Result<(), GameStateError> {
    for (si, req) in requirements.iter().enumerate() {
        if let TargetRequirement::TargetPermanentDistinctFrom(other) = req {
            let a = slot_object.get(si).copied().flatten();
            let b = slot_object.get(*other).copied().flatten();
            if let (Some(a), Some(b)) = (a, b) {
                if a == b {
                    return Err(GameStateError::InvalidTarget(
                        "the same permanent cannot be chosen for both targets \
                         ('another target permanent', CR 601.2c)".to_string(),
                    ));
                }
            }
        }
    }
    Ok(())
}
```

**6c. Wire it into `validate_targets_inner`** (L5795): after the best-fit assignment completes and the "unassigned" check passes (~L5897), and BEFORE `target_slot` is consumed by `.into_iter()` at L5902, build `slot_object` and enforce:
```rust
// CR 601.2c inter-target distinctness. Build slot -> bound ObjectId from target_slot
// (object targets only; players never satisfy a permanent-distinct requirement).
let mut slot_object: Vec<Option<ObjectId>> = vec![None; requirements.len()];
for (ti, slot) in target_slot.iter().enumerate() {
    if let (Some(si), Target::Object(id)) = (slot, &targets[ti]) {
        slot_object[*si] = Some(*id);
    }
}
enforce_inter_target_distinctness(requirements, &slot_object)?;
```
Note: `requirements.is_empty()` short-circuits before this (the `else` branch that builds `target_slot`); guard so the distinctness pass only runs when `!requirements.is_empty()` (there can be no `DistinctFrom` req otherwise). Keep the borrow of `target_slot` before the existing `target_slot.into_iter()...` line.

**6d. Wire it into `validate_targets_positional`** (L5930, modal per-mode path): positional — slot index == target index. Before `validate_mapped_targets`, build `slot_object[i] = matches!(targets[i], Target::Object(id)).then(id)` and call `enforce_inter_target_distinctness(requirements, &slot_object)?`. Defensive/general (Hidden Strings is not modal, but a future modal "another target" card would route here). Low cost, keeps the primitive whole.

**6e. `validate_player_satisfies_requirement`** (L6091): NO change needed — its final arm is a catch-all `_ => Err(...)`, so a player offered for a `TargetPermanentDistinctFrom` slot is correctly rejected (permanents only). Confirm the catch-all still compiles (it does).

### Change 7 — auto-target picker arm (EXHAUSTIVE match)

**File**: `crates/engine/src/rules/abilities.rs` (L7299 `match req`, exhaustive, ends ~L7404 with `TargetSpellWithSingleTarget => false` then `UpToN`)
**Action**: add
```rust
TargetRequirement::TargetPermanentDistinctFrom(_) => true,
```
(identical to `TargetPermanent => true` at L7301). REQUIRED for compilation. This picker only pre-filters candidate objects by type; the distinctness constraint is enforced at declaration validation (Change 6), which is sufficient (a bot picking two identical targets is rejected there — acceptable, no infinite loop since Hidden Strings stays `known_wrong` and is not bot-exercised in a flip test).

### Change 8 — HashInto arms (EXHAUSTIVE matches; SR-8 closure)

**File**: `crates/engine/src/state/hash.rs`

**8a. `TargetRequirement`** (impl ends L5335; current max discriminant 19 = `TargetSpellWithSingleTarget`): add
```rust
// PB-OS10: TargetPermanentDistinctFrom -- CR 601.2c "another target" (discriminant 20)
TargetRequirement::TargetPermanentDistinctFrom(idx) => {
    20u8.hash_into(hasher);
    idx.hash_into(hasher);
}
```
(Verify `usize: HashInto` exists; if not, hash `(*idx as u64)`.)

**8b. `TriggerEvent`** (impl ends L3175; current max 47 = `PermanentBecomesTarget`): add
```rust
// PB-OS10: equipped creature deals combat damage, any recipient — discriminant 48
TriggerEvent::EquippedCreatureDealsCombatDamage => 48u8.hash_into(hasher),
```

**8c. `TriggerCondition`** (impl ends L5795; current max 47 = `WhenBecomesTarget`): add
```rust
// PB-OS10: "Whenever equipped creature deals combat damage" (any recipient) — discriminant 48
TriggerCondition::WhenEquippedCreatureDealsCombatDamage => 48u8.hash_into(hasher),
```

### Change 9 — exhaustive-match audit (verify no other break sites)

The complete set of exhaustive matches on the three enums, derived from where PB-EF11's `TargetSpellWithSingleTarget` and the existing `EquippedCreatureDealsCombatDamageToPlayer` are matched:
| Enum | Match site | Change |
|------|-----------|--------|
| `TargetRequirement` | `casting.rs:6244` (`let valid = match`) | Change 6a |
| `TargetRequirement` | `casting.rs:6427` region? — NO: 6420–6433 is the SAME `let valid` match's tail (one match). Covered by 6a. | — |
| `TargetRequirement` | `abilities.rs:7299` (auto-target picker) | Change 7 |
| `TargetRequirement` | `hash.rs:5292` impl | Change 8a |
| `TriggerEvent` | `hash.rs:3155` impl (ONLY exhaustive match) | Change 8b |
| `TriggerCondition` | `hash.rs:5745` impl (ONLY exhaustive match; enrich conversion is if-let) | Change 8c |

`validate_player_satisfies_requirement` (casting.rs:6091) and the effects/simulator crates have catch-alls or do not match these variants — no new arms. **Runner must still run `cargo build --workspace`** after Changes 1–8 to catch any match the map missed (per gotchas-infra: replay-viewer `view_model.rs` / TUI `stack_view.rs` match `StackObjectKind`/`KeywordAbility`, NOT these three enums — so they should not break, but build anyway).

---

## Card Definition Fixes

### hidden_strings.rs
**Oracle**: "You may tap or untap target permanent, then you may tap or untap another target permanent. / Cipher"
**Current**: `known_wrong` — modeled as unconditional double-tap; drops untap mode, drops "may" optionality, does not enforce distinctness.
**Fix**: change the second target requirement (L37) from `TargetRequirement::TargetPermanent` to `TargetRequirement::TargetPermanentDistinctFrom(0)`. Leave the `Effect::Sequence(TapPermanent, TapPermanent)` approximation as-is. Update the `known_wrong` note to state that inter-target distinctness is now **enforced** (a single permanent can no longer be chosen for both slots), but the card **remains `known_wrong`** because (i) "tap OR untap" player choice is unmodeled (always taps), and (ii) the "you MAY" optionality is dropped. No `Effect::Choose` is introduced (would violate the §5 no-gated-stub guardrail).
**Final completeness**: **stays `known_wrong`.** AC1 is satisfied by the primitive being enforced and pinned by a test (Change/Tests below), NOT by a flip.

### umezawas_jitte.rs
**Oracle**: "Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte. / Remove a charge counter from Umezawa's Jitte: Choose one — • Equipped creature gets +2/+2 until end of turn. • Target creature gets -1/-1 until end of turn. • You gain 2 life. / Equip {2}"
**Current**: `known_wrong` — trigger fires only on damage to players; modal ability uses non-interactive `Effect::Choose` (barred from Complete).
**Fix (two edits):**

1. **Trigger** (L42–55): change `trigger_condition` from `WhenEquippedCreatureDealsCombatDamageToPlayer` to `WhenEquippedCreatureDealsCombatDamage`. Keep `effect: Effect::AddCounter { target: EffectTarget::Source, counter: CounterType::Charge, count: 2 }`, `once_per_turn: false`, `targets: vec![]`, `modes: None`.

2. **Modal activated ability** (L61–107): convert from `Effect::Choose` to `AbilityDefinition::Activated::modes` (`ModeSelection`), mirroring `goblin_cratermaker.rs`:
```rust
AbilityDefinition::Activated {
    cost: Cost::RemoveCounter { counter: CounterType::Charge, count: 1 },
    effect: Effect::Sequence(vec![]),          // placeholder; real effects live in `modes`
    timing_restriction: None,
    targets: vec![],                            // MUST be empty when mode_targets is Some
    activation_condition: None,
    activation_zone: None,
    once_per_turn: false,
    modes: Some(ModeSelection {
        min_modes: 1,
        max_modes: 1,
        allow_duplicate_modes: false,
        mode_costs: None,
        modes: vec![
            // Mode 0: equipped creature +2/+2 EOT (no target)
            Effect::ApplyContinuousEffect { effect_def: Box::new(ContinuousEffectDef {
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyBoth(2),
                filter: EffectFilter::AttachedCreature,
                duration: EffectDuration::UntilEndOfTurn,
                condition: None,
            })},
            // Mode 1: target creature -1/-1 EOT (1 target)
            Effect::ApplyContinuousEffect { effect_def: Box::new(ContinuousEffectDef {
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyBoth(-1),
                filter: EffectFilter::DeclaredTarget { index: 0 },
                duration: EffectDuration::UntilEndOfTurn,
                condition: None,
            })},
            // Mode 2: gain 2 life (no target)
            Effect::GainLife { player: PlayerTarget::Controller, amount: EffectAmount::Fixed(2) },
        ],
        mode_targets: Some(vec![
            vec![],                                     // Mode 0: no targets
            vec![TargetRequirement::TargetCreature],    // Mode 1: one target creature
            vec![],                                     // Mode 2: no targets
        ]),
    }),
},
```
**Final completeness**: **`Complete` — CONTINGENT on execution verification** (see below). If any contingency fails, keep `known_wrong` with a truthful note naming the blocking clause.

**Jitte chain verification (runner MUST prove by execution, not source-tracing — SR-34/36 / §5):**
- (i) **Trigger**: equip a creature, deal combat damage to a CREATURE (a blocker); assert 2 charge counters land on Jitte (proves any-recipient firing, the surviving OOS-EF7-1 blocker). Also test damage to a player.
- (ii) **Cost**: `Cost::RemoveCounter { Charge, 1 }` flattens to `ac.remove_counter_cost` (replay_harness.rs:4086) and is paid on the activation path; assert the counter is removed on activation and that the ability is **unactivatable with 0 charge counters**.
- (iii) **Mode selection is real**: activate choosing **each** mode and assert the correct effect:
  - Mode 0 selected → equipped creature is +2/+2 (proves `EffectFilter::AttachedCreature` resolves for an *activated* modal ability whose source stays on the battlefield — no precedent in goblin_cratermaker, so this is the highest-risk clause).
  - Mode 1 selected → target creature is -1/-1 (proves a targeted mode among untargeted siblings).
  - Mode 2 selected → controller gains 2 life (proves an untargeted mode with an EMPTY `mode_targets` slice — verify the engine tolerates a `vec![]` mode-target slice; pb_ac4 is the reference).
- **If mode 0's `AttachedCreature` does not resolve in the activated-modal context, or an empty `mode_targets` slice is rejected**: keep `known_wrong`, note which clause blocks, and file a follow-up seed (OOS-OS10-1). Do NOT ship a partial-but-Complete card.

---

## New Card Definitions

None. Both cards already exist.

---

## Wire Bump (single batched PROTOCOL 24 → 25 / HASH 61 → 62)

Both primitives extend SR-8-closure enums (`TargetRequirement` is in the wire closure — PB-EF11 v17 precedent; `TriggerEvent`/`TriggerCondition` are in the hash closure via `GameState`). The machine gates (`tests/core/protocol_schema.rs`, `tests/core/hash_schema.rs`) will force both bumps. Procedure (mirror PB-OS9's 23→24 / 60→61):

### PROTOCOL 24 → 25
1. `crates/engine/src/rules/protocol.rs` L225: `PROTOCOL_VERSION: u32 = 25;`
2. Same file, above L225: add a `/// - 25: PB-OS10 (2026-07-19, OOS-XS-1 + OOS-EF7-1): TargetRequirement gains TargetPermanentDistinctFrom (CR 601.2c); ...` doc line (note: TriggerCondition/TriggerEvent are NOT in the wire closure, so the wire-relevant change is the `TargetRequirement` variant — say so).
3. Append a `ProtocolEpoch { version: 25, fingerprint: <recomputed> }` row to `PROTOCOL_HISTORY` (after the version-24 row at L433–438).
4. L242: set `PROTOCOL_SCHEMA_FINGERPRINT` to the recomputed digest (read from the `protocol_schema.rs` failure text).
5. `crates/engine/tests/core/protocol_schema.rs`: `protocol_version_sentinel` (L872) → `25`; update `FROZEN_HISTORY_PREFIX_DIGEST` (L152) to the value the `frozen_prefix_is_pinned` failure prints.

### HASH 61 → 62
6. `crates/engine/src/state/hash.rs` L550: `HASH_SCHEMA_VERSION: u8 = 62;`
7. Add a `/// - 62: PB-OS10 ...` doc line above L550 (TargetRequirement + TriggerEvent + TriggerCondition all gained variants; all in the GameState closure).
8. Append a `HashSchemaEpoch { version: 62, decl_fingerprint: <recomputed>, stream_fingerprint: <recomputed> }` row to `HASH_SCHEMA_HISTORY` (after the version-61 row at L825).
9. `crates/engine/tests/core/hash_schema.rs`: update the `HASH_SCHEMA_VERSION` sentinel and `FROZEN_*_PREFIX_DIGEST` values from the failure text (both decl and stream digests).

### Scattered live-version sentinels (mechanical bulk edit — HIGH churn, do not skip)
These assert the *live* const and redden on every bump:
- **42 occurrences of `HASH_SCHEMA_VERSION, 61`** across 41 files → `62`. (Includes `tests/core/hash_schema.rs` plus 40 test files: all `crates/engine/tests/primitives/*`, `mechanics_e_l/effect_sacrifice_permanents_filter.rs`, `casting/optional_cost_and_counter_tax.rs`, `rules/loyalty_target_validation.rs`.)
- **9 occurrences of `PROTOCOL_VERSION, 24`** → `25` (`tests/core/protocol_schema.rs` + 8 `tests/primitives/pb_*` files).
Do a scoped find/replace on `HASH_SCHEMA_VERSION,\s*61` → `62` and `PROTOCOL_VERSION,\s*24` → `25`, then `cargo test --all` to confirm none missed. (Prior PBs bumped all of these each time — the version numbers already drifted forward from each card's ship version, confirming they track the live const.)

---

## Unit Tests

**New file**: `crates/engine/tests/primitives/pb_os10_singleton_cleanup.rs`
**Register**: add `mod pb_os10_singleton_cleanup;` to `crates/engine/tests/primitives/main.rs` (alongside the other `pb_os*` mods, L37–42).

**Distinctness (OOS-XS-1)** — pattern: `pb_ef1_exclude_self_enforcement.rs` / `pb_ef11_spell_single_target.rs` (direct `validate_targets*` unit calls + a full cast path):
- `test_distinct_from_rejects_same_permanent` — a 2-slot spell `[TargetPermanent, TargetPermanentDistinctFrom(0)]`; declare the SAME permanent for both → `validate_targets_with_source` (or the cast command) returns `InvalidTarget`. CR 601.2c.
- `test_distinct_from_accepts_two_different` — two different permanents → Ok, both bound. CR 601.2c.
- `test_distinct_from_type_legality` — `TargetPermanentDistinctFrom(0)` accepts any battlefield permanent (artifact/land/etc.), rejects a nonpermanent/player (behaves as `TargetPermanent`).
- `test_hidden_strings_second_slot_distinct` — use `hidden_strings::card()`; assert its slot-1 requirement is `TargetPermanentDistinctFrom(0)` and that casting it at one permanent twice is rejected. (Card stays `known_wrong`; this only pins the primitive.)

**Jitte trigger + modal (OOS-EF7-1)** — pattern: `pb_ef7_modal_activated.rs` (goblin_cratermaker modal activation) + an existing equipment-combat-damage test:
- `test_jitte_triggers_on_damage_to_creature` — equip a creature, it deals combat damage to a BLOCKER creature; assert 2 charge counters on Jitte. (Core OOS-EF7-1 proof.)
- `test_jitte_triggers_on_damage_to_player` — damage to a player also adds 2 counters.
- `test_jitte_no_trigger_on_noncombat_damage` — deal NON-combat damage (e.g., a burn spell) via the equipped creature → NO counters (decoy; the trigger is combat-only).
- `test_jitte_no_trigger_when_unequipped` — Jitte on battlefield but not attached, a creature deals combat damage → NO counters (decoy; requires attachment).
- `test_jitte_fires_once_per_multiblock` — equipped attacker blocked by two creatures, assigns damage to both → exactly 2 counters (ONE trigger), not 4 (proves dedupe-by-source, CR 603.2c).
- `test_jitte_distinct_from_toplayer_variant` — an Equipment carrying the OLD `...ToPlayer` trigger does NOT fire from the new any-recipient path when damaging a creature (proves discriminant separation).
- `test_jitte_mode0_pumps_equipped` — accumulate ≥1 counter, activate choosing mode 0 → equipped creature +2/+2, one counter removed.
- `test_jitte_mode1_shrinks_target` — activate choosing mode 1 targeting a creature → that creature -1/-1.
- `test_jitte_mode2_gains_life` — activate choosing mode 2 → controller +2 life (untargeted empty-slice mode).
- `test_jitte_cost_requires_counter` — with 0 charge counters, the modal ability is not activatable / cost cannot be paid.
- `test_jitte_counter_accumulation_roundtrip` — two combat-damage events → 4 counters; spend 2 across two activations → 2 remain (accumulation + spend).

**Version sentinel** (mirror pb_os9 L884/888):
- `test_pb_os10_version_sentinel` — `assert_eq!(PROTOCOL_VERSION, 25)` and `assert_eq!(HASH_SCHEMA_VERSION, 62u8)` with the standard drift message.

---

## Verification Checklist

- [ ] `cargo check -p mtg-card-types` (DSL enums compile: Changes 1–3)
- [ ] `cargo build --workspace` (all exhaustive matches updated — Changes 6a/7/8; catches any missed match site)
- [ ] `hidden_strings.rs` uses `TargetPermanentDistinctFrom(0)`; note updated; stays `known_wrong`
- [ ] `umezawas_jitte.rs`: trigger repointed; modal converted to `modes`; `Effect::Choose` removed
- [ ] Jitte chain execution-verified (trigger any-recipient + all 3 modes + RemoveCounter cost) → flip to `Complete`, OR keep `known_wrong` with named blocker + OOS-OS10-1 filed
- [ ] Single wire bump applied: PROTOCOL 24→25, HASH 61→62 (consts + history rows + fingerprints + both `_schema.rs` gates)
- [ ] All 42 `HASH_SCHEMA_VERSION, 61` and 9 `PROTOCOL_VERSION, 24` sentinels bumped
- [ ] `cargo test --all` green (includes `tools/check-defs-fmt.sh` via `core card_defs_fmt`)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` + `tools/check-defs-fmt.sh` clean (SR-35)
- [ ] No `Effect::Choose` / gated-stub in any `Complete` card (§5 guardrail)

---

## Risks & Edge Cases

- **Blanket-distinctness trap**: CR 601.2c makes same-target reuse the DEFAULT across multiple "target" instances. A global duplicate-rejection pass would break legitimate cards (e.g., Cryptic Command's two `TargetPermanent` modes, any card that lets you tap the same thing twice). The per-slot `DistinctFrom` variant is the correct, minimal expression. Do NOT add a blanket pass.
- **Best-fit slot reassignment vs. distinctness**: `validate_targets_inner` reorders targets to requirement slots by best-fit. For Hidden Strings (two homogeneous permanent slots) the two identical targets necessarily occupy slots 0 and 1, so `slot_object[0] == slot_object[1]` correctly triggers rejection. Build `slot_object` from `target_slot` BEFORE the `into_iter()` consumption at L5902.
- **Jitte double strike**: two combat-damage steps → this collector runs twice → 4 counters total. Correct (CR 510.5 / 702.4e). Single step with multiple recipients (trample/multi-block) → dedupe-by-source → 2 counters. Both pinned by tests.
- **Jitte mode 0 `AttachedCreature` in an activated context** (highest-risk clause): no existing Complete card exercises `EffectFilter::AttachedCreature` inside a modal *activated* ability. If it fails to resolve to the equipped creature at resolution, Jitte stays `known_wrong`. Runner must execute-verify.
- **Empty `mode_targets` slice** (modes 0 and 2): pb_ac4 established per-mode target slices, but verify a `vec![]` slice for a chosen mode is accepted (no phantom target demanded). If rejected, that is the blocking clause.
- **`combat_damage_amount`/`damaged_player` on the new trigger**: recipient may be a creature/planeswalker, so `damaged_player` is left `None`. Any card reading `damaged_player` on this trigger would misbehave — Jitte does not; if a future card needs the recipient, that is a separate primitive. Note in the close-out.
- **Wire-bump churn (51 sentinels)**: the single most error-prone step. A missed sentinel fails `cargo test --all` loudly (not silently), so it is self-correcting, but budget time for the bulk edit + a full test run.
- **`usize: HashInto`**: verify the trait is implemented for `usize`/`u64` before Change 8a; fall back to `(*idx as u64)` if not.

---

## Summary of Plan Decisions (for the coordinator)

- **Distinctness: OPTION (a)** — new `TargetRequirement::TargetPermanentDistinctFrom(usize)` variant, enforced by a small post-slot-assignment pass in `validate_targets_inner` (+ positional path). Chosen because CR 601.2c makes same-target reuse the default, so option (b)'s "blanket duplicate rejection" is CR-wrong and would still need a per-slot marker — collapsing into option (a). Lowest blast radius: one enum variant + 3 exhaustive-match arms + one ~15-line helper. `hidden_strings` wires the variant but **stays `known_wrong`** (tap/untap choice + optionality still unmodeled); AC1 met by the primitive being enforced + test-pinned.
- **Jitte: recommend FLIP to `Complete`**, contingent on execution-verifying three clauses (any-recipient trigger, `Cost::RemoveCounter` payment, and all three modes actually selecting). The plan **converts the modal ability from `Effect::Choose` to the PB-EF7 `modes`/`ModeSelection` primitive** (required — `Effect::Choose` is barred from `Complete` by the §5 no-gated-stub guardrail, and it is non-interactive). Highest-risk clause is mode 0's `AttachedCreature` filter in an activated-modal context (no precedent) and the empty `mode_targets` slices; if either fails, Jitte stays `known_wrong` with a truthful named blocker and a new OOS-OS10-1 seed.
- **Wire: single batched PROTOCOL 24→25 / HASH 61→62.** Only the `TargetRequirement` variant is wire-closure-relevant; all three variants are hash-closure-relevant. 51 scattered live-version sentinels (42 HASH + 9 PROTOCOL) plus both machine-gate files must be updated.
