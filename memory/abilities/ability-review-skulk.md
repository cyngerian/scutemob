# Ability Review: Skulk

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.118
**Files reviewed**:
- `crates/engine/src/state/types.rs:605-608` (enum variant)
- `crates/engine/src/state/hash.rs:461-462` (hash discriminant 72)
- `tools/replay-viewer/src/view_model.rs:681` (format_keyword arm)
- `crates/engine/src/rules/combat.rs:520-534` (blocking restriction enforcement)
- `crates/engine/tests/skulk.rs` (7 tests, 489 lines)

## Verdict: clean

The Skulk implementation is correct and complete. The enforcement logic in `combat.rs` matches
CR 702.118b exactly: one-directional check on the attacker only, strictly-greater-than comparison
(`blocker_power > attacker_power`), using post-layer power values from `calculate_characteristics`.
The hash discriminant (72) has no collisions within the `KeywordAbility` enum. All 7 tests cover
the critical boundary conditions (greater, equal, lesser, one-directional, flying interaction,
zero power, pump effects). One LOW finding for a missing negative-power edge case test, which
does not affect correctness since `i32` comparison handles it natively.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/skulk.rs` | **Missing negative-power test.** Plan mentions -1 power edge case but no test covers it. |

### Finding Details

#### Finding 1: Missing negative-power test

**Severity**: LOW
**File**: `crates/engine/tests/skulk.rs`
**CR Rule**: 702.118b -- "A creature with skulk can't be blocked by creatures with greater power."
**Ruling**: 2016-04-08 (Furtive Homunculus) -- zero or negative power uses actual value.
**Issue**: The plan identifies that a creature with skulk and -1 power should only be blockable
by creatures with power -1 or less. Test 6 (`test_702_118_skulk_zero_power_attacker`) covers the
zero-power boundary (0 vs 1 = blocked, 0 vs 0 = allowed), but no test covers a negative-power
skulk attacker (e.g., a creature with -1 power due to effects). The implementation handles this
correctly because `i32` comparison naturally works for negative values, but a test would document
and protect this edge case.
**Fix**: Add an optional 8th test `test_702_118_skulk_negative_power_attacker` using a continuous
effect to reduce the skulk attacker's power to -1, then verify a 0-power blocker is rejected
(0 > -1) and a -1-power blocker is allowed (-1 is not > -1). Low priority since the implementation
is already correct.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.118a (skulk is an evasion ability) | Yes | Implicit | Doc comment cites it; enforcement is in the evasion section of combat.rs |
| 702.118b (can't be blocked by creatures with greater power) | Yes | Yes | Tests 1-3 (greater/equal/lesser), test 6 (zero power), test 7 (pump) |
| 702.118b (one-directional: only restricts blockers of skulk creature) | Yes | Yes | Test 4 (skulk blocker can block anything) |
| 702.118b (uses post-layer power) | Yes | Yes | Test 7 (continuous +2/+0 effect changes comparison) |
| 702.118b (stacks with other evasion) | Yes | Yes | Test 5 (skulk + flying: three sub-cases) |
| 702.118c (multiple instances redundant) | Yes | Implicit | OrdSet deduplication; no explicit test but structural guarantee |
| Ruling 2016-04-08 (zero/negative power) | Yes | Partial | Zero tested (test 6); negative not tested (Finding 1) |
| Ruling 2016-04-08 (checked at declaration time only) | Yes | Implicit | Engine naturally checks at DeclareBlockers command time |

## Previous Findings (re-review only)

N/A -- first review.
