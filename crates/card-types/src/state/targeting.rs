//! Spell targeting types (CR 601.2c, 608.2b).
//!
//! Targets are announced when a spell is cast and validated again at resolution.
//! The fizzle rule (CR 608.2b) applies when ALL targets are illegal at resolution:
//! the spell is countered without effect and its card goes to the graveyard.
//!
//! Partial fizzle (some but not all targets illegal): spell resolves normally,
//! but illegal targets are unaffected by the spell's effect (M7+).
use super::game_object::ObjectId;
use super::player::PlayerId;
use super::zone::ZoneId;
use serde::{Deserialize, Serialize};
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
impl SpellTarget {
    /// CR 601.2c (PB-DP8 fix cycle, Finding 6): the placeholder that holds an
    /// un-taken "up to N" slot's position in a flat declared-target list.
    ///
    /// A spell or ability carries its targets as one flat `Vec<SpellTarget>` and
    /// its card definition reads them by absolute index
    /// (`EffectTarget::DeclaredTarget { index }`). A `TargetRequirement::UpToN`
    /// slot therefore has a fixed *width* (`count`), and answering it with fewer
    /// than `count` targets would otherwise shift every later slot down — so
    /// "destroy up to one target planeswalker and up to one target artifact"
    /// answered `[[], [artifact]]` would resolve the **planeswalker** clause
    /// against the artifact.
    ///
    /// `ObjectId::SENTINEL` is never assigned to a real object (the counter starts
    /// at 1), so `resolve_effect_target_list_indexed` finds no object for it and
    /// contributes nothing (the CR 608.2b partial-fizzle skip) — exactly what an
    /// out-of-range index already did.
    ///
    /// **Only interior holes are padded.** Trailing un-taken slots are simply
    /// omitted, so an all-empty announcement still yields an EMPTY target list and
    /// cannot trip CR 608.2b's "all targets are illegal" fizzle.
    pub const fn unchosen_slot() -> SpellTarget {
        SpellTarget {
            target: Target::Object(ObjectId::SENTINEL),
            zone_at_cast: None,
        }
    }
    /// True for the [`SpellTarget::unchosen_slot`] placeholder (CR 601.2c).
    pub fn is_unchosen_slot(&self) -> bool {
        matches!(self.target, Target::Object(ObjectId::SENTINEL))
    }
}
