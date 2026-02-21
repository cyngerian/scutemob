//! Placeholder types for systems implemented in later milestones.
//!
//! These exist so GameState can compile with all fields from the architecture
//! doc. Each type will be fully fleshed out in its respective milestone.

use serde::{Deserialize, Serialize};

use super::game_object::ObjectId;
use super::player::PlayerId;

/// A continuous effect modifying game objects (CR 611). Implemented in M5.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContinuousEffect {
    pub source: ObjectId,
    pub timestamp: u64,
}

/// A delayed trigger waiting for a condition (CR 603.7). Implemented in M3.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayedTrigger {
    pub source: ObjectId,
}

/// A replacement effect that modifies events (CR 614). Implemented in M8.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplacementEffect {
    pub source: ObjectId,
}

/// A triggered ability queued to go on the stack (CR 603.3).
///
/// Collected after each event in `GameState::pending_triggers`; placed on
/// the stack in APNAP order the next time a player would receive priority.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingTrigger {
    /// The source object of the triggered ability.
    pub source: ObjectId,
    /// Index into `source.characteristics.triggered_abilities`.
    pub ability_index: usize,
    /// The player who controls this triggered ability.
    pub controller: PlayerId,
}

// StackObject has moved to `state/stack.rs` (M3-A).

/// Combat state tracking (CR 506-511). Implemented in M6.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatState {
    pub attacking_player: PlayerId,
}

// GameEvent has moved to crate::rules::events (M2).
