//! MTG Commander game simulator — bot framework, fuzzer, and game driver.
//!
//! # Architecture
//!
//! - `LegalActionProvider` trait with a `StubProvider` for basic move enumeration
//! - `Bot` trait with `RandomBot` (fuzzing) and `HeuristicBot` (realistic play)
//! - `GameDriver<P>` runs complete games with bots making all decisions
//! - `invariants` module checks game state consistency after every transition
//! - `mana_solver` provides greedy mana payment
//! - `deck` builds random Commander decks from available CardDefinitions
//! - `setup` builds a deterministic, `validate_deck`-admitted pregame `GameState` from a
//!   single seed, and re-deals a seat's opening hand for a pregame mulligan (M11-local
//!   Session 2)

pub mod bot;
pub mod deck;
pub mod driver;
pub mod heuristic_bot;
pub mod invariants;
pub mod legal_actions;
pub mod local_game;
pub mod mana_solver;
pub mod params;
pub mod random_bot;
pub mod report;
pub mod setup;

// Re-export key types for convenience
pub use bot::Bot;
pub use deck::{build_registry, random_deck, DeckConfig};
pub use driver::GameDriver;
pub use heuristic_bot::HeuristicBot;
pub use invariants::{check_all as check_invariants, InvariantViolation};
pub use legal_actions::{LegalAction, LegalActionProvider, StubProvider};
pub use local_game::{
    human_only_actions, AdvanceOutcome, CommandRecord, DecisionKind, HaltReason, LocalGame,
    LocalGameError, LocalGameLimits, PendingDecision,
};
pub use mana_solver::solve_mana_payment;
pub use params::{action_to_command_with_params, ActionParams, HumanChoice, ParamError};
pub use random_bot::RandomBot;
pub use report::{CrashReport, GameDriverError, GameResult};
pub use setup::{build_initial_state, redeal, BotKind, DeckSource, LocalGameConfig, SetupError};
