# Ability Review: Ward (Re-Review)

**Date**: 2026-02-24
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.21
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Ward(u32) variant, line 128-135)
- `crates/engine/src/state/hash.rs` (hash entries for Ward L287-290, TriggerEvent L876, PendingTrigger.targeting_stack_id L845-846, GameEvent::PermanentTargeted L1571-1581, TriggerCondition L1834)
- `crates/engine/src/state/game_object.rs` (TriggerEvent::SelfBecomesTargetByOpponent, L114-117)
- `crates/engine/src/state/stubs.rs` (PendingTrigger.targeting_stack_id, L49-56)
- `crates/engine/src/state/builder.rs` (Ward keyword -> TriggeredAbilityDef translation, L336-372)
- `crates/engine/src/cards/card_definition.rs` (TriggerCondition::WhenBecomesTargetByOpponent, L506-508)
- `crates/engine/src/rules/events.rs` (GameEvent::PermanentTargeted, L597-609)
- `crates/engine/src/rules/casting.rs` (PermanentTargeted emission after SpellCast, L193-256)
- `crates/engine/src/rules/abilities.rs` (PermanentTargeted emission after AbilityActivated L217-266; check_triggers handler L362-388; flush_pending_triggers targeting L513-523; collect_triggers_for_event default L450)
- `crates/engine/src/effects/mod.rs` (CounterSpell dual-match L506-569; DeclaredTarget stack-aware resolution L1044-1066; ControllerOf stack-aware resolution L1188-1213; MayPayOrElse L984-987)
- `crates/engine/src/testing/replay_harness.rs` (enrich_spec_from_def keyword handling, L363-368 -- no ward-specific logic, relies on builder.rs)
- `tools/replay-viewer/src/view_model.rs` (Ward display, L585)
- `crates/engine/tests/ward.rs` (7 tests, all pass)

## Verdict: clean

Both MEDIUM findings from the initial review have been correctly resolved. Finding 1 (wrong payer) was fixed by changing `PlayerTarget::Controller` to `PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))` in `builder.rs:363`, with supporting changes in `effects/mod.rs` to make `ControllerOf` stack-object-aware. Finding 2 (`.clone()` bug in test 5) was fixed by removing the spurious `.clone()` at line 590, so the ward trigger now resolves on the real state before the spell resolution pass. The 3 LOW findings remain deferred as planned. No new HIGH or MEDIUM issues were introduced by the fixes.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 3 | LOW | `card_definition.rs:508` | **TriggerCondition::WhenBecomesTargetByOpponent is dead code.** No card definition or runtime dispatch uses it. Harmless but speculative. |
| 4 | LOW | `ward.rs` | **Missing test for ward on non-creature permanents.** All 7 tests use creatures; CR 702.21a says "permanent." |
| 5 | LOW | `builder.rs:344` | **Redundant spec_keywords variable.** Keywords iterated twice (once for spec_keywords clone, once for ward check). |

### Finding Details

#### Finding 3: TriggerCondition::WhenBecomesTargetByOpponent is dead code

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs:508`
**CR Rule**: N/A (code quality)
**Issue**: The variant was added to TriggerCondition for future card definitions that might have non-ward "when targeted" triggers, but no card definition currently references it and there is no runtime translation from TriggerCondition to TriggerEvent for this variant. Ward uses TriggerEvent::SelfBecomesTargetByOpponent directly (auto-generated from the keyword in builder.rs).
**Fix**: Keep the variant (it will be useful for future card definitions), but consider adding a comment noting it is not currently wired to runtime dispatch.

#### Finding 4: Missing test for ward on non-creature permanents

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/ward.rs`
**CR Rule**: 702.21a -- "Whenever this **permanent** becomes the target..."
**Issue**: All 7 tests use `ObjectSpec::creature(...)`. The code correctly checks `obj.zone == ZoneId::Battlefield` (permanent-agnostic), but a test for ward on an artifact or planeswalker would confirm this.
**Fix**: Add a test `test_ward_on_noncreature_permanent` using `ObjectSpec::card(...).with_types(vec![CardType::Artifact]).with_keyword(KeywordAbility::Ward(2))` on the battlefield.

#### Finding 5: Redundant spec_keywords variable in builder.rs

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs:344`
**CR Rule**: N/A (code quality)
**Issue**: `spec_keywords` is created as `spec.keywords.iter().cloned().collect()` at line 344, then `spec.keywords` is iterated again at line 345 for the ward translation. The `spec_keywords` clone is only used at line 384.
**Fix**: No action required. Cosmetic.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.21a (ward is triggered ability) | Yes | Yes | test_ward_basic_counter_on_targeting |
| 702.21a (opponent-only) | Yes | Yes | test_ward_does_not_trigger_for_controller, test_ward_multiplayer_opponent_check |
| 702.21a (spells AND abilities) | Yes | Yes | test_ward_triggers_for_activated_ability_targeting |
| 702.21a (counter unless pays cost) | Yes (deterministic: always counters) | Yes | test_ward_basic_counter_on_targeting |
| 702.21a (per-permanent trigger) | Yes | Yes | test_ward_multiple_targets_trigger_separately |
| 702.21a (non-targeting = no trigger) | Yes | Yes | test_ward_does_not_trigger_for_non_targeting_spell |
| 702.21a (can't-be-countered interaction) | Yes | Yes | test_ward_cant_be_countered_spell_resolves_normally |
| 702.21a (non-creature permanents) | Yes (code is permanent-agnostic) | No | LOW Finding 4 |
| 702.21a (ward payer = opponent) | Yes | Not directly testable in deterministic mode | Fixed in builder.rs:363 |
| 702.21b (variable X costs) | No (deferred) | No | Plan explicitly defers variable ward costs |
| Multiplayer (4-player) | Yes | Yes | test_ward_multiplayer_opponent_check |
| Interactive payment (MayPayOrElse) | Deferred (M10+) | N/A | Always applies or_else (counter) |

## Previous Findings (re-review)

| # | Previous Status | Current Status | Notes |
|---|----------------|----------------|-------|
| 1 | OPEN (MEDIUM) | RESOLVED | `builder.rs:363` now uses `PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))`. `ControllerOf` in `effects/mod.rs:1188-1213` now falls back to `state.stack_objects` for stack-resident targets. `resolve_effect_target_list_indexed` at `effects/mod.rs:1056-1061` also checks stack objects for `DeclaredTarget` existence. The full resolution chain correctly identifies the controller of the targeting spell/ability. |
| 2 | OPEN (MEDIUM) | RESOLVED | `.clone()` removed at `ward.rs:590`. The ward trigger now resolves on the real state, and the subsequent spell resolution uses the post-ward-resolution state. Test flow is: cast (produces state) -> pass_all for ward trigger resolution (consumes+produces state) -> pass_all for spell resolution (consumes+produces state). All assertions check the correct state. |
| 3 | OPEN (LOW) | DEFERRED | Dead TriggerCondition variant kept for future use |
| 4 | OPEN (LOW) | DEFERRED | Non-creature ward test not added |
| 5 | OPEN (LOW) | DEFERRED | Cosmetic redundancy in builder.rs |
