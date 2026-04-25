//! Property-based tests for game state invariants using proptest.
//!
//! These tests verify that state invariants hold across random state
//! configurations and operations.

use mtg_engine::state::*;
use proptest::prelude::*;

/// Verify that every object is in exactly one zone.
fn check_zone_integrity(state: &GameState) {
    for (obj_id, obj) in state.objects.iter() {
        // Object's zone exists
        let zone = state.zone(&obj.zone);
        assert!(
            zone.is_ok(),
            "object {:?} references non-existent zone {:?}",
            obj_id,
            obj.zone
        );

        // Zone contains this object
        assert!(
            zone.unwrap().contains(obj_id),
            "object {:?} not found in its zone {:?}",
            obj_id,
            obj.zone
        );

        // Object is in exactly one zone
        let mut count = 0;
        for (_, z) in state.zones.iter() {
            if z.contains(obj_id) {
                count += 1;
            }
        }
        assert_eq!(count, 1, "object {:?} found in {} zones", obj_id, count);
    }

    // Every ID in a zone is in the objects map
    for (zone_id, zone) in state.zones.iter() {
        for obj_id in zone.object_ids() {
            assert!(
                state.objects.contains_key(&obj_id),
                "zone {:?} has orphaned object {:?}",
                zone_id,
                obj_id
            );
        }
    }
}

/// Verify all ObjectIds are unique.
fn check_unique_ids(state: &GameState) {
    let ids: Vec<ObjectId> = state.objects.keys().cloned().collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "duplicate ObjectId found");
        }
    }
}

proptest! {
    /// Zone integrity holds for random object counts across 4 players.
    #[test]
    fn prop_zone_integrity(
        creature_count in 0u32..20,
        hand_count in 0u32..10,
        gy_count in 0u32..5,
    ) {
        let p1 = PlayerId(1);
        let mut builder = GameStateBuilder::four_player();

        for i in 0..creature_count {
            builder = builder.object(
                ObjectSpec::creature(p1, &format!("Creature {}", i), 1, 1),
            );
        }
        for i in 0..hand_count {
            builder = builder.object(
                ObjectSpec::card(p1, &format!("Card {}", i))
                    .in_zone(ZoneId::Hand(p1)),
            );
        }
        for i in 0..gy_count {
            builder = builder.object(
                ObjectSpec::card(p1, &format!("GY {}", i))
                    .in_zone(ZoneId::Graveyard(p1)),
            );
        }

        let state = builder.build().unwrap();
        check_zone_integrity(&state);
    }

    /// All ObjectIds are unique regardless of how many objects are created.
    #[test]
    fn prop_unique_object_ids(obj_count in 1u32..50) {
        let p1 = PlayerId(1);
        let mut builder = GameStateBuilder::four_player();

        for i in 0..obj_count {
            builder = builder.object(
                ObjectSpec::creature(p1, &format!("Obj {}", i), 1, 1),
            );
        }

        let state = builder.build().unwrap();
        check_unique_ids(&state);
        prop_assert_eq!(state.total_objects(), obj_count as usize);
    }

    /// Player count is preserved correctly.
    #[test]
    fn prop_player_count(num_players in 1u64..=8) {
        let mut builder = GameStateBuilder::new();
        for i in 1..=num_players {
            builder = builder.add_player(PlayerId(i));
        }
        let state = builder.build().unwrap();
        prop_assert_eq!(state.players.len(), num_players as usize);
        prop_assert_eq!(state.turn.turn_order.len(), num_players as usize);
    }

    /// Move preserves total object count.
    #[test]
    fn prop_move_preserves_object_count(
        initial_count in 1u32..10,
    ) {
        let p1 = PlayerId(1);
        let mut builder = GameStateBuilder::four_player();
        for i in 0..initial_count {
            builder = builder.object(
                ObjectSpec::creature(p1, &format!("C {}", i), 1, 1),
            );
        }
        let mut state = builder.build().unwrap();
        let count_before = state.total_objects();

        // Move the first object to graveyard
        let first_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;
        state
            .move_object_to_zone(first_id, ZoneId::Graveyard(p1))
            .unwrap();

        // Object count unchanged (old removed, new created)
        prop_assert_eq!(state.total_objects(), count_before);
        check_zone_integrity(&state);
    }

    /// Zone integrity holds after a sequence of moves.
    #[test]
    fn prop_zone_integrity_after_moves(num_moves in 1u32..10) {
        let p1 = PlayerId(1);
        let mut state = GameStateBuilder::four_player()
            .object(ObjectSpec::creature(p1, "Nomad", 2, 2))
            .build().unwrap();

        let zones = [
            ZoneId::Graveyard(p1),
            ZoneId::Exile,
            ZoneId::Hand(p1),
            ZoneId::Library(p1),
            ZoneId::Battlefield,
        ];

        for i in 0..num_moves {
            let target_zone = zones[i as usize % zones.len()];
            // Find the object (it might be in any zone after previous moves)
            let obj_id = state.objects.keys().next().copied().unwrap();
            state.move_object_to_zone(obj_id, target_zone).unwrap();
        }

        check_zone_integrity(&state);
        prop_assert_eq!(state.total_objects(), 1);
    }
}
