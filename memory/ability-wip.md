# Ability WIP: Ingest

ability: Ingest
cr: 702.115
priority: P4
started: 2026-02-28
phase: closed
plan_file: memory/abilities/ability-plan-ingest.md

## Step Checklist
- [x] 1. Enum variant — types.rs:629; hash.rs:466 (discriminant 75); view_model.rs:684
- [x] 2. Rule enforcement — stubs.rs:230 (PendingTrigger fields); hash.rs:1087 (PendingTrigger hash)
- [x] 3. Trigger wiring — stack.rs:407 (IngestTrigger variant); hash.rs:1368 (discriminant 18); tui/stack_view.rs:79; view_model.rs:472; abilities.rs:1748 (dispatch in CombatDamageDealt); abilities.rs:2214 (flush handler); resolution.rs:1632 (resolution arm)
- [x] 4. Unit tests — crates/engine/tests/ingest.rs (5 tests: basic, blocked, empty_library, multiple_instances, multiplayer) — 1204 total passing
- [x] 5. Card definition — crates/engine/src/cards/defs/mist_intruder.rs (Mist Intruder, {1}{U}, 1/2, Devoid+Flying+Ingest)
- [x] 6. Game script — test-data/generated-scripts/baseline/113_mist_intruder_ingest_exile.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 6/88, 98 total validated)

## Review
findings: 1 MEDIUM, 1 LOW — both fixed 2026-03-01
- MEDIUM (Finding 1): Renamed test to `test_702_115a_ingest_two_creatures_each_trigger`; added `test_702_115b_ingest_single_creature_multiple_instances` with a CardDefinition having two Keyword(Ingest) entries — crates/engine/tests/ingest.rs
- LOW (Finding 2): Replaced `unreachable!()` with safe `let CombatDamageTarget::Player(...) = ... else { continue; }` — crates/engine/src/rules/abilities.rs:1768
review_file: memory/abilities/ability-review-ingest.md
