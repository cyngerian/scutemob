# Ability Review: Adapt

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.46
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 560-571)
- `crates/engine/src/state/hash.rs` (lines 445-449, 2431-2435)
- `crates/engine/src/cards/card_definition.rs` (lines 786-793)
- `crates/engine/src/effects/mod.rs` (lines 2731-2738)
- `tools/replay-viewer/src/view_model.rs` (line 673)
- `crates/engine/src/state/builder.rs` (verified no exhaustive match on KeywordAbility)
- `crates/engine/tests/adapt.rs` (full file, 593 lines)

## Verdict: clean

The Adapt implementation is correct, complete, and well-tested. The CR rule 701.46a
is simple (only one subrule) and the implementation faithfully models it as an activated
ability with a `Conditional` effect that checks `SourceHasNoCountersOfType` at resolution
time. All edge cases identified in the plan and card rulings are covered by tests. The
enum variant, hash discriminant, view model display, and condition evaluation are all
properly wired. No HIGH or MEDIUM findings. Two LOW findings for minor improvements.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `crates/engine/tests/adapt.rs:357` | **Missing insufficient-mana negative test.** No test verifies that activation fails when the player has insufficient mana. **Fix:** Add a test where P1 has 0 mana, attempts ActivateAbility, and asserts the command returns an error. |
| 2 | LOW | `crates/engine/tests/adapt.rs:416` | **Direct execute_effect bypasses command pipeline.** Test 4 uses `execute_effect` directly to remove counters rather than using a Command. This is an established test pattern in the codebase (used in replacement_effects tests) so it is not a violation, but it means the counter removal is not exercised through the normal command pipeline. **Fix:** No change required -- this is an accepted test pattern. Noted for completeness. |

### Finding Details

#### Finding 1: Missing insufficient-mana negative test

**Severity**: LOW
**File**: `crates/engine/tests/adapt.rs` (absent test)
**CR Rule**: 602.2 -- "To activate an activated ability, a player [...] pays any costs required."
**Issue**: The plan specified test 5 should verify "If P1 had insufficient mana, activation
would fail" but the actual test only checks the positive case (sufficient mana, pool drained).
There is no negative test asserting that `ActivateAbility` returns `Err` when the player
cannot pay the mana cost. While this is thoroughly tested elsewhere in the engine's activated
ability infrastructure, a dedicated negative case here would improve coverage completeness
for the Adapt ability specifically.
**Fix**: Add a seventh test `test_adapt_insufficient_mana_fails` that creates a state with
0 mana available, attempts `Command::ActivateAbility`, and asserts the result is `Err`.

#### Finding 2: Direct execute_effect usage in test 4

**Severity**: LOW
**File**: `crates/engine/tests/adapt.rs:416`
**CR Rule**: N/A -- test methodology observation
**Issue**: Test 4 (`test_adapt_after_losing_counters`) calls `execute_effect` directly to
remove +1/+1 counters from the creature, bypassing the command pipeline. This is functionally
correct and is an established pattern used in 10+ tests in `replacement_effects.rs`. However,
the test also directly mutates the `ManaPool` via `state.players.get_mut()` (lines 426-437)
which bypasses the event system. This is acceptable in tests (conventions allow `unwrap()` in
tests and direct state setup) but means no `CounterRemoved` or `ManaAdded` events are emitted
for these intermediate steps.
**Fix**: No change required. This is standard test practice in this codebase.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.46a ("Adapt N" definition) | Yes | Yes | test_adapt_basic_adds_counters, test_adapt_counter_added_event_emitted |
| 701.46a (no counters placed if counters present) | Yes | Yes | test_adapt_does_nothing_with_existing_counters |
| 701.46a (activation always legal) | Yes | Yes | test_adapt_activation_always_legal |
| 701.46a (re-adapt after losing counters) | Yes | Yes | test_adapt_after_losing_counters |
| Cost payment at activation | Yes | Yes | test_adapt_pays_mana_cost |
| CounterAdded event emission | Yes | Yes | test_adapt_counter_added_event_emitted |
| P/T modification via counters | Yes | Yes | test_adapt_basic_adds_counters (layer-aware check) |

## Implementation Quality Summary

**Enum variant** (`types.rs:560-571`): Correctly placed after Hideaway. Doc comment cites
CR 701.46, explains the marker purpose, and documents the resolution-time check. The `u32`
parameter follows the established pattern of Modular, Crew, Afterlife, etc.

**Hash coverage** (`hash.rs:445-449, 2431-2435`): Both `KeywordAbility::Adapt(n)` (discriminant
67) and `Condition::SourceHasNoCountersOfType` (discriminant 8) have unique discriminants with
no collisions. The inner `n` and `counter` values are correctly hashed.

**Condition evaluation** (`effects/mod.rs:2731-2738`): The implementation correctly checks for
zero counters of the specified type. The `unwrap_or(0)` handles the case where the counter
type is absent from the map (no entry = 0 counters = condition true). The `unwrap_or(true)`
handles the case where the source object no longer exists (safe default -- AddCounter will
silently no-op). Both defaults are correct per CR 701.46a.

**View model** (`view_model.rs:673`): `format!("Adapt {n}")` is correct display text.

**Builder.rs**: No arm needed. The keyword loop uses `if let`/`if matches!` pattern matching,
not an exhaustive `match`. Keywords that don't generate auto-triggers are simply skipped.

**Tests** (`tests/adapt.rs`): Six well-structured tests covering all CR 701.46a behavior.
Each test cites the relevant CR rule or ruling. The four-player setup confirms multiplayer
correctness. The helper function `adapt_ability()` correctly models the Conditional effect
with SourceHasNoCountersOfType. Tests verify both state changes (counter counts, P/T) and
event emissions (CounterAdded, ManaCostPaid).
