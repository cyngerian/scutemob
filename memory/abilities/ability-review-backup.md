# Ability Review: Backup

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.165
**Files reviewed**: `crates/engine/src/state/types.rs:1144-1152`, `crates/engine/src/state/hash.rs:629-633,1822-1835,1232-1286`, `crates/engine/src/state/stubs.rs:93-96,305-316`, `crates/engine/src/state/stack.rs:990-1008`, `crates/engine/src/rules/abilities.rs:2280-2349,4499-4517`, `crates/engine/src/rules/resolution.rs:2741-2815,4727-4735`, `tools/replay-viewer/src/view_model.rs:564-566,836`, `tools/tui/src/play/panels/stack_view.rs:174-176`, `crates/engine/tests/backup.rs`

## Verdict: needs-fix

The implementation correctly handles the core Backup mechanic -- ETB trigger detection, abilities-below filtering, Backup keyword exclusion, abilities snapshot at trigger time (CR 702.165d), self-targeting counter placement, and counter/fizzle arms. However, there is one HIGH finding (missing hash fields on PendingTrigger) and one MEDIUM finding (the "target another creature" ability-granting path through the engine is completely untested). The "another creature" path in resolution.rs appears correct on inspection but has zero integration test coverage.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `hash.rs:1232-1286` | **Missing hash fields.** `backup_abilities` and `backup_n` not hashed in PendingTrigger. **Fix:** add both fields to hash. |
| 2 | MEDIUM | `tests/backup.rs` | **No integration test for "another creature" path.** Ability-granting CE creation (resolution.rs:2787-2807) never exercised. **Fix:** add test with pre-placed target creature. |
| 3 | LOW | `tests/backup.rs:4,6` | **Tests replicate engine logic instead of testing through engine.** Tests 4, 5, 6, 10 manually replicate the filtering logic rather than observing engine output. |

### Finding Details

#### Finding 1: Missing hash fields for backup_abilities and backup_n on PendingTrigger

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:1286` (end of PendingTrigger hash impl)
**CR Rule**: 702.165d -- "The abilities that a backup ability grants are determined as the ability is put on the stack."
**Architecture Invariant**: Hash coverage -- all state-significant fields must be hashed for deterministic state verification.
**Issue**: The `PendingTrigger::backup_abilities` and `PendingTrigger::backup_n` fields are not included in the `HashInto` implementation for `PendingTrigger`. Two pending triggers with different Backup N values or different ability lists would produce identical hashes, breaking distributed state verification. This is the same category as the architecture invariant "Hash coverage for new fields" cited in CLAUDE.md's Critical Gotchas.
**Fix**: Add the following two lines before the closing brace of `PendingTrigger`'s `hash_into` method at line 1286:
```rust
// CR 702.165a: backup-specific fields
self.backup_abilities.hash_into(hasher);
self.backup_n.hash_into(hasher);
```
**Note**: Pre-existing gap -- `echo_cost`, `cumulative_upkeep_cost`, `recover_cost`, `recover_card` are also missing from PendingTrigger's hash (Batch 8). Consider fixing all in one pass.

#### Finding 2: No integration test for the "target another creature" path

**Severity**: MEDIUM
**File**: `crates/engine/tests/backup.rs`
**CR Rule**: 702.165a -- "If that's another creature, it also gains the non-backup abilities of this creature printed below this one until end of turn."
**Issue**: The deterministic bot always self-targets (flush_pending_triggers line 4503: `let target = trigger.source`). This means the continuous effect creation logic in resolution.rs:2787-2807 -- which creates a Layer 6 UntilEndOfTurn effect granting keyword abilities -- is never exercised by any test. The code looks correct on inspection, but untested resolution logic is fragile. A test should place a second creature on the battlefield and directly construct a `BackupTrigger` StackObject targeting it (bypassing the flush default), then verify: (a) counters placed on target, (b) abilities granted via CE, (c) abilities expire at end of turn.
**Fix**: Add a test `test_backup_another_creature_gets_counters_and_abilities` that manually pushes a `StackObjectKind::BackupTrigger` onto the stack with `target_creature` pointing to a different creature and `abilities_to_grant` containing `[Flying, FirstStrike]`, then resolves it and verifies: the target has N +1/+1 counters, a `ContinuousEffect` with `EffectFilter::SingleObject(target)` and `LayerModification::AddKeywords` exists, and the CE has `EffectDuration::UntilEndOfTurn`.

#### Finding 3: Tests replicate engine logic instead of testing through engine

**Severity**: LOW
**File**: `crates/engine/tests/backup.rs` (tests 4, 5, 6, 10)
**Issue**: Tests 4 (`test_backup_does_not_include_backup_keyword_in_grant`), 5 (`test_backup_multiple_instances_trigger_separately` -- fine, this one does go through engine), 6 (`test_backup_only_abilities_below_are_granted`), and 10 (`test_backup_abilities_locked_at_trigger_time`) manually replicate the filtering logic from `check_triggers` rather than observing engine behavior. If the engine's actual filtering logic diverges from the test's copy, the tests would still pass. This is a minor concern since the filter logic is simple, but ideally tests should observe engine output.
**Fix**: No immediate action needed. When Finding 2 is addressed with a proper integration test, these become less critical.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.165a (ETB trigger) | Yes | Yes | test_backup_etb_generates_trigger |
| 702.165a (N counters) | Yes | Yes | test_backup_n_counters_quantity, test_backup_self_target_gets_counters_only |
| 702.165a (another creature = abilities) | Yes | **No** | Resolution code exists but never exercised (Finding 2) |
| 702.165a (self = no abilities) | Yes | Yes | test_backup_self_target_gets_counters_only, test_backup_trigger_stack_object_structure |
| 702.165a (non-backup abilities only) | Yes | Yes (static) | test_backup_does_not_include_backup_keyword_in_grant |
| 702.165a (printed below) | Yes | Yes (static) | test_backup_only_abilities_below_are_granted |
| 702.165b (copy order maintained) | N/A | N/A | Handled by CardDefinition ordering; no separate engine logic needed |
| 702.165c (only printed abilities) | Yes | Yes (static) | Reads from CardDefinition, not layer-resolved characteristics |
| 702.165d (abilities locked at trigger time) | Yes | Yes (static) | test_backup_abilities_locked_at_trigger_time; stored on PendingTrigger |
| Multiple instances | Yes | Yes | test_backup_multiple_instances_trigger_separately |
| Fizzle (target gone) | Yes | Partial | Resolution has fizzle check; no dedicated fizzle test |
| Counter/Stifle | Yes | No | BackupTrigger in counter arm (resolution.rs:4732); no test |

## Previous Findings (re-review only)

N/A -- first review.
