//! Fundamental MTG type enums used throughout the engine.

use serde::{Deserialize, Serialize};

/// The five colors of Magic (CR 105.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

/// Mana colors including colorless, for mana pool tracking (CR 106).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ManaColor {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

/// Card supertypes (CR 205.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SuperType {
    Basic,
    Legendary,
    Snow,
    World,
    Ongoing,
}

/// Card types (CR 205.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CardType {
    Artifact,
    Battle,
    Conspiracy,
    Creature,
    Dungeon,
    Enchantment,
    Instant,
    Kindred,
    Land,
    Phenomenon,
    Plane,
    Planeswalker,
    Scheme,
    Sorcery,
    Vanguard,
}

/// Card subtypes (CR 205.3). Open-ended — 280+ creature types exist.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SubType(pub String);

/// Counter types that can be placed on objects or players (CR 122).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CounterType {
    PlusOnePlusOne,
    MinusOneMinusOne,
    Loyalty,
    Charge,
    Energy,
    Experience,
    Level,
    Lore,
    Oil,
    Poison,
    Shield,
    Stun,
    Time,
    /// Catch-all for counter types not explicitly enumerated.
    Custom(String),
}

/// Keyword abilities (CR 702). Common keywords used in rules processing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KeywordAbility {
    Deathtouch,
    Defender,
    DoubleStrike,
    Enchant,
    Equip,
    FirstStrike,
    Flash,
    Flying,
    Haste,
    Hexproof,
    Indestructible,
    Intimidate,
    Landwalk,
    Lifelink,
    Menace,
    Protection,
    Prowess,
    Reach,
    Shroud,
    Trample,
    Vigilance,
    Ward,
    /// CR 702.124: Partner keyword — allows two legendary creatures to serve as
    /// commanders together. Both commanders must have partner.
    Partner,
    /// CR 402.2: "You have no maximum hand size."
    ///
    /// Placed on permanents (Thought Vessel, Reliquary Tower). When a permanent
    /// with this keyword is on the battlefield under a player's control, the
    /// `no_max_hand_size` flag is set on that player's `PlayerState`, skipping
    /// the cleanup discard step (CR 514.1).
    NoMaxHandSize,
}
