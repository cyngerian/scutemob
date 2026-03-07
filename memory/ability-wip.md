# Ability WIP: Bloodrush

ability: Bloodrush
cr: ability word (no CR section — activated ability: discard this card to pump attacking creature)
priority: P4
started: 2026-03-07
phase: closed

## Review
findings: 1 MEDIUM (FIXED) + 2 LOW (deferred)
review_file: memory/abilities/ability-review-bloodrush.md
plan_file: memory/abilities/ability-plan-bloodrush.md

## Step Checklist
- [x] 1. Enum variant — n/a: ability word (CR 207.2c), no KeywordAbility variant. AbilityDefinition::Bloodrush disc 52 in card_definition.rs:577; StackObjectKind::BloodrushAbility disc 51 in stack.rs:1096; Command::ActivateBloodrush in command.rs:453; hash.rs updated
- [x] 2. Rule enforcement — handle_activate_bloodrush in abilities.rs:970; resolution arm in resolution.rs; engine dispatch in engine.rs; view_model.rs + stack_view.rs updated; replay_harness.rs: activate_bloodrush action
- [x] 3. Trigger wiring — n/a: no triggers (pure activated ability)
- [x] 4. Unit tests — 8 tests in crates/engine/tests/bloodrush.rs: basic_pump, grants_keyword, target_must_be_attacking, no_combat_fails, card_discarded_as_cost, insufficient_mana_fails, not_in_hand_fails, pump_expires_end_of_turn
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update
