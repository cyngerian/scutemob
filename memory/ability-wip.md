# Ability WIP: Bloodthirst

ability: Bloodthirst
cr: 702.54
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 2 (0 HIGH, 0 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-bloodthirst.md
plan_file: memory/abilities/ability-plan-bloodthirst.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1127 (KW 123), hash.rs:619, view_model.rs:831, player.rs:132, builder.rs:288, hash.rs:831, turn_actions.rs:1198, effects/mod.rs:180+200, combat.rs:1440+1453
- [x] 2. Rule enforcement — resolution.rs (after Amplify block), lands.rs (after Amplify block)
- [x] 3. Trigger wiring — n/a (static ability, not triggered)
- [x] 4. Unit tests — crates/engine/tests/bloodthirst.rs (8 tests, all pass)
- [x] 5. Card definition — crates/engine/src/cards/defs/stormblood_berserker.rs
- [x] 6. Game script — test-data/generated-scripts/stack/160_bloodthirst_stormblood_berserker.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Bloodthirst: validated)
