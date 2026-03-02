# Ability WIP: Blitz

ability: Blitz
cr: 702.152
priority: P4
started: 2026-03-02
phase: closed
plan_file: memory/abilities/ability-plan-blitz.md

## Step Checklist
- [x] 1. Enum variant — types.rs:848 (KeywordAbility::Blitz), types.rs:109 (AltCostKind::Blitz), card_definition.rs:342 (AbilityDefinition::Blitz), stack.rs:165 (was_blitzed), stack.rs:727 (BlitzSacrificeTrigger), stubs.rs:71 (PendingTriggerKind::BlitzSacrifice), hash.rs updated (disc 96/29/32)
- [x] 2. Rule enforcement — casting.rs: cast_with_blitz, casting_with_blitz block, get_blitz_cost helper, blitz cost branch; resolution.rs: cast_alt_cost chain, haste+draw trigger at ETB; turn_actions.rs: end-step sacrifice queuing; abilities.rs: flush_pending_triggers BlitzSacrifice arm; replay_harness.rs: cast_spell_blitz action
- [x] 3. Trigger wiring — resolution.rs:BlitzSacrificeTrigger resolution arm (sacrifice with replacement), counter arm updated; abilities.rs: BlitzSacrifice flush arm
- [x] 4. Unit tests — crates/engine/tests/blitz.rs (9 tests: basic cast, normal cast negative, sacrifice at end step, draw on death, draw+sacrifice combined, creature left before end step, rejected non-blitz card, alt cost exclusivity, commander tax)
- [x] 5. Card definition — Riveteers Requisitioner (crates/engine/src/cards/defs/riveteers_requisitioner.rs)
- [x] 6. Game script — 133_blitz_riveteers_requisitioner.json (stack/)
- [x] 7. Coverage doc update — Blitz → validated (P4: 27/88, total validated: 119)

## Review
findings: 3 (1 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-blitz.md

## Fix Notes
- MEDIUM Finding 1 fixed: test_blitz_draw_card_on_death rewritten to kill creature via SBA
  (CR 704.5g lethal damage path). goblin_in_hand helper now sets power=Some(2)/toughness=Some(2)
  so the layer system can see toughness for SBA checks. Test now verifies CreatureDied event,
  AbilityTriggered for SelfDies draw, and hand count +1.
- LOW Finding 2 (is_creature check in BlitzSacrificeTrigger): deferred per instructions.
- LOW Finding 3 (Mezzio Mugger ruling citation): citation added to test_blitz_draw_card_on_death
  doc comment as part of the rewrite.
