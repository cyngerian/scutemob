//! Dungeon data model tests — Session 1 (CR 309.1, 309.4).
//!
//! Tests the structural integrity of dungeon definitions, default state,
//! and hash determinism for the new dungeon fields on GameState and PlayerState.

use mtg_engine::{get_dungeon, DungeonId, DungeonState, GameStateBuilder, PlayerId};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 309.4: Verify all 4 dungeon graph definitions are well-formed.
///
/// A well-formed dungeon satisfies:
/// - Room 0 is always the entry (topmost) room
/// - Every exit index points to a valid room in the same dungeon
/// - The bottommost room has empty exits
/// - `bottommost_room` index matches the room with empty exits
/// - No room can exit to itself (no self-loops)
#[test]
fn test_dungeon_def_structure() {
    let all_dungeons = [
        DungeonId::LostMineOfPhandelver,
        DungeonId::DungeonOfTheMadMage,
        DungeonId::TombOfAnnihilation,
        DungeonId::TheUndercity,
    ];

    for id in all_dungeons {
        let def = get_dungeon(id);
        let n = def.rooms.len();
        assert!(n > 0, "{}: dungeon has no rooms", def.name);

        // Check that bottommost_room is a valid index
        assert!(
            def.bottommost_room < n,
            "{}: bottommost_room {} is out of bounds (len={})",
            def.name,
            def.bottommost_room,
            n
        );

        // Check that the bottommost room actually has empty exits
        assert!(
            def.rooms[def.bottommost_room].exits.is_empty(),
            "{}: bottommost room {} ('{}') has exits {:?} — should be empty",
            def.name,
            def.bottommost_room,
            def.rooms[def.bottommost_room].name,
            def.rooms[def.bottommost_room].exits
        );

        // Check that every room's exits point to valid indices
        for (i, room) in def.rooms.iter().enumerate() {
            for &exit in room.exits {
                assert!(
                    exit < n,
                    "{}: room {} ('{}') has exit {} which is out of bounds (len={})",
                    def.name,
                    i,
                    room.name,
                    exit,
                    n
                );
                assert_ne!(
                    exit, i,
                    "{}: room {} ('{}') has a self-loop exit",
                    def.name, i, room.name
                );
            }
        }

        // Check that no rooms other than the bottommost have empty exits
        for (i, room) in def.rooms.iter().enumerate() {
            if i != def.bottommost_room {
                assert!(
                    !room.exits.is_empty(),
                    "{}: room {} ('{}') has empty exits but is not the bottommost room {}",
                    def.name,
                    i,
                    room.name,
                    def.bottommost_room
                );
            }
        }

        // Verify room effects can be called without panicking
        for (i, room) in def.rooms.iter().enumerate() {
            let _effect = (room.effect)();
            // If we got here without panic, the effect function is sound
            let _ = i;
        }
    }

    // Verify specific expected room counts per dungeon (CR 309.2a)
    assert_eq!(
        get_dungeon(DungeonId::LostMineOfPhandelver).rooms.len(),
        7,
        "Lost Mine of Phandelver should have 7 rooms"
    );
    assert_eq!(
        get_dungeon(DungeonId::DungeonOfTheMadMage).rooms.len(),
        9,
        "Dungeon of the Mad Mage should have 9 rooms"
    );
    assert_eq!(
        get_dungeon(DungeonId::TombOfAnnihilation).rooms.len(),
        5,
        "Tomb of Annihilation should have 5 rooms"
    );
    assert_eq!(
        get_dungeon(DungeonId::TheUndercity).rooms.len(),
        7,
        "The Undercity should have 7 rooms"
    );
}

/// CR 309.4: Verify that a new game has empty dungeon_state for all players.
///
/// At game start, no player has a dungeon in their command zone.
/// The `dungeon_state` map should be empty, and `has_initiative` should be None.
#[test]
fn test_dungeon_state_default() {
    let state = GameStateBuilder::four_player()
        .build()
        .expect("build failed");

    // No player should have a dungeon at game start
    assert!(
        state.dungeon_state.is_empty(),
        "dungeon_state should be empty at game start, got: {:?}",
        state.dungeon_state
    );

    // No player should have the initiative at game start
    assert!(
        state.has_initiative.is_none(),
        "has_initiative should be None at game start, got: {:?}",
        state.has_initiative
    );

    // All players should have 0 dungeons_completed
    for (player_id, player) in &state.players {
        assert_eq!(
            player.dungeons_completed, 0,
            "player {:?} should have 0 dungeons_completed at game start",
            player_id
        );
    }
}

/// CR 309.4, 725.1: Verify that dungeon-related state contributes to the public state hash
/// and that identical states produce the same hash (determinism).
///
/// This tests that `hash.rs` correctly includes the new fields.
#[test]
fn test_dungeon_hash_determinism() {
    // Two identical states should produce the same hash
    let state1 = GameStateBuilder::four_player()
        .build()
        .expect("build failed");
    let state2 = GameStateBuilder::four_player()
        .build()
        .expect("build failed");

    let hash1 = state1.public_state_hash();
    let hash2 = state2.public_state_hash();
    assert_eq!(
        hash1, hash2,
        "identical states must produce the same public state hash"
    );

    // A state with a dungeon entry should produce a different hash than one without
    let mut state_with_dungeon = GameStateBuilder::four_player()
        .build()
        .expect("build failed");

    state_with_dungeon.dungeon_state.insert(
        p(1),
        DungeonState {
            dungeon: DungeonId::LostMineOfPhandelver,
            current_room: 0,
        },
    );

    let hash_no_dungeon = state1.public_state_hash();
    let hash_with_dungeon = state_with_dungeon.public_state_hash();
    assert_ne!(
        hash_no_dungeon, hash_with_dungeon,
        "state with dungeon must hash differently from state without dungeon"
    );

    // Changing the dungeon type should change the hash
    let mut state_other_dungeon = GameStateBuilder::four_player()
        .build()
        .expect("build failed");
    state_other_dungeon.dungeon_state.insert(
        p(1),
        DungeonState {
            dungeon: DungeonId::TombOfAnnihilation,
            current_room: 0,
        },
    );
    let hash_other_dungeon = state_other_dungeon.public_state_hash();
    assert_ne!(
        hash_with_dungeon, hash_other_dungeon,
        "different dungeon types must produce different hashes"
    );

    // Advancing to a different room should change the hash
    let mut state_room1 = GameStateBuilder::four_player()
        .build()
        .expect("build failed");
    state_room1.dungeon_state.insert(
        p(1),
        DungeonState {
            dungeon: DungeonId::LostMineOfPhandelver,
            current_room: 1,
        },
    );
    let hash_room1 = state_room1.public_state_hash();
    assert_ne!(
        hash_with_dungeon, hash_room1,
        "different current_room must produce different hashes"
    );

    // A state with has_initiative set should hash differently
    let mut state_with_initiative = GameStateBuilder::four_player()
        .build()
        .expect("build failed");
    state_with_initiative.has_initiative = Some(p(1));
    let hash_with_initiative = state_with_initiative.public_state_hash();
    assert_ne!(
        hash_no_dungeon, hash_with_initiative,
        "state with initiative must hash differently from state without initiative"
    );
}
