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

/// Protection quality: what a permanent is protected from (CR 702.16a).
///
/// Used in `KeywordAbility::ProtectionFrom(ProtectionQuality)` to specify
/// which sources are blocked by protection (DEBT: Damage, Enchanting, Blocking, Targeting).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProtectionQuality {
    /// Protection from a specific color (e.g., "protection from red").
    FromColor(Color),
    /// Protection from a card type (e.g., "protection from artifacts").
    FromCardType(CardType),
    /// Protection from a subtype (e.g., "protection from Goblins").
    FromSubType(SubType),
    /// Protection from everything (e.g., "protection from everything").
    FromAll,
}

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
///
/// Note: `Copy` is not derived because `ProtectionFrom(ProtectionQuality)` contains
/// `ProtectionQuality` which can hold a `SubType(String)` (not `Copy`).
/// Use `.clone()` where a copy was previously implicit.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    /// CR 702.16a: Protection from a quality (DEBT: Damage, Enchanting, Blocking, Targeting).
    ///
    /// Replaces the former bare `Protection` variant. Use `ProtectionFrom(ProtectionQuality)`
    /// to specify what the permanent is protected from (e.g., `ProtectionFrom(FromColor(Red))`).
    ProtectionFrom(ProtectionQuality),
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
    /// CR 509.1: "This creature can't be blocked."
    ///
    /// Checked in `rules/combat.rs:handle_declare_blockers`. Any blocker assignment
    /// targeting a creature with this keyword is rejected.
    CantBeBlocked,
    /// CR 702.40: Storm — when you cast this spell, copy it for each other spell
    /// cast before it this turn. You may choose new targets for each copy.
    ///
    /// Checked in `rules/resolution.rs` at cast time: when a spell with Storm is
    /// put on the stack, the storm trigger is queued. On resolution, copies equal
    /// to `spells_cast_this_turn - 1` are pushed above the original.
    Storm,
    /// CR 702.85: Cascade — when you cast this spell, exile cards from top of
    /// library until you exile a nonland card with mana value strictly less than
    /// this spell's mana value. You may cast that card without paying its mana
    /// cost. Put the rest on the bottom of your library in a random order.
    ///
    /// Cascade triggers when the spell is cast (not when it resolves). Handled
    /// in `rules/casting.rs:handle_cast_spell` and `rules/copy.rs:resolve_cascade`.
    Cascade,
}
