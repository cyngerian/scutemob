# Ability WIP: Squad

ability: Squad
cr: 702.157
priority: P4
started: 2026-03-07
phase: closed
plan_file: memory/abilities/ability-plan-squad.md

## Step Checklist
- [x] 1. Enum variant — `crates/engine/src/state/types.rs` KeywordAbility::Squad (disc 137); hash.rs arms for KW/SOK/GO/AbilDef; view_model.rs + stack_view.rs match arms
- [x] 2. Rule enforcement — `crates/engine/src/rules/casting.rs` squad_count validation + cost charging; `command.rs` squad_count field; `stack.rs` squad_count on StackObject + SquadTrigger SOK (disc 52); `game_object.rs` squad_count field; `card_definition.rs` AbilityDefinition::Squad (disc 54)
- [x] 3. Trigger wiring — `crates/engine/src/rules/resolution.rs` ETB trigger placement (Ravenous pattern) + SquadTrigger resolution (Myriad token loop); `rules/abilities.rs` flush_pending_triggers SquadETB->SquadTrigger; `state/stubs.rs` PendingTriggerKind::SquadETB + squad_count on PendingTrigger; `testing/replay_harness.rs` cast_spell_squad action
- [x] 4. Unit tests — `crates/engine/tests/squad.rs` (6 tests: zero_payments, basic_one_payment, multiple_payments, tokens_are_copies, rejected_without_keyword, tokens_not_cast)
- [x] 5. Card definition
- [x] 6. Game script
- [x] 7. Coverage doc update

## Review
findings: 2 LOW
verdict: clean
review_file: memory/abilities/ability-review-squad.md
