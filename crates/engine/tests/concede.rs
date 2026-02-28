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
    builder.build().unwrap()
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
        .build()
        .unwrap();

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

#[test]
/// MR-M2-03: Concede while active, with all non-active players already having
/// passed priority, must NOT double-advance (no step advance before turn advance).
///
/// Without the fix, handle_all_passed fires (advancing the step into combat)
/// AND THEN advance_turn fires — leaving the game in the middle of a wrong step.
fn test_concede_active_player_with_all_others_passed_no_double_advance() {
    let state = four_player_at(Step::PreCombatMain);

    // Manually set players_passed so P2/P3/P4 have already passed but P1 hasn't.
    // (Simulates: P1 took an action, then P2/P3/P4 passed back to P1.)
    let mut state = state;
    state.turn.players_passed.insert(PlayerId(2));
    state.turn.players_passed.insert(PlayerId(3));
    state.turn.players_passed.insert(PlayerId(4));
    state.turn.priority_holder = Some(PlayerId(1));

    // P1 (active) concedes — next_priority_player(P1) returns None (all others passed).
    let (state, events) = concede(state, PlayerId(1));

    // P2 should now be the active player (next in turn order after P1).
    assert_eq!(
        state.turn.active_player,
        PlayerId(2),
        "P2 should be active after P1 concedes"
    );

    // There must NOT be a StepChanged event to BeginningOfCombat (or later combat
    // steps) — that would indicate handle_all_passed fired before advance_turn.
    let bad_step_change = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::StepChanged {
                step: Step::BeginningOfCombat
                    | Step::DeclareAttackers
                    | Step::DeclareBlockers
                    | Step::CombatDamage,
                ..
            }
        )
    });
    assert!(
        !bad_step_change,
        "should not see combat steps from P1's interrupted turn"
    );

    // P2's new turn should be announced.
    let turn_started = events
        .iter()
        .any(|e| matches!(e, GameEvent::TurnStarted { player, .. } if *player == PlayerId(2)));
    assert!(turn_started, "P2's turn should have started");
}

#[test]
/// MR-M2-15: Conceding while active with in-progress combat must clear
/// state.combat so the next player doesn't inherit a stale combat state.
fn test_concede_active_player_during_combat_clears_combat_state() {
    // Set up P1 in BeginningOfCombat (combat state is initialised).
    let state = four_player_at(Step::BeginningOfCombat);

    // Initialize a combat state (simulating that begin_combat fired).
    let mut state = state;
    state.combat = Some(mtg_engine::state::CombatState::new(PlayerId(1)));

    // P1 concedes while active.
    let (state, _events) = concede(state, PlayerId(1));

    // Combat state must be cleared.
    assert!(
        state.combat.is_none(),
        "combat state should be cleared when active player concedes"
    );
}

#[test]
/// MR-M2-08 / CR 104.3a — active player concedes while all non-active players
/// have already passed priority. Tests that the engine correctly handles the
/// priority-resolution path (engine.rs) when the priority holder concedes and
/// all remaining players had already passed (making the turn advance clean).
///
/// Uses a non-first active player (P3) to exercise the general-case turn order.
fn test_concede_active_player_all_others_passed_turn_advances_cleanly() {
    // Build a state where P3 is the active player.
    let state = four_player_at(Step::PreCombatMain);

    // Manually override active player to P3 and set players_passed to reflect
    // that P1, P2, P4 have all passed (P3 now holds priority about to cycle back).
    // Also set last_regular_active so advance_turn() knows to advance from P3 → P4.
    let mut state = state;
    state.turn.active_player = PlayerId(3);
    state.turn.last_regular_active = PlayerId(3);
    state.turn.priority_holder = Some(PlayerId(3));
    state.turn.players_passed.insert(PlayerId(1));
    state.turn.players_passed.insert(PlayerId(2));
    state.turn.players_passed.insert(PlayerId(4));

    // P3 (active, all others passed) concedes.
    let (state, events) = concede(state, PlayerId(3));

    // P3 should be eliminated.
    assert!(
        state.players[&PlayerId(3)].has_conceded,
        "P3 should be eliminated after conceding"
    );

    // P4 should be the next active player (next in turn order after P3).
    assert_eq!(
        state.turn.active_player,
        PlayerId(4),
        "P4 should be the next active player after P3 concedes"
    );

    // The game should still be going (3 players remain).
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::GameOver { .. })));
    assert_eq!(
        state
            .players
            .values()
            .filter(|p| !p.has_conceded && !p.has_lost)
            .count(),
        3,
        "3 players should remain after P3 concedes"
    );
}

#[test]
/// MR-M2-17 / CR 104.3a — a non-active player concedes during an active combat
/// phase. Tests that the combat state is correctly cleaned up (their pending
/// blocks are removed, game continues cleanly for surviving players).
fn test_concede_non_active_player_during_combat() {
    // P1 is active and has initiated combat (BeginningOfCombat step).
    // P2 is a non-active player who decides to concede.
    let mut state = four_player_at(Step::BeginningOfCombat);

    // Set up a minimal combat state for P1.
    state.combat = Some(mtg_engine::state::CombatState::new(PlayerId(1)));

    // P2 concedes during combat.
    let (state, events) = concede(state, PlayerId(2));

    // P2 should be eliminated.
    assert!(
        state.players[&PlayerId(2)].has_conceded,
        "P2 should be eliminated after conceding"
    );

    // Combat state remains (P1 is still the active player in combat).
    assert!(
        state.combat.is_some(),
        "combat state should NOT be cleared when a non-active player concedes"
    );

    // Game still ongoing — P1/P3/P4 remain.
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::GameOver { .. })));

    // P1 still has priority (still in combat).
    assert_eq!(
        state.turn.active_player,
        PlayerId(1),
        "P1 should still be the active player after non-active P2 concedes"
    );
}
