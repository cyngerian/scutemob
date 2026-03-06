# Ability Review: Cleave

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.148
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Cleave, AltCostKind::Cleave)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Cleave, Condition::WasCleaved)
- `crates/engine/src/state/hash.rs` (discriminants 108, 37, 11)
- `crates/engine/src/state/stack.rs` (was_cleaved field)
- `crates/engine/src/rules/casting.rs` (cleave cost lookup, validation, StackObject construction)
- `crates/engine/src/rules/resolution.rs` (was_cleaved propagation to EffectContext)
- `crates/engine/src/effects/mod.rs` (EffectContext.was_cleaved, Condition::WasCleaved evaluation)
- `crates/engine/src/rules/copy.rs` (was_cleaved on copies)
- `crates/engine/tests/cleave.rs` (7 tests)

## Verdict: needs-fix

One MEDIUM finding: the test suite cannot distinguish between the cleaved and un-cleaved
branches because both mock card definitions use identical effects in their `if_true` and
`if_false` branches. The implementation logic is correct, but the tests do not actually
verify that `Condition::WasCleaved` routes to different behavior. One LOW finding for
missing defense-in-depth alt cost exclusivity checks.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `tests/cleave.rs:128-158` | **Mock card definitions have identical branches; tests cannot verify condition routing.** Fix: make branches differ. |
| 2 | LOW | `casting.rs:1566-1607` | **Missing defense-in-depth alt cost exclusivity checks.** Cleave only checks 5 of 17 alt costs. |

### Finding Details

#### Finding 1: Mock card definitions have identical Conditional branches

**Severity**: MEDIUM
**File**: `crates/engine/tests/cleave.rs:128-158` (mock_path_of_peril_def) and `:206-209` (mock_fierce_retribution_def)
**CR Rule**: 702.148a -- "If this spell's cleave cost was paid, change its text by removing all text found within square brackets"
**Issue**: Both `mock_path_of_peril_def()` and `mock_fierce_retribution_def()` define `Effect::Conditional` with `Condition::WasCleaved`, but the `if_true` and `if_false` branches are functionally identical. In `mock_path_of_peril_def`, both branches use `ForEach` over all creatures with `has_card_type: Some(CardType::Creature)` -- no MV filter on either. In `mock_fierce_retribution_def`, both branches use `DestroyPermanent { target: DeclaredTarget { index: 0 } }`. Tests 4, 5, and 7 assert that creatures are destroyed, but the same result would occur regardless of which branch was taken. The tests pass even if `Condition::WasCleaved` always returned `true` or always returned `false`. This means the Conditional branching logic is untested.
**Fix**: Make the mock card definitions use genuinely different effects in the two branches. For example, `mock_path_of_peril_def` `if_false` branch could use `Effect::Nothing` (or a no-op) while `if_true` uses the ForEach destroy-all. Then test 7 (boardwipe) verifies that casting with cleave destroys all creatures (if_true), and a NEW test casting without cleave verifies no creatures are destroyed (if_false). Alternatively, use two different effects (e.g., `DealDamage` with different amounts) whose outcomes can be independently verified.

#### Finding 2: Missing defense-in-depth alt cost exclusivity checks

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:1566-1607`
**CR Rule**: 118.9a -- "A player can't apply two alternative methods of casting or two alternative costs to a single spell."
**Issue**: The cleave validation block only checks exclusivity against flashback, overload, surge, spectacle, and emerge (5 checks). It does not check against evoke, bestow, madness, miracle, escape, foretell, retrace, jump-start, aftermath, dash, blitz, plot, or impending (12 missing checks). Similarly, the existing alt cost validation blocks for those 12 abilities do not check against cleave. This is not a correctness bug because `alt_cost: Option<AltCostKind>` structurally enforces mutual exclusivity (only one variant can be selected). However, it deviates from the pattern established by Dash/Blitz/Plot/Impending which check against ALL other alt costs.
**Fix**: Add the missing defense-in-depth checks to the cleave validation block (check all other `casting_with_X` flags), and add `casting_with_cleave` checks to the other alt cost validation blocks. This can be deferred as a LOW batch cleanup.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.148a (alt cost) | Yes | Yes | Tests 1, 2, 6 verify cost payment and validation |
| 702.148a (text change) | Yes | Weak | Tests 4, 5, 7 use Conditional but branches are identical (Finding 1) |
| 702.148a (601.2b/601.2f-h) | Yes | Yes | Test 3 verifies mutual exclusivity error |
| 702.148b (text-changing effect) | Yes | No | Modeled as Condition::WasCleaved; no separate test for CR 612 interaction |
| Copy interaction (707.2) | Yes | No | Correctly sets `was_cleaved: false` on copies per CR 707.2 (text-changing effects not copied); no test |
| Commander tax (118.9d) | Yes | No | Handled by existing casting pipeline; no cleave-specific test |
