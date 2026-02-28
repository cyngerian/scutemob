# Ability Review: Modular

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.43
**Files reviewed**:
- `crates/engine/src/rules/resolution.rs` (lines 309-351, 958-1010, 1101-1113)
- `crates/engine/src/state/builder.rs` (lines 643-661)
- `crates/engine/src/state/stubs.rs` (lines 141-157)
- `crates/engine/src/rules/abilities.rs` (lines 1178-1218, 1525-1593)
- `crates/engine/src/state/stack.rs` (lines 256-272)
- `crates/engine/src/state/hash.rs` (lines 423-427, 1228-1236, 990-1021)
- `tools/replay-viewer/src/view_model.rs` (lines 449-451, 647)
- `crates/engine/tests/modular.rs` (all 798 lines)

## Verdict: needs-fix

One HIGH finding: the `PendingTrigger` hash implementation is missing coverage for
the two new Modular fields (`is_modular_trigger`, `modular_counter_count`). This violates
the architecture invariant that all state fields must participate in hashing for
deterministic game state comparison, replay, and loop detection. One MEDIUM finding:
test 7 claims to test CR 702.43b (multiple instances work separately on death) but
only verifies ETB counters, leaving the two-trigger-on-death scenario completely untested.

All other aspects are correct: ETB counter placement follows CR 702.43a accurately,
the dies trigger uses pre_death_counters for last-known information (Arcbound Worker ruling),
target validation checks artifact + creature types, fizzle checking follows CR 608.2b,
no-target skipping follows CR 603.3d, and the StackObjectKind hash coverage is complete.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `hash.rs:1021` | **Missing PendingTrigger hash fields.** `is_modular_trigger` and `modular_counter_count` are not hashed. **Fix:** Add both fields to the `HashInto for PendingTrigger` impl. |
| 2 | MEDIUM | `modular.rs:631` | **Test 7 does not test CR 702.43b death triggers.** Only ETB is verified. **Fix:** Extend test or add a new test that kills a dual-Modular creature and asserts two ModularTrigger entries on the stack. |
| 3 | LOW | `abilities.rs:1181` | **String-based Modular detection is fragile.** Relies on `description.contains("Modular (CR 702.43a)")`. **Fix:** None required (matches established pattern for other abilities), but note for future robustness. |

### Finding Details

#### Finding 1: Missing PendingTrigger Hash Fields

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:1021`
**CR Rule**: N/A -- architecture invariant (hash coverage)
**Issue**: The `HashInto for PendingTrigger` implementation (lines 990-1021) hashes all
existing fields up through `is_exploit_trigger` but omits the two new Modular fields:
`is_modular_trigger: bool` and `modular_counter_count: Option<u32>`. Two `PendingTrigger`
values that differ only in their Modular fields would hash identically, breaking the
deterministic state hashing invariant required for loop detection, replay correctness,
and state comparison.

Every other `is_*_trigger` flag on `PendingTrigger` (evoke, madness, miracle, unearth,
exploit) is hashed. The Modular fields were simply missed.

**Fix**: Add these two lines before the closing brace of `HashInto for PendingTrigger`
(before line 1021):

```rust
// CR 702.43a: is_modular_trigger -- modular dies trigger marker
self.is_modular_trigger.hash_into(hasher);
self.modular_counter_count.hash_into(hasher);
```

#### Finding 2: Test 7 Missing Death-Trigger Verification for CR 702.43b

**Severity**: MEDIUM
**File**: `crates/engine/tests/modular.rs:631`
**CR Rule**: 702.43b -- "If a creature has multiple instances of modular, each one works separately."
**Issue**: `test_modular_multiple_instances_etb` verifies that a creature with Modular 1
and Modular 2 enters with 3 +1/+1 counters (ETB side of CR 702.43b). The doc comment
claims "On death, two separate Modular triggers fire" but the test never kills the
creature or checks for two triggers. The "each one works separately" rule for the
triggered ability half is completely untested.

This matters because the builder generates one `TriggeredAbilityDef` per Modular instance
(via the `for kw in spec.keywords.iter()` loop), and each trigger should independently
go on the stack at death. If the implementation were to accidentally deduplicate or
merge these triggers, no test would catch it.

**Fix**: Add a new test (or extend test 7) that:
1. Places a creature with Modular 1 + Modular 2 on the battlefield with 3 +1/+1 counters and lethal damage.
2. Also places an artifact creature target on the battlefield.
3. Passes priority to trigger SBAs.
4. Asserts exactly 2 `ModularTrigger` entries are on the stack.
5. Each trigger should carry `counter_count: 3` (the full pre_death_counters count, not just the individual N).

#### Finding 3: String-Based Modular Detection

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:1181`
**CR Rule**: 702.43a
**Issue**: The Modular trigger is detected at trigger-check time via
`trigger_def.description.contains("Modular (CR 702.43a)")`. This relies on the exact
string format set in `builder.rs:654`. If the description format ever changes (e.g.,
during a refactor or localization), the detection silently breaks and Modular triggers
would be treated as generic `TriggeredAbility` entries without counter data.

This pattern is consistent with how the project has handled similar abilities (checking
descriptions), so it is not a deviation from conventions. However, a more robust
approach would be to check the source object's keywords directly, or add a dedicated
field to `TriggeredAbilityDef` indicating the originating keyword.

**Fix**: No immediate fix required. This is an established pattern. If a refactor of
triggered ability identification is undertaken, Modular should be included.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.43a (static -- ETB counters) | Yes | Yes | test_modular_etb_counters, test_modular_etb_counters_n |
| 702.43a (triggered -- dies transfer) | Yes | Yes | test_modular_dies_transfers_counters |
| 702.43a (last-known info / pre_death_counters) | Yes | Yes | test_modular_dies_extra_counters |
| 702.43a (target: artifact creature) | Yes | Yes | test_modular_dies_no_artifact_creature_target |
| 702.43a (zero counters edge case) | Yes | Yes | test_modular_dies_zero_counters |
| 702.43a (0/0 base + counter survives) | Yes | Yes | test_modular_0_0_base_stats_survives_etb |
| 702.43b (multiple instances ETB) | Yes | Yes | test_modular_multiple_instances_etb |
| 702.43b (multiple instances death) | Yes | **No** | **Missing test -- see Finding 2** |
| 608.2b (fizzle check at resolution) | Yes | No | Implemented in resolution.rs:967-983 but no dedicated fizzle test |
| 603.3d (no legal target = no trigger) | Yes | Yes | test_modular_dies_no_artifact_creature_target |
