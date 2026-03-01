# Ability Review: Melee

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.121
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 701-711)
- `crates/engine/src/state/stack.rs` (lines 514-533)
- `crates/engine/src/state/stubs.rs` (lines 300-306)
- `crates/engine/src/state/hash.rs` (lines 495-496, 1135-1136, 1466-1470)
- `crates/engine/src/state/builder.rs` (lines 513-530)
- `crates/engine/src/rules/abilities.rs` (lines 1477-1492, 2748-2753; 13 PendingTrigger sites)
- `crates/engine/src/rules/resolution.rs` (lines 1888-1962, 2075)
- `crates/engine/src/effects/mod.rs` (2 PendingTrigger sites)
- `crates/engine/src/rules/turn_actions.rs` (3 PendingTrigger sites)
- `crates/engine/src/rules/miracle.rs` (1 PendingTrigger site)
- `tools/replay-viewer/src/view_model.rs` (lines 488-490, 710)
- `tools/tui/src/play/panels/stack_view.rs` (lines 95-97)
- `crates/engine/tests/melee.rs` (full file, 579 lines, 7 tests)

## Verdict: clean

The Melee implementation is correct and complete. It faithfully implements CR 702.121a and
702.121b. The resolution-time computation of the bonus from `state.combat` matches the
2016-08-23 ruling. Planeswalker attacks are correctly excluded from the opponent count.
Multiple instances trigger separately. The code follows the established Rampage pattern
exactly. All hash fields are covered, all match arms are present, and all 20
PendingTrigger construction sites include `is_melee_trigger: false`. The 7 tests cover
all key scenarios including multiplayer (2, 3, and 4 opponents), planeswalker exclusion,
multiple instances, and source-leaves-battlefield. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `resolution.rs:1934` | **Effect ID via next_object_id().0 is unconventional.** The EffectId is generated from `state.next_object_id().0`, which increments the object counter. This is the same pattern used by Rampage (resolution.rs:1762) and replacement.rs:1233, so it is consistent. Not a bug -- just noting that effect IDs share the object ID namespace. No fix needed. |
| 2 | LOW | `tests/melee.rs` | **No test for combat-ends-before-resolution edge case.** If combat ends (e.g., via a "remove from combat" or "end the combat phase" effect) while the Melee trigger is on the stack, `state.combat` would be `None` and the bonus would be 0. The code handles this correctly (line 1920: `.unwrap_or(0)`), but no test exercises it. **Fix:** Consider adding a test that sets `state.combat` to `None` before resolving the trigger, to verify the no-op behavior. Low priority since the code path is correct. |
| 3 | LOW | `hash.rs:495` | **Discriminant diverges from plan (83 vs 84).** The plan reserved 83 for Toxic and specified 84 for Melee. The implementer used 83 since Toxic was not implemented. This is correct -- no collision exists. However, the plan file at `memory/abilities/ability-plan-melee.md` still says 84. **Fix:** Update `ability-plan-melee.md` to note that 83 was used instead, or leave as-is since the plan is historical. No functional impact. |

### Finding Details

#### Finding 1: Effect ID via next_object_id().0

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:1934`
**CR Rule**: N/A (infrastructure pattern)
**Issue**: `state.next_object_id().0` is used to generate the `EffectId`. This shares the ObjectId counter namespace with game objects. The same pattern is used in Rampage resolution (line 1762) and `replacement.rs:1233`, so this is consistent across the codebase.
**Fix**: No fix needed. This is an established codebase pattern. Documenting for awareness only.

#### Finding 2: No test for combat-ends-before-resolution

**Severity**: LOW
**File**: `crates/engine/tests/melee.rs`
**CR Rule**: 702.121a -- bonus computed at resolution time
**Issue**: The code correctly handles `state.combat == None` by returning 0 opponents attacked (line 1920: `.unwrap_or(0)`). However, no test exercises this path. The existing test 6 (source-leaves-battlefield) tests a different edge case. A test where combat state is cleared before resolution would increase coverage.
**Fix**: Optionally add a test that manipulates state to clear combat before trigger resolution. Low priority since the code is provably correct and the `unwrap_or(0)` pattern is trivially safe.

#### Finding 3: Discriminant diverges from plan

**Severity**: LOW
**File**: `crates/engine/src/state/hash.rs:495`
**CR Rule**: N/A (coordination)
**Issue**: Plan specified discriminant 84 (reserving 83 for Toxic). Implementation uses 83 since Toxic was not implemented in parallel. No collision exists. When Toxic is eventually implemented, it should use the next available discriminant after 83.
**Fix**: No fix needed for the implementation. The plan file is historical context. Future implementers should check `hash.rs` for the actual next available discriminant rather than relying on plan files.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.121a (trigger definition) | Yes | Yes | test_702_121a_melee_basic_one_opponent_attacked (test 1) |
| 702.121a (bonus = distinct opponents) | Yes | Yes | test_702_121a_melee_multiplayer_two_opponents (test 2), test_702_121a_melee_multiplayer_three_opponents (test 3) |
| 702.121a (planeswalker exclusion) | Yes | Yes | test_702_121a_melee_does_not_count_planeswalker_attacks (test 4) |
| 702.121a (resolution-time computation) | Yes | Yes | All tests verify P/T after resolution |
| 702.121a (attacking alone still counts) | Yes | Yes | test_702_121a_melee_attacking_alone_still_counts (test 7) |
| 702.121b (multiple instances) | Yes | Yes | test_702_121b_melee_multiple_instances (test 5) |
| Ruling: source leaves BF | Yes | Yes | test_702_121a_melee_source_leaves_battlefield_no_bonus (test 6) |
| Ruling: creatures entering BF attacking | Yes (natural) | No | Not explicitly tested; naturally correct because `SelfAttacks` only fires from `AttackersDeclared` |
| Ruling: attackers leaving BF | Yes (natural) | No | Not explicitly tested; naturally correct because `combat.attackers` retains all declared attackers |
| Ruling: eliminated opponents | Yes (natural) | No | Not explicitly tested; naturally correct because `combat.attackers` preserves original targets |
| Combat ends before resolution | Yes | No | `unwrap_or(0)` handles `None` combat; not tested (LOW finding 2) |
