# Ability Review: Impending

**Date**: 2026-03-02
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.176
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 100-113, 870-891)
- `crates/engine/src/cards/card_definition.rs` (lines 383-391)
- `crates/engine/src/state/stack.rs` (lines 150-193, 760-773)
- `crates/engine/src/state/stubs.rs` (lines 30-75)
- `crates/engine/src/state/hash.rs` (lines 533-534, 1578-1586, 1656-1657, 3220-3225)
- `crates/engine/src/rules/casting.rs` (lines 83, 797-800, 876-878, 960-962, 988-1079, 1190-1193, 1767-1768, 1899, 1949, 3171-3211)
- `crates/engine/src/rules/resolution.rs` (lines 284-375, 1226-1279, 1753, 3307-3317)
- `crates/engine/src/rules/layers.rs` (lines 85-109)
- `crates/engine/src/rules/turn_actions.rs` (lines 302-355)
- `crates/engine/src/rules/abilities.rs` (lines 3679-3688)
- `crates/engine/src/rules/copy.rs` (lines 206-210, 398-399)
- `tools/tui/src/play/panels/stack_view.rs` (lines 128-129)
- `tools/replay-viewer/src/view_model.rs` (lines 521-522, 759)
- `crates/engine/tests/impending.rs` (full file, 1005 lines)

## Verdict: clean

The Impending implementation is correct and thorough. All four abilities described in CR 702.176a are faithfully implemented: (1) alternative cost on the stack, (2) ETB replacement effect adding time counters, (3) Layer 4 type-removal static ability, and (4) end-step triggered ability with intervening-if. Hash coverage is complete for all new fields and variants. All StackObject construction sites (17 total across casting.rs, resolution.rs, copy.rs, abilities.rs) include `was_impended: false`. Copy handling correctly prevents impending status from being inherited. Mutual exclusion with all 13 other alternative costs is fully wired. Commander tax correctly applies on top of the impending cost. The multiplayer-specific "your end step" constraint is implemented in turn_actions.rs by filtering on active player. The only findings are LOW-severity test gaps for edge cases that are nonetheless covered by the code paths and by tests for similar abilities.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/impending.rs` | **Missing intervening-if test.** Plan specified `test_impending_intervening_if` (CR 603.4) -- remove counters between trigger and resolution, verify trigger does nothing. Code at resolution.rs:1240-1248 handles this correctly, but there is no dedicated regression test. **Fix:** Add a test that queues the ImpendingCounter trigger, then manually removes all time counters before resolution, and asserts the trigger does nothing. |
| 2 | LOW | `tests/impending.rs` | **Missing copy test.** Plan specified `test_impending_copy_no_counters` (ruling 2024-09-20) -- verify a copy of an impending permanent has no time counters, `cast_alt_cost: None`, and IS a creature. Code at copy.rs:210 handles this correctly. **Fix:** Add a test creating a copy of an impending permanent and asserting the copy has 0 time counters and is a creature. |
| 3 | LOW | `tests/impending.rs` | **Missing zone-change reset test.** Plan specified `test_impending_zone_change_resets` (CR 400.7) -- blink an impending permanent and verify the new object has `cast_alt_cost: None`, no time counters, and IS a creature. Zone-change resets are already handled by the general `move_object_to_zone` code (which sets `cast_alt_cost: None`). **Fix:** Add a test that moves an impending permanent off and back onto the battlefield, verifying reset. |
| 4 | LOW | `tests/impending.rs` | **Missing Stifle test.** Plan specified `test_impending_stifle_counter_removal` (CR 702.176a + CR 603.4) -- counter the ImpendingCounterTrigger and verify the permanent retains its time counters. Code at resolution.rs:3307 handles this. **Fix:** Add a test that counters the trigger and verifies the permanent still has its time counters. |

### Finding Details

#### Finding 1: Missing intervening-if test

**Severity**: LOW
**File**: `crates/engine/tests/impending.rs`
**CR Rule**: 603.4 -- "the ability checks whether the stated condition is true [...] it checks the stated condition again as it resolves. If the condition isn't true at that time, the ability is removed from the stack and does nothing."
**Issue**: The plan called for a test that queues the end-step counter-removal trigger, then removes all time counters from the permanent before the trigger resolves, and verifies the trigger does nothing. The intervening-if recheck logic at resolution.rs:1240-1248 is correctly implemented, but there is no dedicated regression test.
**Fix**: Add `test_impending_intervening_if` -- set up a permanent with `cast_alt_cost == Impending` and 1 time counter, advance to end step to queue the trigger, then manually set counters to 0 before resolution, and assert the trigger does nothing (counters remain at 0, no `CounterRemoved` event emitted).

#### Finding 2: Missing copy test

**Severity**: LOW
**File**: `crates/engine/tests/impending.rs`
**CR Rule**: Ruling 2024-09-20 -- "If an object enters as a copy of a permanent that was cast with its impending cost, it won't enter with time counters, and it will be a creature."
**Issue**: The copy path at copy.rs:206-210 correctly sets `was_impended: false` with a CR comment and ruling citation, but there is no test exercising this path for impending specifically.
**Fix**: Add `test_impending_copy_no_counters` -- create a copy of an impending permanent on the battlefield and assert it has 0 time counters, `cast_alt_cost: None`, and IS a creature via `calculate_characteristics`.

#### Finding 3: Missing zone-change reset test

**Severity**: LOW
**File**: `crates/engine/tests/impending.rs`
**CR Rule**: 400.7 -- "An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence."
**Issue**: The plan called for a blink test verifying that zone changes reset impending status. The general `move_object_to_zone` code already handles this (sets `cast_alt_cost: None`), but there is no impending-specific regression test.
**Fix**: Add `test_impending_zone_change_resets` -- move an impending permanent off the battlefield and back, verify `cast_alt_cost: None`, 0 time counters, and IS a creature.

#### Finding 4: Missing Stifle test

**Severity**: LOW
**File**: `crates/engine/tests/impending.rs`
**CR Rule**: 702.176a (counter-removal is a triggered ability that uses the stack)
**Issue**: The plan called for a test showing that if the counter-removal trigger is countered (e.g., by Stifle), the time counter is NOT removed. The countered-abilities match arm at resolution.rs:3307 is correct and includes a comment, but there is no test exercising this for impending.
**Fix**: Add `test_impending_stifle_counter_removal` -- queue the ImpendingCounterTrigger on the stack, counter it (or remove it from the stack before resolution), and verify the permanent retains its time counters.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.176a (1st ability: alt cost) | Yes | Yes | test_impending_basic_cast, test_impending_alt_cost_exclusivity |
| 702.176a (2nd ability: ETB time counters) | Yes | Yes | test_impending_basic_cast (asserts 4 time counters after resolution) |
| 702.176a (3rd ability: not a creature) | Yes | Yes | test_impending_not_a_creature_while_counters, test_impending_creature_card_type_restored |
| 702.176a (4th ability: end-step counter removal) | Yes | Yes | test_impending_counter_removed_at_end_step, test_impending_multiple_end_steps |
| 702.176a ("your end step" -- controller only) | Yes | Yes | test_impending_counter_removal_only_on_controller_end_step |
| 702.176a (casting follows CR 601.2b/f-h) | Yes | Yes | test_impending_basic_cast validates correct mana payment |
| CR 118.9a (mutual exclusion) | Yes | Yes | test_impending_alt_cost_exclusivity + casting.rs Step 1m covers all 13 alt costs |
| CR 118.9d (commander tax on alt cost) | Yes | Yes | test_impending_commander_tax |
| CR 603.4 (intervening-if) | Yes | No | Logic at resolution.rs:1240-1248 but no dedicated test (Finding 1) |
| CR 400.7 (zone change resets) | Yes | No | General move_object_to_zone handles this but no impending test (Finding 3) |
| CR 707 (copy does not inherit) | Yes | No | copy.rs:210 sets was_impended: false but no impending test (Finding 2) |
| Layer 4 type removal | Yes | Yes | test_impending_not_a_creature_while_counters, test_impending_creature_card_type_restored |
| SBA non-interaction | Yes | Yes | test_impending_sba_while_not_creature |
| Normal cast (negative) | Yes | Yes | test_impending_normal_cast |
| Stifle counter-removal | Yes | No | resolution.rs:3307 match arm but no test (Finding 4) |
