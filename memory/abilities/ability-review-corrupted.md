# Ability Review: Corrupted

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 207.2c (ability word -- no dedicated CR section)
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs:1203-1208`
- `crates/engine/src/state/hash.rs:3235-3239`
- `crates/engine/src/effects/mod.rs:3121-3127`
- `crates/engine/src/rules/replacement.rs:946-963`
- `crates/engine/tests/corrupted.rs` (full file, 497 lines)

## Verdict: clean

The Corrupted ability word implementation is correct. The `Condition::OpponentHasPoisonCounters(u32)`
variant correctly models the "if an opponent has N or more poison counters" condition. The check
iterates all players, excludes the controller, excludes eliminated players (`has_lost == true`),
and uses `>=` for the threshold comparison. Hash discriminant 12 is unique among Condition
variants (0-11 were already taken). All six tests are meaningful, well-structured, and cover the
key edge cases (threshold met, below threshold, any-opponent multiplayer, controller exclusion,
eliminated opponent exclusion, boundary value). Two LOW findings are noted below but neither
affects correctness.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `replacement.rs:950-958` | **Duplicated condition logic.** The inline `match cond` duplicates the logic from `effects/mod.rs:3055 check_condition()`. **Fix:** Make `check_condition` `pub(crate)` and call it from replacement.rs instead of inlining. |
| 2 | LOW | `replacement.rs:958` | **Wildcard catch-all defaults to true.** The `_ => true` arm means any future Condition variant used as an `intervening_if` on a WhenEntersBattlefield trigger will silently pass. **Fix:** When more conditions are wired into the inline ETB path, replace `_ => true` with explicit arms or call the shared `check_condition` function (see Finding 1). |

### Finding Details

#### Finding 1: Duplicated condition logic in replacement.rs

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:950-958`
**CR Rule**: 207.2c / 603.4 -- intervening-if condition check
**Issue**: The `OpponentHasPoisonCounters` check is implemented identically in two places:
(1) `effects/mod.rs:3124-3127` in `check_condition()`, and (2) `replacement.rs:951-954` inline
in `fire_when_enters_triggered_effects`. The duplication exists because `check_condition` is
private to the effects module. While both copies are currently identical and correct, future
changes to one could create a divergence.
**Fix**: Make `check_condition` in `effects/mod.rs` `pub(crate)` and call it from
`replacement.rs` instead of inlining the match. This requires constructing an `EffectContext`
before the condition check, which the code already does at line 964-965 -- just move it earlier.
This is a code quality improvement, not a correctness fix. Defer to LOW remediation.

#### Finding 2: Wildcard catch-all in replacement.rs intervening-if

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:958`
**CR Rule**: 603.4 -- intervening-if conditions must be checked
**Issue**: The `_ => true` arm means any `Condition` variant not explicitly handled in this
inline match will be treated as "always true," causing the trigger to fire unconditionally.
Currently this is acceptable because `OpponentHasPoisonCounters` is the only Condition used
as an `intervening_if` on `WhenEntersBattlefield` triggers. However, if a future card uses
a different Condition (e.g., `ControllerLifeAtLeast`) as an intervening-if on an ETB trigger,
it would silently pass without being checked.
**Fix**: When Finding 1 is addressed (making `check_condition` pub(crate)), this catch-all
becomes unnecessary. Until then, document the assumption that only
`OpponentHasPoisonCounters` uses this path, and add a `debug_assert!` or log warning on
the wildcard arm.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 207.2c (ability word, no rules meaning) | Yes | Yes | Correctly modeled as Condition, not KeywordAbility |
| "an opponent has 3+ poison" threshold | Yes | Yes | test 1 (=3 passes), test 2 (=2 fails), test 6 (boundary) |
| "an opponent" = any opponent (multiplayer) | Yes | Yes | test 3: 4-player, only P3 has 3 poison |
| Controller excluded | Yes | Yes | test 4: P1 has 5 poison, P2 has 1, no fire |
| Eliminated opponents excluded | Yes | Yes | test 5: P2 eliminated with 10 poison, no fire |
| 603.4 intervening-if at trigger time | Yes | Yes | All tests check inline in fire_when_enters_triggered_effects |
| 603.4 intervening-if at resolution time | Partial | No | check_condition arm exists in effects/mod.rs but tests only exercise the inline ETB path. Deferred: no standard way to remove poison counters between trigger and resolution. |
| Static corrupted ("as long as...") | No | No | Deferred per plan -- requires conditional continuous effects infrastructure |

## Previous Findings (re-review only)

N/A -- first review.
