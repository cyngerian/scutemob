# Ability Review: Venture into the Dungeon (Session 2)

**Date**: 2026-03-09
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.49, 309, 725
**Files reviewed**:
- `crates/engine/src/state/dungeon.rs` (dungeon data model + all 4 static definitions)
- `crates/engine/src/rules/engine.rs:596-617` (Command dispatch), `:2076-2238` (handle_venture_into_dungeon)
- `crates/engine/src/effects/mod.rs:2047-2070` (Effect::VentureIntoDungeon, Effect::TakeTheInitiative)
- `crates/engine/src/effects/mod.rs:3466-3489` (Condition::CompletedADungeon, CompletedSpecificDungeon)
- `crates/engine/src/cards/card_definition.rs:1145-1156` (Effect enum variants), `:1474-1486` (Condition variants)
- `crates/engine/src/rules/command.rs:554-569` (Command variants)
- `crates/engine/src/state/stack.rs:778-791` (StackObjectKind::RoomAbility)
- `crates/engine/src/state/hash.rs:999-1018,2096-2105,4537-4543` (hash impls)
- `crates/engine/src/state/mod.rs:198-215` (GameState dungeon_state + has_initiative)
- `crates/engine/src/state/player.rs:156-159` (dungeons_completed)
- `crates/engine/src/state/builder.rs:288-360` (builder defaults)
- `crates/engine/src/rules/resolution.rs:7126-7146,7348-7350` (RoomAbility resolution + counter arm)
- `crates/engine/tests/dungeon_venture.rs` (5 tests)
- `tools/replay-viewer/src/view_model.rs:540` (RoomAbility arm)
- `tools/tui/src/play/panels/stack_view.rs:146-148` (RoomAbility arm)

## Verdict: needs-fix

The core venture mechanic is well-structured with correct three-case dispatch (701.49a/b/c), proper room ability stacking (309.4c), and clean hash coverage. However, there are two MEDIUM findings: CompletedSpecificDungeon ignores the dungeon_id parameter (producing incorrect results for Acererak), and the CR 309.6 SBA for removing completed dungeons from the command zone is missing. There are also notable scope gaps for CR 725.2-5 initiative triggers that should be documented.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | ~~MEDIUM~~ FIXED | `effects/mod.rs:3476-3489` | **CompletedSpecificDungeon ignores dungeon_id.** Fixed: added `dungeons_completed_set: OrdSet<DungeonId>` to PlayerState, populated on completion, condition now checks `set.contains(dungeon_id)`. |
| 2 | MEDIUM | `rules/sba.rs` (missing) | **CR 309.6 SBA not implemented.** Dungeon on bottommost room should be removed by SBA when no pending room ability on stack. **Fix:** Add SBA check or document as deferred (current code removes dungeon via venture handler, not SBA). |
| 3 | LOW | `rules/engine.rs:2170` | **Recursive call in completion path.** handle_venture_into_dungeon calls itself recursively for the post-completion new-dungeon entry. Correct behavior but unbounded recursion possible in theory. |
| 4 | LOW | `effects/mod.rs:2049-2053` | **Error silently swallowed.** `if let Ok(...)` discards the error from handle_venture_into_dungeon. **Fix:** Log or propagate the error. |
| 5 | LOW | `rules/engine.rs:2049-2053,2065-2068` | **Same silent error swallowing in TakeTheInitiative.** |
| 6 | LOW | (missing) | **CR 725.2 inherent triggers not implemented.** Upkeep venture for initiative holder and combat-damage initiative steal are not wired. Acceptable scope gap for S2 but should be documented. |
| 7 | LOW | (missing) | **CR 725.4 not implemented.** Initiative transfer when a player leaves the game is not handled. |
| 8 | LOW | `tests/dungeon_venture.rs` | **No test for force_undercity=true (Initiative/Undercity path).** TakeTheInitiative effect execution is untested. |
| 9 | LOW | `tests/dungeon_venture.rs` | **No test for multiplayer isolation.** Player 2 venturing should not affect Player 1's dungeon state. |
| 10 | LOW | `tests/dungeon_venture.rs` | **No negative test for CR 309.3.** Should verify a player cannot have two dungeons simultaneously. |

### Finding Details

#### Finding 1: CompletedSpecificDungeon ignores dungeon_id

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:3476-3489`
**CR Rule**: 309.7 -- "A player completes a dungeon as that dungeon card is removed from the game."
**Issue**: The `CompletedSpecificDungeon(dungeon_id)` condition uses `let _ = dungeon_id;` and falls back to checking `dungeons_completed > 0`, which is identical to `CompletedADungeon`. This means `CompletedSpecificDungeon(TombOfAnnihilation)` returns true even if the player completed Lost Mine of Phandelver. Acererak the Archlich's "if you haven't completed Tomb of Annihilation" check would be incorrect.
**Fix**: Either (a) add a `dungeons_completed_set: OrdSet<DungeonId>` field to `PlayerState` and populate it when a dungeon is completed, then check `set.contains(dungeon_id)` here; or (b) change the comment to explicitly note this is a known simplification and add a TODO with the CR rule. Option (a) is preferred since the data is cheap to track and the condition is already defined.

#### Finding 2: CR 309.6 SBA missing

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/sba.rs` (no dungeon-related code)
**CR Rule**: 309.6 -- "If a player's venture marker is on the bottommost room of a dungeon card, and that dungeon card isn't the source of a room ability that has triggered but not yet left the stack, the dungeon card's owner removes it from the game. (This is a state-based action. See rule 704.)"
**Issue**: The current implementation handles dungeon completion inside `handle_venture_into_dungeon` (CR 701.49c path) -- the dungeon is removed when the player next ventures while on the bottommost room. CR 309.6 says this should be an SBA: the dungeon is removed as soon as the bottommost room's ability leaves the stack (even without another venture trigger). In practice this matters if a player enters the bottommost room and then the room ability resolves -- the SBA should remove the dungeon before the next SBA check, without requiring another venture action.
**Fix**: Add an SBA check in `sba.rs` that iterates `state.dungeon_state`, finds any player whose venture marker is on the bottommost room, checks that no `StackObjectKind::RoomAbility { owner, dungeon, room }` matching that player+dungeon+room exists on the stack, and if so removes the dungeon and increments `dungeons_completed`. Then adjust `handle_venture_into_dungeon`'s bottommost-room case (line 2159-2171) to not redundantly remove+increment (the SBA will do it), but still emit `DungeonCompleted` and re-enter a new dungeon. Alternatively, document as a known deviation if the SBA approach is too complex for this session.

#### Finding 3: Recursive call in completion path

**Severity**: LOW
**File**: `crates/engine/src/rules/engine.rs:2170`
**CR Rule**: 701.49c -- After completing, "They then choose an appropriate dungeon card..."
**Issue**: `handle_venture_into_dungeon` calls itself recursively when the player is on the bottommost room. The recursion is bounded to depth 1 (the recursive call enters the `None` branch since `dungeon_state` was just removed), so this is correct. However, if a future bug leaves the dungeon_state populated after removal, it could infinite-loop. An iterative approach would be more defensive.
**Fix**: No immediate fix required. Add a comment noting the recursion is bounded to depth 1.

#### Finding 4: Error silently swallowed in VentureIntoDungeon effect

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:2049-2053`
**CR Rule**: Architecture invariant: Events are the single source of truth.
**Issue**: `if let Ok(venture_events) = ... { events.extend(venture_events); }` silently discards errors. If `handle_venture_into_dungeon` fails, no error is surfaced.
**Fix**: Either propagate the error or add a debug_assert/log. Most effect handlers in this codebase use a similar pattern so this is consistent, but worth noting.

#### Finding 6: CR 725.2 inherent triggers not implemented

**Severity**: LOW
**File**: (not implemented)
**CR Rule**: 725.2 -- "At the beginning of the upkeep of the player who has the initiative, that player ventures into Undercity" and "Whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the controller of those creatures takes the initiative."
**Issue**: The `TakeTheInitiative` effect correctly sets `has_initiative` and ventures into the Undercity (the third inherent trigger from 725.2). However, the upkeep-venture trigger and the combat-damage-steal trigger are not implemented anywhere in the codebase. These are inherent triggered abilities with no source (CR 725.2 exception to 113.8).
**Fix**: Document as deferred to a future session. When implemented, the upkeep trigger should be added to `upkeep_actions()` or the generic CardDef upkeep sweep, and the combat damage trigger should be checked in combat damage resolution.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.49a (no dungeon -- enter new) | Yes | Yes | test_venture_enters_first_room |
| 701.49b (mid-dungeon -- advance) | Yes | Yes | test_venture_advances_room |
| 701.49c (bottommost -- complete + restart) | Yes | Yes | test_venture_completes_dungeon, test_venture_starts_new_after_completion |
| 701.49d (venture into [quality] -- Undercity) | Yes | No | force_undercity param exists but no test |
| 309.3 (one dungeon at a time) | Yes (implicit) | No | Enforced by single-key OrdMap, but no explicit test |
| 309.4a (marker on topmost room) | Yes | Yes | room 0 assertion in test_venture_enters_first_room |
| 309.4b (room names are flavor text) | Yes | N/A | Names stored but not used in gameplay |
| 309.4c (room abilities are triggered) | Yes | Yes | test_room_ability_goes_on_stack |
| 309.5a (branching -- choose exit) | Partial | No | Deterministic first-exit fallback; no test for branching |
| 309.6 (SBA: remove completed dungeon) | **No** | No | Completion handled inline, not as SBA (Finding 2) |
| 309.7 (completing = removing from game) | Yes | Yes | dungeons_completed incremented |
| 725.1 (initiative designation) | Yes | No | has_initiative field on GameState |
| 725.2 (taking initiative ventures Undercity) | Yes | No | TakeTheInitiative effect wired |
| 725.2 (upkeep trigger for initiative holder) | **No** | No | Not implemented (Finding 6) |
| 725.2 (combat damage steals initiative) | **No** | No | Not implemented (Finding 6) |
| 725.3 (only one player has initiative) | Yes | No | Option<PlayerId> enforces single holder |
| 725.4 (initiative transfer on player leave) | **No** | No | Not implemented (Finding 7) |
| 725.5 (re-taking initiative triggers) | Partial | No | Trigger fires but no test |

## Summary of Dungeon Definitions

All 4 dungeons are defined with correct room graphs:
- Lost Mine of Phandelver: 7 rooms, bottommost=6, 2 branching points
- Dungeon of the Mad Mage: 9 rooms, bottommost=8, 2 branching points
- Tomb of Annihilation: 5 rooms, bottommost=4, 1 branching point
- The Undercity: 7 rooms, bottommost=6, 2 branching points

Room effects use deterministic fallbacks (placeholders) for interactive targeting effects (marked with TODO(M10+)). This is acceptable for the current milestone.

## Architecture Notes

- Hash coverage is complete: DungeonId, DungeonState, RoomAbility SOK, dungeon_state on GameState, dungeons_completed on PlayerState, has_initiative on GameState -- all hashed.
- StackObject construction for RoomAbility uses inline struct literals with all ~25 boolean fields set to false. This is verbose but correct. Future consolidation of StackObject construction into a builder/default would reduce this.
- The handler is public (`pub fn handle_venture_into_dungeon`) to allow direct testing without going through the full command pipeline. This is pragmatic for unit testing.
- Replay viewer and TUI both have RoomAbility arms -- no missing match coverage.
- Resolution.rs handles both the resolution case (line 7128-7146) and the counter case (line 7348-7350) for RoomAbility. Both are correct.
