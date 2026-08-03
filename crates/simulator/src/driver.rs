//! Game driver — runs a complete game with bots making decisions.
//!
//! `GameDriver<P>` is generic over the `LegalActionProvider`, allowing
//! both the stub provider (Phase 1) and full provider (Phase 4).
//!
//! M11-local Session 1: `run_game` is re-expressed on top of `LocalGame`
//! (`local_game.rs`) with `human_seats` empty — `LocalGame::advance()` never
//! returns `AwaitingHuman` in that configuration, so every step here is either
//! `GameOver` or `Halted`. This is the single loop `LocalGame` and `GameDriver`
//! now share.
//!
//! **Evidence for behavioural parity**, stated precisely because the obvious check
//! is not currently available: the port was verified by a byte-identical *single-seed*
//! command-trace replay across the refactor, plus a statement-by-statement review of
//! the counters, the tracked/untracked invariant-check asymmetry, the pass-count reset,
//! every error variant, the `CastSpell` auto-tap pre-pass and the sequence `break`.
//! A full fuzzer-baseline diff is **not** currently a usable oracle: the fuzzer is not
//! run-to-run deterministic for very long games, and that reproduces on pristine
//! pre-refactor code — see `OOS-M11-3` in `memory/m11-session-plan.md` §4 Session 1.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{GameState, PlayerId};

use crate::bot::Bot;
use crate::legal_actions::LegalActionProvider;
use crate::local_game::{
    AdvanceOutcome, LocalGame, LocalGameError, LocalGameLimits, MechanicsTally,
};
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
        self.run_game_with_mechanics(initial_state, seed).0
    }

    /// As [`Self::run_game`], and additionally returns the game's [`MechanicsTally`].
    ///
    /// PB-DX22 fix cycle (review Finding 1): `mtg-fuzzer` reports commander mechanics and
    /// first-cast depth in its own summary, so those numbers are re-derivable from
    /// committed code rather than from a deleted scratch instrument. The tally is a
    /// constant-size counter set folded from events already in hand — it does **not**
    /// require the journal, which this driver keeps off on purpose.
    ///
    /// It is a separate method rather than a new field on [`GameResult`] because
    /// `GameResult` is constructed outside this crate (`tools/play-server`), and PB-DX22
    /// may not touch `tools/`.
    pub fn run_game_with_mechanics(
        self,
        initial_state: GameState,
        seed: u64,
    ) -> (GameResult, MechanicsTally) {
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
            // The fuzzer discards events and runs thousands of long games in parallel;
            // journalling every command would retain memory the pre-M11 driver never did.
            record_journal: false,
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
                // Unwrap `LocalGameError::Engine` so crash reports keep the pre-M11
                // shape — `EngineError("IncompleteCardsInGame { … }")`, not
                // `EngineError("Engine(IncompleteCardsInGame { … })")`.
                let message = match e {
                    LocalGameError::Engine(inner) => format!("{:?}", inner),
                    other => format!("{:?}", other),
                };
                return (
                    GameResult {
                        seed,
                        winner: None,
                        turn_count: 0,
                        total_commands: 0,
                        violations: Vec::new(),
                        error: Some(GameDriverError::EngineError(message)),
                    },
                    MechanicsTally::default(),
                );
            }
        };

        // A single `advance()` call runs the whole game to conclusion: with
        // `human_seats` empty it never yields `AwaitingHuman`, so `advance()`'s own
        // internal loop only stops at `GameOver` or `Halted`.
        let result = match game.advance() {
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
        };
        // Read AFTER the game has run: the CR 903.10a half of the census is a final-state
        // read, not an event fold.
        let mechanics = game.mechanics();
        (result, mechanics)
    }
}
