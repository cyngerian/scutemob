# Ability WIP: Impending

ability: Impending
cr: 702.176
priority: P4
started: 2026-03-02
phase: closed
plan_file: memory/abilities/ability-plan-impending.md

## Step Checklist
- [x] 1. Enum variant — types.rs:AltCostKind::Impending+KeywordAbility::Impending; card_definition.rs:AbilityDefinition::Impending; stack.rs:was_impended+ImpendingCounterTrigger; stubs.rs:ImpendingCounter; hash.rs (all); TUI stack_view.rs; replay-viewer view_model.rs
- [x] 2. Rule enforcement — casting.rs:casting_with_impending validation+mutual-exclusion+cost-selection+helpers; resolution.rs:cast_alt_cost transfer+ETB time counters; layers.rs:Layer4 type-removal
- [x] 3. Trigger wiring — turn_actions.rs:end-step impending scanning; abilities.rs:ImpendingCounter flush; resolution.rs:ImpendingCounterTrigger resolution arm
- [x] 4. Unit tests — crates/engine/tests/impending.rs:11 tests (1393 total passing)
- [x] 5. Card definition — overlord_of_the_hauntwoods.rs
- [x] 6. Game script — 136_impending_overlord_of_the_hauntwoods.json
- [x] 7. Coverage doc update — P4 29→30 validated

## Review
findings: 4 LOW
review_file: memory/abilities/ability-review-impending.md
verdict: clean
