# Ability Review: Emerge

**Date**: 2026-03-03
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.119
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 112-114, 906-912)
- `crates/engine/src/cards/card_definition.rs` (lines 392-399)
- `crates/engine/src/state/hash.rs` (lines 537-538, 3236-3239)
- `crates/engine/src/rules/command.rs` (lines 175-185)
- `crates/engine/src/rules/engine.rs` (lines 95-116)
- `crates/engine/src/rules/casting.rs` (lines 70, 86, 805-807, 889-891, 978-980, 1089-1091, 1105-1257, 1372-1382, 1953-1965, 3482-3548)
- `crates/engine/src/testing/replay_harness.rs` (lines 253-256, 1004-1028, all `emerge_sacrifice: None` sites)
- `crates/engine/src/testing/script_schema.rs` (lines 286-291)
- `crates/engine/tests/emerge.rs` (all 875 lines)
- `crates/engine/tests/script_replay.rs` (lines 154, 177)
- `tools/replay-viewer/src/view_model.rs` (line 762)

## Verdict: clean

The Emerge implementation is correct and thorough. All CR 702.119 subrules are implemented
faithfully. The alternative-cost pattern follows the established Bargain/Dash/Blitz/Impending
precedent. Validation is comprehensive (creature-type check, controller check, mutual exclusion
with all 15 other alternative costs, bidirectional cross-checks). The cost-reduction function
`reduce_cost_by_mv` correctly handles generic-first, then colorless, then colored pips in WUBRG
order, with a floor of zero. The sacrifice happens during cost payment (before the spell moves
to the stack), which is correct per CR 601.2h. The YAGNI decision to omit `was_emerged` from
StackObject/GameObject is sound -- no current cards check "if this spell's emerge cost was paid."
Ten well-structured tests cover all planned scenarios with CR citations.

No HIGH or MEDIUM findings. Two LOW findings noted below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `crates/engine/tests/emerge.rs:198` | **Generic mana used as colorless in test.** Test adds `ManaColor::Colorless` to pay the `generic: 2` portion of the reduced emerge cost, which works because the engine's mana payment system treats colorless mana as payable for generic costs, but the variable naming/semantics are slightly misleading. Not a correctness bug. |
| 2 | LOW | `crates/engine/tests/emerge.rs` | **No test for emerge + convoke interaction.** The plan (section "Interactions to Watch" item 2) identifies that emerge + convoke is a legal combination -- sacrifice one creature for emerge cost reduction, tap others for convoke. No test exists for this interaction. |

### Finding Details

#### Finding 1: Generic mana used as colorless in test

**Severity**: LOW
**File**: `crates/engine/tests/emerge.rs:198`
**CR Rule**: 202.1 -- generic mana symbol vs. colorless mana
**Issue**: The test adds `ManaColor::Colorless` (2 mana) to pay for the `generic: 2` portion
of the reduced emerge cost `{2}{U}{U}`. The engine correctly allows colorless mana to pay
generic costs (CR 117.6), so this is not a bug. However, using `ManaColor::Colorless` where
`generic` mana is being paid could be confusing to readers who might expect a different mana
type. Other tests in the codebase follow the same pattern, so this is consistent.
**Fix**: No fix required. Optionally, add a brief comment at line 198 noting that colorless
mana is used to pay the generic portion per CR 117.6.

#### Finding 2: No test for emerge + convoke interaction

**Severity**: LOW
**File**: `crates/engine/tests/emerge.rs`
**CR Rule**: 702.119a + 702.51a -- emerge is an alternative cost; convoke is a cost-reduction
mechanic (not an alternative cost). They can legally be combined.
**Issue**: The plan identifies emerge + convoke as a notable legal interaction (sacrifice one
creature for emerge, tap others for convoke). No test validates this combination works
correctly. Since convoke is already well-tested in isolation and emerge properly selects the
emerge cost as the base (which convoke then reduces), this is unlikely to fail, but a test
would document the interaction.
**Fix**: Optionally add a test `test_emerge_plus_convoke_legal_combination` that sets up an
emerge spell with convoke, sacrifices one creature for emerge, and taps another for convoke.
Not blocking -- this can be added when a card definition needs it.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.119a (emerge cost = alt cost + sacrifice creature) | Yes | Yes | test_emerge_basic_sacrifice_reduces_cost |
| 702.119a (cost reduced by sacrificed creature's MV) | Yes | Yes | test_emerge_basic_sacrifice_reduces_cost, test_emerge_sacrifice_high_mv_creature |
| 702.119a (sacrifice must be a creature) | Yes | Yes | test_emerge_sacrifice_must_be_creature |
| 702.119a (only caster's creatures) | Yes | Yes | test_emerge_sacrifice_must_be_own_creature |
| 702.119a (follows rules for alt costs CR 601.2b, 601.2f-h) | Yes | Yes | test_emerge_mutual_exclusion_with_flashback, test_emerge_sacrifice_without_emerge_altcost_fails |
| 702.119b (sacrifice at same time as cost payment) | Yes | Yes | Sacrifice at lines 1957-1965, before stack move at 1967 |
| CR 118.9a (mutual exclusion with other alt costs) | Yes | Yes (partial) | test_emerge_mutual_exclusion_with_flashback; all 15 alt costs checked in casting.rs |
| CR 202.3b (tokens have MV 0) | Yes | Yes | test_emerge_sacrifice_token_mv_zero |
| Cost floor (MV > emerge cost = free) | Yes | Yes | test_emerge_sacrifice_high_mv_creature |
| Normal cast without emerge | Yes | Yes | test_emerge_normal_cast_without_emerge |
| No-keyword rejection | Yes | Yes | test_emerge_no_keyword_rejects_emerge |
| Emerge without sacrifice rejected | Yes | Yes | test_emerge_without_sacrifice_fails |
| Sacrifice without emerge flag rejected | Yes | Yes | test_emerge_sacrifice_without_emerge_altcost_fails |
