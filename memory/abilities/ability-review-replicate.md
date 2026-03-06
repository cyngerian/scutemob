# Ability Review: Replicate

**Date**: 2026-03-05
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.56
**Files reviewed**:
- `crates/engine/src/state/types.rs:951-960` (KeywordAbility::Replicate)
- `crates/engine/src/cards/card_definition.rs:413-423` (AbilityDefinition::Replicate)
- `crates/engine/src/state/stack.rs:822-838` (StackObjectKind::ReplicateTrigger)
- `crates/engine/src/state/hash.rs:550-551, 1619-1629, 3292-3296` (hash arms)
- `crates/engine/src/rules/command.rs:212-219` (replicate_count field)
- `crates/engine/src/rules/engine.rs:82-125` (CastSpell dispatch)
- `crates/engine/src/rules/casting.rs:74, 1760-1798, 2784-2831, 2898-2917` (cost validation, trigger creation, get_replicate_cost)
- `crates/engine/src/rules/resolution.rs:829-858` (ReplicateTrigger resolution)
- `crates/engine/src/rules/resolution.rs:3394` (counter arm)
- `crates/engine/src/effects/mod.rs:850` (ward counter catch-all)
- `tools/tui/src/play/panels/stack_view.rs:135-137` (TUI match arm)
- `tools/replay-viewer/src/view_model.rs:528-530` (replay viewer match arm)
- `crates/engine/tests/replicate.rs` (6 tests)

## Verdict: clean

The Replicate implementation is correct and complete for all practical purposes. It follows
the established Storm/Casualty pattern closely, correctly reuses `create_storm_copies`, properly
validates the keyword before accepting payment, correctly adds replicate cost N times to the
total mana, and creates a trigger that resolves to produce copies. All hash discriminants are
unique within their respective enums. All exhaustive match arms are covered. The only gaps are
one missing planned test (LOW) and the documented limitation where copies fail if the original
is countered before trigger resolution (also documented as LOW in both the plan and the code).
No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/replicate.rs` | **Missing test_replicate_with_targets.** Plan listed 7 tests but only 6 implemented. **Fix:** Add `test_replicate_with_targets` per plan Step 6 description. |
| 2 | LOW | `resolution.rs:838-840` | **Countered-original copy gap (documented).** If the original spell is countered before ReplicateTrigger resolves, `create_storm_copies` returns no events because `copy_spell_on_stack` returns Err. Per Shattering Spree ruling 2024-01-12, copies should still be created. Already documented in code and plan as a known limitation of the copy infrastructure (same gap exists for Storm). No fix needed now. |

### Finding Details

#### Finding 1: Missing test_replicate_with_targets

**Severity**: LOW
**File**: `crates/engine/tests/replicate.rs`
**CR Rule**: 702.56a -- "If the spell has any targets, you may choose new targets for any of the copies."
**Issue**: The plan (Step 6) listed `test_replicate_with_targets` as one of seven tests to write, verifying that copies of targeted spells retain the same targets (since choose-new-targets is not yet interactive). Only 6 tests were implemented; this one was omitted.
**Fix**: Add `test_replicate_with_targets` that casts a targeted Replicate spell (e.g., a synthetic "deal 1 damage to target creature" with Replicate), pays replicate_count=1, resolves the trigger, and asserts the copy targets the same object as the original.

#### Finding 2: Countered-original copy gap

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:838-840`
**CR Rule**: 702.56a -- copies are created "for each time its replicate cost was paid"
**Issue**: If the original spell is countered between casting and ReplicateTrigger resolution, `create_storm_copies` calls `copy_spell_on_stack` which returns `Err` when the original stack object is gone, and the loop breaks with zero copies. The Shattering Spree ruling states copies should still be created. This is the same limitation as Storm and is documented in both the code comments and the plan. No action needed until the copy infrastructure is enhanced.
**Fix**: No fix needed now. When copy infrastructure is enhanced to support copying from a "snapshot" rather than a live stack object, apply to both Storm and Replicate triggers.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.56a (static: additional cost) | Yes | Yes | `test_replicate_mana_cost_added`, cost validation in casting.rs:1760-1798 |
| 702.56a (triggered: copy N times) | Yes | Yes | `test_replicate_one_copy`, `test_replicate_basic_two_copies` |
| 702.56a (zero payment = no trigger) | Yes | Yes | `test_replicate_zero_copies` |
| 702.56a (copies not cast) | Yes | Yes | `test_replicate_copies_not_cast` |
| 702.56a (choose new targets) | Partial | No | Choose-new-targets not yet interactive; copies keep same targets. Missing test. |
| 702.56a (CR 601.2b/601.2f-h) | Yes | Yes | Cost addition follows kicker pattern |
| 702.56b (multiple instances) | No | No | No cards have multiple replicate instances; data model does not preclude it (plan note) |
| Keyword validation | Yes | Yes | `test_replicate_no_keyword_rejected` |
