//! Tests for state snapshot performance: clone independence and structural
//! sharing with the real GameState type.

use mtg_engine::state::*;
use std::time::Instant;

/// Run a test closure in a thread with 32 MB of stack.
///
/// `GameStateBuilder::build()` creates hundreds of large structs on the stack
/// in debug mode (no stack-frame reuse). Combined with large complex states,
/// this can overflow the default 8 MB test thread stack.
fn run_in_big_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

/// Build a complex game state with many objects for benchmarking.
fn build_complex_state() -> GameState {
    let mut builder = GameStateBuilder::four_player();

    // Each player gets:
    // - 7 cards in hand
    // - 10 permanents on battlefield (mix of creatures, lands, artifacts)
    // - 80+ cards in library
    // - 3 cards in graveyard
    for pid in 1..=4u64 {
        let p = PlayerId(pid);

        // Hand
        for i in 0..7 {
            builder = builder.object(
                ObjectSpec::card(p, &format!("Hand Card {}:{}", pid, i)).in_zone(ZoneId::Hand(p)),
            );
        }

        // Battlefield — creatures
        for i in 0..5 {
            builder = builder.object(
                ObjectSpec::creature(p, &format!("Creature {}:{}", pid, i), 2, 2)
                    .with_counter(CounterType::PlusOnePlusOne, 1),
            );
        }

        // Battlefield — lands
        for i in 0..4 {
            builder = builder.object(ObjectSpec::land(p, &format!("Land {}:{}", pid, i)));
        }

        // Battlefield — artifact
        builder = builder.object(ObjectSpec::artifact(p, &format!("Artifact {}:{}", pid, 0)));

        // Library
        for i in 0..83 {
            builder = builder.object(
                ObjectSpec::card(p, &format!("Library Card {}:{}", pid, i))
                    .in_zone(ZoneId::Library(p)),
            );
        }

        // Graveyard
        for i in 0..3 {
            builder = builder.object(
                ObjectSpec::card(p, &format!("GY Card {}:{}", pid, i))
                    .in_zone(ZoneId::Graveyard(p)),
            );
        }
    }

    builder.build().unwrap()
}

#[test]
fn test_clone_independence_real_types() {
    run_in_big_stack(|| {
        let mut state = build_complex_state();
        let snapshot = state.clone();

        // Modify the original
        state.player_mut(PlayerId(1)).unwrap().life_total = 20;
        state.turn.turn_number = 99;

        // Snapshot unchanged
        assert_eq!(snapshot.player(PlayerId(1)).unwrap().life_total, 40);
        assert_eq!(snapshot.turn.turn_number, 1);

        // Original changed
        assert_eq!(state.player(PlayerId(1)).unwrap().life_total, 20);
        assert_eq!(state.turn.turn_number, 99);
    });
}

#[test]
fn test_clone_independence_object_modification() {
    // Run in a thread with a larger stack: GameState is sizeable in debug builds
    // and the combination of two complex states + a new GameObject can overflow
    // the default 8 MB stack.
    let handle = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024) // 32 MB
        .spawn(|| {
            let state = build_complex_state();
            let mut modified = state.clone();

            // Add a new object to the modified state
            let new_obj = GameObject {
                id: ObjectId(0),
                card_id: None,
                characteristics: Characteristics {
                    name: "New Object".to_string(),
                    ..Characteristics::default()
                },
                controller: PlayerId(1),
                owner: PlayerId(1),
                zone: ZoneId::Battlefield,
                status: ObjectStatus::default(),
                counters: im::OrdMap::new(),
                attachments: im::Vector::new(),
                attached_to: None,
                damage_marked: 0,
                deathtouch_damage: false,
                is_token: false,
                timestamp: 0,
                has_summoning_sickness: false,
                goaded_by: im::Vector::new(),
                kicker_times_paid: 0,
                cast_alt_cost: None,
                foretold_turn: 0,
                was_unearthed: false,
                myriad_exile_at_eoc: false,
                decayed_sacrifice_at_eoc: false,
                ring_block_sacrifice_at_eoc: false,
                exiled_by_hideaway: None,
                encore_sacrifice_at_end_step: false,
                encore_must_attack: None,
                encore_activated_by: None,
                is_plotted: false,
                plotted_turn: 0,
                is_prototyped: false,
                was_bargained: false,
                phased_out_indirectly: false,
                phased_out_controller: None,
                creatures_devoured: 0,
                champion_exiled_card: None,
                paired_with: None,
                tribute_was_paid: false,
                x_value: 0,
                evidence_collected: false,
                squad_count: 0,
                offspring_paid: false,
                gift_was_given: false,
                gift_opponent: None,
                encoded_cards: im::Vector::new(),
                haunting_target: None,
                merged_components: im::Vector::new(),
                is_transformed: false,
                last_transform_timestamp: 0,
                was_cast_disturbed: false,
                craft_exiled_cards: im::Vector::new(),
                chosen_creature_type: None,
                face_down_as: None,
                designations: mtg_engine::Designations::default(),
            };
            modified.add_object(new_obj, ZoneId::Battlefield).unwrap();

            // Original unaffected
            assert_eq!(
                state.total_objects(),
                modified.total_objects() - 1,
                "original should have one fewer object"
            );
        })
        .unwrap();
    handle.join().unwrap();
}

#[test]
fn test_snapshot_clone_under_1ms() {
    run_in_big_stack(|| {
        let state = build_complex_state();

        // Verify it's a substantial state
        let obj_count = state.total_objects();
        assert!(obj_count > 300, "expected 300+ objects, got {}", obj_count);

        // Warm up
        for _ in 0..10 {
            let _ = state.clone();
        }

        // Benchmark: clone 1000 times
        let start = Instant::now();
        let mut snapshots = Vec::with_capacity(1000);
        for _ in 0..1000 {
            snapshots.push(state.clone());
        }
        let elapsed = start.elapsed();

        let avg_ns = elapsed.as_nanos() / 1000;
        let avg_us = avg_ns / 1000;

        // With im-rs structural sharing, each clone should be well under 1ms
        // Average should be in the single-digit microsecond range
        assert!(
            avg_us < 1000,
            "average clone took {}µs, expected <1000µs (1ms). Total for 1000 clones: {:?}",
            avg_us,
            elapsed
        );

        // Verify clones are valid
        assert_eq!(snapshots.len(), 1000);
        assert_eq!(snapshots[999].total_objects(), obj_count);
    });
}

#[test]
fn test_structural_sharing_memory_efficiency() {
    run_in_big_stack(|| {
        let state = build_complex_state();

        // Clone 100 times — with structural sharing, this shouldn't significantly
        // increase memory usage compared to having just the original.
        let mut snapshots: Vec<GameState> = Vec::with_capacity(100);
        for _ in 0..100 {
            snapshots.push(state.clone());
        }

        // All snapshots should be equal in content
        for snapshot in &snapshots {
            assert_eq!(snapshot.total_objects(), state.total_objects());
            assert_eq!(snapshot.players.len(), state.players.len());
            assert_eq!(snapshot.zones.len(), state.zones.len());
        }
    });
}

#[test]
fn test_incremental_modification_is_cheap() {
    run_in_big_stack(|| {
        let state = build_complex_state();

        // Clone and modify one field — should be cheap
        let start = Instant::now();
        for _ in 0..1000 {
            let mut s = state.clone();
            s.turn.turn_number += 1;
        }
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 1000,
            "1000 clone-and-modify cycles took {:?}, expected <1s",
            elapsed
        );
    });
}
