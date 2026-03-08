//! Fundamental MTG type enums used throughout the engine.

use serde::{Deserialize, Serialize};

use super::game_object::ManaCost;

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

/// Which alternative casting cost was used to cast this spell.
/// Used by CastSpell.alt_cost and GameObject.cast_alt_cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AltCostKind {
    Flashback,
    Buyback,
    Escape,
    Evoke,
    Bestow,
    Miracle,
    Foretell,
    Overload,
    Retrace,
    JumpStart,
    Aftermath,
    Dash,
    Blitz,
    Plot,
    Impending,
    /// CR 702.119a: Emerge alternative cost — pay [emerge cost] and sacrifice a creature.
    Emerge,
    /// CR 702.137a: Spectacle alternative cost — pay spectacle cost instead of mana cost
    /// if an opponent lost life this turn.
    Spectacle,
    /// CR 702.117a: Surge alternative cost -- pay surge cost instead of mana cost
    /// if you or a teammate has cast another spell this turn.
    Surge,
    /// CR 702.148a: Cleave [cost] -- alternative cost. When paid, the spell's
    /// square-bracketed text is removed (modeled as Condition::WasCleaved branching).
    Cleave,
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
    /// CR 702.32a: Fade counters are placed on permanents with Fading N.
    ///
    /// Distinct from Time counters (used by Vanishing). Fading uses "fade counters"
    /// in its oracle text and in the activated abilities of cards like Parallax Wave.
    Fade,
    /// CR 702.24a: Age counters are placed on permanents with cumulative upkeep.
    /// One age counter is added at the beginning of each upkeep before the
    /// payment check. The total number of age counters determines the total cost.
    Age,
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

/// CR 702.24a: The cost paid for each age counter on a permanent with
/// cumulative upkeep. Multiplied by the number of age counters at
/// resolution time.
///
/// Most common variants are mana costs (Mystic Remora: {1}) and life
/// costs (Glacial Chasm: "Pay 2 life"). Complex action-based costs
/// (Herald of Leshrac) are not yet supported.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CumulativeUpkeepCost {
    /// Pay a mana cost for each age counter (most common).
    /// Example: Mystic Remora with "{1}" -- pay {1} per counter.
    Mana(ManaCost),
    /// Pay life for each age counter.
    /// Example: Glacial Chasm with "Pay 2 life" -- pay 2 life per counter.
    Life(u32),
}

/// CR 702.41a: Specifies the quality for affinity cost reduction.
///
/// "Affinity for [text]" means "This spell costs {1} less to cast for
/// each [text] you control." The quality determines which permanents
/// on the battlefield are counted for the reduction.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AffinityTarget {
    /// "Affinity for artifacts" — count artifacts you control (most common).
    Artifacts,
    /// "Affinity for [basic land type]" — count lands of that subtype you control.
    /// Example: "Affinity for Plains" counts all lands with the Plains subtype.
    BasicLandType(SubType),
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
    /// CR 702.124j: "Partner with [name]" represents two abilities:
    /// (1) Deck construction: allows this card and the named card as co-commanders,
    ///     provided each has a 'partner with [name]' ability naming the other.
    /// (2) ETB trigger: "When this permanent enters, target player may search their
    ///     library for a card named [name], reveal it, put it into their hand, then
    ///     shuffle."
    ///
    /// The `String` is the exact name of the partner card. The deck validation
    /// in `commander.rs` checks that both commanders have matching PartnerWith
    /// names. The ETB trigger is wired in `abilities.rs` via PendingTrigger.
    ///
    /// CR 702.124f: PartnerWith cannot combine with plain Partner or other
    /// partner variants.
    PartnerWith(String),
    /// CR 702.124i: "Partner--Friends forever" — both commanders must have this
    /// same ability. Structurally identical to plain Partner but distinct per
    /// CR 702.124f (different partner abilities cannot be combined).
    FriendsForever,
    /// CR 702.124k: "Choose a Background" — this commander pairs with a legendary
    /// Background enchantment card as the second commander. The Background does NOT
    /// need this keyword; it qualifies by being a legendary enchantment with the
    /// Background subtype. The Background enchantment is exempt from the normal
    /// "must be a legendary creature" commander type requirement.
    ChooseABackground,
    /// CR 702.124m: "Doctor's companion" — this commander pairs with a legendary
    /// Time Lord Doctor creature card that has no other creature types. The Doctor
    /// does NOT need this keyword; it qualifies by type alone.
    DoctorsCompanion,
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
    /// CR 702.51: Convoke — tap creatures to pay mana costs.
    /// "For each colored mana in this spell's total cost, you may tap an untapped
    /// creature of that color you control rather than pay that mana. For each generic
    /// mana in this spell's total cost, you may tap an untapped creature you control
    /// rather than pay that mana."
    /// CR 702.51d: Multiple instances are redundant.
    Convoke,
    /// CR 702.66: Delve — exile cards from your graveyard to pay for generic mana.
    /// "For each generic mana in this spell's total cost, you may exile a card
    /// from your graveyard rather than pay that mana."
    /// CR 702.66b: Not an additional or alternative cost; applies after total cost determined.
    /// CR 702.66c: Multiple instances are redundant.
    Delve,
    /// CR 702.33: Kicker [cost] — optional additional cost for enhanced effect.
    ///
    /// This variant is a marker for quick presence-checking (`keywords.contains`).
    /// The kicker cost itself is stored in `AbilityDefinition::Kicker { cost, is_multikicker }`.
    Kicker,
    /// CR 702.61: Split second — as long as this spell is on the stack,
    /// players can't cast other spells or activate abilities that aren't
    /// mana abilities.
    /// CR 702.61b: Mana abilities and special actions are still allowed.
    /// CR 702.61b: Triggered abilities still trigger and resolve normally.
    /// CR 702.61c: Multiple instances are redundant.
    SplitSecond,
    /// CR 702.83: Exalted — "Whenever a creature you control attacks alone,
    /// that creature gets +1/+1 until end of turn."
    ///
    /// Implemented as a triggered ability. builder.rs auto-generates a
    /// TriggeredAbilityDef from this keyword at object-construction time.
    /// Multiple instances on different permanents each trigger separately.
    Exalted,
    /// CR 702.86: Annihilator N — "Whenever this creature attacks, defending
    /// player sacrifices N permanents."
    ///
    /// Implemented as a triggered ability. builder.rs auto-generates a
    /// TriggeredAbilityDef from this keyword at object-construction time.
    /// Multiple instances each trigger separately (CR 702.86b).
    Annihilator(u32),
    /// CR 702.79: Persist — "When this permanent is put into a graveyard from
    /// the battlefield, if it had no -1/-1 counters on it, return it to the
    /// battlefield under its owner's control with a -1/-1 counter on it."
    ///
    /// Translated to a TriggeredAbilityDef at object-construction time in
    /// `state/builder.rs`. The trigger fires on SelfDies events; the
    /// intervening-if checks pre-death counters via the CreatureDied event.
    Persist,
    /// CR 702.93: Undying -- "When this permanent is put into a graveyard from
    /// the battlefield, if it had no +1/+1 counters on it, return it to the
    /// battlefield under its owner's control with a +1/+1 counter on it."
    ///
    /// Translated to a TriggeredAbilityDef at object-construction time in
    /// `state/builder.rs`. The trigger fires on SelfDies events; the
    /// intervening-if checks pre-death counters via the CreatureDied event.
    Undying,
    /// CR 702.73: Changeling -- "This object is every creature type."
    ///
    /// Characteristic-defining ability (CDA). Applied as a type-change in Layer 4
    /// before non-CDA effects (CR 613.3). Functions in all zones (CR 604.3).
    /// The full creature type list (CR 205.3m) is added to the object's subtypes
    /// inline in `calculate_characteristics` when this keyword is present.
    Changeling,
    /// CR 702.74: Evoke [cost] — alternative cost; sacrifice on ETB if evoked.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The evoke cost itself is stored in `AbilityDefinition::Evoke { cost }`.
    ///
    /// Casting with evoke pays the evoke cost instead of the mana cost (CR 118.9).
    /// When the permanent enters the battlefield, if its evoke cost was paid, its
    /// controller sacrifices it (CR 702.74a).
    Evoke,
    /// CR 702.122: Crew N — "Tap any number of other untapped creatures you control
    /// with total power N or greater: This permanent becomes an artifact creature
    /// until end of turn."
    ///
    /// Marker keyword for quick presence-checking. The crew cost (N) is the
    /// minimum total power of creatures that must be tapped.
    /// The actual command is `Command::CrewVehicle`, handled in `handle_crew_vehicle`.
    Crew(u32),
    /// CR 702.91: Battle Cry -- "Whenever this creature attacks, each other
    /// attacking creature gets +1/+0 until end of turn."
    ///
    /// Implemented as a triggered ability. builder.rs auto-generates a
    /// TriggeredAbilityDef from this keyword at object-construction time.
    /// Multiple instances each trigger separately (CR 702.91b).
    BattleCry,
    /// CR 702.135: Afterlife N -- "When this permanent is put into a graveyard
    /// from the battlefield, create N 1/1 white and black Spirit creature tokens
    /// with flying."
    ///
    /// Implemented as a triggered ability. builder.rs auto-generates a
    /// TriggeredAbilityDef from this keyword at object-construction time.
    /// Multiple instances each trigger separately (CR 702.135b).
    Afterlife(u32),
    /// CR 702.101: Extort -- "Whenever you cast a spell, you may pay {W/B}.
    /// If you do, each opponent loses 1 life and you gain life equal to the
    /// total life lost this way."
    ///
    /// Implemented as a triggered ability. builder.rs auto-generates a
    /// TriggeredAbilityDef from this keyword at object-construction time.
    /// Multiple instances each trigger separately (CR 702.101b).
    Extort,
    /// CR 702.126: Improvise -- tap artifacts to pay generic mana costs.
    /// "For each generic mana in this spell's total cost, you may tap an untapped
    /// artifact you control rather than pay that mana."
    /// CR 702.126b: Not an additional or alternative cost; applies after total cost determined.
    /// CR 702.126c: Multiple instances are redundant.
    Improvise,
    /// CR 702.41: Affinity for [quality] — this spell costs {1} less for each
    /// [quality] you control.
    ///
    /// Static ability that functions while the spell is on the stack (CR 702.41a).
    /// Automatically reduces generic mana in the total cost based on the count
    /// of qualifying permanents the caster controls. Multiple instances are
    /// cumulative (CR 702.41b).
    ///
    /// Unlike Convoke/Improvise/Delve, Affinity requires no player decisions —
    /// the engine counts matching permanents automatically.
    Affinity(AffinityTarget),
    /// CR 702.125: Undaunted -- "This spell costs {1} less to cast for each
    /// opponent you have."
    ///
    /// Static ability that functions while the spell is on the stack (CR 702.125a).
    /// Automatically reduces generic mana in the total cost based on the number
    /// of opponents the caster has. Multiple instances are cumulative (CR 702.125c).
    /// Players who have left the game are not counted (CR 702.125b).
    Undaunted,
    /// CR 702.105: Dethrone -- "Whenever this creature attacks the player with
    /// the most life or tied for most life, put a +1/+1 counter on this creature."
    ///
    /// Implemented as a triggered ability. builder.rs auto-generates a
    /// TriggeredAbilityDef from this keyword at object-construction time.
    /// Multiple instances each trigger separately (CR 702.105b).
    Dethrone,
    /// CR 702.103: Bestow [cost] -- alternative cost; becomes Aura with enchant creature.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The bestow cost itself is stored in `AbilityDefinition::Bestow { cost }`.
    ///
    /// When cast bestowed (CR 702.103b): spell becomes an Aura enchantment, gains
    /// enchant creature, loses creature type. When unattached (CR 702.103f): ceases
    /// to be bestowed, reverts to enchantment creature.
    Bestow,
    /// CR 702.36: Fear -- evasion ability.
    /// "A creature with fear can't be blocked except by artifact creatures
    /// and/or black creatures." (CR 702.36b)
    /// Multiple instances are redundant (CR 702.36c).
    Fear,
    /// CR 702.92: Living Weapon -- "When this Equipment enters, create a 0/0
    /// black Phyrexian Germ creature token, then attach this Equipment to it."
    ///
    /// Implemented as a triggered ability. builder.rs auto-generates a
    /// TriggeredAbilityDef from this keyword at object-construction time.
    /// The trigger fires on SelfEntersBattlefield. The effect creates a Germ
    /// token and attaches the source Equipment to it atomically.
    LivingWeapon,
    /// CR 702.35: Madness [cost] -- when discarded, exile instead of graveyard;
    /// owner may cast for madness cost from exile.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The madness cost itself is stored in `AbilityDefinition::Madness { cost }`.
    ///
    /// Two sub-abilities: (1) static replacement on discard (hand -> exile instead
    /// of graveyard), (2) triggered ability "when exiled this way, may cast for
    /// [cost] or put into graveyard."
    /// Casting ignores timing restrictions (sorceries can be cast at instant speed
    /// via madness, per CR 702.35 ruling).
    Madness,
    /// CR 702.94: Miracle [cost] -- static ability linked to triggered ability.
    /// "You may reveal this card from your hand as you draw it if it's the first
    /// card you've drawn this turn. When you reveal this card this way, you may
    /// cast it by paying [cost] rather than its mana cost."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The miracle cost itself is stored in `AbilityDefinition::Miracle { cost }`.
    ///
    /// The card is drawn normally first; then the player may optionally reveal
    /// it and trigger the miracle ability. Casting ignores timing restrictions
    /// (sorceries can be cast at instant speed via miracle, per CR 702.94a ruling).
    Miracle,
    /// CR 702.138: Escape [cost] -- static ability from graveyard.
    /// "You may cast this card from your graveyard by paying [cost] rather
    /// than paying its mana cost."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The escape cost (mana + exile count) is stored in
    /// `AbilityDefinition::Escape { cost, exile_count }`.
    ///
    /// Unlike Flashback, Escape does NOT exile the card on resolution.
    /// The spell resolves normally (permanent to battlefield, instant/sorcery
    /// to graveyard). The permanent tracks `was_escaped` for "escapes with"
    /// abilities (CR 702.138b-d).
    Escape,
    /// CR 702.143: Foretell [cost] -- special action from hand; cast from exile for foretell cost.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The foretell cost itself is stored in `AbilityDefinition::Foretell { cost }`.
    ///
    /// During the owner's turn, they may pay {2} and exile this card face down
    /// (special action, CR 116.2h). On a later turn, they may cast it for its
    /// foretell cost (alternative cost, CR 118.9).
    Foretell,
    /// CR 702.84: Unearth [cost] -- activated ability from graveyard.
    /// "[Cost]: Return this card from your graveyard to the battlefield.
    /// It gains haste. Exile it at the beginning of the next end step.
    /// If it would leave the battlefield, exile it instead of putting it
    /// anywhere else. Activate only as a sorcery."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The unearth cost itself is stored in `AbilityDefinition::Unearth { cost }`.
    Unearth,
    /// CR 702.136: Riot -- "You may have this permanent enter with an additional
    /// +1/+1 counter on it. If you don't, it gains haste."
    ///
    /// Replacement effect (CR 614.1c). Applied inline at the ETB site in
    /// `resolution.rs` (like Escape's counter placement). Each instance
    /// of Riot on a permanent generates a separate choice (CR 702.136b).
    ///
    /// Default: always chooses +1/+1 counter for deterministic testing.
    /// Future: player-interactive choice via a new Command variant.
    Riot,
    /// CR 702.110: Exploit -- "When this creature enters, you may sacrifice a creature."
    ///
    /// Triggered ability keyword. When a creature with exploit enters the battlefield,
    /// the controller may sacrifice a creature (CR 702.110a). A creature with exploit
    /// "exploits a creature" when the sacrifice happens (CR 702.110b).
    ///
    /// Marker for quick presence-checking (`keywords.contains`). No associated cost.
    /// Each instance triggers separately.
    Exploit,
    /// CR 702.80: Wither -- "Damage dealt to a creature by a source with wither isn't
    /// marked on that creature. Rather, it causes that many -1/-1 counters to be put
    /// on that creature."
    ///
    /// Static ability. Functions from any zone (CR 702.80c). Multiple instances are
    /// redundant (CR 702.80d). Modifies both combat and non-combat damage to creatures.
    Wither,
    /// CR 702.43: Modular N -- "This permanent enters with N +1/+1 counters on it"
    /// and "When this permanent is put into a graveyard from the battlefield, you may
    /// put a +1/+1 counter on target artifact creature for each +1/+1 counter on
    /// this permanent."
    ///
    /// Represents both a static ability (ETB counters) and a triggered ability (dies).
    /// Each instance works separately (CR 702.43b).
    Modular(u32),
    /// CR 702.100: Evolve -- "Whenever a creature you control enters, if that creature's
    /// power is greater than this creature's power and/or that creature's toughness is
    /// greater than this creature's toughness, put a +1/+1 counter on this creature."
    ///
    /// Triggered ability with intervening-if. Each instance triggers separately
    /// (CR 702.100d). A creature "evolves" when counters are placed this way (CR 702.100b).
    Evolve,
    /// CR 702.27: Buyback [cost] -- "You may pay an additional [cost] as you cast this
    /// spell" and "If the buyback cost was paid, put this spell into its owner's hand
    /// instead of into that player's graveyard as it resolves."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The buyback cost itself is stored in `AbilityDefinition::Buyback { cost }`.
    Buyback,
    /// CR 702.131: Ascend -- "If you control ten or more permanents and you don't have
    /// the city's blessing, you get the city's blessing for the rest of the game."
    ///
    /// On instants/sorceries: spell ability checked on resolution.
    /// On permanents: static ability checked continuously.
    /// The city's blessing is a designation with no inherent rules meaning (CR 702.131c).
    Ascend,
    /// CR 702.90: Infect -- "Damage dealt to a creature by a source with infect isn't
    /// marked on that creature. Rather, it causes that many -1/-1 counters to be put
    /// on that creature. Damage dealt to a player by a source with infect doesn't cause
    /// that player to lose life. Rather, it causes that player to get that many poison
    /// counters."
    ///
    /// Static ability. Functions from any zone (CR 702.90e). Multiple instances are
    /// redundant (CR 702.90f). Shares creature-damage mechanic with Wither (CR 120.3d);
    /// additionally replaces player damage with poison counters (CR 120.3b).
    Infect,
    /// CR 702.116: Myriad -- "Whenever this creature attacks, for each opponent other
    /// than defending player, you may create a token that's a copy of this creature
    /// that's tapped and attacking that player or a planeswalker they control. If one
    /// or more tokens are created this way, exile the tokens at end of combat."
    ///
    /// Triggered ability. builder.rs auto-generates a TriggeredAbilityDef from this
    /// keyword at object-construction time. Multiple instances each trigger separately
    /// (CR 702.116b). Tokens created are tagged `myriad_exile_at_eoc = true` and exiled
    /// by the end_combat() turn-based action.
    Myriad,
    /// CR 702.62: Suspend N -- [cost]. Three abilities:
    /// (1) Static ability while in hand: pay [cost] and exile from hand with N time
    ///     counters (special action, CR 116.2f).
    /// (2) Triggered ability in exile: at the beginning of the owner's upkeep, if
    ///     this card is suspended, remove a time counter from it.
    /// (3) Triggered ability in exile: when the last time counter is removed, if
    ///     this card is exiled, you may cast it without paying its mana cost. If a
    ///     creature spell is cast this way, it gains haste until you lose control.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The suspend cost and counter count are stored in
    /// `AbilityDefinition::Suspend { cost, time_counters }`.
    Suspend,
    /// CR 702.75: Hideaway N -- "When this permanent enters, look at the top N
    /// cards of your library. Exile one of them face down and put the rest on
    /// the bottom of your library in a random order."
    ///
    /// Triggered ability keyword. `abilities.rs` detects this keyword at trigger-
    /// scan time (ETB event) and queues a `PendingTrigger` with
    /// `is_hideaway_trigger = true`.  Resolution creates a
    /// `StackObjectKind::HideawayTrigger` and executes the look/exile/put-back
    /// sequence in `resolution.rs`.
    ///
    /// The N parameter specifies how many cards to look at.
    ///
    /// The "play the exiled card" part is card-specific and NOT derived from
    /// this keyword — each card defines its own activated ability with a
    /// condition and `Effect::PlayExiledCard`.
    Hideaway(u32),
    /// CR 701.46: Adapt N -- keyword action used as activated ability.
    /// "Adapt N" means "If this permanent has no +1/+1 counters on it, put N
    /// +1/+1 counters on it."
    ///
    /// Marker for quick presence-checking. The activation cost and conditional
    /// effect are stored in `AbilityDefinition::Activated` on the card definition.
    /// The u32 parameter is the N value (number of +1/+1 counters to add).
    ///
    /// Always used as an activated ability on the card (instant speed).
    /// The condition check (no +1/+1 counters) happens at resolution time,
    /// not at activation time (ruling 2019-01-25).
    Adapt(u32),
    /// CR 702.28: Shadow -- evasion ability.
    /// "A creature with shadow can't be blocked by creatures without shadow,
    /// and a creature without shadow can't be blocked by creatures with shadow."
    /// CR 702.28c: Multiple instances are redundant (auto-deduped by OrdSet).
    Shadow,
    /// CR 702.96: Overload [cost] -- alternative cost; replaces "target" with "each".
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The overload cost itself is stored in `AbilityDefinition::Overload { cost }`.
    ///
    /// When cast with overload: the spell has no targets and affects all valid
    /// objects instead of one target. Implements CR 702.96a/b via conditional
    /// effect dispatch (like Kicker), not literal text replacement.
    Overload,
    /// CR 702.31: Horsemanship -- evasion ability (unidirectional).
    /// "A creature with horsemanship can't be blocked by creatures without horsemanship.
    /// A creature with horsemanship can block a creature with or without horsemanship."
    /// CR 702.31c: Multiple instances are redundant (auto-deduped by OrdSet).
    Horsemanship,
    /// CR 702.118: Skulk -- evasion ability (one-directional, power-based).
    /// "A creature with skulk can't be blocked by creatures with greater power."
    /// CR 702.118c: Multiple instances are redundant (auto-deduped by OrdSet).
    Skulk,
    /// CR 702.114: Devoid -- "This object is colorless."
    ///
    /// Characteristic-defining ability (CDA). Applied as a color-change in Layer 5
    /// (ColorChange) before non-CDA effects (CR 613.3). Functions in all zones
    /// (CR 604.3). Clears the object's colors set, making it colorless regardless
    /// of its mana cost (CR 202.2).
    Devoid,
    /// CR 702.147: Decayed -- static ability + triggered ability.
    /// "This creature can't block" (static) and "When this creature attacks,
    /// sacrifice it at end of combat" (triggered).
    ///
    /// The blocking restriction is enforced in `rules/combat.rs`.
    /// The EOC sacrifice uses a tag-on-object pattern (like Myriad):
    /// `decayed_sacrifice_at_eoc` is set on the creature when it attacks,
    /// and `end_combat()` in `turn_actions.rs` sacrifices all tagged creatures.
    ///
    /// Ruling 2021-09-24: "Once a creature with decayed attacks, it will be
    /// sacrificed at end of combat, even if it no longer has decayed at that time."
    /// CR 702.147a: Multiple instances are redundant.
    Decayed,
    /// CR 702.115: Ingest -- triggered ability.
    /// "Whenever this creature deals combat damage to a player, that player
    /// exiles the top card of their library."
    /// CR 702.115b: Multiple instances trigger separately.
    ///
    /// Ruling 2015-08-25: "The card exiled by the ingest ability is exiled
    /// face up." (engine default is face-up; no special handling needed)
    /// Ruling 2015-08-25: "If the player has no cards in their library when
    /// the ingest ability resolves, nothing happens."
    Ingest,
    /// CR 702.25: Flanking -- triggered ability.
    /// "Whenever this creature becomes blocked by a creature without flanking,
    /// the blocking creature gets -1/-1 until end of turn."
    /// CR 702.25b: Multiple instances trigger separately.
    Flanking,
    /// CR 702.45: Bushido N -- triggered ability.
    /// "Whenever this creature blocks or becomes blocked, it gets +N/+N
    /// until end of turn."
    /// CR 702.45b: Multiple instances trigger separately.
    Bushido(u32),
    /// CR 702.23: Rampage N -- triggered ability.
    /// "Whenever this creature becomes blocked, it gets +N/+N until end of
    /// turn for each creature blocking it beyond the first."
    ///
    /// CR 702.23b: Bonus calculated once at resolution time (not trigger time).
    /// Adding/removing blockers after resolution does not change the bonus.
    /// CR 702.23c: Multiple instances trigger separately.
    ///
    /// Implemented via a custom `StackObjectKind::RampageTrigger` so that
    /// the blocker count can be queried from `state.combat` at resolution time.
    Rampage(u32),
    /// CR 702.39: Provoke -- triggered ability.
    /// "Whenever this creature attacks, you may have target creature defending
    /// player controls untap and block this creature this combat if able."
    ///
    /// CR 702.39b: Multiple instances of provoke each trigger separately.
    ///
    /// Implemented via a custom `StackObjectKind::ProvokeTrigger`. At trigger
    /// collection time (AttackersDeclared handler), a target creature controlled
    /// by the defending player is selected deterministically (first by ObjectId
    /// order). The trigger is not placed on the stack if no valid target exists
    /// (CR 603.3d). On resolution: untap the provoked creature, then add a
    /// forced-block requirement to `CombatState::forced_blocks` (enforced in
    /// `handle_declare_blockers` per CR 509.1c).
    Provoke,
    /// CR 702.130: Afflict N -- triggered ability.
    /// "Whenever this creature becomes blocked, defending player loses N life."
    /// CR 702.130b: Multiple instances trigger separately.
    ///
    /// Implemented as a triggered ability. builder.rs auto-generates a
    /// TriggeredAbilityDef from this keyword at object-construction time.
    /// The trigger fires on SelfBecomesBlocked events; the defending player
    /// is resolved at flush time via PendingTrigger.defending_player_id.
    Afflict(u32),
    /// CR 702.112: Renown N -- triggered ability.
    /// "When this creature deals combat damage to a player, if it isn't renowned,
    /// put N +1/+1 counters on it and it becomes renowned."
    /// CR 702.112c: Multiple instances trigger separately.
    ///
    /// Renowned is a designation tracked as `is_renowned` on `GameObject`
    /// (CR 702.112b). Not a copiable value. Resets on zone change (CR 400.7).
    /// Implemented via custom `StackObjectKind::RenownTrigger` with intervening-if
    /// checked at both trigger time and resolution time (CR 603.4).
    Renown(u32),
    /// CR 702.149: Training -- "Whenever this creature and at least one other creature
    /// with power greater than this creature's power attack, put a +1/+1 counter on
    /// this creature."
    ///
    /// Implemented as a triggered ability. builder.rs auto-generates a
    /// TriggeredAbilityDef from this keyword at object-construction time.
    /// Multiple instances each trigger separately (CR 702.149b).
    Training,
    /// CR 702.121: Melee -- triggered ability.
    /// "Whenever this creature attacks, it gets +1/+1 until end of turn for
    /// each opponent you attacked with a creature this combat."
    ///
    /// CR 702.121b: Multiple instances trigger separately.
    ///
    /// Implemented via a custom `StackObjectKind::MeleeTrigger` because the
    /// bonus is computed at resolution time from `state.combat` (ruling
    /// 2016-08-23: "You determine the size of the bonus as the melee ability
    /// resolves"). The trigger fires on `TriggerEvent::SelfAttacks`.
    Melee,
    /// CR 702.70: Poisonous N -- triggered ability.
    /// "Whenever this creature deals combat damage to a player, that player gets N
    /// poison counters."
    /// CR 702.70b: Multiple instances trigger separately.
    ///
    /// Unlike Infect (replacement effect converting damage to poison counters),
    /// Poisonous is a triggered ability that adds poison counters IN ADDITION to
    /// the normal combat damage. The N value is fixed, not based on damage dealt.
    /// Reuses existing poison counter infrastructure (PlayerState.poison_counters,
    /// PoisonCountersGiven event, 10-poison SBA CR 704.5c).
    Poisonous(u32),
    /// CR 702.164: Toxic N -- static ability.
    /// "Combat damage dealt to a player by a creature with toxic causes that
    /// creature's controller to give the player a number of poison counters
    /// equal to that creature's total toxic value, in addition to the damage's
    /// other results."
    ///
    /// CR 702.164b: Multiple instances are cumulative -- total toxic value is
    /// the sum of all N values.
    /// CR 120.3g: Only combat damage to a player; does not apply to creatures,
    /// planeswalkers, or non-combat damage.
    Toxic(u32),
    /// CR 702.154: Enlist -- "As this creature attacks, you may tap up to one
    /// untapped creature you control that you didn't choose to attack with and
    /// that either has haste or has been under your control continuously since
    /// this turn began. When you do, this creature gets +X/+0 until end of turn,
    /// where X is the tapped creature's power."
    ///
    /// Static ability: optional cost to attack (CR 508.1g). Expressed as an
    /// enlist_choices field on the DeclareAttackers command.
    /// Triggered ability: linked to the static ability (CR 607.2h). Goes on
    /// the stack after attackers are declared. Resolves to +X/+0.
    /// Multiple instances function independently (CR 702.154d).
    Enlist,
    /// CR 702.49: Ninjutsu -- activated ability from hand.
    /// "Pay [cost], Reveal this card from your hand, Return an unblocked
    /// attacking creature you control to its owner's hand: Put this card onto
    /// the battlefield from your hand tapped and attacking."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The ninjutsu cost is stored in `AbilityDefinition::Ninjutsu { cost }`.
    Ninjutsu,
    /// CR 702.49d: Commander Ninjutsu -- variant that also works from the
    /// command zone. "Pay [cost], Reveal this card from your hand or from the
    /// command zone, Return an unblocked attacking creature you control to its
    /// owner's hand: Put this card onto the battlefield tapped and attacking."
    ///
    /// Bypasses commander tax entirely (it is an activated ability, not a
    /// spell cast). See ruling 2020-11-10.
    CommanderNinjutsu,
    /// CR 702.81: Retrace -- card may be cast from the owner's graveyard
    /// by discarding a land card as an additional cost (CR 118.8).
    /// Unlike Flashback, the card returns to the graveyard on resolution
    /// (not exiled). The normal mana cost is paid (not an alternative cost).
    ///
    /// This variant is a marker for quick presence-checking (`keywords.contains`).
    /// No `AbilityDefinition::Retrace` is needed because there is no separate
    /// cost to store -- the card uses its normal mana cost.
    Retrace,
    /// CR 702.133: Jump-start -- cast from graveyard by paying mana cost + discarding a card.
    /// Jump-start is NOT an alternative cost: the card's normal mana cost is paid PLUS the
    /// player discards any card from hand as an additional cost (CR 601.2f-h).
    /// Unlike Retrace, the discarded card may be any card (not just a land).
    /// Unlike Flashback, alternative costs can still be applied on top of jump-start (since
    /// jump-start is additional, not alternative -- 2018-10-05 ruling on Radical Idea).
    /// The card is exiled on departure (resolves, countered, or fizzles) -- same as Flashback.
    ///
    /// This variant is a marker for quick presence-checking (`keywords.contains`).
    /// No `AbilityDefinition::JumpStart` is needed because there is no per-card cost --
    /// jump-start always uses the card's printed mana cost.
    JumpStart,
    /// CR 702.127: Aftermath -- the second half of a split card can only be cast from
    /// the graveyard. Represents three static abilities:
    /// (a) You may cast this half from your graveyard.
    /// (b) This half can't be cast from any zone other than a graveyard.
    /// (c) If cast from a graveyard, exile it instead of putting it anywhere else when
    ///     it leaves the stack.
    ///
    /// This variant is a marker for quick presence-checking (`keywords.contains`).
    /// The aftermath half's spell data (name, cost, card_type, effect, targets) is stored
    /// in `AbilityDefinition::Aftermath`.
    Aftermath,
    /// CR 702.128: Embalm [cost] -- activated ability from graveyard.
    /// "[Cost], Exile this card from your graveyard: Create a token that's a copy
    /// of this card, except it's white, it has no mana cost, and it's a Zombie in
    /// addition to its other types. Activate only as a sorcery."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The embalm cost itself is stored in `AbilityDefinition::Embalm { cost }`.
    Embalm,
    /// CR 702.129: Eternalize [cost] -- activated ability from graveyard.
    /// "[Cost], Exile this card from your graveyard: Create a token that's a copy
    /// of this card, except it's black, it's 4/4, it has no mana cost, and it's a
    /// Zombie in addition to its other types. Activate only as a sorcery."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The eternalize cost itself is stored in `AbilityDefinition::Eternalize { cost }`.
    Eternalize,
    /// CR 702.141: Encore [cost] -- activated ability from graveyard.
    /// "[Cost], Exile this card from your graveyard: For each opponent, create
    /// a token that's a copy of this card that attacks that opponent this turn
    /// if able. The tokens gain haste. Sacrifice them at the beginning of the
    /// next end step. Activate only as a sorcery."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The encore cost itself is stored in `AbilityDefinition::Encore { cost }`.
    Encore,
    /// CR 702.109: Dash [cost] -- alternative cost granting haste and
    /// return-to-hand at end step.
    ///
    /// "You may cast this card by paying [cost] rather than its mana cost.
    /// If you do, it gains haste, and it's returned from the battlefield to
    /// its owner's hand at the beginning of the next end step."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The dash cost itself is stored in `AbilityDefinition::Dash { cost }`.
    Dash,
    /// CR 702.152: Blitz [cost] -- alternative cost granting haste,
    /// sacrifice at end step, and draw-a-card on death.
    ///
    /// "You may cast this card by paying [cost] rather than its mana cost.
    /// If you do, the permanent gains haste, is sacrificed at the beginning
    /// of the next end step, and gains 'When this permanent is put into a
    /// graveyard from the battlefield, draw a card.'"
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The blitz cost itself is stored in `AbilityDefinition::Blitz { cost }`.
    Blitz,
    /// CR 702.170: Plot [cost] -- special action from hand; cast from exile for free.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The plot cost itself is stored in `AbilityDefinition::Plot { cost }`.
    ///
    /// During the owner's main phase with empty stack, they may pay the plot cost
    /// and exile this card face up (special action, CR 116.2k). On a later turn,
    /// during their main phase with empty stack, they may cast it without paying
    /// its mana cost (alternative cost, CR 702.170d).
    Plot,
    /// CR 702.160: Prototype [mana cost] -- [power]/[toughness].
    ///
    /// Static ability on prototype cards. When casting, the player may choose to
    /// cast the card "prototyped" using the alternative mana cost, power, and
    /// toughness (CR 718.3).
    ///
    /// IMPORTANT: Prototype is NOT an alternative cost (CR 118.9, ruling 2022-10-14).
    /// It can combine with alternative costs like Flashback or Escape.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The prototype data (cost, power, toughness) is stored in `AbilityDefinition::Prototype`.
    Prototype,
    /// CR 702.176: Impending N--[cost]. Alternative cost; enters with N time counters;
    /// not a creature while it has time counters and was cast for impending cost;
    /// remove a time counter at beginning of controller's end step.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The impending cost and counter count are stored in
    /// `AbilityDefinition::Impending { cost, count }`.
    Impending,
    /// CR 702.166: Bargain -- optional additional cost: sacrifice an artifact,
    /// enchantment, or token.
    ///
    /// "As an additional cost to cast this spell, you may sacrifice an artifact,
    /// enchantment, or token."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// No `AbilityDefinition::Bargain` needed -- bargain has no per-card cost
    /// to store. The sacrifice target is provided via `CastSpell.bargain_sacrifice`.
    ///
    /// Cards check "if this spell was bargained" via `Condition::WasBargained`.
    /// CR 702.166c: Multiple instances are redundant.
    Bargain,
    /// CR 702.119: Emerge [cost] -- alternative cost: pay [cost] and sacrifice a creature.
    /// The total cost is reduced by the sacrificed creature's mana value.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The emerge cost is stored in `AbilityDefinition::Emerge { cost }`.
    /// The sacrifice target is provided via `CastSpell.emerge_sacrifice`.
    Emerge,
    /// CR 702.137a: Spectacle [cost] -- alternative cost if an opponent lost life this turn.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The spectacle cost is stored in `AbilityDefinition::Spectacle { cost }`.
    Spectacle,
    /// CR 702.117a: Surge [cost] -- alternative cost if you or a teammate cast another spell this turn.
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The surge cost is stored in `AbilityDefinition::Surge { cost }`.
    Surge,
    /// CR 702.153a: Casualty N -- optional additional cost: sacrifice a creature with power N or
    /// greater. If paid, when you cast this spell, copy it.
    ///
    /// "As an additional cost to cast this spell, you may sacrifice a creature with power N or
    /// greater." and "When you cast this spell, if a casualty cost was paid for it, copy it."
    ///
    /// The `u32` value is N (the minimum power of the sacrificed creature).
    /// The sacrifice target is provided via `CastSpell.casualty_sacrifice`.
    ///
    /// CR 702.153b: Multiple instances are each paid separately.
    Casualty(u32),
    /// CR 702.132: Assist -- another player may pay generic mana in the spell's total cost.
    /// "If the total cost to cast a spell with assist includes a generic mana component,
    /// before you activate mana abilities while casting it, you may choose another player.
    /// [...] the player you chose may pay for any amount of the generic mana in the spell's
    /// total cost."
    ///
    /// Static ability. Marker for quick presence-checking (`keywords.contains`).
    /// No `AbilityDefinition::Assist` needed -- assist has no per-card data to store.
    /// The assisting player and amount are provided via `CastSpell.assist_player` and
    /// `CastSpell.assist_amount`.
    /// CR 702.132a: Multiple instances are redundant.
    Assist,
    /// CR 702.56a: Replicate [cost] -- optional additional cost: pay [cost] any
    /// number of times when casting this spell. When you cast this spell, if a
    /// replicate cost was paid, copy it for each time its replicate cost was paid.
    /// If the spell has any targets, you may choose new targets for any of the copies.
    ///
    /// Static + triggered ability marker. The replicate cost is stored in
    /// `AbilityDefinition::Replicate { cost }`.
    /// CR 702.56b: Multiple instances trigger independently.
    Replicate,
    /// CR 702.69a: Gravestorm -- "When you cast this spell, copy it for each
    /// permanent that was put into a graveyard from the battlefield this turn.
    /// If the spell has any targets, you may choose new targets for any of the copies."
    ///
    /// Triggered ability. The count is captured at trigger-creation time (at cast,
    /// not resolution) from `GameState::permanents_put_into_graveyard_this_turn`.
    /// CR 702.69b: Multiple instances trigger independently.
    Gravestorm,
    /// CR 702.148a: Cleave [cost] -- alternative cost. When paid, the spell's
    /// square-bracketed text is removed -- modeled as conditional effect dispatch
    /// via `Condition::WasCleaved`. Cards with cleave should also include
    /// `AbilityDefinition::Cleave { cost }` for the cost lookup.
    Cleave,
    /// CR 702.47a: Splice onto [subtype] [cost] -- static ability that functions
    /// while a card is in the player's hand. When casting a spell of the matching
    /// subtype (e.g., Arcane), the player may reveal this card from hand and pay
    /// the splice cost as an additional cost. The spell gains this card's rules
    /// text, but the spliced card stays in the player's hand.
    ///
    /// Static ability. Marker for quick presence-checking (`keywords.contains`).
    /// The splice cost, subtype qualifier, and effect are stored in
    /// `AbilityDefinition::Splice { cost, onto_subtype, effect }`.
    /// CR 702.47b: Multiple splice abilities are evaluated independently.
    Splice,
    /// CR 702.42a: Entwine [cost] -- optional additional cost on modal spells.
    /// When paid, the caster chooses all modes of this modal spell instead of just
    /// the number specified. The entwine cost itself is stored in
    /// `AbilityDefinition::Entwine { cost }`.
    ///
    /// Static ability. Marker for quick presence-checking (`keywords.contains`).
    Entwine,
    /// CR 702.120a: Escalate [cost] -- optional additional cost on modal spells.
    /// For each mode chosen beyond the first, the escalate cost is paid once.
    /// The escalate cost itself is stored in `AbilityDefinition::Escalate { cost }`.
    ///
    /// Static ability. Marker for quick presence-checking (`keywords.contains`).
    /// Unlike Entwine (pay once, choose ALL modes), Escalate scales with the number
    /// of extra modes chosen. `escalate_modes` in `Command::CastSpell` tracks
    /// how many additional modes beyond the first are chosen.
    Escalate,
    /// CR 702.59a: Recover [cost] -- triggered ability from graveyard.
    ///
    /// "When a creature is put into your graveyard from the battlefield, you may pay
    /// [cost]. If you do, return this card from your graveyard to your hand. Otherwise,
    /// exile this card."
    ///
    /// Static marker for quick presence-checking. The actual cost is stored in
    /// `AbilityDefinition::Recover { cost }`. Discriminant 116.
    Recover,
    /// CR 702.63a: Vanishing N -- "This permanent enters with N time counters on it,"
    /// "At the beginning of your upkeep, if this permanent has a time counter on it,
    /// remove a time counter from it," and "When the last time counter is removed from
    /// this permanent, sacrifice it."
    ///
    /// CR 702.63b: Vanishing without a number -- only the upkeep counter-removal and
    /// sacrifice triggered abilities; no ETB counter placement. Represented as `Vanishing(0)`.
    ///
    /// CR 702.63c: Each instance works separately.
    Vanishing(u32),
    /// CR 702.32a: Fading N -- "This permanent enters with N fade counters on it"
    /// and "At the beginning of your upkeep, remove a fade counter from this
    /// permanent. If you can't, sacrifice the permanent."
    ///
    /// Unlike Vanishing, Fading always has a number (no "Fading without a number").
    /// The upkeep trigger is a SINGLE trigger that handles both counter removal and
    /// sacrifice (if removal fails).
    Fading(u32),
    /// CR 702.30a: Echo [cost] -- "At the beginning of your upkeep, if this permanent
    /// came under your control since the beginning of your last upkeep, sacrifice it
    /// unless you pay [cost]."
    ///
    /// The ManaCost parameter is the echo cost. For most Urza block cards, this equals
    /// the card's mana cost (CR 702.30b). Later cards may have different echo costs.
    ///
    /// CR 702.30a: Each instance triggers separately (standard triggered ability rule).
    Echo(ManaCost),
    /// CR 702.24a: Cumulative upkeep [cost] -- "At the beginning of your upkeep,
    /// if this permanent is on the battlefield, put an age counter on this
    /// permanent. Then you may pay [cost] for each age counter on it. If you
    /// don't, sacrifice it."
    ///
    /// The CumulativeUpkeepCost parameter is the per-counter cost.
    /// CR 702.24b: Each instance triggers separately, but all share age counters.
    CumulativeUpkeep(CumulativeUpkeepCost),
    /// CR 702.57a: Forecast -- activated ability from hand during owner's upkeep.
    /// Static marker for quick presence-checking (`keywords.contains`).
    /// The forecast cost and effect are stored in `AbilityDefinition::Forecast`.
    ///
    /// Discriminant 117.
    Forecast,
    /// CR 702.26a: Phasing -- static ability modifying untap step rules.
    /// "During each player's untap step, before the active player untaps permanents,
    /// all phased-in permanents with phasing that player controls phase out.
    /// Simultaneously, all phased-out permanents that had phased out under that
    /// player's control phase in."
    ///
    /// CR 702.26p: Multiple instances are redundant (auto-deduped by OrdSet).
    /// Phasing does NOT use the stack (CR 702.26d -- no zone change, no triggers).
    ///
    /// Infrastructure: `ObjectStatus::phased_out` (already exists at game_object.rs).
    /// Controller tracking: `GameObject::phased_out_controller` (added for phase-in routing).
    ///
    /// Discriminant 118.
    Phasing,
    /// CR 702.58: Graft N -- "This permanent enters with N +1/+1 counters on it"
    /// and "Whenever another creature enters, if this permanent has a +1/+1
    /// counter on it, you may move a +1/+1 counter from this permanent onto
    /// that creature."
    ///
    /// Represents both a static ability (ETB counters) and a triggered ability
    /// (counter transfer). Each instance works separately (CR 702.58b).
    ///
    /// Discriminant 119.
    Graft(u32),
    /// CR 702.97: Scavenge [cost] -- activated ability from graveyard.
    /// "[Cost], Exile this card from your graveyard: Put a number of +1/+1
    /// counters equal to the power of the card you exiled on target creature.
    /// Activate only as a sorcery."
    ///
    /// Discriminant 120.
    Scavenge,
    /// CR 702.107: Outlast [cost] -- activated ability on the battlefield.
    /// "[Cost], {T}: Put a +1/+1 counter on this creature. Activate only as a sorcery."
    ///
    /// Discriminant 121.
    Outlast,
    /// CR 702.38: Amplify N -- "As this object enters, reveal any number of cards from
    /// your hand that share a creature type with it. This permanent enters with N +1/+1
    /// counters on it for each card revealed this way."
    ///
    /// Static ability / ETB replacement effect (CR 614.1c). Multiple instances work
    /// separately (CR 702.38b).
    ///
    /// Discriminant 122.
    Amplify(u32),
    /// CR 702.54: Bloodthirst N -- "If an opponent was dealt damage this turn,
    /// this permanent enters with N +1/+1 counters on it."
    ///
    /// Static ability / ETB replacement effect (CR 614.1c). Multiple instances
    /// work separately (CR 702.54c).
    ///
    /// Discriminant 123.
    Bloodthirst(u32),
    /// CR 702.82: Devour N -- "As this object enters, you may sacrifice any number of
    /// creatures. This permanent enters with N +1/+1 counters on it for each creature
    /// sacrificed this way."
    ///
    /// Static ability / ETB replacement effect (CR 614.1c). Multiple instances work
    /// separately (CR 702.82c).
    ///
    /// Discriminant 124.
    Devour(u32),
    /// CR 702.165: Backup N -- "When this creature enters, put N +1/+1 counters
    /// on target creature. If that's another creature, it also gains the non-backup
    /// abilities of this creature printed below this one until end of turn."
    ///
    /// Triggered ability (CR 702.165a). The N value is the number of +1/+1 counters.
    /// Multiple instances each trigger separately.
    ///
    /// Discriminant 125.
    Backup(u32),
    /// CR 702.72: Champion an [object] -- "When this permanent enters, sacrifice
    /// it unless you exile another [object] you control. When this permanent
    /// leaves the battlefield, return the exiled card to the battlefield under
    /// its owner's control."
    ///
    /// Two linked triggered abilities (CR 607.2k). The champion filter
    /// (creature, Faerie, etc.) is carried by `ChampionFilter` in the
    /// `PendingTriggerKind::ChampionETB` trigger and looked up from the
    /// card registry at trigger time.
    ///
    /// Discriminant 126.
    Champion,

    /// CR 702.89a: Umbra armor -- "If enchanted permanent would be destroyed,
    /// instead remove all damage marked on it and destroy this Aura."
    ///
    /// Static ability on Auras. Creates a continuous replacement effect while
    /// the Aura is on the battlefield. Unlike regeneration, the protected
    /// permanent is NOT tapped and NOT removed from combat.
    ///
    /// Discriminant 127.
    UmbraArmor,

    /// CR 702.161a: Living metal -- "During your turn, this permanent is an
    /// artifact creature in addition to its other types."
    ///
    /// Static ability on Vehicles. Applied inline in `calculate_characteristics`
    /// at Layer 4 (TypeChange). Adds the Creature card type when the active
    /// player is the permanent's controller. No Layer 7b needed -- the Vehicle
    /// already has printed P/T.
    ///
    /// Discriminant 128.
    LivingMetal,

    /// CR 702.95a: Soulbond -- "When this creature enters, if you control both
    /// this creature and another creature and both are unpaired, you may pair
    /// this creature with another unpaired creature you control for as long as
    /// both remain creatures on the battlefield under your control." (plus the
    /// symmetric other-ETB trigger sentence.)
    ///
    /// Discriminant 129.
    Soulbond,

    /// CR 702.67a: Fortify [cost] -- "[Cost]: Attach this Fortification to target
    /// land you control. Activate only as a sorcery." Fortify is an activated
    /// ability of Fortification artifacts analogous to Equip for Equipment.
    ///
    /// Discriminant 130.
    Fortify,

    /// CR 702.104: Tribute N -- "As this creature enters, choose an opponent.
    /// That player may put an additional N +1/+1 counters on it as it enters."
    ///
    /// Static ability that functions at ETB time (CR 702.104a). The creature's
    /// controller chooses an opponent, who may pay tribute (place N counters)
    /// or decline (triggering the "tribute wasn't paid" ability).
    ///
    /// Discriminant 131.
    Tribute(u32),

    /// CR 702.123: Fabricate N -- "When this permanent enters, you may put N
    /// +1/+1 counters on it. If you don't, create N 1/1 colorless Servo
    /// artifact creature tokens."
    ///
    /// Triggered ability (CR 702.123a). Multiple instances trigger separately
    /// (CR 702.123b).
    ///
    /// Discriminant 132.
    Fabricate(u32),

    /// CR 702.102: Fuse — if a split card has fuse, the controller may cast
    /// both halves from their hand, paying both costs and executing both effects
    /// in order (left first, then right — CR 702.102d).
    ///
    /// Static ability (CR 702.102a). No parameter.
    ///
    /// Discriminant 133.
    Fuse,

    /// CR 702.172a: Spree — static ability on modal spells. "Choose one or more
    /// modes. As an additional cost to cast this spell, pay the costs associated
    /// with those modes." Each mode has its own additional cost (CR 700.2h).
    ///
    /// Static ability. Marker for quick presence-checking (`keywords.contains`).
    /// The per-mode costs are stored in `ModeSelection.mode_costs`.
    ///
    /// Discriminant 134.
    Spree,
    /// CR 702.156: Ravenous -- "This permanent enters with X +1/+1 counters on it"
    /// and "When this permanent enters, if X is 5 or more, draw a card."
    /// X is the value chosen when the spell was cast (CR 107.3m).
    ///
    /// Discriminant 135.
    Ravenous,
    /// CR 701.57: Discover N — keyword action. Exile cards from the top of your
    /// library until you exile a nonland card with mana value N or less. You may
    /// cast that card without paying its mana cost. If you don't cast it, put that
    /// card into your hand. Put the remaining exiled cards on the bottom of your
    /// library in a random order.
    ///
    /// Discover is a keyword action (CR 701.57), not a triggered ability like
    /// Cascade (CR 702.85). Cards bear this keyword as a marker; the actual
    /// discover action is invoked by their triggered abilities via Effect::Discover.
    ///
    /// Key differences from Cascade:
    /// - MV threshold is <= N (not < spell_MV like Cascade)
    /// - Declined discovered card goes to hand (not library bottom)
    /// - Uses a fixed N parameter, not the spell's MV
    ///
    /// Discriminant 136.
    Discover,
    /// CR 702.157a: Squad -- additional cost paid N times at cast; creates N token copies
    /// of the creature on ETB. "As an additional cost to cast this spell, you may pay
    /// [cost] any number of times" and "When this creature enters, if its squad cost was
    /// paid, create a token that's a copy of it for each time its squad cost was paid."
    ///
    /// Unit variant: the actual cost is stored in `AbilityDefinition::Squad { cost }`.
    /// This variant is used for presence-checking (layer-resolved characteristics).
    ///
    /// Discriminant 137.
    Squad,
    /// CR 702.175a: Offspring -- two linked abilities. "You may pay an additional [cost]
    /// as you cast this spell" and "When this permanent enters, if its offspring cost was
    /// paid, create a token that's a copy of it, except it's 1/1."
    ///
    /// Binary: paid once or not at all (unlike Squad which can be paid N times).
    /// The token is a copy of the entering permanent with base P/T overridden to 1/1 (CR 707.9d).
    ///
    /// Unit variant: the actual cost is stored in `AbilityDefinition::Offspring { cost }`.
    /// This variant is used for presence-checking (layer-resolved characteristics).
    ///
    /// Discriminant 138.
    Offspring,
    /// CR 702.174: Gift a [something] -- optional additional cost: choose an opponent.
    ///
    /// "As an additional cost to cast this spell, you may choose an opponent."
    /// If the gift cost was paid, the chosen opponent receives a gift (defined by
    /// the card's AbilityDefinition::Gift variant) and the caster may get an
    /// enhanced effect.
    ///
    /// For permanents: ETB trigger "When this enters, if its gift cost was paid, [effect]."
    /// For instants/sorceries: inline conditional "If this spell's gift cost was paid, [effect]."
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The gift type is stored in `AbilityDefinition::Gift { gift_type }`.
    /// The chosen opponent is provided via `CastSpell.gift_opponent`.
    /// Cards check "if gift was given" via `Condition::GiftWasGiven`.
    ///
    /// CR 702.174a: Multiple instances are redundant (only one opponent can be chosen).
    ///
    /// Discriminant 139.
    Gift,
    /// CR 702.171: Saddle N -- "Tap any number of other untapped creatures you control
    /// with total power N or greater: This permanent becomes saddled until end of turn.
    /// Activate only as a sorcery."
    ///
    /// Marker keyword for quick presence-checking. The saddle cost (N) is the
    /// minimum total power of creatures that must be tapped.
    /// The actual command is `Command::SaddleMount`, handled in `handle_saddle_mount`.
    /// Unlike Crew, Saddle has a sorcery-speed restriction (CR 702.171a) and sets
    /// a boolean designation (`is_saddled`) instead of adding the Creature type.
    ///
    /// Discriminant 140.
    Saddle(u32),
    /// CR 702.99a: Cipher -- two linked abilities.
    ///
    /// First ability (spell ability): "If this spell is represented by a card,
    /// you may exile this card encoded on a creature you control."
    ///
    /// Second ability (static, while in exile): "For as long as this card is
    /// encoded on that creature, that creature has 'Whenever this creature deals
    /// combat damage to a player, you may copy the encoded card and you may cast
    /// the copy without paying its mana cost.'"
    ///
    /// Encoding happens at resolution. The card goes directly from the stack to
    /// exile. It never enters the graveyard. (ruling 2013-04-15)
    ///
    /// Discriminant 141.
    Cipher,
    /// CR 702.55a: Haunt -- two linked triggered abilities.
    ///
    /// On a creature: "When this creature dies, exile it haunting target creature."
    /// On a spell: "When this spell is put into a graveyard during its resolution,
    /// exile it haunting target creature."
    ///
    /// The exiled card then has a second trigger: "When the creature it haunts
    /// dies, [effect]." This trigger fires from exile (CR 702.55c).
    ///
    /// Discriminant 142.
    Haunt,
    /// CR 702.151: Reconfigure [cost] -- two activated abilities.
    /// "[Cost]: Attach this permanent to another target creature you control.
    /// Activate only as a sorcery." and "[Cost]: Unattach this permanent.
    /// Activate only if this permanent is attached to a creature and only
    /// as a sorcery."
    ///
    /// CR 702.151b: While attached, the Equipment stops being a creature
    /// (and loses creature subtypes).
    ///
    /// Marker for quick presence-checking (`keywords.contains`).
    /// The reconfigure cost is stored in `AbilityDefinition::Reconfigure { cost }`.
    ///
    /// Discriminant 143.
    Reconfigure,
}

/// CR 702.72a: The filter for what can be championed.
///
/// "Champion a creature" = `AnyCreature`
/// "Champion a [subtype]" = `Subtype(SubType)` (e.g., Faerie, Goblin)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChampionFilter {
    /// "Champion a creature" -- any creature you control (other than self).
    AnyCreature,
    /// "Champion a [subtype]" -- any permanent with this subtype you control (other than self).
    Subtype(SubType),
}

/// All creature subtypes from CR 205.3m.
///
/// Used by Changeling (CR 702.73a) and "is every creature type" effects such as
/// Maskwood Nexus. Lazily initialized on first use; `im::OrdSet` clones are O(1)
/// due to structural sharing.
pub static ALL_CREATURE_TYPES: std::sync::LazyLock<im::OrdSet<SubType>> =
    std::sync::LazyLock::new(|| {
        #[rustfmt::skip]
        let types: &[&str] = &[
            "Advisor", "Aetherborn", "Alien", "Ally", "Angel", "Antelope", "Ape",
            "Archer", "Archon", "Army", "Artificer", "Assassin", "Assembly-Worker",
            "Atog", "Aurochs", "Avatar", "Azra", "Badger", "Balloon", "Barbarian",
            "Bard", "Bear", "Beast", "Beaver", "Beeble", "Beholder", "Berserker",
            "Bird", "Blinkmoth", "Boar", "Bringer", "Brushwagg", "Camarid", "Camel",
            "Caribou", "Carrier", "Cat", "Centaur", "Cephalid", "Child", "Chimera",
            "Citizen", "Cleric", "Cockroach", "Construct", "Coward", "Crab",
            "Crocodile", "Cyclops", "Dauthi", "Demigod", "Demon", "Deserter",
            "Detective", "Devil", "Dinosaur", "Djinn", "Dog", "Dragon", "Drake",
            "Dreadnought", "Drone", "Druid", "Dryad", "Dwarf", "Efreet", "Egg",
            "Elder", "Eldrazi", "Elemental", "Elephant", "Elf", "Elves", "Elk",
            "Employee", "Eye", "Faerie", "Ferret", "Fish", "Flagbearer", "Fox",
            "Fractal", "Frog", "Fungus", "Gargoyle", "Germ", "Giant", "Gith",
            "Gnoll", "Gnome", "Goat", "Goblin", "God", "Golem", "Gorgon", "Graveborn",
            "Gremlin", "Griffin", "Guest", "Hag", "Halfling", "Hamster", "Harpy",
            "Hellion", "Hippo", "Hippogriff", "Homarid", "Homunculus", "Horror",
            "Horse", "Human", "Hydra", "Hyena", "Illusion", "Imp", "Incarnation",
            "Inkling", "Inquisitor", "Insect", "Jackal", "Jellyfish", "Juggernaut",
            "Kavu", "Kirin", "Kithkin", "Knight", "Kobold", "Kor", "Kraken", "Lamia",
            "Lammasu", "Leech", "Leviathan", "Lhurgoyf", "Licid", "Lizard", "Manticore",
            "Masticore", "Mercenary", "Merfolk", "Metathran", "Minion", "Minotaur",
            "Mite", "Mole", "Monger", "Mongoose", "Monk", "Monkey", "Moonfolk",
            "Mouse", "Mutant", "Myr", "Mystic", "Naga", "Nautilus", "Nephilim",
            "Nightmare", "Nightstalker", "Ninja", "Noble", "Noggle", "Nomad", "Nymph",
            "Octopus", "Ogre", "Ooze", "Orb", "Orc", "Orgg", "Otter", "Ouphe", "Ox",
            "Oyster", "Pangolin", "Peasant", "Pegasus", "Pentavite", "Performer",
            "Pest", "Phelddagrif", "Phoenix", "Phyrexian", "Pilot", "Pincher",
            "Pirate", "Plant", "Praetor", "Primarch", "Prism", "Processor",
            "Rabbit", "Raccoon", "Ranger", "Rat", "Rebel", "Reflection", "Rhino",
            "Rigger", "Robot", "Rogue", "Sable", "Salamander", "Samurai", "Sand",
            "Saproling", "Satyr", "Scarecrow", "Scientist", "Scion", "Scorpion",
            "Scout", "Sculpture", "Serf", "Serpent", "Servo", "Shade", "Shaman",
            "Shapeshifter", "Shark", "Sheep", "Siren", "Skeleton", "Slith",
            "Sliver", "Slug", "Snake", "Soldier", "Soltari", "Spawn", "Specter",
            "Spellshaper", "Sphinx", "Spider", "Spike", "Spirit", "Splinter",
            "Sponge", "Squid", "Squirrel", "Starfish", "Surrakar", "Survivor",
            "Tentacle", "Tetravite", "Thalakos", "Thopter", "Thrull", "Tiefling",
            "Time Lord", "Treefolk", "Trilobite", "Triskelavite", "Troll", "Turtle",
            "Tyranid", "Unicorn", "Vampire", "Vedalken", "Viashino", "Volver",
            "Wall", "Walrus", "Warlock", "Warrior", "Weird", "Werewolf", "Whale",
            "Wizard", "Wolf", "Wolverine", "Wombat", "Worm", "Wraith", "Wurm",
            "Yeti", "Zombie", "Zubera",
        ];
        types.iter().map(|s| SubType(s.to_string())).collect()
    });
