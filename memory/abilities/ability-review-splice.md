# Ability Review: Splice

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.47
**Files reviewed**:
- `crates/engine/src/state/types.rs:977-988`
- `crates/engine/src/cards/card_definition.rs:431-444`
- `crates/engine/src/state/hash.rs:555-559, 1727-1731, 3331-3340`
- `crates/engine/src/rules/command.rs:223-231`
- `crates/engine/src/state/stack.rs:225-243`
- `crates/engine/src/rules/casting.rs:1849-1949`
- `crates/engine/src/rules/resolution.rs:215-236`
- `crates/engine/src/rules/copy.rs:223-225, 423-425`
- `crates/engine/src/testing/replay_harness.rs:1295-1327`
- `crates/engine/src/testing/script_schema.rs:317-320`
- `crates/engine/tests/splice.rs` (full)

## Verdict: needs-fix

One MEDIUM finding (harness silently drops unresolvable splice card names instead of
failing the test) and two LOW findings (doc comment inaccuracy and test ordering
assertion weakness). The core engine enforcement is correct and matches CR 702.47a-e.
The copy interaction (707.2 excludes text-changing effects from copiable values) is
handled correctly with `spliced_effects: vec![]` on copies. All validation checks
(zone, keyword, subtype, duplicate, self-splice) are present and correct.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `replay_harness.rs:1303-1306` | **Harness silently drops unresolvable splice cards.** `filter_map` skips names not found in hand instead of failing. **Fix:** use `map` + `expect`/error. |
| 2 | LOW | `types.rs:986` | **Inaccurate doc comment.** Says "Multiple instances each trigger independently" but Splice is a static ability, not triggered. **Fix:** change to "Multiple instances are evaluated independently." |
| 3 | LOW | `splice.rs:827-913` | **Ordering test doesn't verify ordering.** `test_splice_main_effect_first` only checks total life, which is commutative. Cannot distinguish execution order. **Fix:** acceptable for P4; consider using non-commutative effects (e.g., DealDamage + GainLife based on life total) if ordering precision matters later. |

### Finding Details

#### Finding 1: Harness silently drops unresolvable splice cards

**Severity**: MEDIUM
**File**: `crates/engine/src/testing/replay_harness.rs:1303-1306`
**CR Rule**: 702.47a -- "You may reveal this card from your hand"
**Issue**: The `cast_spell_splice` harness arm uses `filter_map(|n| find_in_hand(...))` to
resolve splice card names to ObjectIds. If a splice card name is misspelled or the card
is not in hand, it is silently dropped from the list. This means a test script that
declares `splice_card_names: ["Glacial Ray"]` when Glacial Ray is not in hand will
silently cast the spell without splice, and the test may pass for the wrong reason.
This is inconsistent with other harness arms (e.g., `find_in_hand` returns `Option`
and the arm returns `None` to skip the action on failure). The splice path should fail
loudly if a declared splice card cannot be found.
**Fix**: Replace `filter_map` with a loop that calls `find_in_hand` and returns `None`
(or panics with a descriptive message) if any splice card name cannot be resolved. For
example:
```rust
let mut splice_ids = Vec::new();
for name in splice_card_names {
    let id = find_in_hand(state, player, name.as_str())
        .unwrap_or_else(|| panic!("splice card '{}' not found in hand", name));
    splice_ids.push(id);
}
```

#### Finding 2: Inaccurate doc comment on KeywordAbility::Splice

**Severity**: LOW
**File**: `crates/engine/src/state/types.rs:986`
**CR Rule**: 702.47a -- "Splice is a static ability"
**Issue**: The doc comment on `KeywordAbility::Splice` says "CR 702.47b: Multiple instances
each trigger independently." Splice is explicitly defined as a static ability by CR 702.47a,
not a triggered ability. The word "trigger" is misleading here. This appears to be
copy-paste from the Replicate variant's doc comment (which is a triggered ability per
CR 702.56b).
**Fix**: Change line 986 from `/// CR 702.47b: Multiple instances each trigger independently.`
to `/// CR 702.47b: Multiple splice abilities are evaluated independently.`

#### Finding 3: Ordering test uses commutative effects

**Severity**: LOW
**File**: `crates/engine/tests/splice.rs:827-913`
**CR Rule**: 702.47b -- "The effects of the main spell must happen first."
**Issue**: `test_splice_main_effect_first` verifies that life total is 23 (20 + 1 + 2), but
since GainLife is commutative, this total would be the same regardless of execution order.
The test validates that both effects execute but cannot detect a violation of the "main
spell first" ordering requirement from CR 702.47b.
**Fix**: This is acceptable for P4 priority. If ordering precision becomes important (e.g.,
effects that depend on game state modified by earlier effects), replace with
non-commutative effects where the final state differs based on order. No action required
now.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.47a (static ability, reveal from hand, pay splice cost) | Yes | Yes | `test_splice_basic_onto_arcane`, `test_splice_cost_added`, `test_splice_card_stays_in_hand`, `test_splice_not_in_hand_rejected` |
| 702.47a (splice card stays in hand) | Yes | Yes | `test_splice_card_stays_in_hand` -- checks before and after resolution |
| 702.47b (can't splice same card twice) | Yes | Yes | `test_splice_same_card_twice_rejected` |
| 702.47b (multiple splices in declared order) | Yes | Yes | `test_splice_multiple_cards` |
| 702.47b (main spell effect first) | Yes | Partial | `test_splice_main_effect_first` -- uses commutative effects (Finding 3) |
| 702.47b (can't splice if required choices impossible) | No | No | Not enforced; would require target validation for spliced effects at cast time. Edge case, acceptable for P4. |
| 702.47c (only gains rules text, not characteristics) | Yes (implicit) | No | The engine stores only `Effect`, never copies name/cost/types. Correct by construction. |
| 702.47c (name references refer to spell, not spliced card) | Yes (implicit) | No | `EffectContext.source` is the spell's source object. Correct by construction. |
| 702.47d (choose targets for added text) | Partial | No | Spliced effects share the main spell's target list. Full per-splice targeting not implemented. Acceptable for P4. |
| 702.47e (splice lost when spell leaves stack) | Yes (implicit) | No | `spliced_effects` lives on `StackObject`, which is consumed on resolution/counter. Correct by construction. |
| Ruling: can't splice onto itself | Yes | Yes | `test_splice_onto_itself_rejected` -- explicit card==splice check at casting.rs:1876 |
| 707.2: copies don't inherit splice (text-changing effects not copiable) | Yes | No | `copy.rs:223-225` correctly uses `spliced_effects: vec![]`. No test for this interaction. |

## Previous Findings (re-review only)

N/A -- first review.
