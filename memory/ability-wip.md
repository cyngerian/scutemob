# Ability WIP: Assist

ability: Assist
cr: 702.132
priority: P4
started: 2026-03-05
phase: closed
plan_file: memory/abilities/ability-plan-assist.md

## Step Checklist
- [x] 1. Enum variant — state/types.rs:939, state/hash.rs:548, tools/replay-viewer/src/view_model.rs:769
- [x] 2. Rule enforcement — rules/command.rs:198, rules/casting.rs:2172, rules/engine.rs:97
- [x] 3. Trigger wiring — n/a (assist is static, no triggers)
- [x] 4. Unit tests — crates/engine/tests/assist.rs (11 tests)
- [x] 5. Card definition — huddle_up.rs (two target players each draw 1)
- [x] 6. Game script — 142_assist_huddle_up.json
- [x] 7. Coverage doc update — P4 35->36 validated

## Review
findings: 2 LOW
review_file: memory/abilities/ability-review-assist.md
verdict: clean
