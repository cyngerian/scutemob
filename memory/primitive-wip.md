# Primitive WIP: PB-J — Copy/Redirect Spells

batch: PB-J
title: Copy/redirect spells — spell copy + target changing on stack
cards_affected: 4
started: 2026-04-09
phase: closed
plan_file: memory/primitives/pb-plan-J.md

## Deferred from Prior PBs
- none directly; existing copy_spell_on_stack in copy.rs (Storm/Cascade) is foundation

## Known Cards with Copy/Redirect TODOs
Cards with TODOs referencing these gaps:
- Bolt Bend: DONE — Effect::ChangeTargets { must_change: true } + TargetSpellOrAbilityWithSingleTarget
- Deflecting Swat: DONE — AltCastAbility(CommanderFreeCast) + Effect::ChangeTargets { must_change: false }
- Complete the Circuit: PARTIAL — GrantFlash correct; delayed copy trigger deferred (TODO updated)
- Untimely Malfunction: DONE — Mode 1 fixed with ChangeTargets { must_change: true }, target index 1

## Step Checklist
- [x] 1. Engine changes (new Effect variants, dispatch logic)
  - Added TargetRequirement::TargetSpellOrAbilityWithSingleTarget (card_definition.rs:2233+)
  - Added Effect::CopySpellOnStack { target, count } (card_definition.rs ~L2041)
  - Added Effect::ChangeTargets { target, must_change } (card_definition.rs ~L2055)
  - Added GameEvent::TargetsChanged { stack_object_id, old_targets, new_targets } (events.rs L1238+)
  - Updated hash.rs: TargetRequirement disc 16, Effect discs 82+83, GameEvent disc 126
  - Added validate_object_satisfies_requirement arm for TargetSpellOrAbilityWithSingleTarget (casting.rs ~L5418)
  - Added match arm in abilities.rs exhaustive TargetRequirement match (L6414+)
  - Dispatched Effect::CopySpellOnStack and Effect::ChangeTargets in effects/mod.rs (~L5057)
- [x] 2. Card definition fixes
  - bolt_bend.rs: ChangeTargets must_change: true + TargetSpellOrAbilityWithSingleTarget
  - deflecting_swat.rs: AltCastAbility(CommanderFreeCast) + ChangeTargets must_change: false
  - untimely_malfunction.rs: Mode 1 fixed (ChangeTargets + target index 1), mode 2 corrected to index 2
  - complete_the_circuit.rs: TODO comment updated to reference PB-J partial completion
- [x] 3. New card definitions (if any) — none needed
- [x] 4. Unit tests
  - crates/engine/tests/copy_redirect.rs: 8 tests all passing
  - test_copy_spell_on_stack_basic, test_copy_spell_on_stack_twice
  - test_change_targets_must_change_redirects_to_new_player
  - test_change_targets_no_alternative_leaves_unchanged
  - test_change_targets_may_choose_new_leaves_unchanged
  - test_change_targets_accepts_single_target_spell
  - test_change_targets_object_redirect
  - test_bolt_bend_redirects_single_target_spell
- [x] 5. Workspace build verification
  - cargo test --all: all pass, 0 failures
  - cargo clippy -- -D warnings: 0 warnings
  - cargo build --workspace: clean
  - cargo fmt --check: clean

## Fix Phase (review findings applied 2026-04-09)
- [x] MEDIUM-1: Added `has_lost` guard before controller-prefer in ChangeTargets (effects/mod.rs ~L5129)
- [x] MEDIUM-2: Added self-targeting prevention for TargetSpellOrAbilityWithSingleTarget (casting.rs ~L5421)
  - Refactored `validate_targets` → `validate_targets_inner` + `validate_targets_with_source` wrapper
  - Added `self_id: Option<ObjectId>` param to `validate_object_satisfies_requirement`
  - `handle_cast_spell` now calls `validate_targets_with_source` passing `card` as source
  - Added unit test `test_target_spell_single_target_self_targeting_prevented` in casting.rs
- [x] LOW-3: Added KNOWN LIMITATION comment at object redirect site (effects/mod.rs ~L5161)
- [x] LOW-4: Added variable target count TODO to untimely_malfunction.rs mode 2 (line 52)
- All tests pass; 0 clippy warnings; workspace build clean
