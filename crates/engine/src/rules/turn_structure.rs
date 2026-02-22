//! Turn structure FSM: step ordering, turn advancement (CR 500-514).

use im::OrdSet;

use crate::state::player::PlayerId;
use crate::state::turn::{Phase, Step, TurnState};
use crate::state::GameState;

use super::combat;
use super::events::GameEvent;

/// All steps in a normal turn, in order.
/// FirstStrikeDamage is excluded — M6 will conditionally insert it.
pub const STEP_ORDER: &[Step] = &[
    Step::Untap,
    Step::Upkeep,
    Step::Draw,
    Step::PreCombatMain,
    Step::BeginningOfCombat,
    Step::DeclareAttackers,
    Step::DeclareBlockers,
    Step::CombatDamage,
    Step::EndOfCombat,
    Step::PostCombatMain,
    Step::End,
    Step::Cleanup,
];

/// Advance to the next step within the current turn.
/// Returns the updated TurnState and any events generated.
/// Returns None if the turn is over (past Cleanup).
///
/// When leaving `DeclareBlockers`, checks whether any combatant has FirstStrike
/// or DoubleStrike (CR 510.4); if so, inserts `Step::FirstStrikeDamage` before
/// the normal `Step::CombatDamage`.
pub fn advance_step(state: &GameState) -> Option<(TurnState, Vec<GameEvent>)> {
    // CR 510.4: Conditionally insert FirstStrikeDamage between DeclareBlockers and CombatDamage.
    let next = if state.turn.step == Step::DeclareBlockers
        && combat::should_have_first_strike_step(state)
    {
        Step::FirstStrikeDamage
    } else {
        state.turn.step.next()?
    };

    let mut turn = state.turn.clone();
    let mut events = Vec::new();

    turn.step = next;
    turn.phase = next.phase();
    turn.priority_holder = None;
    turn.players_passed = OrdSet::new();

    events.push(GameEvent::StepChanged {
        step: next,
        phase: next.phase(),
    });

    Some((turn, events))
}

/// Advance to the next player's turn. Handles extra turns (LIFO) and
/// skips eliminated players.
///
/// Returns the updated TurnState and events. Resets per-turn state.
///
/// # Errors
/// Returns `GameStateError::NoActivePlayers` if no active player can be found
/// for the next turn (all players eliminated or conceded).
pub fn advance_turn(
    state: &GameState,
) -> Result<(TurnState, Vec<GameEvent>), crate::state::error::GameStateError> {
    let mut turn = state.turn.clone();
    let mut events = Vec::new();

    // Determine who takes the next turn — MR-M2-02: typed error instead of expect.
    let next_player = if let Some(extra_turn_player) = turn.extra_turns.pop_back() {
        // LIFO: most recently added extra turn goes first.
        // Don't update last_regular_active — extra turns don't advance normal order.
        extra_turn_player
    } else {
        // Normal turn order: resume from last regular active player.
        let next = next_player_in_turn_order(state, turn.last_regular_active)
            .ok_or(crate::state::error::GameStateError::NoActivePlayers)?;
        turn.last_regular_active = next;
        next
    };

    turn.turn_number += 1;
    turn.active_player = next_player;
    turn.step = Step::Untap;
    turn.phase = Phase::Beginning;
    turn.priority_holder = None;
    turn.players_passed = OrdSet::new();
    turn.extra_combats = 0;
    turn.in_extra_combat = false;
    turn.cleanup_sba_rounds = 0;
    // After the first turn of the game, this flag stays false.
    if turn.is_first_turn_of_game {
        turn.is_first_turn_of_game = false;
    }

    events.push(GameEvent::TurnStarted {
        player: next_player,
        turn_number: turn.turn_number,
    });
    events.push(GameEvent::StepChanged {
        step: Step::Untap,
        phase: Phase::Beginning,
    });

    Ok((turn, events))
}

/// Find the next active (non-eliminated) player in turn order after `current`.
/// Returns None if no active players remain.
pub fn next_player_in_turn_order(state: &GameState, current: PlayerId) -> Option<PlayerId> {
    let order = &state.turn.turn_order;
    let len = order.len();
    if len == 0 {
        return None;
    }

    // Find current player's position in turn order
    let current_pos = order.iter().position(|&p| p == current)?;

    // Search through all other positions
    for offset in 1..=len {
        let idx = (current_pos + offset) % len;
        let candidate = order[idx];
        if let Some(player) = state.players.get(&candidate) {
            if !player.has_lost && !player.has_conceded {
                return Some(candidate);
            }
        }
    }

    None
}
