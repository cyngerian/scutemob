# Ability WIP: Escalate

ability: Escalate
cr: 702.120
priority: P4
started: 2026-03-06
phase: closed
plan_file: memory/abilities/ability-plan-escalate.md

## Step Checklist
- [x] 1. Enum variant — types.rs:994, card_definition.rs:453, hash.rs:559/3350
- [x] 2. Rule enforcement — command.rs:238, stack.rs:231, casting.rs:76/1882/3222, resolution.rs:216, engine.rs:102/128, hash.rs:1730; replay_harness.rs:cast_spell_escalate, script_schema.rs:escalate_modes
- [x] 3. Trigger wiring — N/A (Escalate is a static/additional-cost ability)
- [x] 4. Unit tests — crates/engine/tests/escalate.rs (8 tests)
- [x] 5. Card definition — crates/engine/src/cards/defs/blessed_alliance.rs
- [x] 6. Game script — test-data/generated-scripts/stack/148_escalate_blessed_alliance.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Escalate: validated)
