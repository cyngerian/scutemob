# Primitive WIP: PB-AC0 — ETBTriggerFilter subtype/nontoken fields (creature-ETB filter forwarding)

batch: PB-AC0
title: ETBTriggerFilter carries + honors creature-subtype and token/nontoken constraints on the WheneverCreatureEntersBattlefield trigger path
started: 2026-05-18
phase: review
plan_file: memory/primitives/pb-plan-AC0.md

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
  - test_etb_subtype_filter_fires_on_match (CR 603.2/205.3)
  - test_etb_subtype_filter_no_fire_on_mismatch (CR 603.2)
  - test_etb_nontoken_filter_fires_on_nontoken (CR 111.1)
  - test_etb_nontoken_filter_no_fire_on_token (CR 111.1)
  - test_etb_subtype_and_nontoken_combined (CR 603.2/205.3/111.1)
  - test_etb_exclude_self_with_subtype (CR 603.2, Gatherer 2024-11-08)
  - test_etb_subtype_filter_layer_resolved (CR 613.1d)
  - test_etb_ganax_treasure_integration (CR 603.2)
  - test_etb_lathliss_token_integration (CR 111.1)
  - test_etb_great_henge_counter_on_entering_creature (CR 603.2)
  - test_etb_death_path_unaffected (CR 603.10a)

### Gate results
- cargo test --all: 2871 passed (was 2860; +11 new)
- cargo clippy --all-targets -- -D warnings: 0 warnings
- cargo build --workspace: clean
- cargo fmt --check: clean
- HASH_SCHEMA_VERSION: unchanged at 27 (no new fields)

### Deviations from plan
- test_etb_exclude_self_with_subtype: rewritten to place Lathliss on battlefield via ObjectSpec::with_triggered_ability() instead of casting from CardDefinition. Root cause: enrich_spec_from_def (which converts WheneverCreatureEntersBattlefield → runtime TriggeredAbilityDef) only runs at build_initial_state time; CastSpell resolution does not re-enrich. Test verifies: trigger fires for another nontoken Dragon; exclude_self semantics correct.
- test_etb_great_henge_counter_on_entering_creature: added `.in_zone(ZoneId::Battlefield)` to Henge ObjectSpec (was defaulting to Hand). Trigger + counter placement works correctly when Henge is on the battlefield.

## Planner notes (handoff to implement phase)
- Shape: approach (b). NO struct/enum change -> NO HASH bump (current HASH_SCHEMA_VERSION is 27, not 26).
- Roster: 6 cards. ganax + lathliss re-authored; the_great_henge forced add (TODO sweep);
  miirym one-field edit (is_nontoken); dragons_hoard + bloomvine_regent verified no-edit.
  encroaching_dragonstorm is NOT unblocked (blocked on Effect::ReturnToHand, different gap).
- the_great_henge carries a 2nd latent bug: EffectTarget::Source -> TriggeringCreature for the
  +1/+1 counter; corrected in-scope (oracle-unambiguous, same file). Flagged for reviewer.
- Highest-risk item: scoping the new triggering_creature_filter check to ETB defs only
  (place inside the etb_filter block in abilities.rs ~L6181) so the death/attack paths,
  which also use that field, are not double-consumed.
