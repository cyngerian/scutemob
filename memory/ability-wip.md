# Ability WIP: Vanishing

ability: Vanishing
cr: 702.63
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 4 (2 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-vanishing.md
plan_file: memory/abilities/ability-plan-vanishing.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1003 (Vanishing(u32)), card_definition.rs:462 (AbilityDefinition::Vanishing), hash.rs (disc 112 KW, 41 AbilDef, 37-38 SOK)
- [x] 2. Rule enforcement — resolution.rs:430 (ETB counter placement), lands.rs:115 (ETB hook for lands)
- [x] 3. Trigger wiring — stubs.rs (VanishingCounter/VanishingSacrifice PendingTriggerKind), stack.rs (VanishingCounterTrigger/VanishingSacrificeTrigger SOK 37/38), turn_actions.rs:102 (upkeep queueing), abilities.rs (flush), resolution.rs (resolution handlers), stack_view.rs (TUI arms)
- [x] 4. Unit tests — crates/engine/tests/vanishing.rs (8 tests, all passing)
- [x] 5. Card definition — crates/engine/src/cards/defs/aven_riftwatcher.rs
- [x] 6. Game script — test-data/generated-scripts/stack/149_vanishing_aven_riftwatcher.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Vanishing: validated)
