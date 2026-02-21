//! Concession and player elimination tests (CR 104).

use mtg_engine::rules::engine::process_command;
use mtg_engine::{
    Command, GameEvent, GameState, GameStateBuilder, GameStateError, ObjectSpec, PlayerId, Step,
    ZoneId,
};

fn pass(state: GameState, player: PlayerId) -> (GameState, Vec<GameEvent>) {
    process_command(state, Command::PassPriority { player }).unwrap()
}

fn concede(state: GameState, player: PlayerId) -> (GameState, Vec<GameEvent>) {
    process_command(state, Command::Concede { player }).unwrap()
}

/// Build a 4-player state with library cards so nobody decks.
fn four_player_at(step: Step) -> GameState {
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
    builder.build()
}

#[test]
/// Conceded player is skipped in priority ordering
fn test_concede_player_skipped_in_priority() {
    let state = four_player_at(Step::PreCombatMain);

    // P2 concedes
    let (state, _) = concede(state, PlayerId(2));

    // P1 passes → should skip P2 → P3
    let (state, _) = pass(state, PlayerId(1));
    assert_eq!(state.turn.priority_holder, Some(PlayerId(3)));
}

#[test]
/// Conceded player's turn is skipped
fn test_concede_player_turn_skipped() {
    let state = four_player_at(Step::End);

    // P2 concedes during P1's turn
    let (state, _) = concede(state, PlayerId(2));

    // Complete P1's turn by passing for remaining active players
    let mut state = state;
    loop {
        if let Some(holder) = state.turn.priority_holder {
            let old_turn = state.turn.turn_number;
            let (new_state, _) = pass(state, holder);
            state = new_state;
            if state.turn.turn_number > old_turn {
                break;
            }
        } else {
            break;
        }
    }

    // P2's turn should be skipped → P3 is active
    assert_eq!(state.turn.active_player, PlayerId(3));
}

#[test]
/// Game continues with remaining players after concession
fn test_concede_game_continues() {
    let state = four_player_at(Step::PreCombatMain);

    let (state, events) = concede(state, PlayerId(3));

    // No GameOver event — 3 players still alive
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::GameOver { .. })));

    // Game should continue
    assert!(state.turn.priority_holder.is_some());
}

#[test]
/// Last player standing wins
fn test_concede_last_player_wins() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .build();

    let (_state, events) = concede(state, PlayerId(2));

    // Game should be over with P1 as winner
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::GameOver { winner: Some(p) } if *p == PlayerId(1)
    )));
}

#[test]
/// Eliminated player can't act (pass priority or concede again)
fn test_eliminated_player_cannot_act() {
    let state = four_player_at(Step::PreCombatMain);

    let (state, _) = concede(state, PlayerId(2));

    // P2 trying to pass priority should error
    let result = process_command(
        state.clone(),
        Command::PassPriority {
            player: PlayerId(2),
        },
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        GameStateError::PlayerEliminated(p) => assert_eq!(p, PlayerId(2)),
        e => panic!("expected PlayerEliminated, got {:?}", e),
    }

    // P2 trying to concede again should also error
    let result = process_command(
        state,
        Command::Concede {
            player: PlayerId(2),
        },
    );
    assert!(result.is_err());
}
