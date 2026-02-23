//! Replacement and prevention effect type definitions (CR 614, 615).
//!
//! Replacement effects watch for events as they happen and modify or replace
//! them before they occur. They are NOT triggers — they don't use the stack.
//! Prevention effects are a subset that specifically prevent damage.
//!
//! Key rules:
//! - CR 614.5: A replacement effect can apply to a given event at most once.
//! - CR 614.15: Self-replacement effects apply before other replacements.
//! - CR 615.7: Prevention shields reduce by the amount of damage prevented.
//! - CR 616.1: When multiple replacements apply, affected player chooses order.

use serde::{Deserialize, Serialize};

use super::continuous_effect::EffectDuration;
use super::game_object::ObjectId;
use super::player::{CardId, PlayerId};
use super::types::{CardType, CounterType};
use super::zone::ZoneType;

/// Unique identifier for a replacement effect instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ReplacementId(pub u64);

/// Which event a replacement effect watches for (CR 614.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplacementTrigger {
    /// An object would change zones (CR 614.1a).
    /// E.g., "If ~ would die" / "If a creature would be put into a graveyard".
    WouldChangeZone {
        from: Option<ZoneType>,
        to: ZoneType,
        filter: ObjectFilter,
    },
    /// A player would draw a card (CR 614.11).
    /// E.g., "If a player would draw a card" / "skip that draw".
    WouldDraw { player_filter: PlayerFilter },
    /// A permanent would enter the battlefield (CR 614.12).
    /// E.g., "enters the battlefield tapped" / "enters with N counters".
    WouldEnterBattlefield { filter: ObjectFilter },
    /// A player would gain life.
    /// E.g., "If you would gain life, draw that many cards instead".
    WouldGainLife { player_filter: PlayerFilter },
    /// Damage would be dealt (CR 614.2, 615.1).
    /// Used by both replacement and prevention effects on damage.
    DamageWouldBeDealt { target_filter: DamageTargetFilter },
}

/// What a replacement effect does when it applies (CR 614.6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplacementModification {
    /// Redirect a zone change to a different zone (CR 614.1a).
    /// E.g., "exile it instead" / "put it into the command zone instead".
    RedirectToZone(ZoneType),
    /// Permanent enters the battlefield tapped (CR 614.1c).
    /// E.g., "~ enters the battlefield tapped".
    EntersTapped,
    /// Permanent enters the battlefield with counters (CR 614.1c).
    /// E.g., "~ enters the battlefield with N +1/+1 counters".
    EntersWithCounters { counter: CounterType, count: u32 },
    /// Skip the draw entirely (CR 614.10).
    /// E.g., "skip that draw" / replacement that prevents the draw.
    SkipDraw,
    /// Prevent exactly N damage from this event (CR 615.7).
    /// The shield is tracked separately and decrements on each application.
    PreventDamage(u32),
    /// Prevent all damage from this event (CR 615.1).
    PreventAllDamage,
}

/// Filters which objects a replacement trigger matches.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectFilter {
    /// Matches any object.
    Any,
    /// Matches a specific permanent by ObjectId.
    SpecificObject(ObjectId),
    /// Matches any object controlled by a specific player.
    ControlledBy(PlayerId),
    /// Matches any creature.
    AnyCreature,
    /// Matches any object with a specific card type.
    HasCardType(CardType),
    /// Matches any commander.
    Commander,
    /// Matches an object with a specific CardId (persistent across zone changes).
    /// Used for commander zone-change replacements: the CardId survives zone changes
    /// even though the ObjectId doesn't (CR 400.7).
    HasCardId(CardId),
}

/// Filters which players a replacement trigger matches.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerFilter {
    /// Matches any player.
    Any,
    /// Matches a specific player.
    Specific(PlayerId),
    /// Matches any opponent of a specific player.
    OpponentsOf(PlayerId),
}

/// Filters which damage targets a prevention/replacement applies to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageTargetFilter {
    /// Any target (creature, player, or planeswalker).
    Any,
    /// Only damage dealt to a specific player.
    Player(PlayerId),
    /// Only damage dealt to a specific permanent.
    Permanent(ObjectId),
    /// Only damage dealt by a specific source.
    FromSource(ObjectId),
}

/// A replacement or prevention effect active in the game (CR 614, 615).
///
/// Replacement effects intercept events as they happen and modify them
/// inline — they do NOT use the stack. When multiple replacement effects
/// apply to the same event, the affected player or controller chooses
/// the order (CR 616.1), but self-replacement effects always apply
/// first (CR 614.15).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplacementEffect {
    /// Unique identifier for this effect instance.
    pub id: ReplacementId,
    /// The source permanent generating this effect, or `None` for
    /// non-permanent sources (e.g., a resolved spell, game rules).
    pub source: Option<ObjectId>,
    /// The player who controls this replacement effect.
    pub controller: PlayerId,
    /// How long this effect lasts.
    pub duration: EffectDuration,
    /// CR 614.15: If true, this is a self-replacement effect and applies
    /// before other replacement effects on the same event.
    pub is_self_replacement: bool,
    /// Which event this effect watches for.
    pub trigger: ReplacementTrigger,
    /// What this effect does when it applies.
    pub modification: ReplacementModification,
}

/// Tracks a zone change that is waiting for the affected player to choose
/// which replacement effect to apply (CR 616.1).
///
/// When multiple replacement effects apply to the same zone change (e.g.,
/// commander dies with Rest in Peace active), the object stays on the
/// battlefield until the player submits `Command::OrderReplacements`.
/// The SBA loop skips objects with pending zone changes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingZoneChange {
    /// The object that's waiting to be moved.
    pub object_id: ObjectId,
    /// Where the object was going to go before replacement.
    pub original_destination: ZoneType,
    /// The affected player who must choose (typically the object's owner).
    pub affected_player: PlayerId,
    /// Replacement effects already applied in this chain (CR 614.5).
    pub already_applied: Vec<ReplacementId>,
}
