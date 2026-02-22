//! Extra turn tests.

use im::Vector;
use mtg_engine::rules::engine::process_command;
use mtg_engine::{
    Command, GameEvent, GameState, GameStateBuilder, ObjectSpec, PlayerId, Step, ZoneId,
};

fn pass(state: GameState, player: PlayerId) -> (GameState, Vec<GameEvent>) {
    process_command(state, Command::PassPriority { player }).unwrap()
}

/// Build a 4-player state with enough library cards that nobody decks out.
fn four_player_with_libraries(step: Step) -> GameState {
    let mut builder = GameStateBuilder::four_player().at_step(step);
    for pid in 1..=4 {
        let player = PlayerId(pid);
        for i in 0..10 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Card {} P{}", i, pid))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder.build().unwrap()
}

/// Complete the current turn by passing priority until the turn number increases.
fn complete_turn(mut state: GameState) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let start_turn = state.turn.turn_number;
    loop {
        let holder = state.turn.priority_holder.expect("no priority holder");
        let (new_state, events) = pass(state, holder);
        all_events.extend(events);
        state = new_state;
        if state.turn.turn_number > start_turn {
            return (state, all_events);
        }
    }
}

#[test]
/// Extra turns are LIFO — most recently added goes first
fn test_extra_turns_lifo() {
    let mut state = four_player_with_libraries(Step::End);

    // Add extra turns: P2 first, then P3
    state.turn.extra_turns = Vector::from(vec![PlayerId(2), PlayerId(3)]);

    // Complete current turn (P1's End step)
    let (state, _) = complete_turn(state);

    // P3 should get the next turn (LIFO — last added goes first)
    assert_eq!(state.turn.active_player, PlayerId(3));
}

#[test]
/// Extra turn: designated player becomes active
fn test_extra_turn_designated_player_active() {
    let mut state = four_player_with_libraries(Step::End);

    state.turn.extra_turns.push_back(PlayerId(3));

    let (state, events) = complete_turn(state);

    assert_eq!(state.turn.active_player, PlayerId(3));
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::TurnStarted { player, .. } if *player == PlayerId(3)
    )));
}

#[test]
/// After an extra turn, normal turn order resumes
fn test_extra_turn_normal_order_resumes() {
    let mut state = four_player_with_libraries(Step::End);

    // P1's turn, with an extra turn for P3 queued
    state.turn.extra_turns.push_back(PlayerId(3));

    // Complete P1's turn → P3's extra turn
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(3));

    // Complete P3's extra turn → normal order resumes (P2 is next after P1)
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(2));
}

#[test]
/// Multiple extra turns stack LIFO
fn test_multiple_extra_turns_stack() {
    let mut state = four_player_with_libraries(Step::End);

    // Stack three extra turns
    state.turn.extra_turns.push_back(PlayerId(2));
    state.turn.extra_turns.push_back(PlayerId(3));
    state.turn.extra_turns.push_back(PlayerId(4));

    // P4 goes first (LIFO)
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(4));

    // Then P3
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(3));

    // Then P2
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(2));

    // Then back to normal order (P2 is after P1)
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(2));
}
