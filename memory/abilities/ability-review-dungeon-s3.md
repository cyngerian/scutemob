# Ability Review: Dungeon Session 3 (Resolution, SBA 704.5t, Initiative)

**Date**: 2026-03-09
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 309.4c, 309.6, 704.5t, 725.1-725.5, 701.49
**Files reviewed**:
- `crates/engine/src/rules/sba.rs:230-233,264-301,308-359,1273-1337` (check_dungeon_completion_sba, transfer_initiative_on_player_leave, initiative hooks in check_player_sbas)
- `crates/engine/src/rules/turn_actions.rs:576-583,1763-1790,1795-1835` (upkeep initiative venture, first_strike_damage_step, combat_damage_step initiative steal)
- `crates/engine/src/rules/engine.rs:1897-1900,2160-2177` (concede initiative transfer, handle_venture_into_dungeon 701.49c path)
- `crates/engine/src/rules/resolution.rs:7126-7146,7348-7350` (RoomAbility resolution + counter arm)
- `crates/engine/tests/dungeon_resolution.rs` (6 tests)

## Verdict: fixed (2026-03-09 — both MEDIUM findings resolved)

The SBA 704.5t implementation is correct and well-structured. Room ability resolution through the standard effect path is clean. Initiative upkeep and combat damage triggers are wired. However, there are two MEDIUM findings: CR 725.4 initiative transfer does not prioritize the active player as the rule requires, and the first-strike combat damage step is missing the initiative steal check (only the regular damage step has it). There are also several LOW findings around missing tests and a minor doc gap.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `sba.rs:314-359` | **CR 725.4: Initiative transfer does not prioritize active player.** Searches turn order from leaving player instead of checking active player first. **Fix:** check `state.turn.active_player` first. **FIXED** (2026-03-09) |
| 2 | **MEDIUM** | `turn_actions.rs:1763-1790` | **First-strike damage step missing initiative steal.** Only `combat_damage_step` checks for initiative theft; `first_strike_damage_step` does not. **Fix:** add the same initiative check after `apply_combat_damage(state, true)`. **FIXED** (2026-03-09) — extracted `check_initiative_steal_from_combat_damage` helper, called from both steps. |
| 3 | LOW | `sba.rs:13-25` | **SBA header missing 704.5t.** Module doc lists implemented SBAs but omits 704.5t (dungeon completion). **Fix:** add `//! - 704.5t: Dungeon on bottommost room with no room ability on stack -> removed.` |
| 4 | LOW | `turn_actions.rs:581` | **Upkeep initiative venture swallows errors.** `.unwrap_or_default()` silently discards `handle_venture_into_dungeon` errors. Consistent with other call sites but violates error-handling convention. |
| 5 | LOW | `sba.rs:356` | **transfer_initiative_on_player_leave swallows venture errors.** Same `.unwrap_or_default()` pattern. |
| 6 | LOW | `dungeon_resolution.rs` | **No test for CR 725.4 initiative transfer on player leave.** Neither SBA-based death nor concession scenarios are tested. |
| 7 | LOW | `dungeon_resolution.rs` | **No test for first-strike initiative steal.** First-strike creature dealing damage to initiative holder is untested. |
| 8 | LOW | `engine.rs:2164-2177` | **701.49c completion duplicates SBA logic.** Both `handle_venture_into_dungeon` (701.49c path) and `check_dungeon_completion_sba` increment `dungeons_completed` and insert into `dungeons_completed_set`. Not a bug (paths are mutually exclusive in practice) but fragile — if both run, double-counting would occur. |

### Finding Details

#### Finding 1: CR 725.4 — Initiative transfer does not prioritize active player

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/sba.rs:314-359`
**CR Rule**: 725.4 -- "If the player who has the initiative leaves the game, the active player takes the initiative at the same time that player leaves the game. If the active player is leaving the game or if there is no active player, the next player in turn order takes the initiative."
**Issue**: The implementation iterates through turn order starting from the leaving player's position (`leaving_pos + 1`), finding the first active player. It does not check whether the active player should receive the initiative first. CR 725.4 has a two-step priority: (1) active player gets it, unless (2) the active player is also leaving or there is no active player, in which case it goes to the next player in turn order.

Example: In a 4-player game with turn order [P1, P2, P3, P4], P3 is the active player, P2 has the initiative and dies. The implementation gives the initiative to P3 (next after P2 in turn order) -- this happens to be correct. But if P4 has the initiative and dies, the implementation gives it to P1 (next after P4), when CR 725.4 says it should go to P3 (the active player).

**Fix**: Before the turn-order search, check if `state.turn.active_player` is a valid candidate (not the leaving player, not lost/conceded). If so, give them the initiative directly. Only fall through to the turn-order search if the active player is ineligible.

```rust
// CR 725.4: Active player gets initiative first.
let active = state.turn.active_player;
if active != leaving_player {
    if let Some(ap) = state.players.get(&active) {
        if !ap.has_lost && !ap.has_conceded {
            state.has_initiative = Some(active);
            // ... emit events, venture
            return events;
        }
    }
}
// Fallback: next in turn order
```

#### Finding 2: First-strike damage step missing initiative steal

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/turn_actions.rs:1763-1790`
**CR Rule**: 725.2 -- "Whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the controller of those creatures takes the initiative."
**Issue**: `first_strike_damage_step` calls `super::combat::apply_combat_damage(state, true)` and returns the result directly. It does not check the resulting events for combat damage dealt to the initiative holder. A creature with First Strike or Double Strike that deals combat damage to the initiative holder in the first-strike damage step will NOT cause the initiative to transfer. Only `combat_damage_step` (regular damage) has this check (lines 1808-1835). CR 725.2's trigger condition is "deal combat damage" -- it applies to all combat damage, regardless of whether it's first-strike or regular.
**Fix**: Extract the initiative-steal logic (lines 1808-1835 of `combat_damage_step`) into a helper function, and call it at the end of both `first_strike_damage_step` and `combat_damage_step`.

#### Finding 3: SBA header missing 704.5t

**Severity**: LOW
**File**: `crates/engine/src/rules/sba.rs:13-25`
**CR Rule**: 704.5t
**Issue**: The module doc comment lists all implemented SBAs but omits 704.5t, which is now implemented by `check_dungeon_completion_sba`.
**Fix**: Add `//! - 704.5t: Dungeon on bottommost room with no room ability on stack -> removed from game.` to the header list, after the 704.5u entry.

#### Finding 8: 701.49c completion duplicates SBA logic

**Severity**: LOW
**File**: `crates/engine/src/rules/engine.rs:2164-2177`
**CR Rule**: 701.49c, 704.5t
**Issue**: When a player ventures while on the bottommost room (701.49c), `handle_venture_into_dungeon` removes the dungeon from `dungeon_state`, increments `dungeons_completed`, and inserts into `dungeons_completed_set`. The SBA `check_dungeon_completion_sba` does the exact same operations. These two paths are mutually exclusive in practice: the 701.49c path only fires when the player is on the bottommost room AND ventures again, at which point it removes the dungeon before SBA can see it. But if a future change causes both paths to fire for the same dungeon, `dungeons_completed` would be double-incremented. Consider removing the completion logic from the 701.49c path and relying solely on the SBA, or adding a guard in the SBA to skip if already completed.
**Fix**: No immediate fix required. Add a code comment in `handle_venture_into_dungeon` noting the dual-path risk: `// Note: this mirrors check_dungeon_completion_sba logic. The two paths are mutually exclusive because this removes dungeon_state before SBA runs.`

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 309.4c (room abilities are triggered) | Yes | Yes | test_room_ability_resolves_scry, test_room_ability_resolves_create_token |
| 309.6 / 704.5t (SBA: remove completed dungeon) | Yes | Yes | test_sba_704_5t_removes_completed_dungeon, test_sba_704_5t_waits_for_room_ability |
| 725.1 (initiative designation) | Yes | Yes (implicit) | has_initiative field |
| 725.2 (take initiative -> venture Undercity) | Yes | Yes | test_initiative_combat_damage_steal verifies venture |
| 725.2 (upkeep venture for initiative holder) | Yes | Yes | test_initiative_upkeep_venture |
| 725.2 (combat damage steals initiative) | Partial | Partial | Regular damage only (Finding 2); no first-strike test (Finding 7) |
| 725.3 (only one player has initiative) | Yes | Implicit | Option type enforces |
| 725.4 (initiative transfer on player leave) | **Partial** | **No** | Active player not prioritized (Finding 1); no test (Finding 6) |
| 725.5 (re-taking initiative triggers venture) | Yes | No | Implementation doesn't guard against self-assignment; CR 725.5 says this is correct |

## Previous Findings (from S1 and S2 reviews)

| # | Previous ID | Previous Status | Current Status | Notes |
|---|-------------|----------------|----------------|-------|
| S2-F2 | MEDIUM | OPEN | **RESOLVED** | CR 309.6 SBA now implemented as `check_dungeon_completion_sba`. Correct: checks bottommost room + no matching RoomAbility on stack. |
| S2-F6 | LOW | OPEN (scope gap) | **RESOLVED** | CR 725.2 upkeep trigger now implemented in `upkeep_actions()`. |
| S2-F7 | LOW | OPEN (scope gap) | **PARTIALLY RESOLVED** | CR 725.4 implemented but has active-player priority bug (Finding 1). |
| S2-F8 | LOW | OPEN (no test) | **RESOLVED** | Upkeep venture tested in `test_initiative_upkeep_venture`. |

## Architecture Notes

- The SBA is correctly placed after commander zone-return SBA in the fixed-point loop, at the end of `check_and_apply_all_sbas`. This ordering is fine since dungeon completion has no dependency on other SBAs.
- Room ability resolution at `resolution.rs:7128-7146` uses a sentinel `ObjectId(0)` for the effect context source. This is pragmatic since dungeons are not permanent objects, but any effect that references `source` will get a non-existent object. The counter arm at line 7348-7350 correctly handles the RoomAbility being countered (Stifle) — no room effect fires, venture marker stays advanced.
- The `transfer_initiative_on_player_leave` function is called from three sites: 704.5a (life loss), 704.5c (poison), 704.5u (commander damage), plus the concede handler. All four call sites correctly invoke it after marking the player as lost/conceded.
- The initiative combat damage check uses `find_map` to find the first creature that dealt damage. This correctly handles the CR 725.2 "one or more" grouping (a single trigger regardless of creature count). In Commander, only one player attacks at a time, so multiple-controller scenarios are not practically relevant.
