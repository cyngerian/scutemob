//! Engine integration: command processing and game loop (CR 500-514).
//!
//! `process_command` is the single public entry point. It takes an immutable
//! GameState and a Command, produces a new GameState and a list of events.
//! State module = data, rules module = behavior.

use crate::state::error::GameStateError;
use crate::state::player::PlayerId;
use crate::state::GameState;

use super::abilities;
use super::casting;
use super::combat;
use super::command::Command;
use super::commander;
use super::events::GameEvent;
use super::lands;
use super::mana;
use super::priority::{self, PriorityResult};
use super::replacement;
use super::resolution;
use super::sba;
use super::turn_actions;
use super::turn_structure;

/// Process a player command against the current game state.
///
/// Returns the new game state and a list of events describing what happened.
/// The old state is not modified (immutable state model).
pub fn process_command(
    state: GameState,
    command: Command,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    let mut all_events = Vec::new();

    // Validate: game not over
    if is_game_over(&state) {
        return Err(GameStateError::GameAlreadyOver);
    }

    match command {
        Command::PassPriority { player } => {
            validate_player_active(&state, player)?;
            let events = handle_pass_priority(&mut state, player)?;
            all_events.extend(events);
        }
        Command::Concede { player } => {
            validate_player_exists(&state, player)?;
            let events = handle_concede(&mut state, player)?;
            all_events.extend(events);
        }
        Command::TapForMana {
            player,
            source,
            ability_index,
        } => {
            validate_player_active(&state, player)?;
            let events = mana::handle_tap_for_mana(&mut state, player, source, ability_index)?;
            all_events.extend(events);
        }
        Command::PlayLand { player, card } => {
            validate_player_active(&state, player)?;
            let events = lands::handle_play_land(&mut state, player, card)?;
            all_events.extend(events);
        }
        Command::CastSpell {
            player,
            card,
            targets,
        } => {
            validate_player_active(&state, player)?;
            let mut events = casting::handle_cast_spell(&mut state, player, card, targets)?;
            // CR 603.3: Check for triggered abilities arising from casting this spell
            // (e.g., "Whenever an opponent casts a spell" — Rhystic Study).
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
            all_events.extend(events);
        }
        Command::ActivateAbility {
            player,
            source,
            ability_index,
            targets,
        } => {
            validate_player_active(&state, player)?;
            let events = abilities::handle_activate_ability(
                &mut state,
                player,
                source,
                ability_index,
                targets,
            )?;
            all_events.extend(events);
        }
        Command::DeclareAttackers { player, attackers } => {
            validate_player_active(&state, player)?;
            let events = combat::handle_declare_attackers(&mut state, player, attackers)?;
            all_events.extend(events);
        }
        Command::DeclareBlockers { player, blockers } => {
            validate_player_active(&state, player)?;
            let events = combat::handle_declare_blockers(&mut state, player, blockers)?;
            all_events.extend(events);
        }
        Command::OrderBlockers {
            player,
            attacker,
            order,
        } => {
            validate_player_active(&state, player)?;
            let events = combat::handle_order_blockers(&mut state, player, attacker, order)?;
            all_events.extend(events);
        }
        Command::OrderReplacements { player, ids } => {
            validate_player_active(&state, player)?;
            let events = replacement::handle_order_replacements(&mut state, player, ids)?;
            all_events.extend(events);
        }
        Command::ReturnCommanderToCommandZone { player, object_id } => {
            // CR 903.9a / CR 704.6d: owner chooses to return their commander
            // from graveyard or exile to the command zone. Clears the pending
            // commander zone-return choice recorded by the SBA.
            validate_player_exists(&state, player)?;
            let events =
                commander::handle_return_commander_to_command_zone(&mut state, player, object_id)?;
            all_events.extend(events);
        }

        Command::LeaveCommanderInZone { player, object_id } => {
            // CR 903.9a: owner chooses to leave their commander in graveyard or
            // exile rather than returning it to the command zone.
            validate_player_exists(&state, player)?;
            let events = commander::handle_leave_commander_in_zone(&mut state, player, object_id)?;
            all_events.extend(events);
        }

        // ── M9: Mulligan commands (CR 103.5 / CR 103.5c) ─────────────────
        Command::TakeMulligan { player } => {
            validate_player_exists(&state, player)?;
            let events = commander::handle_take_mulligan(&mut state, player)?;
            all_events.extend(events);
        }
        Command::KeepHand {
            player,
            cards_to_bottom,
        } => {
            validate_player_exists(&state, player)?;
            let events = commander::handle_keep_hand(&mut state, player, cards_to_bottom)?;
            all_events.extend(events);
        }

        // ── M9: Companion command (CR 702.139a) ───────────────────────────
        Command::BringCompanion { player } => {
            validate_player_active(&state, player)?;
            let events = commander::handle_bring_companion(&mut state, player)?;
            all_events.extend(events);
        }
    }

    // Record events in history
    for event in &all_events {
        state.history.push_back(event.clone());
    }

    Ok((state, all_events))
}

/// Handle a PassPriority command.
fn handle_pass_priority(
    state: &mut GameState,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let (result, mut events) = priority::pass_priority(state, player)?;

    match result {
        PriorityResult::PlayerHasPriority { player: next } => {
            state.turn.players_passed.insert(player);
            state.turn.priority_holder = Some(next);
        }
        PriorityResult::AllPassed => {
            // All players passed with empty stack — advance the game
            state.turn.players_passed.insert(player);
            state.turn.priority_holder = None;
            let advance_events = handle_all_passed(state)?;
            events.extend(advance_events);
        }
    }

    Ok(events)
}

/// Handle when all players have passed priority in succession.
///
/// CR 608.1: If the stack is non-empty, resolve the top of the stack.
/// CR 500.4: If the stack is empty, empty mana pools and advance step or turn.
fn handle_all_passed(state: &mut GameState) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    if !state.stack_objects.is_empty() {
        // CR 608.1: Stack is non-empty — resolve the top object.
        let resolve_events = resolution::resolve_top_of_stack(state)?;
        events.extend(resolve_events);
    } else {
        // Stack is empty — advance step or turn.

        // Empty mana pools at step transition (CR 500.4)
        let mana_events = turn_actions::empty_all_mana_pools(state);
        events.extend(mana_events);

        // CR 514.3a: When all pass with empty stack in Cleanup, do NOT advance
        // to the next step — run another cleanup round instead.  `enter_step`
        // will execute cleanup actions, check SBAs, and either grant priority
        // again (if SBAs fired) or auto-advance to the next turn (if none).
        if state.turn.step != crate::state::turn::Step::Cleanup {
            // Advance to next step or next turn
            if let Some((new_turn, step_events)) = turn_structure::advance_step(state) {
                state.turn = new_turn;
                events.extend(step_events);
            } else {
                // Past cleanup — advance to next turn
                let (new_turn, turn_events) = turn_structure::advance_turn(state)?;
                state.turn = new_turn;
                events.extend(turn_events);
                // Reset per-turn state for new active player
                turn_actions::reset_turn_state(state, state.turn.active_player);
            }
        }

        // Enter the new step (execute turn-based actions, grant priority or auto-advance)
        let enter_events = enter_step(state)?;
        events.extend(enter_events);
    }

    Ok(events)
}

/// Enter a step: execute turn-based actions, then either grant priority or
/// auto-advance if the step has no priority (Untap, Cleanup).
///
/// Uses a loop (not recursion) to handle steps that auto-advance.
fn enter_step(state: &mut GameState) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    loop {
        // Execute turn-based actions for this step
        let action_events = turn_actions::execute_turn_based_actions(state)?;
        events.extend(action_events);

        // Check if game ended due to turn-based actions (e.g., draw from empty library)
        if is_game_over(state) {
            let game_over_events = check_game_over(state);
            events.extend(game_over_events);
            return Ok(events);
        }

        // CR 514.3a: After cleanup turn-based actions, check SBAs and triggers.
        // If any events are produced, grant priority to the active player.
        // The active player (and others) then pass; `handle_all_passed` will
        // call `enter_step` again for another cleanup round instead of advancing.
        // A safety counter (max 100) guards against pathological infinite loops.
        if state.turn.step == crate::state::turn::Step::Cleanup {
            const MAX_CLEANUP_SBA_ROUNDS: u32 = 100;
            let sba_events = sba::check_and_apply_sbas(state);
            let sba_triggers = abilities::check_triggers(state, &sba_events);
            for t in sba_triggers {
                state.pending_triggers.push_back(t);
            }
            events.extend(sba_events.clone());

            let trigger_events = abilities::flush_pending_triggers(state);
            events.extend(trigger_events.clone());

            let had_events = !sba_events.is_empty() || !trigger_events.is_empty();
            if had_events && state.turn.cleanup_sba_rounds < MAX_CLEANUP_SBA_ROUNDS {
                state.turn.cleanup_sba_rounds += 1;
                // Grant priority — when all pass, handle_all_passed will re-enter cleanup.
                let active = state.turn.active_player;
                let (passed, priority_events) = priority::grant_initial_priority(state);
                state.turn.players_passed = passed;
                state.turn.priority_holder = Some(active);
                events.extend(priority_events);
                return Ok(events);
            }
            // No SBAs (or safety limit reached) — fall through to auto-advance.
        }

        if state.turn.step.has_priority() {
            // CR 704.3: Check and apply all SBAs before granting priority.
            let sba_events = sba::check_and_apply_sbas(state);
            // Any SBA events may have triggered abilities — queue them.
            let sba_triggers = abilities::check_triggers(state, &sba_events);
            for t in sba_triggers {
                state.pending_triggers.push_back(t);
            }
            events.extend(sba_events);

            // If all players lost due to SBAs, end the game.
            if is_game_over(state) {
                events.extend(check_game_over(state));
                return Ok(events);
            }

            // Flush any pending triggers before granting priority (CR 603.3).
            let trigger_events = abilities::flush_pending_triggers(state);
            events.extend(trigger_events);

            // Grant priority to active player (if still alive)
            let active = state.turn.active_player;
            let is_alive = state
                .players
                .get(&active)
                .map(|p| !p.has_lost && !p.has_conceded)
                .unwrap_or(false);

            if is_alive {
                let (passed, priority_events) = priority::grant_initial_priority(state);
                state.turn.players_passed = passed;
                state.turn.priority_holder = Some(active);
                events.extend(priority_events);
            } else {
                // Active player lost (e.g., drew from empty library).
                // Find next player in APNAP order.
                if let Some(next) = priority::next_priority_player(state, active) {
                    state.turn.players_passed = im::OrdSet::new();
                    state.turn.priority_holder = Some(next);
                    events.push(GameEvent::PriorityGiven { player: next });
                } else {
                    state.turn.priority_holder = None;
                }
            }
            return Ok(events);
        }

        // No priority in this step — auto-advance
        // Empty mana pools at step transition
        let mana_events = turn_actions::empty_all_mana_pools(state);
        events.extend(mana_events);

        if let Some((new_turn, step_events)) = turn_structure::advance_step(state) {
            state.turn = new_turn;
            events.extend(step_events);
            // Loop to enter the next step
        } else {
            // Past cleanup — advance to next turn
            let (new_turn, turn_events) = turn_structure::advance_turn(state)?;
            state.turn = new_turn;
            events.extend(turn_events);
            turn_actions::reset_turn_state(state, state.turn.active_player);
            // Loop to enter the first step of the new turn
        }
    }
}

/// Handle a Concede command.
fn handle_concede(
    state: &mut GameState,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // Mark player as conceded
    if let Some(p) = state.players.get_mut(&player) {
        if p.has_lost || p.has_conceded {
            return Err(GameStateError::PlayerEliminated(player));
        }
        p.has_conceded = true;
    } else {
        return Err(GameStateError::PlayerNotFound(player));
    }

    events.push(GameEvent::PlayerConceded { player });

    // Check game over
    let game_over_events = check_game_over(state);
    events.extend(game_over_events);

    if !is_game_over(state) {
        // If the conceding player held priority, advance priority
        if state.turn.priority_holder == Some(player) {
            let next = priority::next_priority_player(state, player);
            match next {
                Some(next_player) => {
                    state.turn.priority_holder = Some(next_player);
                    events.push(GameEvent::PriorityGiven {
                        player: next_player,
                    });
                }
                None => {
                    // All remaining have passed. MR-M2-03: if the conceding
                    // player is also the active player, do NOT call
                    // handle_all_passed (which would advance the step); the
                    // turn-advance block below handles that path.
                    state.turn.priority_holder = None;
                    if state.turn.active_player != player {
                        let advance_events = handle_all_passed(state)?;
                        events.extend(advance_events);
                    }
                }
            }
        }

        // If it was the conceding player's turn, advance to next turn
        if state.turn.active_player == player {
            // MR-M2-15: Clear stale combat state so the next player doesn't
            // inherit an in-progress combat from the conceded turn.
            state.combat = None;

            let mana_events = turn_actions::empty_all_mana_pools(state);
            events.extend(mana_events);

            let (new_turn, turn_events) = turn_structure::advance_turn(state)?;
            state.turn = new_turn;
            events.extend(turn_events);
            turn_actions::reset_turn_state(state, state.turn.active_player);

            let enter_events = enter_step(state)?;
            events.extend(enter_events);
        }
    }

    Ok(events)
}

/// Check if the game is over (one or fewer active players).
/// Returns GameOver event if applicable.
fn check_game_over(state: &GameState) -> Vec<GameEvent> {
    let active = state.active_players();
    match active.len() {
        0 => vec![GameEvent::GameOver { winner: None }],
        1 => vec![GameEvent::GameOver {
            winner: Some(active[0]),
        }],
        _ => Vec::new(),
    }
}

/// Returns true if the game is over.
fn is_game_over(state: &GameState) -> bool {
    let active = state.active_players();
    active.len() <= 1
}

fn validate_player_active(state: &GameState, player: PlayerId) -> Result<(), GameStateError> {
    let p = state.player(player)?;
    if p.has_lost || p.has_conceded {
        return Err(GameStateError::PlayerEliminated(player));
    }
    Ok(())
}

fn validate_player_exists(state: &GameState, player: PlayerId) -> Result<(), GameStateError> {
    state.player(player)?;
    Ok(())
}

/// Start the game: set up the first turn and enter the first step.
/// Call this after building the initial state to begin gameplay.
pub fn start_game(state: GameState) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    let mut events = Vec::new();

    let active = state.turn.active_player;
    turn_actions::reset_turn_state(&mut state, active);

    // Set to the beginning of the turn
    state.turn.step = crate::state::turn::Step::Untap;
    state.turn.phase = crate::state::turn::Phase::Beginning;
    state.turn.is_first_turn_of_game = true;

    events.push(GameEvent::TurnStarted {
        player: active,
        turn_number: state.turn.turn_number,
    });
    events.push(GameEvent::StepChanged {
        step: crate::state::turn::Step::Untap,
        phase: crate::state::turn::Phase::Beginning,
    });

    // Enter the first step
    let enter_events = enter_step(&mut state)?;
    events.extend(enter_events);

    // Record events in history
    for event in &events {
        state.history.push_back(event.clone());
    }

    Ok((state, events))
}
