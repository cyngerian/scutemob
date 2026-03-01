# Ability WIP: Partner With

ability: Partner With
cr: 702.124
priority: P3
started: 2026-02-28
phase: closed
plan_file: memory/abilities/ability-plan-partner-with.md

## Step Checklist
- [x] 1. Enum variant (types.rs:199, hash.rs:451, stack.rs:374, stubs.rs:217, hash.rs:PendingTrigger, card_definition.rs:688, effects/mod.rs:2679, view_model.rs:674)
- [x] 2. Rule enforcement (commander.rs:459 validate_partner_commanders extended; resolution.rs:1559 PartnerWithTrigger arm)
- [x] 3. Trigger wiring (abilities.rs:~985 ETB generation; abilities.rs:~2091 flush branch)
- [x] 4. Unit tests (crates/engine/tests/partner_with.rs — 10 tests)
- [x] 5. Card definition — pir_imaginative_rascal.rs + toothy_imaginary_friend.rs
- [x] 6. Game script — test-data/generated-scripts/baseline/107_pir_partner_with_toothy.json
- [x] 7. Coverage doc update

## Review
findings: 1 HIGH, 1 MEDIUM, 3 LOW — HIGH+MEDIUM fixed; 3 LOWs deferred
review_file: memory/abilities/ability-review-partner-with.md
