# Ability Review: Affinity

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.41
**Files reviewed**:
- `crates/engine/src/state/types.rs:138-150` (AffinityTarget enum), `types.rs:347-363` (KeywordAbility::Affinity)
- `crates/engine/src/state/hash.rs:288-298` (AffinityTarget hash), `hash.rs:408-413` (Affinity arm, discriminant 53)
- `crates/engine/src/state/mod.rs:43-46` (AffinityTarget re-export)
- `crates/engine/src/lib.rs:19-28` (AffinityTarget public re-export)
- `crates/engine/src/rules/casting.rs:691-695` (pipeline call site), `casting.rs:1905-1986` (apply_affinity_reduction, count_affinity_permanents, matches_affinity_target)
- `tools/replay-viewer/src/view_model.rs:10-13` (import), `view_model.rs:632-635` (format_keyword Affinity arm)
- `crates/engine/tests/affinity.rs` (12 tests, 861 lines)
- `crates/engine/src/state/builder.rs` (verified no match arm needed -- uses if-let pattern, not exhaustive match)

## Verdict: clean

The Affinity implementation is correct and complete. All CR 702.41 subrules are faithfully implemented. The cost reduction pipeline placement is accurate (after kicker, before convoke/improvise/delve). The implementation correctly reduces only generic mana, floors at zero per CR 601.2f, counts only the caster's controlled permanents, includes tapped permanents, and handles multiple distinct affinity instances cumulatively. The three new engine functions contain no `.unwrap()` calls, all match arms are covered, hash discriminants are assigned, and all 12 tests cite CR rules. Two LOW findings are noted below -- both are minor and do not affect correctness for any real card or game scenario.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `casting.rs:1912-1913` | **OrdSet deduplication prevents identical affinity instances.** Doc comment claims "Two instances of affinity for artifacts with 3 artifacts = 6 generic mana reduction" but this is impossible with `OrdSet<KeywordAbility>` storage. **Fix:** Update doc comment to note the OrdSet limitation or add "(distinct targets only)" qualifier. |
| 2 | LOW | `tests/affinity.rs:680` | **Missing test for insufficient mana WITH affinity.** No negative test confirms that affinity reduces the cost but casting still fails if the player lacks the reduced amount. E.g., spell {4} with 2 artifacts (reduced to {2}) and only {1} in pool should fail. **Fix:** Add a test `test_affinity_insufficient_mana_after_reduction`. |

### Finding Details

#### Finding 1: OrdSet deduplication prevents identical affinity instances

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:1912-1913`
**CR Rule**: 702.41b -- "If a spell has multiple instances of affinity, each of them applies."
**Issue**: The doc comment on `apply_affinity_reduction` states "Two instances of 'affinity for artifacts' with 3 artifacts = 6 generic mana reduction." However, `Characteristics::keywords` is an `OrdSet<KeywordAbility>`, which deduplicates identical values. Two `Affinity(AffinityTarget::Artifacts)` entries collapse to one, making the described scenario impossible in the current data model. The implementation correctly handles distinct affinity targets (test 10 confirms this with Artifacts + BasicLandType(Plains)), but identical targets cannot be tested or applied. No real MTG card has two printed instances of the same affinity keyword, and granting a redundant affinity via external effects is extremely rare and speculative. The test file (test 10, line 680-682) already documents this limitation.
**Fix**: Update the doc comment on `apply_affinity_reduction` (lines 1912-1913) to say "Two instances of affinity with *distinct* targets each reduce independently" or add a note that identical affinity instances are deduplicated by OrdSet. This prevents future confusion about what the function can actually do.

#### Finding 2: Missing negative test for insufficient mana after reduction

**Severity**: LOW
**File**: `crates/engine/tests/affinity.rs`
**CR Rule**: 601.2f -- total cost must be paid after all reductions
**Issue**: Test 5 (`test_affinity_no_keyword_no_reduction`) covers the case where there is no affinity keyword at all. However, there is no test for the scenario where affinity *does* reduce the cost but the player still lacks the *reduced* amount. For example: spell {4} with Affinity for artifacts, player controls 2 artifacts (cost reduced to {2}), but only {1} in pool -- cast should fail. This is a minor gap because the mana payment system is well-tested elsewhere, but having it in the affinity test file would improve coverage completeness.
**Fix**: Add a test `test_affinity_insufficient_mana_after_reduction` that sets up a spell with affinity, partial artifact control, and insufficient mana for the reduced cost. Assert that `cast_spell` returns `Err`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.41a (static ability, spell costs {1} less per [text] you control) | Yes | Yes | Tests 1, 2, 3, 4, 6, 7, 11, 12 |
| 702.41a ("you control" -- only caster's permanents) | Yes | Yes | Test 7 (opponent's artifacts don't count) |
| 702.41a (counts all matching permanents -- tapped and untapped) | Yes | Yes | Test 6 (2 tapped + 2 untapped) |
| 702.41a (reduces generic mana only, not colored pips) | Yes | Yes | Test 4 ({4}{U} with 4 artifacts = pay {U}) |
| 702.41b (multiple instances cumulative) | Yes (distinct targets) | Yes (distinct targets) | Test 10 (Artifacts + Plains). Identical target dedup = OrdSet limitation (LOW finding 1) |
| 601.2f (generic cannot go below 0) | Yes | Yes | Test 3 (6 artifacts for {4} spell) |
| 601.2f (pipeline order: after kicker, before convoke/improvise/delve) | Yes | Yes | casting.rs:695 placement; Test 8 (affinity + improvise combo) |
| 118.9d (cost reductions apply to alternative costs too) | Yes (implicit) | No (explicit) | Affinity reads from `mana_cost` which is already resolved from alt cost. No dedicated test, but alt-cost pipeline is tested elsewhere. |
| Artifact creatures count as artifacts | Yes | Yes | Test 11 |
| Affinity for basic land type | Yes | Yes | Test 12 (Plains) |
| Commander tax interaction | Yes | Yes | Test 9 (tax + affinity) |
| Negative: no keyword = no reduction | Yes | Yes | Test 5 |

## Previous Findings (re-review only)

N/A -- first review.
