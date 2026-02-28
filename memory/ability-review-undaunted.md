# Ability Review: Undaunted

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.125
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 364-371)
- `crates/engine/src/state/hash.rs` (lines 413-414)
- `tools/replay-viewer/src/view_model.rs` (line 636)
- `crates/engine/src/rules/casting.rs` (lines 695-701 call site, lines 1993-2042 function)
- `crates/engine/tests/undaunted.rs` (780 lines, 11 tests)
- `crates/engine/src/state/builder.rs` (no Undaunted arm needed -- uses if-let, not exhaustive match)
- `crates/engine/src/state/mod.rs` (lines 387-399 -- `active_players()`)

## Verdict: clean

The Undaunted implementation is correct and well-structured. All three CR 702.125 subrules
are properly implemented. The enforcement logic correctly counts opponents of the caster
(not the active player), excludes eliminated players via `active_players()`, reduces only
generic mana, floors at zero per CR 601.2f, and is positioned correctly in the casting
pipeline (after affinity, before convoke/improvise/delve). The 11 tests cover all major
scenarios including multiplayer scaling, eliminated opponents, commander tax interaction,
and affinity composition. No HIGH or MEDIUM findings. Two LOW findings relate to test
coverage gaps and a stale CR number in the WIP state file.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `crates/engine/tests/undaunted.rs` | **Missing test: caster != active player.** No test exercises instant-speed Undaunted where the caster is not the active player. |
| 2 | LOW | `memory/ability-wip.md:4` | **Stale CR number in WIP file.** WIP says `cr: 702.124` (Partner) but should say `cr: 702.125` (Undaunted). |

### Finding Details

#### Finding 1: Missing test -- caster != active player

**Severity**: LOW
**File**: `crates/engine/tests/undaunted.rs` (all tests)
**CR Rule**: 702.125a -- "This spell costs {1} less to cast for each opponent you have."
**Issue**: All 11 tests set `active_player(p1)` and cast with `p1`. No test exercises the
scenario where a non-active player casts an Undaunted instant (e.g., during an opponent's
turn). The implementation correctly uses the `player` parameter (caster) rather than
`state.turn.active_player`, but a test would guard against regression. This also tests that
"opponent" means "not the caster" rather than "not the active player" -- an important
semantic distinction in multiplayer.
**Fix**: Add a test where p2 is the active player but p1 casts an instant with Undaunted.
Verify p1's opponent count is N-1 (all players except p1), not N-1 relative to the active
player. This could also catch bugs if someone later refactors to use `state.turn.active_player`
instead of the `player` parameter.

#### Finding 2: Stale CR number in ability-wip.md

**Severity**: LOW
**File**: `memory/ability-wip.md:4`
**CR Rule**: 702.125 (Undaunted), not 702.124 (Partner)
**Issue**: The WIP state file says `cr: 702.124` but Undaunted is CR 702.125. The plan file
correctly identifies this as 702.125, and all implementation code uses 702.125. This is a
documentation-only issue in the WIP tracking file. The ability-coverage doc also had the
stale number (702.124), which the plan file explicitly noted.
**Fix**: Update `memory/ability-wip.md` line 4 to `cr: 702.125`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.125a -- static ability, costs {1} less per opponent | Yes | Yes | Tests 1, 2, 4, 6, 9, 10, 11 |
| 702.125b -- eliminated players not counted | Yes | Yes | Test 7 (has_lost); no test for has_conceded (minor gap, same code path via active_players()) |
| 702.125c -- multiple instances cumulative | Yes (code) | Yes (documents OrdSet limitation) | Test 8 documents OrdSet deduplication; theoretical only since no printed card has 2x Undaunted |
| 601.2f -- generic floor at {0} | Yes | Yes | Tests 2 (exact zero), 3 (excess opponents), 4 (colored pips preserved) |
| Pipeline position (after affinity, before convoke) | Yes | Yes | Test 10 (Affinity + Undaunted composition) |
| Commander tax interaction | Yes | Yes | Test 9 |
| Multiplayer scaling (2p, 4p, 6p) | Yes | Yes | Tests 1 (4p), 6 (2p), 11 (6p) |
| Negative test (no keyword = no reduction) | N/A | Yes | Test 5 |
| Caster vs active player distinction | Yes (code correct) | No | All tests have caster = active player (LOW finding 1) |

## Implementation Quality Notes

- **Doc comments**: Thorough CR citations in types.rs (702.125a/b/c), casting.rs (702.125a/b/c, 601.2f).
- **Hash coverage**: Discriminant 54 added correctly, no collision with existing discriminants.
- **View model**: Display string "Undaunted" added.
- **Builder.rs**: No arm needed -- builder uses if-let chains for trigger generation, not
  exhaustive match. Undaunted is a static cost-reduction ability with no triggers.
- **Pattern consistency**: `apply_undaunted_reduction` closely follows `apply_affinity_reduction`
  (same signature, same structure, same early-return patterns). Good code reuse pattern.
- **No `.unwrap()` in engine library code**: The `apply_undaunted_reduction` function uses `?`
  for `None` cost, no unwraps. Correct per conventions.
- **Test quality**: All 11 tests cite CR rules in doc comments. Assertions check both stack
  state and mana pool depletion. Helper functions reduce boilerplate.
