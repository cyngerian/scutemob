# Ability Review: Bolster

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.39
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs:380-394` (Effect::Bolster variant)
- `crates/engine/src/effects/mod.rs:971-1036` (execution logic)
- `crates/engine/src/state/hash.rs:2696-2701` (hash coverage)
- `crates/engine/tests/bolster.rs` (8 tests, 453 lines)

## Verdict: needs-fix

The implementation correctly follows CR 701.39a in all substantive ways: layer-aware toughness
comparison, non-targeting semantics, controller-only creature pool, deterministic tie-breaking,
and graceful no-op when no creatures exist. Hash coverage is complete. Tests are comprehensive
and well-structured. There are two `.unwrap()` calls in engine library code that are logically
unreachable but violate the project convention ("never `unwrap()` in engine logic"), which is
a MEDIUM per the architecture invariants.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `effects/mod.rs:1009` | **`.unwrap()` in engine library code (min toughness).** Logically unreachable but violates convention. **Fix:** use `.unwrap_or(0)` or refactor. |
| 2 | MEDIUM | `effects/mod.rs:1018` | **`.unwrap()` in engine library code (chosen creature).** Logically unreachable but violates convention. **Fix:** use `if let Some(id) = ...` or `.unwrap_or`. |
| 3 | LOW | `tests/bolster.rs` | **Missing test for Bolster 0.** The early-return path at line 977-980 is untested. |

### Finding Details

#### Finding 1: `.unwrap()` on min toughness in engine library code

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:1009`
**CR Rule**: N/A -- architecture invariant
**Issue**: The call `creatures.iter().map(|(_, t)| *t).min().unwrap()` uses `.unwrap()` in
engine library code. While the `creatures.is_empty()` guard at line 1003 makes this logically
unreachable, the project convention in `memory/conventions.md` states: "Engine crate uses typed
errors -- never `unwrap()` or `expect()` in engine logic." This is the only occurrence of bare
`.unwrap()` in the entire `effects/mod.rs` file (along with Finding 2). Every other potential
`None` in this file uses `.unwrap_or()`, `.unwrap_or_default()`, or `.unwrap_or_else()`.
**Fix**: Replace `.unwrap()` with `.unwrap_or(0)`. Alternatively, combine the empty check and
min calculation: `let Some(min_toughness) = creatures.iter().map(|(_, t)| *t).min() else { continue; };`
which eliminates both the explicit `is_empty()` check and the `.unwrap()`.

#### Finding 2: `.unwrap()` on chosen creature in engine library code

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:1018`
**CR Rule**: N/A -- architecture invariant
**Issue**: The call `.min_by_key(|id| id.0).unwrap()` uses `.unwrap()` in engine library code.
The filter at line 1015 guarantees at least one element (since `min_toughness` came from the
same collection), but this still violates the convention. This is the second of only two bare
`.unwrap()` calls in the entire file.
**Fix**: Use `let Some(chosen_id) = creatures.iter().filter(...).map(...).min_by_key(...) else { continue; };`
or wrap the entire min-toughness + chosen-id block in a single `if let` chain.

#### Finding 3: Missing test for Bolster 0

**Severity**: LOW
**File**: `crates/engine/tests/bolster.rs`
**CR Rule**: 701.39a -- "Put N +1/+1 counters on that creature" (N=0 means no counters)
**Issue**: The implementation has an early return for `n == 0` at line 977-980, but no test
validates this path. While "Bolster 0" is not a common card text, `EffectAmount` could
evaluate to 0 at runtime (e.g., `EffectAmount::PowerOf` on a 0-power creature). A test
would document the intended no-op behavior and protect against regressions.
**Fix**: Add a test `test_bolster_zero_does_nothing` that verifies no events are emitted
and no counters are placed when bolster is called with count 0.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.39a "Choose a creature you control with the least toughness" | Yes | Yes | test_bolster_chooses_least_toughness |
| 701.39a "tied for least toughness" | Yes | Yes | test_bolster_tied_toughness_deterministic |
| 701.39a "Put N +1/+1 counters on that creature" | Yes | Yes | test_bolster_basic_single_creature |
| 701.39a no creatures edge case | Yes | Yes | test_bolster_no_creatures_does_nothing |
| Ruling: layer-aware toughness at resolution | Yes | Yes | test_bolster_uses_layer_aware_toughness |
| Ruling: does not target (protection irrelevant) | Yes | Yes | test_bolster_not_targeting_ignores_protection |
| Edge: source can receive counters | Yes | Yes | test_bolster_can_target_source |
| Multiplayer: controller's creatures only | Yes | Yes | test_bolster_multiplayer_only_controllers_creatures |
| Edge: Bolster 0 (no-op) | Yes (early return) | No | Missing test (Finding 3) |
| Edge: animated non-creatures eligible | Yes (layer-aware card_types) | No | Not tested but code is correct |

## Additional Notes

- **Plan vs implementation discrepancy (hash.rs)**: The plan stated "No hash.rs update needed"
  but the implementer correctly added the hash arm anyway, matching the existing pattern where
  every `Effect` variant has an explicit hash discriminant. The implementer was right; the plan
  was wrong on this point.

- **Code quality (positive)**: The implementation uses `calculate_characteristics` for both
  toughness comparison AND creature type checking (line 993-997), which is superior to many
  other effects in the codebase that check raw `obj.characteristics.card_types`. This correctly
  handles animated non-creatures (e.g., Mutavault, man-lands).

- **CR citation quality (positive)**: Both the `Effect::Bolster` variant doc comment and the
  execution logic cite CR 701.39a and the 2014-11-24 ruling. Test file header cites all key
  rules verified.

- **Serialization**: `Effect::Bolster` automatically derives `Serialize`/`Deserialize` from
  the parent enum's derive, and its fields (`PlayerTarget`, `EffectAmount`) both implement
  these traits. No manual serialization needed.
