# Ability WIP: Jump-Start

ability: Jump-Start
cr: 702.133
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-jump-start.md

## Step Checklist
- [x] 1. Enum variant — types.rs:771, hash.rs:514, view_model.rs:725
- [x] 2. Rule enforcement — casting.rs (detection, validation, discard cost + madness check), resolution.rs (exile-on-departure, 3 sites), effects/mod.rs (exile-on-counter), stack.rs (flag), command.rs (fields), engine.rs (threading)
- [x] 3. Trigger wiring — n/a
- [x] 4. Unit tests — jump_start.rs (12 tests)
- [x] 5. Card definition — radical_idea.rs
- [x] 6. Game script — 127_jump_start_radical_idea.json (validated)
- [x] 7. Coverage doc update — P4 validated 20→21, total 112→113

## Review
findings: 0 HIGH + 1 MEDIUM + 2 LOW
verdict: needs-fix → fixed
review_file: memory/abilities/ability-review-jump-start.md
