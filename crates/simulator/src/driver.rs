//! Game driver — runs a complete game with bots making decisions.
//!
//! `GameDriver<P>` is generic over the `LegalActionProvider`, allowing
//! both the stub provider (Phase 1) and full provider (Phase 4).

use std::collections::HashMap;

use mtg_engine::{process_command, start_game, Command, GameState, PlayerId};

use crate::bot::Bot;
use crate::invariants;
use crate::legal_actions::LegalActionProvider;
use crate::mana_solver;
use crate::report::{GameDriverError, GameResult};

/// Drives a complete game, alternating between legal action enumeration
/// and bot decision-making.
pub struct GameDriver<P: LegalActionProvider> {
    pub provider: P,
    pub bots: HashMap<PlayerId, Box<dyn Bot>>,
    pub max_turns: u32,
    pub max_commands: u32,
    pub check_invariants: bool,
}

impl<P: LegalActionProvider> GameDriver<P> {
    pub fn new(
        provider: P,
        bots: HashMap<PlayerId, Box<dyn Bot>>,
        max_turns: u32,
        _seed: u64,
    ) -> Self {
        Self {
            provider,
            bots,
            max_turns,
            max_commands: max_turns * 200, // Safety valve: ~200 commands per turn max
            check_invariants: true,
        }
    }

    /// Run a complete game from initial state to conclusion.
    pub fn run_game(&mut self, initial_state: GameState, seed: u64) -> GameResult {
        let mut violations = Vec::new();
        let mut total_commands: usize = 0;
        let mut command_count: u32 = 0;

        // Start the game
        let (mut state, _start_events) = match start_game(initial_state) {
            Ok(result) => result,
            Err(e) => {
                return GameResult {
                    seed,
                    winner: None,
                    turn_count: 0,
                    total_commands: 0,
                    violations: Vec::new(),
                    error: Some(GameDriverError::EngineError(format!("{:?}", e))),
                };
            }
        };

        let mut prev_turn = state.turn.turn_number;
        let mut pass_count: u32 = 0;
        let max_consecutive_passes = 500; // Safety: break infinite pass loops

        loop {
            // Check game over
            if is_game_over(&state) {
                let winner = find_winner(&state);
                return GameResult {
                    seed,
                    winner,
                    turn_count: state.turn.turn_number,
                    total_commands,
                    violations,
                    error: None,
                };
            }

            // Check turn limit
            if state.turn.turn_number > self.max_turns {
                return GameResult {
                    seed,
                    winner: None,
                    turn_count: state.turn.turn_number,
                    total_commands,
                    violations,
                    error: Some(GameDriverError::MaxTurnsReached(self.max_turns)),
                };
            }

            // Check command limit (infinite loop protection)
            if command_count >= self.max_commands {
                return GameResult {
                    seed,
                    winner: None,
                    turn_count: state.turn.turn_number,
                    total_commands,
                    violations,
                    error: Some(GameDriverError::InfiniteLoop {
                        turn: state.turn.turn_number,
                    }),
                };
            }

            // Consecutive pass limit (stuck game protection)
            if pass_count >= max_consecutive_passes {
                return GameResult {
                    seed,
                    winner: None,
                    turn_count: state.turn.turn_number,
                    total_commands,
                    violations,
                    error: Some(GameDriverError::InfiniteLoop {
                        turn: state.turn.turn_number,
                    }),
                };
            }

            // Determine acting player
            let acting_player =
                if let Some(pending) = state.pending_commander_zone_choices.iter().next() {
                    pending.0
                } else if let Some(priority) = state.turn.priority_holder {
                    priority
                } else {
                    // No one has priority and no pending choices — pass to advance
                    // This can happen between steps; issue PassPriority for active player
                    let active = state.turn.active_player;
                    let cmd = Command::PassPriority { player: active };
                    match process_command(state.clone(), cmd) {
                        Ok((new_state, _events)) => {
                            state = new_state;
                            command_count += 1;
                            total_commands += 1;
                            pass_count += 1;
                            continue;
                        }
                        Err(e) => {
                            return GameResult {
                                seed,
                                winner: None,
                                turn_count: state.turn.turn_number,
                                total_commands,
                                violations,
                                error: Some(GameDriverError::EngineError(format!("{:?}", e))),
                            };
                        }
                    }
                };

            // Get legal actions
            let legal = self.provider.legal_actions(&state, acting_player);

            if legal.is_empty() {
                // No legal actions — pass priority to advance
                let cmd = Command::PassPriority {
                    player: acting_player,
                };
                match process_command(state.clone(), cmd) {
                    Ok((new_state, _events)) => {
                        state = new_state;
                        command_count += 1;
                        total_commands += 1;
                        pass_count += 1;
                        continue;
                    }
                    Err(_) => {
                        return GameResult {
                            seed,
                            winner: None,
                            turn_count: state.turn.turn_number,
                            total_commands,
                            violations,
                            error: Some(GameDriverError::NoLegalActions {
                                player: acting_player,
                                turn: state.turn.turn_number,
                            }),
                        };
                    }
                }
            }

            // Bot chooses an action
            let cmd = if let Some(bot) = self.bots.get_mut(&acting_player) {
                bot.choose_action(&state, acting_player, &legal)
            } else {
                // No bot assigned — pass priority
                Command::PassPriority {
                    player: acting_player,
                }
            };

            // Track passes for loop detection
            if matches!(cmd, Command::PassPriority { .. }) {
                pass_count += 1;
            } else {
                pass_count = 0;
            }

            // If the command is CastSpell, auto-tap mana sources first
            let commands = if let Command::CastSpell { player, card, .. } = &cmd {
                if let Ok(obj) = state.object(*card) {
                    if let Some(ref cost) = obj.characteristics.mana_cost {
                        let mut cmds = mana_solver::solve_mana_payment(&state, *player, cost)
                            .unwrap_or_default();
                        cmds.push(cmd.clone());
                        cmds
                    } else {
                        vec![cmd.clone()]
                    }
                } else {
                    vec![cmd.clone()]
                }
            } else {
                vec![cmd.clone()]
            };

            // Execute all commands in sequence (tap commands + the action)
            for c in commands {
                match process_command(state.clone(), c) {
                    Ok((new_state, _events)) => {
                        if self.check_invariants {
                            let new_violations = invariants::check_all(&new_state, Some(prev_turn));
                            violations.extend(new_violations);
                        }
                        prev_turn = new_state.turn.turn_number;
                        state = new_state;
                        command_count += 1;
                        total_commands += 1;
                    }
                    Err(e) => {
                        // Command rejected — not necessarily fatal. The stub provider
                        // may produce invalid actions. Log and try passing instead.
                        let fallback = Command::PassPriority {
                            player: acting_player,
                        };
                        match process_command(state.clone(), fallback) {
                            Ok((new_state, _)) => {
                                state = new_state;
                                command_count += 1;
                                total_commands += 1;
                                pass_count += 1;
                            }
                            Err(e2) => {
                                return GameResult {
                                    seed,
                                    winner: None,
                                    turn_count: state.turn.turn_number,
                                    total_commands,
                                    violations,
                                    error: Some(GameDriverError::EngineError(format!(
                                        "Both action and fallback failed: {:?}, {:?}",
                                        e, e2
                                    ))),
                                };
                            }
                        }
                        break; // Don't continue the sequence if a command failed
                    }
                }
            }
        }
    }
}

/// Check if the game is over (one or zero players remain).
fn is_game_over(state: &GameState) -> bool {
    let alive = state.active_players();
    alive.len() <= 1
}

/// Find the winner (last player standing), if any.
fn find_winner(state: &GameState) -> Option<PlayerId> {
    let alive = state.active_players();
    if alive.len() == 1 {
        Some(alive[0])
    } else {
        None
    }
}
