# Ability Review: Entwine

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.42
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 988)
- `crates/engine/src/cards/card_definition.rs` (line 446)
- `crates/engine/src/state/hash.rs` (lines 558, 1729, 3345)
- `crates/engine/src/rules/command.rs` (line 232)
- `crates/engine/src/state/stack.rs` (line 225)
- `crates/engine/src/rules/casting.rs` (lines 1850-1880, 3202-3220)
- `crates/engine/src/rules/resolution.rs` (lines 162-248)
- `crates/engine/src/rules/copy.rs` (lines 223-225, 426-427)
- `crates/engine/src/testing/replay_harness.rs` (lines 1387-1414)
- `crates/engine/tests/entwine.rs` (all 733 lines)

## Verdict: needs-fix

One MEDIUM finding: the StackObject doc comment for `was_entwined` contradicts the
actual copy propagation behavior in `copy.rs`. The propagation is correct per CR 707.10
but the doc comment claims copies must always have `was_entwined: false`. Additionally,
the planned commander-tax stacking test was not implemented.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `stack.rs:228` | **Doc comment contradicts copy propagation.** `was_entwined` doc says "Must always be false for copies" but `copy.rs:225` correctly propagates it from the original per CR 707.10. **Fix:** remove the incorrect sentence. |
| 2 | LOW | `entwine.rs` | **Missing commander tax stacking test.** Plan specified `test_entwine_stacks_with_commander_tax` (CR 601.2f + CR 903.8) but it was replaced with a flag-check test. **Fix:** add the commander tax test in a future pass, or document the gap. |

### Finding Details

#### Finding 1: Doc comment contradicts copy propagation

**Severity**: MEDIUM
**File**: `crates/engine/src/state/stack.rs:228`
**CR Rule**: 707.10 -- "To copy a spell... means to put a copy of it onto the stack; a copy of a spell isn't cast and a copy of an activated ability isn't activated. A copy of a spell or ability copies both the characteristics of the spell or ability and all decisions made for it, including modes, targets, the value of X, and additional or alternative costs."
**Issue**: The doc comment on `StackObject::was_entwined` (line 228) states "Must always be false for copies (`is_copy: true`) -- copies are not cast." However, `copy.rs:225` correctly sets `was_entwined: original.was_entwined` because CR 707.10 mandates that copies preserve all decisions including modes and additional costs. The entwine decision (choosing all modes) is exactly the kind of decision that copies inherit. The doc comment is misleading and could cause a future contributor to "fix" the propagation, introducing a bug.
**Fix**: Change the doc comment at `stack.rs:228` from "Must always be false for copies (`is_copy: true`) -- copies are not cast." to "Propagated to copies per CR 707.10 (copies copy all decisions including modes and additional costs)."

#### Finding 2: Missing commander tax stacking test

**Severity**: LOW
**File**: `crates/engine/tests/entwine.rs`
**CR Rule**: 601.2f + 903.8
**Issue**: The plan specified test 6 as `test_entwine_stacks_with_commander_tax` (verifying total cost = base + commander tax + entwine). The implementation replaced it with `test_entwine_was_entwined_flag_on_stack`, which is a valid test but does not cover the commander tax interaction. The cost pipeline is validated by `test_entwine_insufficient_mana_rejected` for the base+entwine case, but the three-way cost stacking (base + tax + entwine) is untested.
**Fix**: Add the commander tax test in a future pass or during the card definition phase. Not blocking since the cost pipeline is well-tested for other additional costs with commander tax.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.42a (entwine is additional cost, choose all modes) | Yes | Yes | test_entwine_basic_both_modes_execute, test_entwine_insufficient_mana_rejected |
| 702.42a (validation: spell must have entwine) | Yes | Yes | test_entwine_no_keyword_rejected |
| 702.42b (modes in printed order) | Yes | Yes | test_entwine_modes_in_printed_order, test_entwine_basic_both_modes_execute |
| 702.42b (state from mode N visible to mode N+1) | Yes | Partial | test_entwine_modes_in_printed_order checks both effects visible, but doesn't use a mode-interdependent card (e.g., mode 0 adds counters, mode 1 counts them). Acceptable given GainLife+DrawCards are order-independent. |
| 601.2f (additional cost added to total) | Yes | Yes | test_entwine_insufficient_mana_rejected |
| 707.10 (copies preserve entwine choice) | Yes | No | copy.rs:225 propagates correctly but no test covers it |
| Auto-mode[0] fallback (batch plan stub) | Yes | Yes | test_entwine_not_paid_only_first_mode |
| Commander tax stacking (903.8 + 601.2f) | Yes (pipeline) | No | Cost pipeline handles it generically but no entwine-specific test |
| was_entwined flag on StackObject | Yes | Yes | test_entwine_was_entwined_flag_on_stack |
| Hash coverage (KW 110, AbilDef 39, StackObject) | Yes | N/A | Verified in hash.rs |
| Harness support (cast_spell_entwine) | Yes | N/A | replay_harness.rs:1390 |
