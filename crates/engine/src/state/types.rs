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
