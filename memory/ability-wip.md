# Ability WIP: Improvise

ability: Improvise
cr: 702.126
priority: P3
started: 2026-02-26
phase: closed
plan_file: memory/ability-plan-improvise.md

## Step Checklist
- [x] 1. Enum variant — state/types.rs:333, state/hash.rs:378 (discriminant 44), rules/command.rs:72
- [x] 2. Rule enforcement — rules/casting.rs:360-378 (reduction), casting.rs:806-905 (apply_improvise_reduction), engine.rs:70-91, testing/replay_harness.rs, testing/script_schema.rs
- [x] 3. Trigger wiring — N/A (Improvise is a static ability, not a trigger)
- [x] 4. Unit tests — crates/engine/tests/improvise.rs (12 tests, all passing)
- [x] 5. Card definition
- [x] 6. Game script — test-data/generated-scripts/stack/079_reverse_engineer_improvise.json
- [x] 7. Coverage doc update
