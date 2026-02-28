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

pub mod bot;
pub mod deck;
pub mod driver;
pub mod heuristic_bot;
pub mod invariants;
pub mod legal_actions;
pub mod mana_solver;
pub mod random_bot;
pub mod report;

// Re-export key types for convenience
pub use bot::Bot;
pub use deck::{build_registry, random_deck, DeckConfig};
pub use driver::GameDriver;
pub use heuristic_bot::HeuristicBot;
pub use invariants::{check_all as check_invariants, InvariantViolation};
pub use legal_actions::{LegalAction, LegalActionProvider, StubProvider};
pub use mana_solver::solve_mana_payment;
pub use random_bot::RandomBot;
pub use report::{CrashReport, GameDriverError, GameResult};
