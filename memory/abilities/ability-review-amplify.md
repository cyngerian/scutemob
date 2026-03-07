# Ability Review: Amplify

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.38
**Files reviewed**: `crates/engine/src/state/types.rs:1118-1126`, `crates/engine/src/state/hash.rs:614-618`, `tools/replay-viewer/src/view_model.rs:830`, `crates/engine/src/rules/resolution.rs:679-777`, `crates/engine/src/rules/lands.rs:217-296`, `crates/engine/tests/amplify.rs` (825 lines, 8 tests)

## Verdict: clean

The Amplify implementation is correct and complete. All CR 702.38 subrules are faithfully implemented. The ETB replacement pattern follows the established Modular/Graft precedent in both resolution.rs and lands.rs. Creature type sharing uses `calculate_characteristics()` to respect Changeling/CDAs. Multiple instances are processed independently over the same eligible hand cards (CR 702.38b). The hash discriminant (122) includes the N parameter. Tests cover all specified edge cases with correct CR citations. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `resolution.rs:746` | **Redundant clone on OrdSet intersection.** `entering_subtypes.clone().intersection(hand_subtypes)` clones unnecessarily on each iteration. `im::OrdSet::intersection` takes ownership, but since the loop iterates over all hand cards, the entering subtypes are cloned N times. Could use a reference-based intersection check instead. **Fix:** Use `entering_subtypes.iter().any(\|st\| hand_subtypes.contains(st))` to avoid repeated cloning. |
| 2 | LOW | `lands.rs:265-268` | **Same redundant clone pattern in lands.rs Amplify block.** Same optimization opportunity as Finding 1. **Fix:** Same as Finding 1. |
| 3 | LOW | `tests/amplify.rs:67-110` | **cast_creature helper mutates state directly.** The `cast_creature` helper directly mutates `mana_pool` and `priority_holder` on the state, bypassing the Command system. This is an established test pattern (used in modular.rs, graft.rs, etc.) but technically violates Architecture Invariant 3 ("All player actions are Commands"). Acceptable for unit tests. **Fix:** No fix needed -- consistent with existing test patterns. |

### Finding Details

#### Finding 1: Redundant clone on OrdSet intersection

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:746`
**CR Rule**: N/A -- performance, not correctness
**Issue**: `entering_subtypes.clone().intersection(hand_subtypes)` clones the entering subtypes OrdSet for every hand card in the filter closure. While `im::OrdSet` clones are O(1) due to structural sharing, the intersection itself allocates a new set each time just to check emptiness.
**Fix**: Replace with `entering_subtypes.iter().any(|st| hand_subtypes.contains(st))` for an early-exit check without allocation. Apply same fix in lands.rs:265-268.

#### Finding 2: Same redundant clone in lands.rs

**Severity**: LOW
**File**: `crates/engine/src/rules/lands.rs:265-268`
**CR Rule**: N/A -- performance, not correctness
**Issue**: Same pattern as Finding 1.
**Fix**: Same as Finding 1.

#### Finding 3: Test helper direct state mutation

**Severity**: LOW
**File**: `crates/engine/tests/amplify.rs:67-110`
**CR Rule**: Architecture Invariant 3
**Issue**: The `cast_creature` helper directly mutates `state.players.get_mut(&caster).unwrap().mana_pool` and `state.turn.priority_holder` instead of using Command-based mana addition. This is an established pattern across many test files so it is acceptable, but worth noting.
**Fix**: No fix needed -- consistent with existing test conventions.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.38a (static ability, ETB replacement) | Yes | Yes | resolution.rs:679-777 -- replacement, not trigger; no stack |
| 702.38a (reveal cards sharing creature type) | Yes | Yes | test_amplify_basic_one_revealed, test_amplify_multiple_revealed |
| 702.38a (N +1/+1 counters per card revealed) | Yes | Yes | test_amplify_n_multiplier (3x2=6) |
| 702.38a (may reveal zero) | Yes | Yes | test_amplify_empty_hand, test_amplify_no_matching_cards |
| 702.38a (can't reveal itself) | Yes (natural) | Yes (implicit) | Card moves from hand to stack before resolution; not in hand zone during ETB |
| 702.38a (non-creature cards excluded) | Yes | Yes | test_amplify_non_creature_in_hand |
| 702.38a (Changeling/CDA interaction) | Yes | Yes | test_amplify_changeling_in_hand; uses calculate_characteristics() |
| 702.38b (multiple instances work separately) | Yes | Yes | test_amplify_multiple_instances (1x2+2x2=6) |
| lands.rs ETB site consistency | Yes | No (implicit) | Lands never have creature subtypes; block is a defensive no-op |

## Previous Findings (re-review only)

N/A -- first review.
