//! Dungeon venture mechanic tests — Session 2 (CR 701.49, CR 309.4, CR 309.5).
//!
//! Tests the core venture-into-the-dungeon logic: entering a new dungeon,
//! advancing the marker, completing a dungeon, completing and restarting,
//! and the room ability being pushed to the stack.

use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::{DungeonId, DungeonState, GameEvent, GameStateBuilder, PlayerId};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Build a minimal 2-player game state in the player's main phase with priority.
/// We don't need a running game — just test the venture handler directly.
fn simple_state() -> mtg_engine::GameState {
    GameStateBuilder::four_player()
        .build()
        .expect("build failed")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 701.49a: When a player ventures with no dungeon in their command zone,
/// they enter a new dungeon and the venture marker is placed on room 0 (topmost).
///
/// Source: CR 701.49a — "If the player doesn't have a dungeon in the command zone,
/// they choose a dungeon card from outside the game and put it into the command zone,
/// then they put their venture marker on the topmost room."
#[test]
fn test_venture_enters_first_room() {
    let mut state = simple_state();

    // Player 1 has no dungeon yet
    assert!(state.dungeon_state.get(&p(1)).is_none());

    // Venture into the dungeon
    let events = mtg_engine::rules::engine::handle_venture_into_dungeon(&mut state, p(1), false)
        .expect("venture should succeed");

    // Player 1 should now have a dungeon with venture marker at room 0
    let ds = state
        .dungeon_state
        .get(&p(1))
        .expect("player should have dungeon state");
    assert_eq!(ds.current_room, 0, "venture marker should start at room 0");
    assert_eq!(
        ds.dungeon,
        DungeonId::LostMineOfPhandelver,
        "deterministic fallback should choose LostMineOfPhandelver"
    );

    // VenturedIntoDungeon event should be emitted for room 0
    let ventured = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::VenturedIntoDungeon { player, dungeon: DungeonId::LostMineOfPhandelver, room: 0 }
            if *player == p(1)
        )
    });
    assert!(
        ventured,
        "VenturedIntoDungeon event should be emitted for room 0"
    );
}

/// CR 701.49b: When a player ventures and is not on the bottommost room,
/// the venture marker advances to the next room (first exit, deterministic).
///
/// Source: CR 701.49b — "If the player's venture marker is on a non-bottommost room,
/// they move their venture marker to one of the adjacent rooms."
#[test]
fn test_venture_advances_room() {
    let mut state = simple_state();

    // Place player 1 in Lost Mine of Phandelver at room 0 (Cave Entrance, exits: [1, 2])
    state.dungeon_state.insert(
        p(1),
        DungeonState {
            dungeon: DungeonId::LostMineOfPhandelver,
            current_room: 0,
        },
    );

    // Venture again — should advance to room 1 (first exit)
    let events = mtg_engine::rules::engine::handle_venture_into_dungeon(&mut state, p(1), false)
        .expect("venture should succeed");

    let ds = state
        .dungeon_state
        .get(&p(1))
        .expect("player should still have dungeon state");
    assert_eq!(
        ds.current_room, 1,
        "venture marker should advance to room 1 (first exit)"
    );
    assert_eq!(ds.dungeon, DungeonId::LostMineOfPhandelver);

    // VenturedIntoDungeon event should be emitted for room 1
    let ventured = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::VenturedIntoDungeon { player, dungeon: DungeonId::LostMineOfPhandelver, room: 1 }
            if *player == p(1)
        )
    });
    assert!(
        ventured,
        "VenturedIntoDungeon event should be emitted for room 1"
    );
}

/// CR 701.49c: When a player ventures and is on the bottommost room, they complete
/// the dungeon (it is removed from the command zone and dungeons_completed increments).
///
/// Source: CR 701.49c — "If the player's venture marker is on the bottommost room,
/// they complete that dungeon. The dungeon card is removed from the game."
#[test]
fn test_venture_completes_dungeon() {
    use mtg_engine::get_dungeon;

    let mut state = simple_state();

    // Find the bottommost room of Lost Mine of Phandelver
    let bottommost = get_dungeon(DungeonId::LostMineOfPhandelver).bottommost_room;

    // Place player 1 at the bottommost room
    state.dungeon_state.insert(
        p(1),
        DungeonState {
            dungeon: DungeonId::LostMineOfPhandelver,
            current_room: bottommost,
        },
    );

    // Initial dungeons_completed should be 0
    let initial_completed = state.players.get(&p(1)).unwrap().dungeons_completed;
    assert_eq!(initial_completed, 0);

    // Venture — should complete the dungeon and start a new one
    let events = mtg_engine::rules::engine::handle_venture_into_dungeon(&mut state, p(1), false)
        .expect("venture should succeed");

    // DungeonCompleted event should be emitted
    let completed = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::DungeonCompleted { player, dungeon: DungeonId::LostMineOfPhandelver }
            if *player == p(1)
        )
    });
    assert!(completed, "DungeonCompleted event should be emitted");

    // dungeons_completed should now be 1
    let new_completed = state.players.get(&p(1)).unwrap().dungeons_completed;
    assert_eq!(
        new_completed, 1,
        "dungeons_completed should be incremented to 1"
    );
}

/// CR 701.49c: After completing a dungeon, the player immediately starts a new one
/// (enters a new dungeon and the venture marker is placed on room 0).
///
/// Source: CR 701.49c — "...and then venture into a new dungeon."
#[test]
fn test_venture_starts_new_after_completion() {
    use mtg_engine::get_dungeon;

    let mut state = simple_state();

    let bottommost = get_dungeon(DungeonId::LostMineOfPhandelver).bottommost_room;

    // Place player 1 at the bottommost room of Lost Mine
    state.dungeon_state.insert(
        p(1),
        DungeonState {
            dungeon: DungeonId::LostMineOfPhandelver,
            current_room: bottommost,
        },
    );

    // Venture — completes Lost Mine and starts a new dungeon
    let events = mtg_engine::rules::engine::handle_venture_into_dungeon(&mut state, p(1), false)
        .expect("venture should succeed");

    // Player 1 should have a new dungeon (not the old one at bottommost)
    // After completion, a new dungeon is entered at room 0
    let ds = state
        .dungeon_state
        .get(&p(1))
        .expect("player should have a new dungeon state");
    assert_eq!(ds.current_room, 0, "new dungeon should start at room 0");

    // There should be two VenturedIntoDungeon events:
    // The first for starting the new dungeon at room 0 (after completion).
    let new_venture_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::VenturedIntoDungeon { room: 0, player, .. } if *player == p(1)))
        .count();
    assert!(
        new_venture_count >= 1,
        "should have at least 1 VenturedIntoDungeon at room 0 for the new dungeon"
    );

    // dungeons_completed should be 1
    assert_eq!(state.players.get(&p(1)).unwrap().dungeons_completed, 1);
}

/// CR 309.4c: When the venture marker moves into a room, a room ability
/// (triggered ability) is pushed onto the stack.
///
/// Source: CR 309.4c — "Each room has a room ability that triggers when a player
/// moves their venture marker into that room."
#[test]
fn test_room_ability_goes_on_stack() {
    let mut state = simple_state();

    let initial_stack_len = state.stack_objects.len();

    // Venture into the dungeon (player 1 has no dungeon yet)
    let _ = mtg_engine::rules::engine::handle_venture_into_dungeon(&mut state, p(1), false)
        .expect("venture should succeed");

    // A RoomAbility should be on the stack
    assert!(
        state.stack_objects.len() > initial_stack_len,
        "stack should have grown after venturing"
    );

    // The top of the stack should be a RoomAbility for the entered room
    let top = state
        .stack_objects
        .back()
        .expect("stack should not be empty");
    assert!(
        matches!(
            &top.kind,
            StackObjectKind::RoomAbility { owner, dungeon: DungeonId::LostMineOfPhandelver, room: 0 }
            if *owner == p(1)
        ),
        "top of stack should be RoomAbility for Lost Mine room 0, got: {:?}",
        top.kind
    );
    assert_eq!(
        top.controller,
        p(1),
        "room ability controller should be the venturer"
    );
}
