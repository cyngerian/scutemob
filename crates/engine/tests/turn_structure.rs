//! Turn structure tests (CR 500-514).

use mtg_engine::rules::engine::{process_command, start_game};
use mtg_engine::{
    Command, GameEvent, GameState, GameStateBuilder, ObjectSpec, Phase, PlayerId, Step, ZoneId,
};

fn pass(state: GameState, player: PlayerId) -> (GameState, Vec<GameEvent>) {
    process_command(state, Command::PassPriority { player }).unwrap()
}

/// Build a 4-player state with enough library cards that nobody decks out.
fn four_player_with_libraries(cards_per_player: usize) -> GameState {
    let mut builder = GameStateBuilder::four_player();
    for pid in 1..=4 {
        let player = PlayerId(pid);
        for i in 0..cards_per_player {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Card {} P{}", i, pid))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder.build().unwrap()
}

/// Pass priority through all active players until the step advances.
/// Returns the new state and all accumulated events.
fn pass_until_advance(mut state: GameState) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    loop {
        let holder = state.turn.priority_holder.expect("no priority holder");
        let (new_state, events) = pass(state, holder);
        let advanced = events.iter().any(|e| {
            matches!(
                e,
                GameEvent::StepChanged { .. } | GameEvent::TurnStarted { .. }
            )
        });
        all_events.extend(events);
        state = new_state;
        if advanced {
            return (state, all_events);
        }
    }
}

#[test]
/// CR 500-514 — verify the full step order within a single turn
fn test_turn_full_step_order() {
    let state = four_player_with_libraries(5);
    let (mut state, _) = start_game(state).unwrap();

    // After start_game, we should be at Upkeep (Untap auto-advances since no priority)
    assert_eq!(state.turn.step, Step::Upkeep);
    assert_eq!(state.turn.phase, Phase::Beginning);

    // CR 508.8: With no attackers declared, DeclareBlockers and CombatDamage
    // are skipped — DeclareAttackers advances directly to EndOfCombat.
    let expected_steps = vec![
        Step::Draw,
        Step::PreCombatMain,
        Step::BeginningOfCombat,
        Step::DeclareAttackers,
        Step::EndOfCombat, // skips DeclareBlockers + CombatDamage (no attackers)
        Step::PostCombatMain,
        Step::End,
        // Cleanup auto-advances, then new turn starts at Upkeep
    ];

    for expected in &expected_steps {
        let (new_state, _) = pass_until_advance(state);
        state = new_state;
        assert_eq!(
            state.turn.step, *expected,
            "expected step {:?}, got {:?}",
            expected, state.turn.step
        );
    }
}

#[test]
/// CR 500 — verify phase derivation from steps
fn test_turn_phase_transitions() {
    let state = four_player_with_libraries(5);
    let (mut state, _) = start_game(state).unwrap();

    // Track phases as we advance through a full turn
    let mut phases_seen = Vec::new();

    loop {
        phases_seen.push((state.turn.step, state.turn.phase));
        let old_turn = state.turn.turn_number;
        let (new_state, _) = pass_until_advance(state);
        state = new_state;
        if state.turn.turn_number > old_turn {
            // New turn started
            break;
        }
    }

    // Verify each step maps to the right phase
    for (step, phase) in &phases_seen {
        assert_eq!(
            step.phase(),
            *phase,
            "step {:?} should be in phase {:?}",
            step,
            phase
        );
    }
}

#[test]
/// CR 500 — 4 players take turns in order
fn test_four_player_turn_rotation() {
    let state = four_player_with_libraries(5);
    let (mut state, _) = start_game(state).unwrap();

    let expected_active = vec![
        PlayerId(1),
        PlayerId(2),
        PlayerId(3),
        PlayerId(4),
        PlayerId(1), // wraps around
    ];

    for (i, expected) in expected_active.iter().enumerate() {
        assert_eq!(
            state.turn.active_player,
            *expected,
            "turn {} should have active player {:?}",
            i + 1,
            expected
        );

        // Pass through the entire turn
        loop {
            let old_turn = state.turn.turn_number;
            let holder = match state.turn.priority_holder {
                Some(h) => h,
                None => break, // game ended
            };
            let (new_state, _) = pass(state, holder);
            state = new_state;
            if state.turn.turn_number > old_turn {
                break;
            }
        }
    }
}

#[test]
/// Acceptance test: 10 full turn cycles (40 turns) with 4 players
fn test_ten_full_turn_cycles() {
    let state = four_player_with_libraries(15);
    let (mut state, _) = start_game(state).unwrap();

    let target_turns = 40; // 10 cycles × 4 players

    while state.turn.turn_number <= target_turns {
        let holder = match state.turn.priority_holder {
            Some(h) => h,
            None => panic!("no priority holder at turn {}", state.turn.turn_number),
        };
        let (new_state, _) = pass(state, holder);
        state = new_state;
        if state.turn.turn_number > target_turns {
            break;
        }
    }

    // We should have completed at least 40 turns
    assert!(
        state.turn.turn_number > target_turns,
        "expected to reach turn {}, got {}",
        target_turns + 1,
        state.turn.turn_number
    );
}

#[test]
/// Turn number increments each turn
fn test_turn_number_increments() {
    let state = four_player_with_libraries(5);
    let (mut state, _) = start_game(state).unwrap();

    assert_eq!(state.turn.turn_number, 1);

    // Complete first turn
    loop {
        let holder = state.turn.priority_holder.unwrap();
        let old_turn = state.turn.turn_number;
        let (new_state, _) = pass(state, holder);
        state = new_state;
        if state.turn.turn_number > old_turn {
            break;
        }
    }

    assert_eq!(state.turn.turn_number, 2);
}

#[test]
/// Turn order wraps around from last to first player
fn test_turn_order_wraparound() {
    let mut builder = GameStateBuilder::four_player()
        .active_player(PlayerId(4))
        .at_step(Step::PreCombatMain);
    for pid in 1..=4 {
        let player = PlayerId(pid);
        for i in 0..10 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Card {} P{}", i, pid))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    let state = builder.build().unwrap();

    assert_eq!(state.turn.active_player, PlayerId(4));

    // Complete player 4's turn
    let mut state = state;
    loop {
        let holder = state.turn.priority_holder.unwrap();
        let old_turn = state.turn.turn_number;
        let (new_state, _) = pass(state, holder);
        state = new_state;
        if state.turn.turn_number > old_turn {
            break;
        }
    }

    // Should wrap back to player 1
    assert_eq!(state.turn.active_player, PlayerId(1));
}
