//! Priority system tests (CR 116-117).

use mtg_engine::rules::engine::process_command;
use mtg_engine::{Command, GameEvent, GameState, GameStateBuilder, GameStateError, PlayerId, Step};

fn pass(state: GameState, player: PlayerId) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(state, Command::PassPriority { player })
}

#[test]
/// CR 116.3a — active player receives priority first at each step
fn test_priority_active_player_gets_priority_first() {
    let state = GameStateBuilder::four_player()
        .active_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .build();

    assert_eq!(state.turn.priority_holder, Some(PlayerId(2)));
}

#[test]
/// CR 116.3d — APNAP order: active player, then clockwise (1→2→3→4)
fn test_priority_apnap_order() {
    let state = GameStateBuilder::four_player()
        .at_step(Step::PreCombatMain)
        .build();

    // Player 1 (active) has priority
    assert_eq!(state.turn.priority_holder, Some(PlayerId(1)));

    // Player 1 passes → player 2 gets priority
    let (state, events) = pass(state, PlayerId(1)).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::PriorityPassed { player } if *player == PlayerId(1))));
    assert_eq!(state.turn.priority_holder, Some(PlayerId(2)));

    // Player 2 passes → player 3 gets priority
    let (state, _) = pass(state, PlayerId(2)).unwrap();
    assert_eq!(state.turn.priority_holder, Some(PlayerId(3)));

    // Player 3 passes → player 4 gets priority
    let (state, _) = pass(state, PlayerId(3)).unwrap();
    assert_eq!(state.turn.priority_holder, Some(PlayerId(4)));
}

#[test]
/// CR 116.3d — all players pass → step advances
fn test_priority_all_pass_step_advances() {
    let state = GameStateBuilder::four_player()
        .at_step(Step::PreCombatMain)
        .build();

    let (state, _) = pass(state, PlayerId(1)).unwrap();
    let (state, _) = pass(state, PlayerId(2)).unwrap();
    let (state, _) = pass(state, PlayerId(3)).unwrap();
    let (state, events) = pass(state, PlayerId(4)).unwrap();

    // All passed event should have been emitted
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::AllPlayersPassed)));

    // Should have advanced to the next step
    assert_ne!(state.turn.step, Step::PreCombatMain);
}

#[test]
/// Wrong player passing priority → error
fn test_priority_wrong_player_error() {
    let state = GameStateBuilder::four_player()
        .at_step(Step::PreCombatMain)
        .build();

    // Player 1 has priority, but player 2 tries to pass
    let result = pass(state, PlayerId(2));
    assert!(result.is_err());
    match result.unwrap_err() {
        GameStateError::NotPriorityHolder { expected, actual } => {
            assert_eq!(expected, Some(PlayerId(1)));
            assert_eq!(actual, PlayerId(2));
        }
        e => panic!("expected NotPriorityHolder, got {:?}", e),
    }
}

#[test]
/// Eliminated player is skipped in priority
fn test_priority_eliminated_player_skipped() {
    let state = GameStateBuilder::four_player()
        .at_step(Step::PreCombatMain)
        .build();

    // Concede player 2
    let (state, _) = process_command(
        state,
        Command::Concede {
            player: PlayerId(2),
        },
    )
    .unwrap();

    // Player 1 passes → should skip player 2 → player 3
    // Need to get to a state where player 1 has priority
    // After concede, if player 1 still has priority:
    let holder = state.turn.priority_holder.unwrap();
    assert_eq!(holder, PlayerId(1)); // Player 1 should still have priority

    let (state, _) = pass(state, PlayerId(1)).unwrap();
    // Should skip player 2 (conceded) → player 3
    assert_eq!(state.turn.priority_holder, Some(PlayerId(3)));
}

#[test]
/// CR 502.3 — no priority during Untap step
fn test_priority_no_priority_during_untap() {
    let state = GameStateBuilder::four_player().build();

    // Start game — Untap has no priority, should auto-advance to Upkeep
    let (state, events) = mtg_engine::rules::engine::start_game(state).unwrap();

    // Should have advanced past Untap
    assert_ne!(state.turn.step, Step::Untap);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Upkeep step change should appear in events
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::StepChanged {
            step: Step::Upkeep,
            ..
        }
    )));
}

#[test]
/// CR 514.3 — no priority during Cleanup step (normally)
fn test_priority_no_priority_during_cleanup() {
    let state = GameStateBuilder::four_player().at_step(Step::End).build();

    // Pass through End step — all 4 players pass
    let (state, _) = pass(state, PlayerId(1)).unwrap();
    let (state, _) = pass(state, PlayerId(2)).unwrap();
    let (state, _) = pass(state, PlayerId(3)).unwrap();
    let (state, _) = pass(state, PlayerId(4)).unwrap();

    // Should have auto-advanced past Cleanup to the next turn's Upkeep
    assert_ne!(state.turn.step, Step::Cleanup);
    // Next turn starts
    assert_eq!(state.turn.turn_number, 2);
}
