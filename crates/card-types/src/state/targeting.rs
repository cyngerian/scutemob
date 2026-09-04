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
    ///
    /// This is a `state.objects` key. A SPELL on the stack is named this way: CR 601.2a
    /// moves the card into `ZoneId::Stack` with a fresh `ObjectId` (CR 400.7), and that
    /// card id is what the offer layer enumerates and the player announces.
    Object(ObjectId),
    /// An **ability** on the stack, named by its `StackObject::id` (CR 602.2a / CR 603.3,
    /// CR 115.7a
    /// -- "target spell or ability").
    ///
    /// PB-DX52 (`OOS-DX25b-1`). An activated or triggered ability's stack entry is minted
    /// by `state.next_object_id()` and pushed into `state.stack_objects`; it is **never**
    /// added to `state.objects`, because it owns no card
    /// (`state::stack_registry::card_in_stack_zone` returns `None` for every ability
    /// kind). Before this variant existed there was therefore no id space in which a
    /// player could name one, so Bolt Bend's printed "or ability" half was dead and
    /// `TargetSpellOrAbilityWithSingleTarget` was behaviourally identical to the
    /// spell-only `TargetSpellWithSingleTarget` on every production path.
    ///
    /// **Why a third `Target` variant rather than registering ability entries in
    /// `state.objects`.** The CR argues for registration on its face -- CR 113.1c: *"An
    /// ability can be an activated or triggered ability on the stack. This kind of ability
    /// is an object"*, and CR 109.1 lists *"an ability on the stack"* first among the
    /// things an object is. That alternative was costed and REJECTED at PB-DX52 stage 0
    /// (`memory/primitives/pb-DX52-execution-notes.md` §0.3), because `state.objects` is
    /// not this engine's model of CR 109.1's "object" -- it is the CARD-object map, and
    /// CR 113 abilities are modelled by `state.stack_objects`: an entry in that
    /// map must claim a `ZoneId`, and the only honest claim is `ZoneId::Stack`; but
    /// `casting.rs`'s `TargetRequirement::TargetSpell` arm decides "is this a spell" by
    /// `obj.zone == ZoneId::Stack` **alone**, so a registered ability would immediately
    /// become a legal target for "counter target spell" (CR 115.1a-wrong (a spell is not a permanent, and "counter target spell" names a spell)). Registration
    /// also forces zone membership (`simulator::invariants::check_zone_integrity`), which
    /// moves `public_state_hash` for every game with an ability on the stack and
    /// double-counts the entry in `loop_detection::compute_mandatory_state_hash`. And
    /// `GameObject` has no `CardType` that fits an ability and no "kind" discriminator at
    /// all. `state.objects` is this engine's **card**-object map; CR 113 abilities are
    /// modelled by `state.stack_objects`, and this variant names them there.
    ///
    /// **The id is the stack ENTRY's own id, deliberately.**
    /// `state::stack_registry::stack_index_for_announced_target`'s first clause is already
    /// `so.id == announced`, so every existing consumer -- `Effect::ChangeTargets`,
    /// `Effect::CounterSpell`, `Effect::CopySpellOnStack` and both single-target arms of
    /// `casting::validate_object_satisfies_requirement` -- resolves one of these through
    /// the SAME shared arithmetic a card id goes through, with no second lookup to drift.
    ///
    /// **`zone_at_cast` is `None`** for this variant, like a player target: a stack entry
    /// is not in a zone the way a card is. CR 608.2b's own sentence is about a target that
    /// is *"no longer in the zone it was in"*, which is written about a CARD changing zones
    /// (CR 400.7); an ability never does that, because CR 608.2n says that as the final
    /// part of its resolution *"the ability is removed from the stack and ceases to
    /// exist"*. So the equivalent legality question is existence: an ability that has
    /// resolved or been countered is gone from `state.stack_objects`, and is an illegal
    /// target.
    ///
    /// **No CR 702.21a Ward dispatch is owed for one of these.** Ward reads "whenever this
    /// **permanent** becomes the target of a spell or ability an opponent controls", and
    /// an ability on the stack is not a permanent; `rules::events::permanent_targeted_events`
    /// says so in an explicit arm rather than by falling through a wildcard.
    StackObject(ObjectId),
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
    ///
    /// PB-DX52: `Target::StackObject(SENTINEL)` is deliberately NOT recognised here, and
    /// the reason is that it cannot be constructed: [`SpellTarget::unchosen_slot`] is the
    /// only producer of the placeholder and it emits `Target::Object` unconditionally.
    /// Widening this predicate to accept a second spelling of "unchosen" would create a
    /// shape nothing writes and everything would then have to keep handling.
    pub fn is_unchosen_slot(&self) -> bool {
        matches!(self.target, Target::Object(ObjectId::SENTINEL))
    }
}
