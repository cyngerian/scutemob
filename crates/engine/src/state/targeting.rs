//! Spell targeting types (CR 601.2c, 608.2b).
//!
//! Targets are announced when a spell is cast and validated again at resolution.
//! The fizzle rule (CR 608.2b) applies when ALL targets are illegal at resolution:
//! the spell is countered without effect and its card goes to the graveyard.
//!
//! Partial fizzle (some but not all targets illegal): spell resolves normally,
//! but illegal targets are unaffected by the spell's effect (M7+).

use serde::{Deserialize, Serialize};

use super::game_object::ObjectId;
use super::player::PlayerId;
use super::zone::ZoneId;

/// A target that a spell or ability can point at (CR 109.1 / 114).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    /// A player (any active player may be a target unless specified otherwise).
    Player(PlayerId),
    /// A game object (card, token, etc.) in any zone.
    Object(ObjectId),
}

/// A recorded target for a spell or ability on the stack.
///
/// Captures the target at cast time including a zone snapshot for fizzle detection.
/// At resolution, CR 608.2b checks whether each target is still in its original zone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellTarget {
    pub target: Target,
    /// Zone the target object was in at the time of targeting.
    /// `None` for player targets (players are not in a zone).
    /// At resolution: if the object is no longer in `zone_at_cast`, the target is illegal.
    pub zone_at_cast: Option<ZoneId>,
}
