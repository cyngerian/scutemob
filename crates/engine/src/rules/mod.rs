//! Rules engine: turn structure, priority, stack, SBAs, layers, combat.
//!
//! Each submodule corresponds to a section of the MTG Comprehensive Rules.

pub mod command;
pub mod engine;
pub mod events;
pub mod priority;
pub mod turn_actions;
pub mod turn_structure;

pub use command::Command;
pub use engine::process_command;
pub use events::{GameEvent, LossReason};
