# Ability WIP: Saddle

ability: Saddle
cr: 702.171
priority: P4
started: 2026-03-07
phase: closed
plan_file: memory/abilities/ability-plan-saddle.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1304, hash.rs:669, game_object.rs:657, mod.rs:386+553, builder.rs:1001, effects/mod.rs:2922, turn_actions.rs:1296, resolution.rs (6 token sites), view_model.rs:876
- [x] 2. Rule enforcement — command.rs:549, engine.rs:380, abilities.rs:6295, stack.rs:1192, hash.rs:2024, resolution.rs:4056+6308, view_model.rs:592, stack_view.rs:201
- [x] 3. Trigger wiring — replay_harness.rs (saddle_mount action)
- [x] 4. Unit tests — crates/engine/tests/saddle.rs (14 tests, all pass)
- [x] 5. Card definition
- [x] 6. Game script
- [x] 7. Coverage doc update

## Review
findings: 1 LOW
verdict: clean
review_file: memory/abilities/ability-review-saddle.md
