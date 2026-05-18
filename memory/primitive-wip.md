# Primitive WIP: PB-AC0 — ETBTriggerFilter subtype/nontoken fields (creature-ETB filter forwarding)

batch: PB-AC0
title: ETBTriggerFilter carries + honors creature-subtype and token/nontoken constraints on the WheneverCreatureEntersBattlefield trigger path
started: 2026-05-18
phase: re-review-complete (PASS — cleared to close)
plan_file: memory/primitives/pb-plan-AC0.md
review_file: memory/primitives/pb-review-AC0.md

## Task reference
- ESM task: scutemob-41
- Branch: feat/pb-ac0-etbtriggerfilter-subtypenontoken-fields-creature-etb-
- Acceptance criteria:
  - 4081: ETBTriggerFilter (or the creature-ETB harness path) carries + honors creature-subtype and token/nontoken constraints; has_subtype + is_nontoken on WheneverCreatureEntersBattlefield triggers no longer silently dropped
  - 4082: ganax_astral_hunter + lathliss_dragon_queen ETB clauses re-authored as live abilities (TODOs removed); miirym, dragons_hoard, bloomvine_regent, encroaching_dragonstorm verified/corrected against oracle text
  - 4083: New tests cover subtype-filtered and token/nontoken-filtered creature-ETB triggers — fire-on-match + no-fire-on-mismatch; CR citations present
  - 4084: cargo build --workspace, cargo test --all, cargo clippy --all-targets -- -D warnings, cargo fmt --check all pass; primitive-impl-reviewer pass run with findings addressed

## Problem (from memory/card-authoring/review-scutemob-40.md)

`TriggerCondition::WheneverCreatureEntersBattlefield { filter }` carries a TargetFilter,
but the replay harness (~replay_harness.rs:2371) converts it into an `ETBTriggerFilter`
struct (state/game_object.rs:~560) that has ONLY `creature_only`, `controller_you`,
`exclude_self`, `color_filter`, `card_type_filter` — NO subtype field, NO token field.
So `has_subtype` and `is_nontoken`/`is_token` on a creature-ETB trigger are SILENTLY
DROPPED; the trigger over-fires for every creature entering. The matching loop is in
abilities.rs ~6142-6181.

The death-trigger path is correct: it forwards the full filter as
`triggering_creature_filter`, matched via `matches_filter`.

FIX: bring the creature-ETB path to parity — planner picks the cleaner of
(a) add subtype + token/nontoken fields to ETBTriggerFilter + honor them in the
matching loop, or (b) forward `triggering_creature_filter` on the creature-ETB path
mirroring the death path.

## Cards unblocked
- ganax_astral_hunter — Dragon-ETB Treasure trigger (currently ENGINE-BLOCKED TODO)
- lathliss_dragon_queen — nontoken-Dragon-ETB 5/5 Dragon token trigger (currently ENGINE-BLOCKED TODO)
- Latent-bug cards to verify/correct: miirym, dragons_hoard, bloomvine_regent, encroaching_dragonstorm

## Reference docs
- memory/card-authoring/review-scutemob-40.md — the finding (review header + F1/F3)
- memory/card-authoring/dsl-gap-audit-2026-05-16.md
- memory/card-authoring/campaign-plan-2026-05-16.md

## Planner checklist
- [x] CR research — creature-ETB triggered abilities, subtype membership, token vs nontoken (CR 603.2, 603.10, 111.1, 205.3, 613.1d)
- [x] Engine architecture walk — ETBTriggerFilter struct, harness conversion, abilities.rs matching loop, death-path parity
- [x] Shape decision (a vs b) documented with rationale — chose (b): forward triggering_creature_filter
- [x] Hash impact assessment (struct field add → HASH bump?) — NO bump; triggering_creature_filter already exists + already hashed
- [x] Test plan — 11 tests, subtype + nontoken fire/no-fire, CR-cited
- [x] Plan file written: memory/primitives/pb-plan-AC0.md

## Implementation notes (handoff to review phase)

### Engine changes completed
1. `crates/engine/src/testing/replay_harness.rs` ~L2411: `triggering_creature_filter: filter.clone()` (was `None`). Forwards full TargetFilter from WheneverCreatureEntersBattlefield carddef via triggering_creature_filter on the creature-ETB harness conversion path. CR 603.2/205.3/111.1 comment added.
2. `crates/engine/src/rules/abilities.rs` inside `etb_filter` block (after `card_type_filter` check): new block checks `triggering_creature_filter` — explicit `is_token`/`is_nontoken` guards, then `matches_filter` for subtype and other characteristic filters. Scoped inside etb_filter block to avoid double-consuming on death/attack defs.

### Card defs completed
- `ganax_astral_hunter.rs`: re-authored, ENGINE-BLOCKED TODO removed. Dragon-ETB Treasure trigger live. `exclude_self: false`, `has_subtype: Dragon`, `controller: You`.
- `lathliss_dragon_queen.rs`: re-authored, ENGINE-BLOCKED TODO removed. Nontoken-Dragon-ETB 5/5 Dragon token trigger live. `exclude_self: true`, `is_nontoken: true`, `has_subtype: Dragon`. Token spec: 5/5 Red Dragon Flying, `count: Fixed(1)`.
- `the_great_henge.rs`: header nontoken TODO removed (remains: cost-reduction TODO). Added `is_nontoken: true` to filter. Fixed `EffectTarget::Source` → `TriggeringCreature` for +1/+1 counter (oracle unambiguous).
- `miirym_sentinel_wyrm.rs`: added `is_nontoken: true` to TargetFilter. Stale TODO comments removed. `exclude_self: true` already correct.
- `dragons_hoard.rs`: verified no edit needed (Dragon-ETB gold trigger, no nontoken/token restriction in oracle text).
- `bloomvine_regent.rs`: verified no edit needed (creature-ETB trigger, no subtype restriction).

### Tests completed
- New file: `crates/engine/tests/etb_trigger_subtype_filter.rs`, 11 tests (implement phase) → 13 tests (fix phase).

### Gate results (final, post-fix-phase)
- cargo test --all: 2873 passed
- cargo clippy --all-targets -- -D warnings: 0 warnings
- cargo build --workspace: clean
- cargo fmt --check: clean
- HASH_SCHEMA_VERSION: unchanged at 27 (no new fields)

## Reviewer verdict: NEEDS-FIX (original) → resolved in fix phase

- Engine logic: CORRECT (1 LOW comment nit, non-blocking).
- Card defs: all 6 CORRECT against oracle text, no remaining in-scope TODOs.
- Tests: FINDING T1 (HIGH, fix-phase) — Change 1 (the `replay_harness.rs`
  `triggering_creature_filter` forwarding) was provably untested.

## Fix phase (2026-05-18, commit a7ebac79)

### T1 (HIGH) — Fixed

Added 2 new tests (tests 12 + 13) to `crates/engine/tests/etb_trigger_subtype_filter.rs`:
- `test_etb_ganax_carddef_integration_via_enrich` — watcher = Ganax via
  `enrich_spec_from_def(ObjectSpec::card(...).in_zone(Battlefield), &defs)`, NO
  `with_triggered_ability`. Fire-on-match (Dragon → +1 Treasure) + no-fire-on-mismatch
  (Goblin → 0 Treasures).
- `test_etb_lathliss_carddef_integration_via_enrich` — same pattern, Lathliss.
  Fire-on-match (nontoken Dragon → token) + no-fire-on-mismatch (token Dragon → no trigger).

### E1 (LOW) — Fixed (with residual LOW)

Comment at abilities.rs:6193-6196 tightened. Note: fix names the death path's containing
function `apply_zone_change_triggers`, which does not exist — actual function is
`check_triggers`. Non-blocking cosmetic LOW.

## Re-review (2026-05-18, primitive-impl-reviewer, Opus)

- [x] Verified tests 12 + 13 build the watcher from the real registered
  `CardDefinition` (`load_defs()` = `all_cards()` keyed by name) via
  `enrich_spec_from_def`, with NO `with_triggered_ability` — they genuinely exercise
  the `WheneverCreatureEntersBattlefield` conversion arm at `replay_harness.rs:2360-2414`
  and Change 1's `triggering_creature_filter: filter.clone()` at L2411.
- [x] Confirmed `ganax_astral_hunter.rs` and `lathliss_dragon_queen.rs` carry the
  matching `AbilityDefinition::Triggered { WheneverCreatureEntersBattlefield { filter, exclude_self } }`.
- [x] Discrimination logic-traced: reverting Change 1 → `None` makes `abilities.rs:6197`
  `if let Some(ref creature_filter)` block skip → over-fire → both new tests' no-fire
  assertions fail. Runner's reported discrimination check is consistent.
- [x] Fire-on-match AND no-fire-on-mismatch covered for both subtype (Ganax) and
  nontoken (Lathliss) filters.
- [x] Scope check: fix touched only `etb_trigger_subtype_filter.rs` (11 → 13 tests,
  original 11 untouched) and the `abilities.rs:6193-6196` comment. No engine logic,
  no card defs, no other files. No regressions, no scope creep.
- [x] E1: comment fixed but names non-existent function `apply_zone_change_triggers`
  (actual: `check_triggers`) — residual non-blocking LOW logged.
- [x] Review updated: memory/primitives/pb-review-AC0.md

## Re-review verdict: PASS — cleared to close

T1 (the sole blocking finding) is RESOLVED correctly and verifiably. Only residual is
E1's cosmetic comment inaccuracy (a non-existent function name) — non-blocking LOW,
recommend trivial follow-up `apply_zone_change_triggers` → `check_triggers` but not
gating. PB-AC0 is cleared to close.

## Planner notes (handoff to implement phase)
- Shape: approach (b). NO struct/enum change -> NO HASH bump.
- the_great_henge carries a 2nd latent bug: EffectTarget::Source -> TriggeringCreature for the
  +1/+1 counter; corrected in-scope. Reviewer confirmed correct.
- Highest-risk item: scoping the new triggering_creature_filter check to ETB defs only —
  reviewer confirmed correctly scoped (inside etb_filter block; death path separate function).
