# Ability WIP: Shadow

ability: Shadow
cr: 702.28
priority: P4
started: 2026-02-28
phase: closed
plan_file: memory/abilities/ability-plan-shadow.md

## Step Checklist
- [x] 1. Enum variant — types.rs:571, hash.rs:451, view_model.rs:673, helpers.rs:16
- [x] 2. Rule enforcement — combat.rs:491-502 (bidirectional shadow blocking check)
- [x] 3. Trigger wiring — N/A (Shadow is a static evasion keyword, no triggers)
- [x] 4. Unit tests — crates/engine/tests/shadow.rs (7 tests, all passing)
- [x] 5. Card definition — crates/engine/src/cards/defs/dauthi_slayer.rs
- [x] 6. Game script — test-data/generated-scripts/combat/106_dauthi_slayer_shadow_evasion.json
- [x] 7. Coverage doc update

## Review
findings: 0 HIGH/MEDIUM, 4 LOW — clean (no fixes needed)
review_file: memory/abilities/ability-review-shadow.md
