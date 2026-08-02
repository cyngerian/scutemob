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
pub mod targeting;

// Re-export key types for convenience
pub use bot::Bot;
pub use deck::{build_registry, random_deck, DeckConfig};
pub use driver::GameDriver;
pub use heuristic_bot::HeuristicBot;
pub use invariants::{check_all as check_invariants, InvariantViolation};
// SIM-1 (CR 903.8): `pub`, not `pub(crate)` -- OOS-SIM1-2 names a FOURTH printed-cost
// auto-tap site outside this crate (`tools/tui/src/play/app.rs`'s bot path); exporting
// the helper makes that a one-line fix later instead of a copy.
// UI-2 (CR 118.8 / CR 702.157): the additional-cost descriptor is part of this
// crate's public surface -- `tools/play-server` renders it and validates a
// submission against it. `effective_cast_cost_with_additional` is exported for the
// same reason `effective_cast_cost` is: any caller that pays for a cast must use
// the same arithmetic the offer gate used (SR-38).
pub use legal_actions::{
    effective_cast_cost, effective_cast_cost_with_additional, AdditionalCostPlan, LegalAction,
    LegalActionProvider, SacrificeCostOption, SquadCostOption, StubProvider,
};
pub use local_game::{
    human_only_actions, AdvanceOutcome, CommandRecord, DecisionKind, HaltReason, LocalGame,
    LocalGameError, LocalGameLimits, PendingDecision,
};
pub use mana_solver::solve_mana_payment;
pub use params::{action_to_command_with_params, ActionParams, HumanChoice, ParamError};
pub use random_bot::RandomBot;
pub use report::{CrashReport, GameDriverError, GameResult};
pub use setup::{
    build_initial_state, dealt_decks, redeal, BotKind, DeckSource, LocalGameConfig, SetupError,
};
// SIM-5 (CR 601.2c): `plan_targets` is how a bot announces targets, and `TargetPlan`
// is also the predicate a future offer gate needs (G5 fix (4) -- see the module doc).
pub use targeting::{plan_targets, TargetPlan};
