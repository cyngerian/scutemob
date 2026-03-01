# Ability WIP: Enlist

ability: Enlist
cr: 702.154
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-enlist.md

## Step Checklist
- [x] 1. Enum variant — crates/engine/src/state/types.rs:KeywordAbility::Enlist; state/hash.rs discriminant 86; state/stack.rs:StackObjectKind::EnlistTrigger{source_object,enlisted_creature}; hash.rs discriminant 25; tui/stack_view.rs + replay-viewer/view_model.rs arms added
- [x] 2. Rule enforcement — rules/command.rs:DeclareAttackers.enlist_choices; rules/combat.rs:handle_declare_attackers 10-check validation + tap loop + enlist_pairings storage; state/combat.rs:CombatState.enlist_pairings; state/stubs.rs:PendingTrigger.is_enlist_trigger+enlist_enlisted_creature; testing/script_schema.rs:EnlistDeclaration + PlayerAction.enlist; testing/replay_harness.rs enlist resolution; all ~170 test DeclareAttackers struct literals updated
- [x] 3. Trigger wiring — state/builder.rs:Enlist placeholder TriggeredAbilityDef(SelfAttacks); rules/abilities.rs:AttackersDeclared post-processing (tag/remove) + flush_pending_triggers EnlistTrigger arm; rules/resolution.rs:EnlistTrigger resolution (+X/+0 ContinuousEffect UntilEndOfTurn) + counter arm; enlist fields added to all 14 PendingTrigger literals across abilities.rs, effects/mod.rs, miracle.rs, turn_actions.rs
- [x] 4. Unit tests — crates/engine/tests/enlist.rs: 8 tests all passing (basic power addition, no-choice no-trigger, attacker-cannot-be-enlisted, summoning-sickness-rejected, haste-bypasses-sickness, cannot-enlist-self, duplicate-enlisted-rejected, 4-player multiplayer)
- [x] 5. Card definition — crates/engine/src/cards/defs/coalition_skyknight.rs (Coalition Skyknight, {3}{W}, 2/2, Flying + Enlist)
- [x] 6. Game script — test-data/generated-scripts/combat/124_coalition_skyknight_enlist_trigger.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 17/88, 109 total validated, CR corrected to 702.154)

## Review
findings: 4 LOW (no HIGH/MEDIUM)
verdict: clean
review_file: memory/abilities/ability-review-enlist.md
