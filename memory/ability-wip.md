# Ability WIP: Devour

ability: Devour
cr: 702.82
priority: P4
started: 2026-03-06
phase: close

## Review
findings: 4 (0 HIGH, 0 MEDIUM, 4 LOW)
review_file: memory/abilities/ability-review-devour.md
plan_file: memory/abilities/ability-plan-devour.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1134, hash.rs:619, view_model.rs:831
- [x] 2. Rule enforcement — command.rs:247, casting.rs:77+2740, engine.rs:103, stack.rs:258, game_object.rs:553, resolution.rs:840-990, lands.rs:356, replay_harness.rs:1559
- [x] 3. Trigger wiring — n/a (Devour is a replacement effect, CR 614.1c)
- [x] 4. Unit tests — crates/engine/tests/devour.rs (10 tests, all passing)
- [x] 5. Card definition — crates/engine/src/cards/defs/predator_dragon.rs
- [x] 6. Game script — test-data/generated-scripts/stack/162_devour_predator_dragon.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Devour: validated)
