# Primitive WIP: PB-AC0 — ETBTriggerFilter subtype/nontoken fields (creature-ETB filter forwarding)

batch: PB-AC0
title: ETBTriggerFilter carries + honors creature-subtype and token/nontoken constraints on the WheneverCreatureEntersBattlefield trigger path
started: 2026-05-18
phase: fix-complete (DONE)
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
1. `crates/engine/src/testing/replay_harness.rs` ~L2405: `triggering_creature_filter: filter.clone()` (was `None`). Forwards full TargetFilter from WheneverCreatureEntersBattlefield carddef via triggering_creature_filter on the creature-ETB harness conversion path. CR 603.2/205.3/111.1 comment added.
2. `crates/engine/src/rules/abilities.rs` inside `etb_filter` block (after `card_type_filter` check): new block checks `triggering_creature_filter` — explicit `is_token`/`is_nontoken` guards, then `matches_filter` for subtype and other characteristic filters. Scoped inside etb_filter block to avoid double-consuming on death/attack defs.

### Card defs completed
- `ganax_astral_hunter.rs`: re-authored, ENGINE-BLOCKED TODO removed. Dragon-ETB Treasure trigger live. `exclude_self: false`, `has_subtype: Dragon`, `controller: You`.
- `lathliss_dragon_queen.rs`: re-authored, ENGINE-BLOCKED TODO removed. Nontoken-Dragon-ETB 5/5 Dragon token trigger live. `exclude_self: true`, `is_nontoken: true`, `has_subtype: Dragon`. Token spec: 5/5 Red Dragon Flying, `count: Fixed(1)`.
- `the_great_henge.rs`: header nontoken TODO removed (remains: cost-reduction TODO). Added `is_nontoken: true` to filter. Fixed `EffectTarget::Source` → `TriggeringCreature` for +1/+1 counter (oracle unambiguous).
- `miirym_sentinel_wyrm.rs`: added `is_nontoken: true` to TargetFilter. Stale TODO comments removed. `exclude_self: true` already correct.
- `dragons_hoard.rs`: verified no edit needed (Dragon-ETB gold trigger, no nontoken/token restriction in oracle text).
- `bloomvine_regent.rs`: verified no edit needed (creature-ETB trigger, no subtype restriction).

### Tests completed
- New file: `crates/engine/tests/etb_trigger_subtype_filter.rs`, 11 tests all passing.

### Gate results
- cargo test --all: 2871 passed (was 2860; +11 new)
- cargo clippy --all-targets -- -D warnings: 0 warnings
- cargo build --workspace: clean
- cargo fmt --check: clean
- HASH_SCHEMA_VERSION: unchanged at 27 (no new fields)

### Deviations from plan
- test_etb_exclude_self_with_subtype: rewritten to place Lathliss on battlefield via ObjectSpec::with_triggered_ability() instead of casting from CardDefinition.
- test_etb_great_henge_counter_on_entering_creature: added `.in_zone(ZoneId::Battlefield)` to Henge ObjectSpec.

## Reviewer checklist (primitive-impl-reviewer, 2026-05-18)
- [x] CR rules verified independently via mtg-rules MCP — 603.2, 603.10/603.10a, 111.1, 205.3 confirmed; ETB correctly NOT on the 603.10 look-back list
- [x] Oracle text re-verified via MCP lookup_card for all 6 cards — ganax, lathliss, the_great_henge, miirym, dragons_hoard, bloomvine_regent all match defs exactly
- [x] Engine Change 1 (harness forwarding) — type-correct, CR-cited, mirrors death conversion
- [x] Engine Change 2 (abilities.rs matching) — correctly placed inside etb_filter block, reuses layer-resolved entering_chars (CR 613.1d), explicit is_token/is_nontoken guards mirror death path
- [x] Scoping verified — death path is a separate function; attack/combat-damage defs have etb_filter: None; no double-consumption of triggering_creature_filter
- [x] EffectTarget::Source→TriggeringCreature (Great Henge) verified correct end-to-end: entering_object_id → stack_obj.triggering_creature_id → ctx.triggering_creature_id → EffectTarget::TriggeringCreature
- [x] matches_filter confirmed to honor has_subtype; controller field correctly handled by etb_filter.controller_you (not matches_filter)
- [x] Hash impact confirmed — no schema change, HASH_SCHEMA_VERSION 27 correctly unchanged
- [x] Test discrimination analyzed — FINDING T1: Change 1 has zero discrimination; all 11 tests bypass enrich_spec_from_def via with_triggered_ability
- [x] test_etb_death_path_unaffected confirmed to exercise the death-path scoping (death def with etb_filter: None)
- [x] Review written: memory/primitives/pb-review-AC0.md

## Reviewer verdict: NEEDS-FIX (resolved in fix phase)

- Engine logic: CORRECT (1 LOW comment nit, non-blocking).
- Card defs: all 6 CORRECT against oracle text, no remaining in-scope TODOs.
- Tests: FINDING T1 (HIGH, fix-phase per conventions "test-validity MEDIUMs are
  fix-phase HIGHs") — Change 1 (the `replay_harness.rs` `triggering_creature_filter`
  forwarding) is provably untested: every test wires the watcher via
  `with_triggered_ability(<runtime def>)`, so `enrich_spec_from_def`'s
  `WheneverCreatureEntersBattlefield` conversion arm (the Change 1 site) is never
  exercised. Reverting Change 1 leaves all 11 tests green. The three "integration"
  tests do not register/use the actual re-authored card defs.

## Fix phase (2026-05-18)

### T1 (HIGH) — Fixed

Added 2 new tests (tests 12 + 13) to `crates/engine/tests/etb_trigger_subtype_filter.rs`:

- `test_etb_ganax_carddef_integration_via_enrich` — watcher is Ganax, Astral Hunter built
  via `enrich_spec_from_def(ObjectSpec::card(...).in_zone(Battlefield), &defs)` with NO
  `with_triggered_ability` call. Exercises the WheneverCreatureEntersBattlefield conversion
  arm in enrich_spec_from_def (Change 1). Fire-on-match: Dragon enters → 1 Treasure.
  No-fire-on-mismatch: Goblin enters → 0 Treasures.

- `test_etb_lathliss_carddef_integration_via_enrich` — watcher is Lathliss, Dragon Queen
  via enrich_spec_from_def. Fire-on-match: nontoken Dragon enters → Dragon token created.
  No-fire-on-mismatch: token Dragon enters → no trigger (is_nontoken filter honored).

Discrimination check CONFIRMED: with `triggering_creature_filter: None` (Change 1 reverted),
both new tests FAIL (StackNotEmpty error — over-triggered ETB from Ganax/Lathliss stays on
stack). All 11 original tests remain GREEN with Change 1 reverted, confirming they exercise
only Change 2. Change 1 was unexercised before this fix.

With Change 1 restored: all 13 tests pass.

### E1 (LOW) — Fixed

Comment at abilities.rs:6193-6194 tightened from "not double-evaluated here" to clarify
that the death path is in a *separate function* (`apply_zone_change_triggers` ~L4287) and
attack/combat-damage defs have `etb_filter: None` so they never enter this block.

### Fix phase gate results

- cargo test --all: 2873 passed (was 2871 post-implement; +2 new tests)
- cargo clippy --all-targets -- -D warnings: 0 warnings
- cargo build --workspace: clean
- cargo fmt --all -- --check: clean

## Planner notes (handoff to implement phase)
- Shape: approach (b). NO struct/enum change -> NO HASH bump.
- the_great_henge carries a 2nd latent bug: EffectTarget::Source -> TriggeringCreature for the
  +1/+1 counter; corrected in-scope. Reviewer confirmed correct.
- Highest-risk item: scoping the new triggering_creature_filter check to ETB defs only —
  reviewer confirmed correctly scoped (inside etb_filter block; death path separate function).
