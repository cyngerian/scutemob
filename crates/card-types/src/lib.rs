//! Card DSL types and the game-state data model they are built from.
//!
//! This crate holds everything a `CardDefinition` needs to exist, and nothing
//! that needs a running game. It is the bottom of the workspace dependency
//! graph:
//!
//! ```text
//!            mtg-card-types  (this crate: DSL + data model)
//!             ↑           ↑
//!    mtg-card-defs        |     (1,749 per-card files + build.rs discovery)
//!             ↑           |
//!            mtg-engine ──┘     (GameState, rules, effects)
//! ```
//!
//! The split exists so that editing an engine-internal file — anything in
//! `rules/`, `effects/`, or the `GameState` struct itself — cannot invalidate
//! the compilation of the card definitions, which are roughly half the source
//! lines in the project. Card defs are *upstream* of the engine, so cargo
//! never rebuilds them for an engine change.
//!
//! The corollary: changing anything in *this* crate does rebuild every card
//! def. That is correct — the DSL is their language. Keep runtime concerns out
//! of here, and the split keeps paying.
//!
//! Nothing in this crate may depend on `GameState`. The `state` modules here
//! are the pure data types (`ObjectId`, `Characteristics`, `KeywordAbility`,
//! filters, …); `GameState` and its mutation surface live in `mtg-engine`.
pub mod cards;
pub mod state;
