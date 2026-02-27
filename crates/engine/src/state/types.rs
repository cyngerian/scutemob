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
