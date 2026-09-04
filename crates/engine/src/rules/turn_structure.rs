//! Turn structure FSM: step ordering, turn advancement (CR 500-514).
use super::combat;
use super::events::GameEvent;
use crate::state::player::PlayerId;
use crate::state::turn::{Phase, Step, TurnState};
use crate::state::GameState;
use imbl::{OrdSet, Vector};
/// All steps in a normal turn, in order.
/// FirstStrikeDamage is excluded -- M6 will conditionally insert it.
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
///
/// CR 500.8: When leaving `EndOfCombat`, checks `additional_phases`. If the next
/// queued entry is `Phase::Combat`, redirects to `BeginningOfCombat` (extra combat).
/// If it is `Phase::PostCombatMain`, redirects to `PostCombatMain` (extra main).
/// LIFO ordering: most recently created phase occurs first.
///
/// CR 500.8: When leaving `PostCombatMain`, checks `additional_phases`. If the next
/// queued entry is `Phase::Combat`, redirects to `BeginningOfCombat` (for effects
/// that say "after this main phase, there is an additional combat phase").
pub fn advance_step(state: &GameState) -> Option<(TurnState, Vec<GameEvent>)> {
    // CR 508.8, verbatim: "If no creatures are declared as attackers or put onto the
    // battlefield attacking, skip the declare blockers and combat damage steps."
    //
    // PB-DX51 (`OOS-DX21-4`): that predicate is a HISTORICAL fact about two events --
    // a CR 508.1 declaration and a CR 508.4 entry -- and NOT a question about what is
    // in combat at step end. Until PB-DX51 this read `c.attackers.is_empty()` alone,
    // so an instant-speed answer to a lone attacker (CR 506.4: leaves the battlefield,
    // phases out, changes controller, stops being a creature) emptied the map while
    // the step was still open and the engine skipped declare-blockers AND combat-damage
    // in a combat where creatures WERE declared -- taking every other creature's block,
    // every later CR 508.4 entrant, and the whole of CR 510 with it.
    //
    // `had_attackers` is the monotone marker `CombatState::add_attacker` sets on both
    // routes. `!attackers_declared` would NOT do: CR 508.1a's "if any" makes an EMPTY
    // declaration a completed one, and CR 508.8 still demands the skip for it.
    //
    // The `attackers.is_empty()` conjunct is retained deliberately and is CR-grounded
    // in its own right: a creature sitting in `attackers` right now IS attacking, which
    // CR 508.8 says is a reason not to skip. It can only ever PREVENT a skip, never
    // cause one, so it cannot make this less CR-correct -- it is an observational
    // fallback for the CR 508.4 half should a future entry site bypass the mutator, and
    // it is what keeps every hand-built test fixture that writes `attackers` directly
    // behaving exactly as before. It does NOT rescue the removed-attacker case, which
    // is why `had_attackers` is load-bearing (revert row R2).
    //
    // `unwrap_or(true)`: no `CombatState` at all means nothing was ever declared.
    let no_attackers = state
        .combat
        .as_ref()
        .map(|c| !c.had_attackers && c.attackers.is_empty())
        .unwrap_or(true);
    let mut turn = state.turn.clone();
    let mut events = Vec::new();
    let next = if turn.step == Step::DeclareAttackers && no_attackers {
        Step::EndOfCombat
    } else if turn.step == Step::DeclareBlockers && combat::should_have_first_strike_step(state) {
        // CR 510.4: Conditionally insert FirstStrikeDamage.
        Step::FirstStrikeDamage
    } else if turn.step == Step::EndOfCombat {
        // CR 500.8: If additional phases are queued, consume the next one (LIFO pop_back).
        if let Some(phase) = turn.additional_phases.pop_back() {
            match phase {
                Phase::Combat => {
                    // Enter a new extra combat phase.
                    turn.in_extra_combat = true;
                    Step::BeginningOfCombat
                }
                Phase::PostCombatMain => {
                    // Insert an extra main phase (from followed_by_main effects).
                    // CR 500.8: Clear in_extra_combat -- main phases are not combat phases.
                    turn.in_extra_combat = false;
                    Step::PostCombatMain
                }
                _ => {
                    // Other phases are not insertable this way; skip and fall through.
                    turn.in_extra_combat = false;
                    Step::PostCombatMain
                }
            }
        } else {
            // No more extra phases: restore normal flow.
            turn.in_extra_combat = false;
            Step::PostCombatMain
        }
    } else if turn.step == Step::PostCombatMain {
        // CR 500.8: After a postcombat main phase, check for queued Combat phases.
        if let Some(phase) = turn.additional_phases.pop_back() {
            match phase {
                Phase::Combat => {
                    turn.in_extra_combat = true;
                    Step::BeginningOfCombat
                }
                _ => {
                    // Not a combat phase -- push it back and advance normally.
                    turn.additional_phases.push_back(phase);
                    turn.step.next()?
                }
            }
        } else {
            turn.step.next()?
        }
    } else {
        turn.step.next()?
    };
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
    // Determine who takes the next turn -- MR-M2-02: typed error instead of expect.
    let next_player = if let Some(extra_turn_player) = turn.extra_turns.pop_back() {
        // LIFO: most recently added extra turn goes first.
        // Don't update last_regular_active -- extra turns don't advance normal order.
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
    turn.additional_phases = Vector::new();
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
        if let Some(player) = state.expect_player(candidate) {
            if !player.has_lost && !player.has_conceded {
                return Some(candidate);
            }
        }
    }
    None
}
