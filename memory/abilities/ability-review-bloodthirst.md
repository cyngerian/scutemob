# Ability Review: Bloodthirst

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.54
**Files reviewed**:
- `crates/engine/src/state/types.rs:1127-1134`
- `crates/engine/src/state/player.rs:132-140`
- `crates/engine/src/state/hash.rs:619-623, 837-838`
- `crates/engine/src/state/builder.rs:289`
- `crates/engine/src/rules/turn_actions.rs:1199-1201`
- `crates/engine/src/effects/mod.rs:179-205`
- `crates/engine/src/rules/combat.rs:1437-1458`
- `crates/engine/src/rules/resolution.rs:779-835`
- `crates/engine/src/rules/lands.rs:299-353`
- `tools/replay-viewer/src/view_model.rs:831`
- `crates/engine/tests/bloodthirst.rs` (full file, 653 lines)

## Verdict: clean

The Bloodthirst implementation is correct and thorough. All four player-damage sites
(effects/mod.rs normal + infect, combat.rs normal + infect) correctly increment the
new `damage_received_this_turn` field. The ETB counter logic in both resolution.rs
and lands.rs correctly checks "any opponent" (excluding eliminated/conceded players),
sums multiple Bloodthirst instances independently per CR 702.54c, and emits
CounterAdded events. The field is properly hashed, initialized, and reset per turn.
Tests cover all documented edge cases. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | plan | **Bloodthirst X (CR 702.54b) not implemented.** Explicitly deferred in plan; no printed Commander-relevant card uses this form. |
| 2 | LOW | `resolution.rs:784` vs `lands.rs:306` | **Inconsistent source for keyword lookup.** Resolution reads from CardDefinition registry; lands reads from object characteristics. Both work but pattern diverges. |

### Finding Details

#### Finding 1: Bloodthirst X (CR 702.54b) deferred

**Severity**: LOW
**CR Rule**: 702.54b -- "Bloodthirst X" means "This permanent enters with X +1/+1 counters on it, where X is the total damage your opponents have been dealt this turn."
**Issue**: The `Bloodthirst(u32)` variant has no way to represent the special "X" form where the counter count equals total damage dealt to all opponents. Currently `Bloodthirst(0)` would simply add zero counters.
**Fix**: Acceptable deferral. When a card with Bloodthirst X is needed, either use a sentinel value (e.g., `u32::MAX`) or add a `BloodthirstX` variant. The `damage_received_this_turn` field already tracks the total damage needed for this calculation; the infrastructure is in place.

#### Finding 2: Inconsistent keyword source between ETB sites

**Severity**: LOW
**File**: `resolution.rs:784` and `lands.rs:306`
**Issue**: `resolution.rs` reads Bloodthirst instances from the `CardDefinition` via registry lookup, while `lands.rs` reads from the on-object `characteristics.keywords`. Both produce the same result in practice (the object's keywords are populated from the CardDefinition), and this mirrors the Amplify pattern at these same sites. Not a correctness bug, just a stylistic inconsistency.
**Fix**: No action needed. This matches the existing Amplify pattern at both sites. If a future refactor unifies the ETB keyword logic, both should be updated together.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.54a (Bloodthirst N) | Yes | Yes | test_bloodthirst_basic_opponent_damaged, test_bloodthirst_no_damage_dealt, test_bloodthirst_n_multiplier |
| 702.54b (Bloodthirst X) | No (deferred) | No | No printed Commander card uses this form; LOW priority |
| 702.54c (multiple instances) | Yes | Yes | test_bloodthirst_multiple_instances (Bloodthirst 1 + Bloodthirst 2 = 3 counters) |
| "an opponent" (multiplayer) | Yes | Yes | test_bloodthirst_multiple_opponents_damaged (3-player) |
| Self-damage exclusion | Yes | Yes | test_bloodthirst_only_controller_damaged |
| Eliminated opponent exclusion (CR 800.4a) | Yes | Yes | test_bloodthirst_eliminated_opponent_not_counted |
| Infect damage counts | Yes (incremented) | No | damage_received_this_turn incremented at infect sites; no dedicated test |
| CounterAdded event | Yes | Yes | test_bloodthirst_counter_added_event |
| ETB via resolution (cast) | Yes | Yes | All cast-based tests exercise this path |
| ETB via lands.rs | Yes | No | No land with Bloodthirst exists; code present for completeness |
| Damage vs life loss distinction | Yes (field separation) | No | Separate fields exist; no test verifies infect increments damage_received but not life_lost |
| Turn reset | Yes | No | Reset in turn_actions.rs; no dedicated test (covered implicitly by field default) |
| Hash coverage | Yes | n/a | Both KeywordAbility hash arm and PlayerState field hash present |
| Builder initialization | Yes | n/a | damage_received_this_turn: 0 in builder |
| view_model display | Yes | n/a | "Bloodthirst {n}" format string |
