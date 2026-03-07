# Ability Review: Scavenge

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.97
**Files reviewed**: `state/types.rs`, `cards/card_definition.rs`, `state/stack.rs`, `state/hash.rs`, `rules/command.rs`, `rules/engine.rs`, `rules/abilities.rs`, `rules/resolution.rs`, `testing/replay_harness.rs`, `testing/script_schema.rs`, `tools/tui/src/play/panels/stack_view.rs`, `tools/replay-viewer/src/view_model.rs`, `tools/replay-viewer/src/replay.rs`, `crates/engine/tests/scavenge.rs`

## Verdict: clean

The Scavenge implementation is correct with respect to CR 702.97a. All critical behaviors are properly implemented: power snapshot before exile, exile as cost, sorcery-speed restriction, target validation, fizzle on invalid target at resolution, and the ability is correctly treated as an activated ability (not a cast). Hash coverage is complete. TUI and replay-viewer exhaustive matches are updated. One LOW finding exists for test coverage.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/scavenge.rs:389-404` | **Test 3c (non-empty stack) is a no-op.** The test drops state without testing. **Fix:** Push a dummy StackObject, then assert `ScavengeCard` returns error. |

### Finding Details

#### Finding 1: Test 3c (non-empty stack) is a no-op

**Severity**: LOW
**File**: `crates/engine/tests/scavenge.rs:389-404`
**CR Rule**: 702.97a -- "Activate only as a sorcery."
**Issue**: Test 3 sub-case (c) for "cannot scavenge with a non-empty stack" does not actually test the behavior. The code block creates state, suppresses unused warnings, drops state, and exits. The comments acknowledge this limitation but no actual assertion is made. The implementation does have the correct guard (`!state.stack_objects.is_empty()` at abilities.rs:4913), but the test does not exercise it.
**Fix**: Add a dummy `StackObject` to `state.stack_objects` before issuing `ScavengeCard`, and assert the command returns an error. This requires constructing a minimal `StackObject` or using an existing helper.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.97a (activated ability from graveyard) | Yes | Yes | test_scavenge_basic_adds_counters, test_scavenge_requires_graveyard |
| 702.97a (exile as cost) | Yes | Yes | test_scavenge_card_exiled_as_cost |
| 702.97a (power determines counters) | Yes | Yes | test_scavenge_basic_adds_counters (5 counters for 5-power) |
| 702.97a (power snapshot before exile) | Yes | Yes | Power captured at line 4972 before exile at line 4984 |
| 702.97a (target creature) | Yes | Yes | test_scavenge_basic_adds_counters, test_scavenge_fizzles_if_target_leaves |
| 702.97a (activate only as a sorcery) | Yes | Partial | Active player tested; main phase tested; empty stack NOT tested (Finding 1) |
| 702.97a (Scavenge keyword required) | Yes | Yes | test_scavenge_requires_keyword |
| 608.2b (fizzle on illegal target) | Yes | Yes | test_scavenge_fizzles_if_target_leaves |
| Not-a-cast (no SpellCast event) | Yes | Yes | test_scavenge_not_a_cast |
| 0-power edge case | Yes | Yes | test_scavenge_zero_power |
| Mana cost required | Yes | Yes | test_scavenge_requires_mana_payment |
| Multiplayer: non-active player rejected | Yes | Yes | test_scavenge_multiplayer_only_active_player (4-player) |

## Previous Findings (re-review only)

N/A -- first review.
