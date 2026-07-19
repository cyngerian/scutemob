# Primitive WIP — PB-OS10 (IMPLEMENT COMPLETE)

<!-- last_updated: 2026-07-19 -->

**Batch**: PB-OS10 — singleton cleanup pair
**Task**: scutemob-140
**Branch**: feat/pb-os10-singleton-cleanup-pair-inter-target-distinctness-oos
**Phase**: review (done — 0 HIGH/0 MEDIUM/3 LOW dispositioned; Jitte Complete justified)

## Seeds (both confirmed REAL 2026-07-19 — NOT falsified)
- **OOS-XS-1** — inter-target distinctness. Hidden Strings: "you may tap or untap
  target permanent, then you may tap or untap **another** target permanent." Per
  CR 601.2c the same object may be chosen once per instance of "target" UNLESS the
  word "another" forces distinctness — which it does here. Confirmed real.
  Fix: `TargetRequirement::TargetPermanentDistinctFrom(usize)` OR a post-bind
  duplicate-rejection pass in casting.rs target validation.
  NOTE: hidden_strings has ADDITIONAL blockers (tap-or-untap "may" optionality;
  non-interactive Effect::Choose) — so it likely stays `known_wrong` even after
  distinctness. AC1 only requires the distinctness primitive enforced+pinned.
- **OOS-EF7-1** — Jitte counters trigger. Oracle: "Whenever equipped creature deals
  combat damage" (ANY recipient — player OR creature). Only
  `TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer` exists
  (card_definition.rs:3389). Need a `WhenEquippedCreatureDealsCombatDamage`
  (any recipient) variant. umezawas_jitte flips known_wrong->Complete ONLY if the
  full card chains (counters trigger, modal ability, RemoveCounter cost).

## Wire
Both add/extend SR-8 closure enums → single batched **PROTOCOL 24->25 / HASH 61->62**.
Both bumps applied, machine-gate fingerprints recomputed and pinned (protocol_schema.rs
+ hash_schema.rs both green). All 42 `HASH_SCHEMA_VERSION, 61` and 9
`PROTOCOL_VERSION, 24` scattered sentinels bulk-bumped.

## Plan file
memory/primitives/pb-plan-OS10.md

## Implementation record (2026-07-19)

- [x] **Change 1** — `TargetRequirement::TargetPermanentDistinctFrom(usize)` added
  (`crates/card-types/src/cards/card_definition.rs`, adjacent to
  `TargetSpellWithSingleTarget`).
- [x] **Change 2** — `TriggerCondition::WhenEquippedCreatureDealsCombatDamage` added
  (same file, after `WhenEquippedCreatureDealsCombatDamageToPlayer`).
- [x] **Change 3** — `TriggerEvent::EquippedCreatureDealsCombatDamage` added
  (`crates/card-types/src/state/game_object.rs`).
- [x] **Change 4** — enrich-conversion loop added in
  `crates/engine/src/testing/replay_harness.rs` (mirrors the `...ToPlayer` loop).
- [x] **Change 5** — new firing block in `crates/engine/src/rules/abilities.rs`
  (dedupe-by-source-creature per combat-damage step; fires for ANY recipient).
  Uses `state.fizzle_object(source_creature)` (NOT a bare `.objects.get(...)`) to
  keep the SR-25 bare-lookup ratchet green — CR 603.10 "source may have left the
  battlefield" is a legitimate quiet-None fizzle, not an engine bug.
- [x] **Change 6** — inter-target distinctness enforcement: `casting.rs` type-legality
  arm (6a), `enforce_inter_target_distinctness` helper (6b), wired into
  `validate_targets_inner` (6c) and `validate_targets_positional` (6d, defensive).
  `validate_player_satisfies_requirement`'s catch-all already rejects players (6e,
  no change needed).
- [x] **Change 7** — auto-target picker arm in `abilities.rs` (`TargetPermanentDistinctFrom(_) => true`).
- [x] **Change 8** — `HashInto` arms added: `TargetRequirement` (discriminant 20),
  `TriggerEvent` (discriminant 48), `TriggerCondition` (discriminant 48).
- [x] **Change 9** — exhaustive-match audit: `cargo build --workspace` clean; no
  replay-viewer/TUI match sites touch these three enums (confirmed per plan).
- [x] **Card fix — hidden_strings.rs**: second target slot changed to
  `TargetPermanentDistinctFrom(0)`; `known_wrong` note updated to say distinctness
  is now enforced but tap/untap "may" optionality remains the blocker. **Stays
  known_wrong** (per plan).
- [x] **Card fix — umezawas_jitte.rs**: trigger repointed to
  `WhenEquippedCreatureDealsCombatDamage`; modal ability converted from
  `Effect::Choose` to `AbilityDefinition::Activated::modes` (3 modes, exact plan
  shape). **Flipped known_wrong -> Complete** — execution-verified (see below).
  Stale header comment (claimed "NOT flipped") corrected to describe actual state.
- [x] **Wire bump**: PROTOCOL 24->25 (const, doc line, history row, fingerprint,
  `protocol_schema.rs` sentinel + FROZEN_HISTORY_PREFIX_DIGEST) and HASH 61->62
  (const, doc line, history row, both fingerprints, `hash_schema.rs` sentinel +
  FROZEN_HISTORY_PREFIX_DIGEST) all applied; both machine gates green. All 42+9
  scattered live-version sentinels bulk-bumped via scoped find/replace.
- [x] **Tests** — `crates/engine/tests/primitives/pb_os10_singleton_cleanup.rs`
  created (16 tests), `mod` line added to `tests/primitives/main.rs`. All pass.

## Jitte chain execution-verification (SR-34/36 — probed by execution)

All three contingencies PASSED:
- (i) **Trigger**: `test_jitte_triggers_on_damage_to_creature` (damage to a blocker
  creature -> 2 counters) and `test_jitte_triggers_on_damage_to_player` (damage to a
  player -> 2 counters) both pass. Decoys: `test_jitte_no_trigger_on_noncombat_damage`,
  `test_jitte_no_trigger_when_unequipped`, `test_jitte_distinct_from_toplayer_variant`
  (old `...ToPlayer` variant does NOT fire on creature damage), and
  `test_jitte_fires_once_per_multiblock` (2 blockers -> still 2 counters, not 4) all pass.
- (ii) **Cost**: `test_jitte_cost_requires_counter` (0 counters -> activation rejected)
  and `test_jitte_mode0_pumps_equipped` (counter removed at activation, before
  resolution) both pass.
- (iii) **Modes**: `test_jitte_mode0_pumps_equipped` (equipped creature +2/+2, proves
  `EffectFilter::AttachedCreature` resolves in an activated-modal context — the
  highest-risk clause per the plan), `test_jitte_mode1_shrinks_target` (targeted mode
  among untargeted siblings), `test_jitte_mode2_gains_life` (untargeted mode with an
  EMPTY `mode_targets` slice) all pass. `test_jitte_counter_accumulation_roundtrip`
  proves accumulation + spend round-trip (4 counters, spend 2, 2 remain).

**Conclusion: umezawas_jitte flips to `Complete`.** No new seed filed (no contingency failed).

## Final verification (2026-07-19)

- `cargo test --all`: all 29 test groups green (0 failures).
- `cargo clippy --workspace --tests -- -D warnings`: clean.
- `cargo build --workspace`: clean.
- `cargo fmt --check`: clean.
- `tools/check-defs-fmt.sh`: clean (1803 defs).
- No remaining TODO in `umezawas_jitte.rs`; `hidden_strings.rs` retains its one
  genuine remaining TODO (tap/untap player choice), consistent with staying
  `known_wrong`.

Ready for the review phase.
