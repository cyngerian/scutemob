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

/// Specifies which kind of landwalk ability (CR 702.14a).
///
/// Landwalk is a generic term -- each variant specifies what kind of land the
/// defending player must control for the creature to become unblockable.
/// CR 702.14c: "A creature with landwalk can't be blocked as long as the defending
/// player controls at least one land with the specified type."
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LandwalkType {
    /// Checks if defending player controls a land with the given basic land subtype.
    /// Covers: Plainswalk (`SubType("Plains")`), Islandwalk (`SubType("Island")`),
    /// Swampwalk (`SubType("Swamp")`), Mountainwalk (`SubType("Mountain")`),
    /// Forestwalk (`SubType("Forest")`).
    BasicType(SubType),
    /// Nonbasic landwalk: checks if defending player controls a land WITHOUT
    /// the `Basic` supertype (e.g., Dryad Sophisticate).
    Nonbasic,
}

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

/// CR 702.5a: Specifies what an Aura can legally target and enchant.
///
/// The Enchant keyword restricts both the target at cast time (CR 303.4a) and
/// what the Aura can be attached to on the battlefield (CR 704.5m).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EnchantTarget {
    /// "Enchant creature" (most common)
    Creature,
    /// "Enchant permanent" — any permanent type
    Permanent,
    /// "Enchant artifact"
    Artifact,
    /// "Enchant enchantment"
    Enchantment,
    /// "Enchant land"
    Land,
    /// "Enchant planeswalker"
    Planeswalker,
    /// "Enchant player" — can target and attach to players (CR 702.5d)
    Player,
    /// "Enchant creature or planeswalker"
    CreatureOrPlaneswalker,
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
    /// CR 702.5a: Enchant [object or player] — restricts what this Aura can target and enchant.
    Enchant(EnchantTarget),
    Equip,
    FirstStrike,
    Flash,
    Flying,
    Haste,
    Hexproof,
    Indestructible,
    Intimidate,
    /// CR 702.14a: Landwalk -- a creature with landwalk can't be blocked as long as the
    /// defending player controls at least one land matching the `LandwalkType` specification.
    /// CR 702.14b: Landwalk is an evasion ability.
    /// CR 702.14e: Multiple instances of the same landwalk are redundant (auto-deduped by OrdSet).
    Landwalk(LandwalkType),
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
    /// CR 702.21a: Ward [cost] — "Whenever this permanent becomes the target of a spell
    /// or ability an opponent controls, counter that spell or ability unless that player
    /// pays [cost]."
    ///
    /// Implemented as a triggered ability that generates a TriggeredAbilityDef at
    /// object-construction time (see `state/builder.rs` Ward->trigger translation).
    /// The `u32` encodes the generic mana cost (ward {N}).
    Ward(u32),
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
    /// CR 702.34: Flashback — card may be cast from the owner's graveyard by paying
    /// its flashback cost instead of its mana cost. If cast via flashback, the card
    /// is exiled instead of going anywhere else when it leaves the stack.
    ///
    /// This variant is a marker for quick presence-checking (`keywords.contains`).
    /// The flashback cost itself is stored in `AbilityDefinition::Flashback { cost }`.
    Flashback,
    /// CR 702.29: Cycling [cost] — activated ability from hand.
    /// "Cycling [cost]" means "[cost], Discard this card: Draw a card."
    /// Activate only from hand. The keyword exists in all zones (CR 702.29b).
    ///
    /// This variant is a marker for quick presence-checking (`keywords.contains`).
    /// The cycling cost itself is stored in `AbilityDefinition::Cycling { cost }`.
    Cycling,
    /// CR 702.52: Dredge N — if you would draw a card, you may instead mill N cards
    /// and return this card from your graveyard to your hand. Functions only while
    /// this card is in the graveyard. Requires >= N cards in library (CR 702.52b).
    Dredge(u32),
}
