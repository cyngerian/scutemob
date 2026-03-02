# Ability WIP: Encore

ability: Encore
cr: 702.141
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-encore.md

## Step Checklist
- [x] 1. Enum variant — types.rs:KeywordAbility::Encore(94), card_definition.rs:AbilityDefinition::Encore{cost}, stack.rs:EncoreAbility(29)/EncoreSacrificeTrigger(30), game_object.rs:encore_sacrifice_at_end_step/encore_must_attack, stubs.rs:PendingTrigger::is_encore_sacrifice_trigger/encore_activator, hash.rs all entries
- [x] 2. Rule enforcement — abilities.rs:handle_encore_card + get_encore_cost, resolution.rs:EncoreAbility+EncoreSacrificeTrigger arms (per-opponent token creation with haste + sacrifice check)
- [x] 3. Trigger wiring — turn_actions.rs:end_step_actions encore EOC queue, abilities.rs:flush_pending_triggers Encore arm
- [x] 4. Unit tests — crates/engine/tests/encore.rs (10 tests: 4p basic, haste, exile-as-cost, sacrifice, sorcery-speed x2, not-in-graveyard, no-keyword, 2p, eliminated-opponent)
- [x] 5. Card definition — Briarblade Adept (crates/engine/src/cards/defs/briarblade_adept.rs)
- [x] 6. Game script — 131_encore_briarblade_adept.json (stack/)
- [x] 7. Coverage doc update — Encore row → validated (P4: 25/88)

## Review
findings: 6 (1 MEDIUM, 5 LOW)
review_file: memory/abilities/ability-review-encore.md
fix_applied: MEDIUM Finding 1 — added encore_activated_by: Option<PlayerId> to GameObject; set during EncoreAbility resolution; read in end_step encore sacrifice trigger queue (turn_actions.rs). LOW findings 1-5 deferred.

## Discriminants (confirmed after Embalm+Eternalize)
- KeywordAbility::Encore = 94
- AbilityDefinition::Encore = 27
- StackObjectKind::EncoreAbility = 29
- StackObjectKind::EncoreSacrificeTrigger = 30
