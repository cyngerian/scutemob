# Ability WIP: Eternalize

ability: Eternalize
cr: 702.129
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-eternalize.md

## Step Checklist
- [x] 1. Enum variant — `state/types.rs:802` (KeywordAbility::Eternalize), `cards/card_definition.rs:288` (AbilityDefinition::Eternalize), `state/stack.rs:645` (StackObjectKind::EternalizeAbility), `rules/command.rs:454` (Command::EternalizeCard), `state/hash.rs` (discs 93/26/28)
- [x] 2. Rule enforcement — `rules/abilities.rs` (handle_eternalize_card, get_eternalize_cost), `rules/engine.rs` (dispatch arm), `rules/resolution.rs` (EternalizeAbility arm + counter arm), `testing/replay_harness.rs` (eternalize_card action), `tools/tui/stack_view.rs` (match arm), `tools/replay-viewer/view_model.rs` (match arm + keyword)
- [x] 3. Trigger wiring — n/a (not trigger-based; card exiled as cost at activation time)
- [x] 4. Unit tests — `crates/engine/tests/eternalize.rs` (12 tests, all passing)
- [x] 5. Card definition — Proven Combatant (crates/engine/src/cards/defs/proven_combatant.rs)
- [x] 6. Game script — 130_eternalize_proven_combatant.json (stack/)
- [x] 7. Coverage doc update — Eternalize row → validated (P4: 24/88)

## Review
findings: 7 (1 MEDIUM, 6 LOW)
review_file: memory/abilities/ability-review-eternalize.md
fix_applied: LOW #2 (renamed test fn), LOW #3 (fixed mana comment); MEDIUM deferred (TODO in place); LOW #4, #5 deferred; LOW #6, #7 no-action

## Discriminants (confirmed after Embalm)
- KeywordAbility::Eternalize = 93
- AbilityDefinition::Eternalize = 26
- StackObjectKind::EternalizeAbility = 28
