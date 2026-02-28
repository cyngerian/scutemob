# Ability Review: Fear

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.36
**Files reviewed**:
- `crates/engine/src/state/types.rs:338-345` (enum variant + doc comment)
- `crates/engine/src/state/hash.rs:380-381` (hash discriminant)
- `tools/replay-viewer/src/view_model.rs:612` (keyword_to_string)
- `crates/engine/src/rules/combat.rs:475-489` (blocking restriction enforcement)
- `crates/engine/tests/keywords.rs:983-1326` (7 unit tests)

## Verdict: clean

The Fear implementation is correct and complete. All three CR subrules (702.36a, 702.36b,
702.36c) are properly handled. The blocking restriction enforcement in `combat.rs` exactly
matches the CR 702.36b text. The hash discriminant is unique (45). The view model string
is correct. All seven tests are well-structured, cite CR rules, and cover both positive
and negative cases including the key edge cases (colorless non-artifact, attacker color
irrelevance, combined evasion with flying). No HIGH or MEDIUM findings. Two LOW
observations noted below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/keywords.rs` | **Missing multicolored blocker test.** No test for a white-black creature blocking a fear attacker. **Fix:** Add a test with a `[Color::White, Color::Black]` blocker asserting `is_ok()`. |
| 2 | LOW | `rules/combat.rs:479-480` | **Redundant Creature type check in artifact-creature guard.** The `blocker_chars.card_types.contains(&CardType::Creature)` check is always true at this point because line 372 already rejected non-creatures. Not a bug -- it is defensive, matches CR text exactly, and mirrors the Intimidate pattern. No fix needed. |

### Finding Details

#### Finding 1: Missing multicolored blocker test

**Severity**: LOW
**File**: `crates/engine/tests/keywords.rs`
**CR Rule**: 702.36b -- "A creature with fear can't be blocked except by artifact creatures and/or black creatures."
**Issue**: There is no test verifying that a multicolored creature that includes black (e.g., white-black) can block a creature with fear. The Intimidate suite has a multicolored attacker test (`test_702_13_intimidate_multicolor_attacker_allows_partial_color_match`), but Fear's suite does not have an equivalent for the blocker side. The implementation handles this correctly because `colors.contains(&Color::Black)` works with multi-element color sets, but the test gap means this behavior is not regression-protected.
**Fix**: Add a test `test_702_36_fear_allows_multicolor_black_creature_blocker` with a blocker that has `with_colors(vec![Color::White, Color::Black])`, asserting `is_ok()`.

#### Finding 2: Redundant Creature type check in artifact-creature guard

**Severity**: LOW (informational)
**File**: `crates/engine/src/rules/combat.rs:479-480`
**CR Rule**: 702.36b -- "artifact creatures"
**Issue**: The check `blocker_chars.card_types.contains(&CardType::Creature)` at line 480 is always true when reached, because line 372 already validates that the blocker is a creature. The `&& CardType::Creature` portion is dead logic in the current control flow. However, this is correct defensive coding: if the validation order ever changes, this guard would catch the edge case. It also matches the Intimidate pattern at line 460-461 exactly.
**Fix**: None required. Keep for consistency with Intimidate and CR text fidelity.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.36a (fear is an evasion ability) | Yes | Yes (implicitly) | Categorized as evasion via blocking restriction pattern |
| 702.36b (artifact creatures and/or black creatures) | Yes | Yes | 7 tests cover all branches: non-matching rejected, artifact creature accepted, black creature accepted, black artifact creature accepted, colorless non-artifact rejected, attacker color irrelevant, combined evasion with flying |
| 702.36c (multiple instances redundant) | Yes | No (implicit) | OrdSet deduplication handles this; no explicit test, but this is a framework invariant, not fear-specific logic |

## Previous Findings (re-review only)

N/A -- first review.
