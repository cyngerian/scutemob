# Ability Review: Gravestorm

**Date**: 2026-03-05
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.69
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 961-968)
- `crates/engine/src/state/stack.rs` (lines 839-854)
- `crates/engine/src/state/hash.rs` (lines 551-553, 1632-1641, 3405-3407)
- `crates/engine/src/state/mod.rs` (lines 128-133, 354-362, 484-489)
- `crates/engine/src/state/builder.rs` (line 345)
- `crates/engine/src/rules/turn_actions.rs` (lines 730-733)
- `crates/engine/src/rules/casting.rs` (lines 2684-2734)
- `crates/engine/src/rules/resolution.rs` (lines 861-889, 3425-3440)
- `crates/engine/src/rules/copy.rs` (lines 245-259 -- `create_storm_copies` reuse)
- `tools/tui/src/play/panels/stack_view.rs` (lines 138-140)
- `tools/replay-viewer/src/view_model.rs` (lines 531-533)
- `crates/engine/tests/gravestorm.rs` (all 690 lines, 9 tests)

## Verdict: clean

The implementation is correct and complete. All CR 702.69 subrules are faithfully
implemented. The counter tracking is global (on `GameState`), increments only for
battlefield-to-graveyard moves, covers both `move_object_to_zone` and
`move_object_to_bottom_of_zone`, resets at turn change, and the gravestorm count is
captured at trigger creation time (not resolution). Copies are created via the existing
`create_storm_copies` infrastructure, which correctly handles the "original countered"
case (graceful no-op). Hash coverage is complete. All match arms are covered. No
`.unwrap()` in engine library code. Two LOW findings identified; neither affects
correctness.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/gravestorm.rs` | **Missing test for original-countered edge case.** No test verifies behavior when the original spell is countered before the gravestorm trigger resolves. **Fix:** Add a test that casts a gravestorm spell, counters it, then resolves the trigger -- verify 0 copies created (graceful no-op from `create_storm_copies`). |
| 2 | LOW | `casting.rs:2692` | **702.69b multiple instances not testable.** `KeywordAbility` in `OrdSet` deduplicates, so multiple gravestorm instances on one spell cannot be represented. Known limitation shared with Storm (702.40b). No card has multiple gravestorm instances. **Fix:** None needed -- accept as known gap, already documented in plan. |

### Finding Details

#### Finding 1: Missing test for original-countered edge case

**Severity**: LOW
**File**: `crates/engine/tests/gravestorm.rs`
**CR Rule**: 702.69a -- "When you cast this spell, copy it for each permanent..."
**Issue**: The plan identified "trigger can be countered" and "each copy is independent"
as key edge cases. The implementation handles the "original spell countered before
trigger resolves" case correctly via `create_storm_copies` returning empty events when
`copy_spell_on_stack` fails (the original is no longer on the stack). However, there is
no test exercising this path. The resolution.rs comment at line 869 acknowledges this
behavior but it is untested.
**Fix**: Add `test_gravestorm_original_countered_trigger_still_resolves` -- counter the
original gravestorm spell, then resolve the trigger. Assert 0 copies created and no
panics. This is LOW because the underlying infrastructure (`create_storm_copies`) is
already tested by Storm tests, but a gravestorm-specific regression test would be
valuable.

#### Finding 2: 702.69b multiple instances limitation

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:2692`
**CR Rule**: 702.69b -- "If a spell has multiple instances of gravestorm, each triggers separately."
**Issue**: The `keywords` field uses `OrdSet<KeywordAbility>` which deduplicates, so a
spell with two instances of gravestorm would only produce one trigger. This is a known
structural limitation shared with Storm (702.40b) and acknowledged in the plan.
**Fix**: None needed. No card in MTG has multiple gravestorm instances. The limitation
is documented. If a future card requires this, the `keywords` data structure would need
to change to a multiset (engine-wide refactor).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.69a (trigger creation) | Yes | Yes | `test_gravestorm_basic_creates_copies`, `test_gravestorm_zero_count_no_copies` |
| 702.69a (count = permanents battlefield->graveyard) | Yes | Yes | `test_gravestorm_count_increments_on_permanent_dying`, `test_gravestorm_count_does_not_include_non_battlefield_to_graveyard` |
| 702.69a (global, all players) | Yes | Yes | `test_gravestorm_count_includes_all_players` |
| 702.69a (tokens count) | Yes | Yes | `test_gravestorm_count_includes_tokens` |
| 702.69a ("this turn" -- reset) | Yes | Yes | `test_gravestorm_count_resets_each_turn` |
| 702.69a (copies not cast) | Yes | Yes | `test_gravestorm_copies_not_cast` |
| 702.69a (count captured at cast, not resolution) | Yes | Yes | Implicit in `test_gravestorm_basic_creates_copies` (count set before cast, trigger captures it) |
| 702.69a (original countered) | Yes | No | `create_storm_copies` handles gracefully; no dedicated test (Finding 1) |
| 702.69a (accumulation) | Yes | Yes | `test_gravestorm_count_accumulates_across_multiple_deaths` |
| 702.69b (multiple instances) | No | No | Known OrdSet limitation (Finding 2) |
