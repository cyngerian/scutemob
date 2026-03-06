# Ability WIP: Recover

ability: Recover
cr: 702.59
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 3 (0 HIGH, 0 MEDIUM, 3 LOW)
review_file: memory/abilities/ability-review-recover.md
plan_file: memory/abilities/ability-plan-recover.md

## Step Checklist
- [x] 1. Enum variant (types.rs:1040, card_definition.rs:499, stack.rs:937, hash.rs:599/3492/1741/2640, mod.rs:134, builder.rs:347)
- [x] 2. Rule enforcement (stubs.rs:Recover, command.rs:PayRecover, events.rs:RecoverPaymentRequired/RecoverPaid/RecoverDeclined, engine.rs:handle_pay_recover, resolution.rs:RecoverTrigger handler)
- [x] 3. Trigger wiring (abilities.rs:2872 CreatureDied arm, find_recover_cost helper at 648, flush_pending_triggers:Recover at 3893)
- [x] 4. Unit tests (crates/engine/tests/recover.rs: 8 tests)
- [x] 5. Card definition — crates/engine/src/cards/defs/grim_harvest.rs
- [x] 6. Game script — test-data/generated-scripts/stack/153_recover_grim_harvest.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Recover: validated)
