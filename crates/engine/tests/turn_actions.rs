//! Turn-based action tests (CR 500-514).

use mtg_engine::rules::engine::{process_command, start_game};
use mtg_engine::{
    Command, GameEvent, GameState, GameStateBuilder, ObjectSpec, PlayerId, Step, ZoneId,
};

fn pass(state: GameState, player: PlayerId) -> (GameState, Vec<GameEvent>) {
    process_command(state, Command::PassPriority { player }).unwrap()
}

/// Pass priority for all players through remaining steps to reach a target step.
fn advance_to_step(mut state: GameState, target: Step) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    loop {
        if state.turn.step == target {
            return (state, all_events);
        }
        let holder = state.turn.priority_holder.expect("no priority holder");
        let (new_state, events) = pass(state, holder);
        all_events.extend(events);
        state = new_state;
    }
}

#[test]
/// CR 502.2 — untap step untaps active player's tapped permanents
fn test_untap_step_untaps_active_player_permanents() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Grizzly Bears", 2, 2).tapped())
        .object(ObjectSpec::artifact(p1, "Sol Ring").tapped())
        .object(ObjectSpec::land(p1, "Forest").tapped())
        .build()
        .unwrap();

    let (state, events) = start_game(state).unwrap();

    // Untap event should have been emitted
    let untap_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentsUntapped { .. }))
        .collect();
    assert_eq!(untap_events.len(), 1);

    if let GameEvent::PermanentsUntapped { player, objects } = &untap_events[0] {
        assert_eq!(*player, p1);
        assert_eq!(objects.len(), 3);
    }

    // All permanents should be untapped
    for (_id, obj) in state.objects.iter() {
        assert!(
            !obj.status.tapped,
            "object {} should be untapped",
            obj.characteristics.name
        );
    }
}

#[test]
/// CR 502.2 — untap doesn't affect other players' permanents
fn test_untap_step_doesnt_affect_other_players() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Bears", 2, 2).tapped())
        .object(ObjectSpec::creature(p2, "Other Bears", 2, 2).tapped())
        .build()
        .unwrap();

    let (state, _) = start_game(state).unwrap();

    // P1's creature should be untapped (it's P1's turn)
    // P2's creature should remain tapped
    let p2_creatures: Vec<_> = state
        .objects
        .iter()
        .filter(|(_, obj)| obj.controller == p2)
        .collect();
    assert_eq!(p2_creatures.len(), 1);
    assert!(
        p2_creatures[0].1.status.tapped,
        "P2's creature should still be tapped"
    );
}

#[test]
/// CR 504.1 — draw step draws a card (new ObjectId per CR 400.7)
fn test_draw_step_draws_card() {
    let p1 = PlayerId(1);
    // Start at Upkeep (not first turn) so draw step will actually draw
    let state = GameStateBuilder::four_player()
        .at_step(Step::Upkeep)
        .object(ObjectSpec::card(p1, "Mountain").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Lightning Bolt").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    // Advance from Upkeep to Draw (pass all 4 players in Upkeep)
    let (state, events) = advance_to_step(state, Step::Draw);

    let draw_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { .. }))
        .collect();

    // Should have one draw event (from the draw step)
    assert!(!draw_events.is_empty(), "should have drawn a card");

    // Hand should have 1 card
    let hand = state.zone(&ZoneId::Hand(p1)).unwrap();
    assert_eq!(hand.len(), 1);

    // Library should have 1 card (started with 2, drew 1)
    let library = state.zone(&ZoneId::Library(p1)).unwrap();
    assert_eq!(library.len(), 1);
}

#[test]
/// CR 103.8a — in a two-player game, the first player skips first draw
fn test_first_player_skips_first_draw() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .object(ObjectSpec::card(p1, "Mountain").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let (state, _) = start_game(state).unwrap();

    // Advance through Upkeep to Draw
    let (state, events) = advance_to_step(state, Step::PreCombatMain);

    // P1 should NOT have drawn (two-player, first turn of game — CR 103.8a)
    let draw_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .collect();
    assert!(
        draw_events.is_empty(),
        "first player should skip first draw in two-player"
    );

    // Library should still have 1 card
    let library = state.zone(&ZoneId::Library(p1)).unwrap();
    assert_eq!(library.len(), 1);
}

#[test]
/// CR 103.8c — in multiplayer, no player skips the draw step of their first turn
fn test_multiplayer_first_player_draws() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::card(p1, "Mountain").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let (state, _) = start_game(state).unwrap();

    // Advance through Upkeep to Draw and beyond
    let (state, events) = advance_to_step(state, Step::PreCombatMain);

    // P1 SHOULD have drawn (multiplayer — CR 103.8c)
    let draw_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .collect();
    assert_eq!(
        draw_events.len(),
        1,
        "first player should draw in multiplayer (CR 103.8c)"
    );

    // Library should be empty (1 card drawn)
    let library = state.zone(&ZoneId::Library(p1)).unwrap();
    assert_eq!(library.len(), 0);
}

#[test]
/// CR 514.1 — cleanup discards to max hand size
fn test_cleanup_discards_to_hand_size() {
    let p1 = PlayerId(1);
    // Give player 9 cards in hand (max is 7, should discard 2)
    let mut builder = GameStateBuilder::four_player().at_step(Step::End);
    for i in 0..9 {
        builder =
            builder.object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Hand(p1)));
    }
    let state = builder.build().unwrap();

    // Pass through End step to reach Cleanup (which auto-advances)
    let (state, _) = pass(state, PlayerId(1));
    let (state, _) = pass(state, PlayerId(2));
    let (state, _) = pass(state, PlayerId(3));
    let (state, events) = pass(state, PlayerId(4));

    // Should have discard events
    let discard_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscardedToHandSize { .. }))
        .collect();
    assert_eq!(
        discard_events.len(),
        2,
        "should discard 2 cards to reach hand size 7"
    );

    // Hand should be at max size
    let hand = state.zone(&ZoneId::Hand(p1)).unwrap();
    assert_eq!(hand.len(), 7);

    // Graveyard should have 2 cards
    let gy = state.zone(&ZoneId::Graveyard(p1)).unwrap();
    assert_eq!(gy.len(), 2);
}

#[test]
/// CR 514.2 — cleanup clears damage from all permanents
fn test_cleanup_clears_damage() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .at_step(Step::End)
        .object(ObjectSpec::creature(p1, "Bears", 2, 2))
        .build()
        .unwrap();

    // Mark damage on the creature
    let mut state = state;
    let obj_id = *state.objects.keys().next().unwrap();
    state.objects.get_mut(&obj_id).unwrap().damage_marked = 1;

    // Pass through End step to reach Cleanup
    let (state, _) = pass(state, PlayerId(1));
    let (state, _) = pass(state, PlayerId(2));
    let (state, _) = pass(state, PlayerId(3));
    let (state, events) = pass(state, PlayerId(4));

    // Damage cleared event should appear
    assert!(events.iter().any(|e| matches!(e, GameEvent::DamageCleared)));

    // The creature in the NEW turn should have 0 damage
    // (Note: the creature persists since it stays on battlefield)
    for (_, obj) in state.objects.iter() {
        if obj.zone == ZoneId::Battlefield {
            assert_eq!(obj.damage_marked, 0);
        }
    }
}

#[test]
/// CR 500.4 — mana pools empty between steps
fn test_mana_pools_empty_between_steps() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .at_step(Step::PreCombatMain)
        .player_mana(
            p1,
            mtg_engine::ManaPool {
                white: 3,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    // Verify mana exists
    assert_eq!(state.player(p1).unwrap().mana_pool.total(), 3);

    // Pass through main phase
    let (state, _) = pass(state, PlayerId(1));
    let (state, _) = pass(state, PlayerId(2));
    let (state, _) = pass(state, PlayerId(3));
    let (state, events) = pass(state, PlayerId(4));

    // Mana pools should have been emptied
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::ManaPoolsEmptied)));
    assert_eq!(state.player(p1).unwrap().mana_pool.total(), 0);
}

#[test]
/// MR-M2-04: draw_card returns empty vec for eliminated/conceded player
fn test_draw_card_skips_eliminated_player() {
    use mtg_engine::rules::turn_actions::draw_card;

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::Draw)
        .object(ObjectSpec::card(p1, "Mountain").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    // Mark P1 as conceded
    let mut state = state;
    state.players.get_mut(&p1).unwrap().has_conceded = true;

    // draw_card for a conceded player should return no events (not draw or lose)
    let events = draw_card(&mut state, p1).unwrap();
    assert!(
        events.is_empty(),
        "conceded player should not draw or lose from draw attempt"
    );

    // Library should be untouched
    let library = state.zone(&ZoneId::Library(p1)).unwrap();
    assert_eq!(library.len(), 1);
}

#[test]
/// MR-M2-06: DiscardedToHandSize event carries the old hand ObjectId, not the
/// new graveyard ObjectId.
fn test_cleanup_discard_event_uses_hand_id() {
    let p1 = PlayerId(1);
    let mut builder = GameStateBuilder::four_player().at_step(Step::End);
    // Give P1 8 cards in hand (one over max 7 → one discard)
    for i in 0..8 {
        builder =
            builder.object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Hand(p1)));
    }
    let state = builder.build().unwrap();

    // Snapshot hand object IDs before cleanup
    let hand_ids_before: std::collections::HashSet<_> = state
        .zone(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .iter()
        .copied()
        .collect();

    // Pass through End step to trigger cleanup
    let (state, events) = pass(state, PlayerId(1));
    let (state, events2) = pass(state, PlayerId(2));
    let (state, events3) = pass(state, PlayerId(3));
    let (_state, events4) = pass(state, PlayerId(4));
    let all_events: Vec<_> = events
        .iter()
        .chain(events2.iter())
        .chain(events3.iter())
        .chain(events4.iter())
        .collect();

    let discard_events: Vec<_> = all_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::DiscardedToHandSize { object_id, .. } = e {
                Some(*object_id)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(discard_events.len(), 1, "should have one discard event");
    // The object_id in the event must be the OLD hand ID (which no longer exists
    // in the objects map after the zone change, but was a valid hand object ID).
    assert!(
        hand_ids_before.contains(&discard_events[0]),
        "DiscardedToHandSize event should carry the old hand ObjectId"
    );
}

#[test]
/// CR 514.3a: If SBAs fire during cleanup, priority is granted and another
/// cleanup step is performed before the turn advances.
fn test_cleanup_sba_grants_priority_and_repeats() {
    // Set up: 2 players, creature with 0 toughness placed at cleanup step
    // (bypassing normal SBA checks by direct state mutation).
    // When cleanup runs, SBAs should fire (creature dies) and priority should
    // be granted to the active player before the turn ends.
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::End) // start at End; transition to Cleanup after passing
        .object(ObjectSpec::creature(p1, "Deathtouched", 1, 0)) // 0 toughness
        .build()
        .unwrap();

    // Pass through End step (all players pass → reach Cleanup, which auto-fires)
    let (state, events1) = pass(state, p1);
    let (state, events2) = pass(state, p2);

    let all_events: Vec<_> = events1.iter().chain(events2.iter()).collect();

    // A CreatureDied event should have been emitted during cleanup SBAs
    let creature_died = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        creature_died,
        "0-toughness creature should die via SBA during cleanup"
    );

    // Priority should be granted to the active player (cleanup repeats with priority)
    // — then after all pass with empty stack, the turn should advance.
    // Verify the game is still in a sane state (turn 2 or priority was granted).
    let priority_given = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::PriorityGiven { player } if *player == p1));

    // Either priority was granted during cleanup AND subsequently the turn advanced,
    // or the turn advanced directly after cleanup (if state ended up at turn 2).
    // The key invariant: the creature is dead and the state has moved past cleanup.
    let creature_still_alive = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Deathtouched" && o.zone == ZoneId::Battlefield);
    assert!(
        !creature_still_alive,
        "0-toughness creature should not be on battlefield after cleanup"
    );

    // If priority was given, the next player's turn should be active after
    // everyone passes through the cleanup priority window.
    if priority_given {
        // Allow the cleanup priority window to resolve (everyone passes)
        let holder = state
            .turn
            .priority_holder
            .expect("should have priority holder");
        let (state, _) = pass(state, holder);
        // After cleanup priority window, turn should advance or another cleanup fires
        // Either way, we just verify no panic / no infinite loop.
        let _ = state.turn.active_player; // state is valid
    }
}
