//! The card definition corpus: one file per card, discovered at build time.
//!
//! `build.rs` scans `src/defs/` and generates the module list plus
//! [`all_cards`]. Adding a card is adding one file — no shared file is edited,
//! so concurrent authoring sessions never collide on a registry.
//!
//! This crate depends only on `mtg-card-types`, never on `mtg-engine`. That is
//! what buys the compile isolation: an edit to the engine's rules or effects
//! cannot invalidate these 1,749 modules, because they sit upstream of it.
//!
//! ## Why the two re-exports below
//!
//! Every def file opens with `use crate::cards::helpers::*;`, and a handful
//! reach for `crate::state::…` directly. Those paths were written when the defs
//! lived inside `mtg-engine`. Re-exporting the two modules here under the same
//! names keeps all of them resolving unchanged, so the extraction moved the
//! files without touching their contents. Anything a def can name still comes
//! from `mtg-card-types`; this crate adds no vocabulary of its own.
pub(crate) use mtg_card_types::{cards, state};

pub mod defs;

pub use defs::all_cards;
