//! Game driver — runs a complete game with bots making decisions.
//!
//! `GameDriver<P>` is generic over the `LegalActionProvider`, allowing
//! both the stub provider (Phase 1) and full provider (Phase 4).
//!
//! M11-local Session 1: `run_game` is re-expressed on top of `LocalGame`
//! (`local_game.rs`) with `human_seats` empty — `LocalGame::advance()` never
//! returns `AwaitingHuman` in that configuration, so every step here is either
//! `GameOver` or `Halted`. This is the single loop `LocalGame` and `GameDriver`
//! now share; behaviour is unchanged (verified against a pre-refactor fuzzer
//! baseline, see `memory/m11-session-plan.md` §4 Session 1 items 1 and 8).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{GameState, PlayerId};

use crate::bot::Bot;
use crate::legal_actions::LegalActionProvider;
use crate::local_game::{AdvanceOutcome, LocalGame, LocalGameLimits};
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
    ///
    /// Consumes `self`: `LocalGame` owns `provider` and `bots`, and this driver is a
    /// single-use "run one game" object (its sole caller, `mtg-fuzzer`'s
    /// `run_single_game`, constructs a fresh `GameDriver` per game and never reuses it
    /// after calling `run_game`).
    pub fn run_game(self, initial_state: GameState, seed: u64) -> GameResult {
        let GameDriver {
            provider,
            bots,
            max_turns,
            max_commands,
            check_invariants,
        } = self;

        let limits = LocalGameLimits {
            max_turns,
            max_commands,
            max_consecutive_passes: 500, // Safety: break infinite pass loops
        };

        let mut game = match LocalGame::start(
            initial_state,
            seed,
            provider,
            bots,
            BTreeSet::new(), // No human seats: this is the pure-bot driver.
            limits,
            check_invariants,
        ) {
            Ok((game, _start_events)) => game,
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

        // A single `advance()` call runs the whole game to conclusion: with
        // `human_seats` empty it never yields `AwaitingHuman`, so `advance()`'s own
        // internal loop only stops at `GameOver` or `Halted`.
        match game.advance() {
            AdvanceOutcome::GameOver(result) => result,
            AdvanceOutcome::Halted(reason) => GameResult {
                seed,
                winner: None,
                turn_count: game.state().turn().turn_number,
                total_commands: game.command_count() as usize,
                violations: game.violations().to_vec(),
                error: Some(reason.into()),
            },
            AdvanceOutcome::AwaitingHuman(_) => unreachable!(
                "human_seats is empty; LocalGame::advance() must never yield AwaitingHuman"
            ),
        }
    }
}
