//! Placeholder types for systems implemented in later milestones.
//!
//! These exist so GameState can compile with all fields from the architecture
//! doc. Each type will be fully fleshed out in its respective milestone.

use serde::{Deserialize, Serialize};

use super::game_object::ObjectId;
use super::player::PlayerId;

// ContinuousEffect has moved to `state/continuous_effect.rs` (M5).

/// A delayed trigger waiting for a condition (CR 603.7). Implemented in M3.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayedTrigger {
    pub source: ObjectId,
}

// ReplacementEffect has moved to `state/replacement_effect.rs` (M8).

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
    /// The game event that caused this trigger to fire.
    ///
    /// Stored for Panharmonicon-style trigger doubling (CR 603.2d): the doubler
    /// needs to know what event caused the trigger to determine if it applies.
    /// `None` for triggers queued without a specific event context (e.g., delayed triggers).
    #[serde(default)]
    pub triggering_event: Option<super::game_object::TriggerEvent>,
    /// The object that entered the battlefield and caused this trigger to fire, if applicable.
    ///
    /// Populated for `AnyPermanentEntersBattlefield` triggers so that
    /// `TriggerDoublerFilter::ArtifactOrCreatureETB` can verify the entering
    /// object's card types (CR 603.2d — Panharmonicon doubles only when an
    /// artifact or creature enters, not any permanent).
    /// `None` for triggers that are not caused by a specific permanent entering.
    #[serde(default)]
    pub entering_object_id: Option<ObjectId>,
}

// StackObject has moved to `state/stack.rs` (M3-A).

// CombatState has moved to `state/combat.rs` (M6).

// GameEvent has moved to crate::rules::events (M2).

/// Which triggers are doubled by a `TriggerDoubler` (CR 603.2d).
///
/// Used to filter which pending triggers get additional copies queued when
/// `flush_pending_triggers` processes them. Adding new filter variants here
/// enables new Panharmonicon-style cards without touching `flush_pending_triggers`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerDoublerFilter {
    /// "Whenever an artifact or creature enters the battlefield" — Panharmonicon.
    ///
    /// Doubles ETB triggered abilities from artifacts and creatures (CR 603.2d).
    /// Specifically: any triggered ability on a permanent that would trigger from
    /// a creature or artifact permanent entering the battlefield.
    ArtifactOrCreatureETB,
}

/// A Panharmonicon-style trigger-doubling effect (CR 603.2d).
///
/// When a trigger that matches the filter would be queued, it is queued an
/// additional `additional_triggers` times instead. Each instance resolves
/// independently on the stack.
///
/// Registered when a permanent with the appropriate ability enters the
/// battlefield; unregistered (by source ObjectId) when it leaves.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerDoubler {
    /// ObjectId of the permanent generating this effect (Panharmonicon, etc.).
    pub source: ObjectId,
    /// The player who controls the source permanent.
    pub controller: PlayerId,
    /// Which ETB triggers are doubled by this effect.
    pub filter: TriggerDoublerFilter,
    /// How many additional times the trigger fires (usually 1).
    pub additional_triggers: u32,
}
