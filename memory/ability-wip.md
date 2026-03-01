# Ability WIP: Bushido

ability: Bushido
cr: 702.45
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-bushido.md

## Step Checklist
- [x] 1. Enum variant — types.rs:639, hash.rs:471-475, view_model.rs:692
- [x] 2. Rule enforcement — game_object.rs:199-206 (SelfBecomesBlocked), hash.rs:1158-1159
- [x] 3. Trigger wiring — builder.rs:685-726 (two TriggeredAbilityDefs), abilities.rs:1560-1578 (SelfBecomesBlocked dispatch)
- [x] 4. Unit tests — crates/engine/tests/bushido.rs (7 tests, all passing)
- [x] 5. Card definition — crates/engine/src/cards/defs/devoted_retainer.rs (Devoted Retainer, {W}, 1/1, Bushido 1)
- [x] 6. Game script — test-data/generated-scripts/combat/115_devoted_retainer_bushido_block.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 8/88, 100 total validated)
