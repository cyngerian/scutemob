//! Property-based tests for turn structure invariants.

use mtg_engine::rules::engine::{process_command, start_game};
use mtg_engine::{Command, GameState, GameStateBuilder, PlayerId};
use proptest::prelude::*;

/// Run N random PassPriority commands and verify invariants hold.
fn run_pass_sequence(num_passes: usize) -> GameState {
    let state = GameStateBuilder::four_player().build().unwrap();
    let (mut state, _) = start_game(state).unwrap();

    for _ in 0..num_passes {
        let holder = match state.turn.priority_holder {
            Some(h) => h,
            None => return state, // game ended
        };
        match process_command(state.clone(), Command::PassPriority { player: holder }) {
            Ok((new_state, _)) => state = new_state,
            Err(_) => return state,
        }
    }
    state
}

proptest! {
    #[test]
    /// Random PassPriority sequences never produce invalid state
    fn prop_pass_priority_never_invalid(num_passes in 1..500usize) {
        let state = run_pass_sequence(num_passes);

        // State should always be valid
        let active = state.active_players();
        // Either game is ongoing with active players or it ended
        prop_assert!(active.len() >= 1 || state.turn.priority_holder.is_none());
    }

    #[test]
    /// Priority holder is always a valid, non-eliminated player
    fn prop_priority_holder_always_valid(num_passes in 1..500usize) {
        let state = run_pass_sequence(num_passes);

        if let Some(holder) = state.turn.priority_holder {
            let player = state.player(holder).unwrap();
            prop_assert!(!player.has_lost, "priority holder {:?} has lost", holder);
            prop_assert!(!player.has_conceded, "priority holder {:?} has conceded", holder);
        }
    }

    #[test]
    /// Turn number monotonically increases
    fn prop_turn_number_monotonic(num_passes in 1..200usize) {
        let state = GameStateBuilder::four_player().build().unwrap();
        let (mut state, _) = start_game(state).unwrap();
        let mut last_turn = state.turn.turn_number;

        for _ in 0..num_passes {
            let holder = match state.turn.priority_holder {
                Some(h) => h,
                None => break,
            };
            match process_command(state.clone(), Command::PassPriority { player: holder }) {
                Ok((new_state, _)) => {
                    prop_assert!(new_state.turn.turn_number >= last_turn,
                        "turn number decreased: {} -> {}",
                        last_turn, new_state.turn.turn_number);
                    last_turn = new_state.turn.turn_number;
                    state = new_state;
                }
                Err(_) => break,
            }
        }
    }

    #[test]
    /// Eliminated player never gets priority (via concessions)
    fn prop_eliminated_never_gets_priority(
        concede_player in 2u64..=4u64,
        num_passes in 1..300usize
    ) {
        let state = GameStateBuilder::four_player().build().unwrap();
        let (state, _) = start_game(state).unwrap();

        let target = PlayerId(concede_player);
        let (mut state, _) = process_command(
            state,
            Command::Concede { player: target }
        ).unwrap();

        for _ in 0..num_passes {
            // The eliminated player should never hold priority
            prop_assert!(state.turn.priority_holder != Some(target),
                "eliminated player {:?} holds priority", target);

            let holder = match state.turn.priority_holder {
                Some(h) => h,
                None => break,
            };
            match process_command(state.clone(), Command::PassPriority { player: holder }) {
                Ok((new_state, _)) => state = new_state,
                Err(_) => break,
            }
        }
    }
}
