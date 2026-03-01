# Ability Review: Shadow

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.28
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 572-576)
- `crates/engine/src/state/hash.rs` (lines 450-451)
- `tools/replay-viewer/src/view_model.rs` (lines 674)
- `crates/engine/src/rules/combat.rs` (lines 491-501)
- `crates/engine/tests/shadow.rs` (full file, 357 lines)

## Verdict: clean

The Shadow implementation is correct and complete for CR 702.28. All three subrules
(702.28a evasion classification, 702.28b bidirectional blocking restriction, 702.28c
redundancy) are properly handled. The enforcement logic in combat.rs uses a clean
`attacker_has_shadow != blocker_has_shadow` check that correctly encodes the bidirectional
restriction. The hash discriminant (68) is unique and follows the sequential pattern. All 7
tests cover both halves of the bidirectional restriction plus the compound evasion
interaction with Flying/Reach. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `crates/engine/tests/shadow.rs` | **Separate test file instead of keywords.rs.** Tests are in a standalone file rather than appended to `keywords.rs` as the plan specified. This is functionally equivalent and arguably better for modularity, but deviates from the plan. No fix needed. |
| 2 | LOW | `crates/engine/tests/shadow.rs` | **No test for Shadow + Menace compound evasion.** Shadow + Menace is called out in the plan's "Interactions to Watch" section. A test verifying that a shadow+menace creature requires 2+ shadow blockers would strengthen coverage. **Fix:** Consider adding `test_702_28_shadow_plus_menace_requires_two_shadow_blockers` in a future pass. |
| 3 | LOW | `crates/engine/tests/shadow.rs` | **No test for Shadow + Protection compound evasion.** The plan lists Shadow + Protection as an interaction to watch. A test verifying both restrictions compound would be valuable. **Fix:** Consider adding a test in a future pass. |
| 4 | LOW | N/A | **"Can block as though it didn't have shadow" effects not handled.** Cards like Aetherflame Wall and Aether Web override the shadow restriction via "as though" text. This is a card-specific ability, not part of CR 702.28 itself, so it does not affect the correctness of this implementation. When those cards are authored, they will need a mechanism to bypass the shadow check. **Fix:** No action now; note for future card definitions. |

### Finding Details

#### Finding 1: Separate test file

**Severity**: LOW
**File**: `crates/engine/tests/shadow.rs`
**Issue**: The plan specified adding tests to `crates/engine/tests/keywords.rs` (after the Fear
section). The implementation created a standalone `shadow.rs` test file instead. Both approaches
are used in the project (e.g., `combat.rs`, `sba.rs` have standalone test files). This is
acceptable and has the advantage of smaller file sizes.
**Fix**: No action needed.

#### Finding 2: No Shadow + Menace compound test

**Severity**: LOW
**File**: `crates/engine/tests/shadow.rs`
**CR Rule**: 702.28b + 702.110 (Menace)
**Issue**: The plan's "Interactions to Watch" section calls out Shadow + Menace. The sequential
check architecture handles this correctly (shadow check is per-pair, menace check is aggregate
at line 528), but there is no test proving this. The existing Shadow + Flying tests demonstrate
the compound evasion pattern, so this is a minor gap.
**Fix**: Consider adding `test_702_28_shadow_plus_menace_requires_two_shadow_blockers` in a
future test pass.

#### Finding 3: No Shadow + Protection compound test

**Severity**: LOW
**File**: `crates/engine/tests/shadow.rs`
**CR Rule**: 702.28b + 702.16f (Protection)
**Issue**: Shadow + Protection is another compound interaction from the plan's watch list.
Both checks are independent and sequential in combat.rs (shadow at line 496, protection at
line 505). No test verifies the combination.
**Fix**: Consider adding a test in a future pass.

#### Finding 4: "As though" shadow override mechanism

**Severity**: LOW
**CR Rule**: Not CR 702.28 itself -- card-specific oracle text
**Issue**: Aetherflame Wall ("can block creatures with shadow as though they didn't have shadow")
and Aether Web grant similar override abilities. The current shadow check in combat.rs has no
mechanism for "as though" overrides. This is not a gap in CR 702.28 enforcement -- it's a
future card-definition concern. The Aetherflame Wall ruling (2006-09-25) also notes the
interesting edge case: "If Aetherflame Wall gains shadow, it won't be able to block any
creatures (not even those with shadow)" -- meaning the "as though" effect is overridden by
actually having shadow, since the check would then be shadow-vs-shadow-with-override which
collapses. This is only relevant when those specific cards are implemented.
**Fix**: No action now. When implementing Aetherflame Wall or Aether Web, add a mechanism
(e.g., a keyword like `IgnoreShadowRestriction` or a continuous effect) to bypass the shadow
check in combat.rs.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.28a (Shadow is an evasion ability) | Yes | Yes | Classification correct; evasion check in combat.rs per-pair loop |
| 702.28b (bidirectional blocking restriction) | Yes | Yes | `attacker_has_shadow != blocker_has_shadow` at combat.rs:496; 4 tests cover both directions + baseline |
| 702.28b (compound with other evasion) | Yes | Yes | 3 tests: shadow+flying (fail), shadow+flying+flying (pass), shadow+flying+reach (pass) |
| 702.28c (multiple instances redundant) | Yes | No | Handled by `OrdSet` deduplication infrastructure; no explicit test, but this is infrastructure-level (all keywords get this) |

## Previous Findings (re-review only)

N/A -- first review.
