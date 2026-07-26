//! Steppable local-game core (M11-local Session 1).
//!
//! `LocalGame` extracts the loop body of the old `GameDriver::run_game` (driver.rs)
//! so a caller can advance a game until a designated human seat must act, receive a
//! `PendingDecision` describing the legal actions available, and later hand back a
//! chosen `Command` via `submit`. With `human_seats` empty this behaves identically to
//! the pre-existing bot-only driver — `GameDriver::run_game` is re-expressed on top of
//! it in this same file.
//!
//! CR 117.3 governs priority; CR 103.5 mulligans; CR 903.9a the commander zone-change
//! choice; CR 508.1 / 509.1 combat declarations. See `memory/m11-session-plan.md` §3-4.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    process_command, start_game, Command, GameEvent, GameState, GameStateError, PlayerId,
};

use crate::bot::Bot;
use crate::invariants::{self, InvariantViolation};
use crate::legal_actions::{LegalAction, LegalActionProvider};
use crate::mana_solver;
use crate::report::{GameDriverError, GameResult};

/// Turn/command/pass safety valves for a `LocalGame`. Mirrors the constants
/// `GameDriver` used to hardcode (driver.rs) as configurable fields so a human game
/// (which legitimately passes a lot) is not forced to share the fuzzer's defaults.
#[derive(Clone, Copy, Debug)]
pub struct LocalGameLimits {
    pub max_turns: u32,
    pub max_commands: u32,
    pub max_consecutive_passes: u32,
}

/// The outcome of a single `LocalGame::advance()` call.
#[derive(Debug)]
pub enum AdvanceOutcome {
    /// A human-occupied seat must act. The game state has not been advanced any
    /// further than the moment this decision became available.
    AwaitingHuman(PendingDecision),
    /// CR 104.2a/720 etc. — the game concluded normally.
    GameOver(GameResult),
    /// A safety valve tripped, or the engine rejected a command issued on a bot
    /// seat's behalf with no viable fallback.
    Halted(HaltReason),
}

/// Why `advance()` stopped without reaching `GameOver` or a human decision.
#[derive(Clone, Debug)]
pub enum HaltReason {
    /// `LocalGameLimits::max_turns` was exceeded without the game concluding.
    MaxTurns { max_turns: u32, turn: u32 },
    /// `LocalGameLimits::max_commands` or `max_consecutive_passes` was exceeded
    /// (stuck-game / infinite-loop protection).
    InfiniteLoop { turn: u32 },
    /// The legal-action provider returned no actions for the acting player and even
    /// `PassPriority` was rejected by the engine.
    NoLegalActions { player: PlayerId, turn: u32 },
    /// A bot-seat command was rejected by the engine and the `PassPriority` fallback
    /// was also rejected.
    EngineError(String),
}

impl From<HaltReason> for GameDriverError {
    fn from(reason: HaltReason) -> Self {
        match reason {
            HaltReason::MaxTurns { max_turns, .. } => GameDriverError::MaxTurnsReached(max_turns),
            HaltReason::InfiniteLoop { turn } => GameDriverError::InfiniteLoop { turn },
            HaltReason::NoLegalActions { player, turn } => {
                GameDriverError::NoLegalActions { player, turn }
            }
            HaltReason::EngineError(msg) => GameDriverError::EngineError(msg),
        }
    }
}

/// CR 117.3 (priority), CR 103.5 (mulligan), CR 903.9a (commander zone-change choice),
/// CR 508.1 / 509.1 (combat declarations) — what kind of decision a human seat faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionKind {
    Priority,
    Mulligan,
    CommanderZoneChoice,
    DeclareAttackers,
    DeclareBlockers,
}

/// A decision a human-occupied seat must make before the game can advance further.
#[derive(Clone, Debug)]
pub struct PendingDecision {
    /// Monotonically increasing per `LocalGame`. `submit()` rejects a mismatched
    /// `seq` so a stale browser tab cannot act on a superseded action list.
    pub seq: u64,
    pub player: PlayerId,
    pub kind: DecisionKind,
    pub actions: Vec<LegalAction>,
}

/// A command that was actually applied to the game state, and the events it produced.
#[derive(Clone, Debug)]
pub struct CommandRecord {
    pub command: Command,
    pub events: Vec<GameEvent>,
    pub turn: u32,
}

/// A human choice handed to `LocalGame::submit`. Session 1 accepts a pre-built
/// `Command` directly; full parameterization (`ActionParams`, resolving a
/// `LegalAction` + targets/X/modes into a `Command`) arrives in Session 3.
#[derive(Clone, Debug)]
pub enum HumanChoice {
    Command(Command),
}

/// Errors `LocalGame::start` / `submit` can return. Distinct from `GameStateError`
/// (the engine's own rejection reason) so a caller can tell "your seq was stale" apart
/// from "the engine said no" apart from "the engine itself is unusable".
#[derive(Clone, Debug)]
pub enum LocalGameError {
    /// `submit`'s `seq` did not match the currently pending decision's `seq`.
    StaleDecision { expected: u64, got: u64 },
    /// `submit` was called with no outstanding `PendingDecision`.
    NoPendingDecision,
    /// Reserved for Session 3's `action_index` resolution against `LegalAction`s.
    UnknownAction(usize),
    /// Reserved for Session 3's `ActionParams` validation.
    BadParams(String),
    /// The engine rejected the submitted command. `self.state` is left untouched —
    /// `submit` never falls back to `PassPriority` on a human seat's behalf.
    Rejected(GameStateError),
    /// A failure while starting the game or advancing bot seats.
    Engine(GameStateError),
}

/// A steppable local Commander game: owns the `GameState`, a `LegalActionProvider`,
/// and the bots for every non-human seat. `advance()` runs bot seats autonomously
/// (identically to the old `GameDriver::run_game` loop) and stops the moment a
/// human-occupied seat must act. `human_seats` empty makes this behave exactly like
/// the bot-only driver — see `GameDriver::run_game` below.
pub struct LocalGame<P: LegalActionProvider> {
    state: GameState,
    /// The seed the game was started with, carried only so `GameResult.seed` (a
    /// pre-existing, unchanged field on the fuzzer's report type) can be populated by
    /// `advance()` on `GameOver`; `LocalGame` itself never consumes randomness.
    seed: u64,
    provider: P,
    bots: HashMap<PlayerId, Box<dyn Bot>>,
    /// Seats a human occupies. Empty => pure bot game (the `GameDriver` case).
    human_seats: BTreeSet<PlayerId>,
    limits: LocalGameLimits,
    consecutive_passes: u32,
    command_count: u32,
    /// Turn number as of the last *tracked* command (mirrors the old `run_game`'s
    /// local `prev_turn`) — feeds `check_game_progression`'s stagnation check.
    prev_turn: u32,
    /// Monotonic. Every emitted `PendingDecision` carries the post-increment value.
    decision_seq: u64,
    /// The decision `submit` is currently allowed to answer, if any.
    pending: Option<PendingDecision>,
    journal: Vec<CommandRecord>,
    violations: Vec<InvariantViolation>,
    check_invariants: bool,
}

impl<P: LegalActionProvider> LocalGame<P> {
    /// Starts a game from an assembled (but not yet started) `GameState`. Delegates to
    /// `mtg_engine::start_game`, which enforces Architecture Invariant 9 — a game
    /// whose objects reference a non-`Complete` `CardDefinition` is refused with
    /// `GameStateError::IncompleteCardsInGame`, surfaced here as `LocalGameError::Engine`.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        state: GameState,
        seed: u64,
        provider: P,
        bots: HashMap<PlayerId, Box<dyn Bot>>,
        human_seats: BTreeSet<PlayerId>,
        limits: LocalGameLimits,
        check_invariants: bool,
    ) -> Result<(Self, Vec<GameEvent>), LocalGameError> {
        let (state, start_events) = start_game(state).map_err(LocalGameError::Engine)?;
        let prev_turn = state.turn().turn_number;
        let game = LocalGame {
            state,
            seed,
            provider,
            bots,
            human_seats,
            limits,
            consecutive_passes: 0,
            command_count: 0,
            prev_turn,
            decision_seq: 0,
            pending: None,
            journal: Vec::new(),
            violations: Vec::new(),
            check_invariants,
        };
        Ok((game, start_events))
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn command_count(&self) -> u32 {
        self.command_count
    }

    pub fn violations(&self) -> &[InvariantViolation] {
        &self.violations
    }

    pub fn journal(&self) -> &[CommandRecord] {
        &self.journal
    }

    /// Every `CommandRecord` recorded since `cursor` (an index into `journal()`).
    pub fn journal_since(&self, cursor: usize) -> &[CommandRecord] {
        if cursor >= self.journal.len() {
            &[]
        } else {
            &self.journal[cursor..]
        }
    }

    /// The decision currently awaiting a human answer, if any.
    pub fn pending_decision(&self) -> Option<&PendingDecision> {
        self.pending.as_ref()
    }

    /// Advance the game: run bot seats autonomously (identically to the old
    /// `GameDriver::run_game` loop body) until the game ends, a human-occupied seat
    /// must act, or a safety valve trips.
    pub fn advance(&mut self) -> AdvanceOutcome {
        loop {
            if is_game_over(&self.state) {
                let winner = find_winner(&self.state);
                return AdvanceOutcome::GameOver(GameResult {
                    seed: self.seed,
                    winner,
                    turn_count: self.state.turn().turn_number,
                    total_commands: self.command_count as usize,
                    violations: self.violations.clone(),
                    error: None,
                });
            }

            if self.state.turn().turn_number > self.limits.max_turns {
                return AdvanceOutcome::Halted(HaltReason::MaxTurns {
                    max_turns: self.limits.max_turns,
                    turn: self.state.turn().turn_number,
                });
            }

            if self.command_count >= self.limits.max_commands {
                return AdvanceOutcome::Halted(HaltReason::InfiniteLoop {
                    turn: self.state.turn().turn_number,
                });
            }

            if self.consecutive_passes >= self.limits.max_consecutive_passes {
                return AdvanceOutcome::Halted(HaltReason::InfiniteLoop {
                    turn: self.state.turn().turn_number,
                });
            }

            // Determine acting player (CR 903.9a commander zone choice, then CR 117.3
            // priority holder, then a structural pass to advance between steps).
            let acting_player = if let Some(pending) =
                self.state.pending_commander_zone_choices().iter().next()
            {
                let player = pending.0;
                if self.human_seats.contains(&player) {
                    let actions = self.provider.legal_actions(&self.state, player);
                    return self.await_human(player, DecisionKind::CommanderZoneChoice, actions);
                }
                player
            } else if let Some(priority) = self.state.turn().priority_holder {
                if self.human_seats.contains(&priority) {
                    let actions = self.provider.legal_actions(&self.state, priority);
                    let kind = decision_kind_for(&self.state, &actions);
                    return self.await_human(priority, kind, actions);
                }
                priority
            } else {
                // No one has priority and no pending choices — pass to advance. This
                // can happen between steps; issue PassPriority for active player.
                let active = self.state.turn().active_player;
                let cmd = Command::PassPriority { player: active };
                match self.apply_command(cmd, false) {
                    Ok(_events) => {
                        self.consecutive_passes += 1;
                        continue;
                    }
                    Err(e) => {
                        return AdvanceOutcome::Halted(HaltReason::EngineError(format!("{:?}", e)));
                    }
                }
            };

            // Get legal actions for the (bot) acting player.
            let legal = self.provider.legal_actions(&self.state, acting_player);

            if legal.is_empty() {
                // No legal actions — pass priority to advance.
                let cmd = Command::PassPriority {
                    player: acting_player,
                };
                match self.apply_command(cmd, false) {
                    Ok(_events) => {
                        self.consecutive_passes += 1;
                        continue;
                    }
                    Err(_) => {
                        return AdvanceOutcome::Halted(HaltReason::NoLegalActions {
                            player: acting_player,
                            turn: self.state.turn().turn_number,
                        });
                    }
                }
            }

            // Bot chooses an action.
            let cmd = if let Some(bot) = self.bots.get_mut(&acting_player) {
                bot.choose_action(&self.state, acting_player, &legal)
            } else {
                // No bot assigned — pass priority.
                Command::PassPriority {
                    player: acting_player,
                }
            };

            // Track passes for loop detection.
            if matches!(cmd, Command::PassPriority { .. }) {
                self.consecutive_passes += 1;
            } else {
                self.consecutive_passes = 0;
            }

            // If the command is CastSpell, auto-tap mana sources first.
            let commands = if let Command::CastSpell(cast) = &cmd {
                if let Ok(obj) = self.state.object(cast.card) {
                    if let Some(ref cost) = obj.characteristics.mana_cost {
                        let mut cmds =
                            mana_solver::solve_mana_payment(&self.state, cast.player, cost)
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

            // Execute all commands in sequence (tap commands + the action).
            for c in commands {
                match self.apply_command(c, true) {
                    Ok(_events) => {}
                    Err(e) => {
                        // Command rejected — not necessarily fatal. The provider may
                        // produce invalid actions for a bot seat. Fall back to passing.
                        let fallback = Command::PassPriority {
                            player: acting_player,
                        };
                        match self.apply_command(fallback, false) {
                            Ok(_events) => {
                                self.consecutive_passes += 1;
                            }
                            Err(e2) => {
                                return AdvanceOutcome::Halted(HaltReason::EngineError(format!(
                                    "Both action and fallback failed: {:?}, {:?}",
                                    e, e2
                                )));
                            }
                        }
                        break; // Don't continue the sequence if a command failed.
                    }
                }
            }
        }
    }

    /// Submit a human's chosen command for the currently pending decision. Validates
    /// `seq` against the outstanding `PendingDecision`; on engine rejection returns
    /// `LocalGameError::Rejected` and leaves `self.state` untouched. Never falls back
    /// to `PassPriority` — a mis-submitted human action is an error, not a pass.
    pub fn submit(
        &mut self,
        seq: u64,
        choice: HumanChoice,
    ) -> Result<Vec<GameEvent>, LocalGameError> {
        let pending = self
            .pending
            .as_ref()
            .ok_or(LocalGameError::NoPendingDecision)?;
        if pending.seq != seq {
            return Err(LocalGameError::StaleDecision {
                expected: pending.seq,
                got: seq,
            });
        }

        let HumanChoice::Command(command) = choice;

        match process_command(self.state.clone(), command.clone()) {
            Ok((new_state, events)) => {
                if matches!(command, Command::PassPriority { .. }) {
                    self.consecutive_passes += 1;
                } else {
                    self.consecutive_passes = 0;
                }
                if self.check_invariants {
                    let new_violations = invariants::check_all(&new_state, Some(self.prev_turn));
                    self.violations.extend(new_violations);
                }
                self.prev_turn = new_state.turn().turn_number;
                self.state = new_state;
                self.command_count += 1;
                self.journal.push(CommandRecord {
                    command,
                    events: events.clone(),
                    turn: self.state.turn().turn_number,
                });
                self.pending = None;
                Ok(events)
            }
            Err(e) => Err(LocalGameError::Rejected(e)),
        }
    }

    /// Apply a command that is known-legal-to-attempt (bot seat auto-play or a
    /// structural pass). `track` mirrors the old `run_game`'s distinction between
    /// "clean" command execution (invariants checked, `prev_turn` advanced) and the
    /// untracked structural/fallback passes it never ran invariants against.
    fn apply_command(
        &mut self,
        command: Command,
        track: bool,
    ) -> Result<Vec<GameEvent>, GameStateError> {
        match process_command(self.state.clone(), command.clone()) {
            Ok((new_state, events)) => {
                if track {
                    if self.check_invariants {
                        let new_violations =
                            invariants::check_all(&new_state, Some(self.prev_turn));
                        self.violations.extend(new_violations);
                    }
                    self.prev_turn = new_state.turn().turn_number;
                }
                self.state = new_state;
                self.command_count += 1;
                self.journal.push(CommandRecord {
                    command,
                    events: events.clone(),
                    turn: self.state.turn().turn_number,
                });
                Ok(events)
            }
            Err(e) => Err(e),
        }
    }

    fn await_human(
        &mut self,
        player: PlayerId,
        kind: DecisionKind,
        actions: Vec<LegalAction>,
    ) -> AdvanceOutcome {
        self.decision_seq += 1;
        let decision = PendingDecision {
            seq: self.decision_seq,
            player,
            kind,
            actions,
        };
        self.pending = Some(decision.clone());
        AdvanceOutcome::AwaitingHuman(decision)
    }
}

/// CR 117.3 (priority), CR 103.5 (mulligan), CR 508.1 / 509.1 (combat declarations) —
/// classify what a priority-holding human seat is actually being asked to do, from the
/// legal actions the provider already computed for them.
fn decision_kind_for(state: &GameState, actions: &[LegalAction]) -> DecisionKind {
    if state.turn().is_first_turn_of_game && state.turn().turn_number == 0 {
        return DecisionKind::Mulligan;
    }
    if actions
        .iter()
        .any(|a| matches!(a, LegalAction::DeclareAttackers { .. }))
    {
        return DecisionKind::DeclareAttackers;
    }
    if actions
        .iter()
        .any(|a| matches!(a, LegalAction::DeclareBlockers { .. }))
    {
        return DecisionKind::DeclareBlockers;
    }
    DecisionKind::Priority
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
