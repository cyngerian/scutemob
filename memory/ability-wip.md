# Ability WIP: Adapt

ability: Adapt
cr: 701.46
priority: P3
started: 2026-02-28
phase: closed
plan_file: memory/abilities/ability-plan-adapt.md

## Step Checklist
- [x] 1. Enum variant (types.rs:559, hash.rs:441, view_model.rs:672)
- [x] 2. Rule enforcement (card_definition.rs:785, effects/mod.rs:2731, hash.rs:2428)
- [x] 3. Trigger wiring (n/a — uses existing activated ability pipeline)
- [x] 4. Unit tests (crates/engine/tests/adapt.rs — 6 tests)
- [x] 5. Card definition — crates/engine/src/cards/defs/sharktocrab.rs
- [x] 6. Game script — test-data/generated-scripts/baseline/105_sharktocrab_adapt.json
- [x] 7. Coverage doc update

## Review
findings: 0 HIGH/MEDIUM, 2 LOW — clean (no fixes needed)
review_file: memory/abilities/ability-review-adapt.md
