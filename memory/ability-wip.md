# Ability WIP: Melee

ability: Melee
cr: 702.121
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-melee.md

## Step Checklist
- [x] 1. Enum variant — types.rs:700 + hash.rs:494 (disc 83) + view_model.rs:706
- [x] 2. Rule enforcement — stack.rs:514 (MeleeTrigger) + stubs.rs:299 (is_melee_trigger) + builder.rs:512 (auto-generate SelfAttacks trigger) + hash.rs:1462 (disc 23) + tui/stack_view.rs:94 + view_model.rs:485
- [x] 3. Trigger wiring — abilities.rs:1477 (tag melee triggers) + abilities.rs:2732 (flush MeleeTrigger) + resolution.rs:1997 (counter catch-all) + resolution.rs:1886 (MeleeTrigger resolution)
- [x] 4. Unit tests — crates/engine/tests/melee.rs (7 tests, all passing)
- [x] 5. Card definition — crates/engine/src/cards/defs/wings_of_the_guard.rs
- [x] 6. Game script — test-data/generated-scripts/combat/121_wings_of_the_guard_melee_4_player.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 14/88, 106 total validated, CR corrected to 702.121)

## Review
findings: 3 LOW (no HIGH/MEDIUM)
verdict: clean
review_file: memory/abilities/ability-review-melee.md
