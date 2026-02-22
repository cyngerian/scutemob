//! Card definition types: the engine's internal DSL for card behavior.
//!
//! A `CardDefinition` describes what a card does in terms the engine can execute.
//! It is separate from `Characteristics` (which is the game-state representation
//! of an object's observable properties). The definition is static data loaded
//! from the card database; `Characteristics` is per-object runtime state.
//!
//! See architecture doc Section 3.7 for design rationale.

use im::OrdSet;
use serde::{Deserialize, Serialize};

use crate::state::{
    CardId, CardType, Color, CounterType, KeywordAbility, ManaColor, ManaCost, ManaPool, SubType,
    SuperType,
};

// ── Card Definition ───────────────────────────────────────────────────────────

/// A complete card definition: what a card is and what it does (CR Section 2).
///
/// Loaded from the card database at startup. Looked up via `CardRegistry`
/// during effect resolution. Does not change during a game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardDefinition {
    /// Stable card identity across all printings (Scryfall oracle_id).
    pub card_id: CardId,
    pub name: String,
    /// Printed mana cost. None for lands and some tokens.
    pub mana_cost: Option<ManaCost>,
    pub types: TypeLine,
    /// Oracle text for display only. Behavior is encoded in `abilities`.
    pub oracle_text: String,
    /// All abilities on the card in oracle-text order.
    pub abilities: Vec<AbilityDefinition>,
    /// Printed power (creatures only). None for non-creatures.
    #[serde(default)]
    pub power: Option<i32>,
    /// Printed toughness (creatures only). None for non-creatures.
    #[serde(default)]
    pub toughness: Option<i32>,
}

impl Default for CardDefinition {
    fn default() -> Self {
        CardDefinition {
            card_id: CardId(String::new()),
            name: String::new(),
            mana_cost: None,
            types: TypeLine::default(),
            oracle_text: String::new(),
            abilities: vec![],
            power: None,
            toughness: None,
        }
    }
}

/// Type line of a card: supertypes, card types, and subtypes (CR 205).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TypeLine {
    pub supertypes: OrdSet<SuperType>,
    pub card_types: OrdSet<CardType>,
    pub subtypes: OrdSet<SubType>,
}

// ── Ability Definitions ───────────────────────────────────────────────────────

/// One ability on a card (CR 112). Encodes behavior the engine can execute.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AbilityDefinition {
    /// Activated ability: "[Cost]: [Effect]" (CR 602).
    Activated {
        cost: Cost,
        effect: Effect,
        /// If Some, restricts when the ability can be activated (e.g., sorcery speed).
        timing_restriction: Option<TimingRestriction>,
    },
    /// Triggered ability: "When/Whenever/At [event], [Effect]" (CR 603).
    Triggered {
        trigger_condition: TriggerCondition,
        effect: Effect,
        /// Intervening-if condition checked at trigger time and resolution (CR 603.4).
        intervening_if: Option<Condition>,
    },
    /// Static ability that generates a continuous effect while the source is on the battlefield
    /// (CR 604). Handled via the layer system (see `rules/layers.rs`).
    Static {
        continuous_effect: ContinuousEffectDef,
    },
    /// A keyword ability (CR 702). Many are enforced by existing rules code.
    Keyword(KeywordAbility),
    /// The effect of the spell itself when it resolves (for instants and sorceries) (CR 608).
    Spell {
        effect: Effect,
        targets: Vec<TargetRequirement>,
        modes: Option<ModeSelection>,
    },
}

// ── Cost ─────────────────────────────────────────────────────────────────────

/// The cost to activate an ability or cast a spell (CR 118).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cost {
    /// Pay a mana cost (CR 202).
    Mana(ManaCost),
    /// Tap the source permanent (CR 602.2).
    Tap,
    /// Sacrifice a permanent matching the filter.
    Sacrifice(TargetFilter),
    /// Pay N life (CR 119).
    PayLife(u32),
    /// Discard a card.
    DiscardCard,
    /// Multiple costs, all paid simultaneously (CR 601.2g).
    Sequence(Vec<Cost>),
}

// ── Effect ────────────────────────────────────────────────────────────────────

/// A recursive effect primitive: the engine's internal DSL for card behavior.
///
/// Effects are executed by `effects::execute_effect`. Every effect that changes
/// game state emits `GameEvent`s. Effects are composed with `Sequence`, `Conditional`,
/// `Choose`, and `ForEach`.
///
/// See architecture doc Section 3.7 for the full list of primitives.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Effect {
    // ── Damage & Life ───────────────────────────────────────────────────────
    /// CR 119: Deal damage to a target (player, creature, or planeswalker).
    DealDamage {
        target: EffectTarget,
        amount: EffectAmount,
    },
    /// CR 118.4: A player gains life.
    GainLife {
        player: PlayerTarget,
        amount: EffectAmount,
    },
    /// CR 118.4: A player loses life.
    LoseLife {
        player: PlayerTarget,
        amount: EffectAmount,
    },

    // ── Cards ───────────────────────────────────────────────────────────────
    /// CR 121: A player draws one or more cards.
    DrawCards {
        player: PlayerTarget,
        count: EffectAmount,
    },
    /// CR 701.7: A player discards one or more cards.
    DiscardCards {
        player: PlayerTarget,
        count: EffectAmount,
    },
    /// CR 701.13: A player puts the top N cards of their library into their graveyard.
    MillCards {
        player: PlayerTarget,
        count: EffectAmount,
    },

    // ── Permanents ──────────────────────────────────────────────────────────
    /// CR 701.6: Create a token on the battlefield.
    CreateToken { spec: TokenSpec },
    /// CR 701.7: Destroy a permanent (does not apply to indestructible).
    DestroyPermanent { target: EffectTarget },
    /// CR 701.5: Put an object into exile.
    ExileObject { target: EffectTarget },
    /// CR 701.5: Counter a spell or ability on the stack.
    CounterSpell { target: EffectTarget },
    /// CR 701.16: Tap a permanent.
    TapPermanent { target: EffectTarget },
    /// CR 701.17: Untap a permanent.
    UntapPermanent { target: EffectTarget },

    // ── Mana ────────────────────────────────────────────────────────────────
    /// Add mana to a player's pool (CR 106).
    AddMana {
        player: PlayerTarget,
        mana: ManaPool,
    },
    /// Add one mana of any color to a player's pool.
    AddManaAnyColor { player: PlayerTarget },
    /// Add N mana of any one color.
    AddManaChoice {
        player: PlayerTarget,
        count: EffectAmount,
    },

    // ── Counters ─────────────────────────────────────────────────────────────
    /// CR 122: Put one or more counters on a permanent or player.
    AddCounter {
        target: EffectTarget,
        counter: CounterType,
        count: u32,
    },
    /// CR 122: Remove counters from a permanent or player.
    RemoveCounter {
        target: EffectTarget,
        counter: CounterType,
        count: u32,
    },

    // ── Zone ─────────────────────────────────────────────────────────────────
    /// Move an object to a zone (CR 400).
    MoveZone {
        target: EffectTarget,
        to: ZoneTarget,
    },

    // ── Library ─────────────────────────────────────────────────────────────
    /// CR 701.19: Search a library for a card matching a filter.
    SearchLibrary {
        player: PlayerTarget,
        filter: TargetFilter,
        reveal: bool,
        destination: ZoneTarget,
    },
    /// CR 701.20: Shuffle a player's library.
    Shuffle { player: PlayerTarget },

    // ── Continuous Effects ───────────────────────────────────────────────────
    /// Apply a continuous effect until end of turn or for a duration (CR 611).
    ApplyContinuousEffect {
        effect_def: Box<ContinuousEffectDef>,
    },

    // ── Combinators ─────────────────────────────────────────────────────────
    /// Execute `if_true` if condition holds, otherwise `if_false` (may be Nothing).
    Conditional {
        condition: Condition,
        if_true: Box<Effect>,
        if_false: Box<Effect>,
    },
    /// Apply effect once for each element in `over`.
    ForEach {
        over: ForEachTarget,
        effect: Box<Effect>,
    },
    /// Player chooses one of the given effects (modal effects, CR 700.2).
    Choose {
        prompt: String,
        choices: Vec<Effect>,
    },
    /// Execute effects in order (CR 101.2).
    Sequence(Vec<Effect>),
    /// A player may pay a cost; if they don't, apply the effect.
    MayPayOrElse {
        cost: Cost,
        payer: PlayerTarget,
        or_else: Box<Effect>,
    },
    /// No effect (used in Conditional branches, or for keyword-only cards).
    Nothing,
}

// ── Effect Targets ────────────────────────────────────────────────────────────

/// How an effect identifies its primary target.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectTarget {
    /// One of the targets chosen when the spell/ability was put on the stack (0-indexed).
    DeclaredTarget { index: usize },
    /// The controller of the spell or ability.
    Controller,
    /// Each player, simultaneously.
    EachPlayer,
    /// Each opponent of the controller.
    EachOpponent,
    /// Every creature on the battlefield.
    AllCreatures,
    /// Every permanent on the battlefield.
    AllPermanents,
    /// Every permanent matching a filter on the battlefield.
    AllPermanentsMatching(TargetFilter),
    /// The spell/ability's source object.
    Source,
}

/// How an effect identifies a player.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerTarget {
    /// The controller of the spell or ability.
    Controller,
    /// Each player simultaneously.
    EachPlayer,
    /// Each opponent of the controller.
    EachOpponent,
    /// The target player (declared at cast time). Index into the targets list.
    DeclaredTarget { index: usize },
    /// The controller of the specified permanent (used for e.g. Swords to Plowshares:
    /// "its controller gains life equal to its power").
    ControllerOf(Box<EffectTarget>),
}

/// How an effect produces a numeric value.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectAmount {
    /// A fixed number.
    Fixed(i32),
    /// The value of X (from an X-cost spell or ability).
    XValue,
    /// The power of a permanent. Used for e.g. "gain life equal to its power" (Swords to Plowshares).
    PowerOf(EffectTarget),
    /// The toughness of a permanent.
    ToughnessOf(EffectTarget),
    /// The mana value of an object.
    ManaValueOf(EffectTarget),
    /// Number of cards in a zone.
    CardCount {
        zone: ZoneTarget,
        player: PlayerTarget,
        filter: Option<TargetFilter>,
    },
}

// ── Target Requirements ───────────────────────────────────────────────────────

/// A legal target type for a spell or ability (CR 601.2c, CR 115).
///
/// This is declared on the spell/ability at definition time and used to validate
/// targets when the spell or ability is put on the stack.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TargetRequirement {
    /// "target creature"
    TargetCreature,
    /// "target player"
    TargetPlayer,
    /// "target permanent"
    TargetPermanent,
    /// "target creature or player"
    TargetCreatureOrPlayer,
    /// "any target" = creature, planeswalker, or player (CR 115.4)
    TargetAny,
    /// "target spell"
    TargetSpell,
    /// "target artifact"
    TargetArtifact,
    /// "target enchantment"
    TargetEnchantment,
    /// "target land"
    TargetLand,
    /// "target planeswalker"
    TargetPlaneswalker,
    /// "target creature with power N or less" etc.
    TargetCreatureWithFilter(TargetFilter),
    /// "target permanent with filter"
    TargetPermanentWithFilter(TargetFilter),
    /// "target player or planeswalker"
    TargetPlayerOrPlaneswalker,
}

/// A filter on game objects, used for target requirements and `SearchLibrary`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TargetFilter {
    /// Max power (inclusive). None = no restriction.
    pub max_power: Option<i32>,
    /// Min power (inclusive). None = no restriction.
    pub min_power: Option<i32>,
    /// Exactly one of these card types. None = no restriction.
    pub has_card_type: Option<CardType>,
    /// Must have all these keywords.
    pub has_keywords: OrdSet<KeywordAbility>,
    /// Must be one of these colors. None = no restriction.
    pub colors: Option<OrdSet<Color>>,
    /// Must be non-land.
    pub non_land: bool,
    /// Must be basic.
    pub basic: bool,
    /// Controller constraint.
    pub controller: TargetController,
    /// Subtype constraint.
    pub has_subtype: Option<SubType>,
}

/// Whose control an object must be under for a target filter.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetController {
    #[default]
    Any,
    You,
    Opponent,
}

// ── Trigger Conditions ────────────────────────────────────────────────────────

/// What game event causes a triggered ability to fire (CR 603.1).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// "When ~ enters the battlefield" — self-referential ETB.
    WhenEntersBattlefield,
    /// "When ~ dies" — this object goes from battlefield to graveyard.
    WhenDies,
    /// "When ~ attacks" — this creature is declared as an attacker.
    WhenAttacks,
    /// "When ~ blocks" — this creature is declared as a blocker.
    WhenBlocks,
    /// "Whenever ~ deals combat damage to a player."
    WhenDealsCombatDamageToPlayer,
    /// "Whenever an opponent casts a spell."
    WheneverOpponentCastsSpell,
    /// "Whenever a player draws a card."
    WheneverPlayerDrawsCard,
    /// "Whenever a creature dies."
    WheneverCreatureDies,
    /// "Whenever a creature enters the battlefield" (with optional filter).
    WheneverCreatureEntersBattlefield { filter: Option<TargetFilter> },
    /// "Whenever a permanent enters the battlefield" (with optional filter).
    WheneverPermanentEntersBattlefield { filter: Option<TargetFilter> },
    /// "At the beginning of your upkeep."
    AtBeginningOfYourUpkeep,
    /// "At the beginning of each player's upkeep."
    AtBeginningOfEachUpkeep,
    /// "At the beginning of your end step."
    AtBeginningOfYourEndStep,
    /// "At the beginning of combat on your turn."
    AtBeginningOfCombat,
    /// "Whenever you cast a spell."
    WheneverYouCastSpell,
    /// "Whenever you gain life."
    WheneverYouGainLife,
    /// "Whenever you draw a card."
    WheneverYouDrawACard,
}

// ── Conditions ────────────────────────────────────────────────────────────────

/// A boolean condition checked at trigger time or in Conditional effects (CR 603.4).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Condition {
    /// "if your life total is N or more"
    ControllerLifeAtLeast(u32),
    /// "if ~ is still on the battlefield" — source object still exists.
    SourceOnBattlefield,
    /// "if you control a [filter] permanent"
    YouControlPermanent(TargetFilter),
    /// "if an opponent controls a [filter] permanent"
    OpponentControlsPermanent(TargetFilter),
    /// "if the target is still legal" — used for intervening-if checks.
    TargetIsLegal { index: usize },
    /// "if ~ has N or more [counter] counters on it"
    SourceHasCounters { counter: CounterType, min: u32 },
    /// Always true (for Conditional branches that always fire).
    Always,
}

// ── Mode Selection ────────────────────────────────────────────────────────────

/// Modal spells/abilities: choose N of M modes (CR 700.2).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModeSelection {
    /// Minimum number of modes the caster must choose.
    pub min_modes: usize,
    /// Maximum number of modes the caster may choose.
    pub max_modes: usize,
    /// The available modes (effects). Indexed by the `ChooseOption` command.
    pub modes: Vec<Effect>,
}

// ── Token Specification ───────────────────────────────────────────────────────

/// Everything needed to create a token (CR 111).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenSpec {
    pub name: String,
    pub power: i32,
    pub toughness: i32,
    pub colors: OrdSet<Color>,
    pub card_types: OrdSet<CardType>,
    pub subtypes: OrdSet<SubType>,
    pub keywords: OrdSet<KeywordAbility>,
    /// How many tokens to create.
    pub count: u32,
    /// True if the tokens enter the battlefield tapped.
    pub tapped: bool,
    /// Color override: all created under the controller's control.
    pub mana_color: Option<ManaColor>,
}

impl Default for TokenSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            power: 1,
            toughness: 1,
            colors: OrdSet::new(),
            card_types: OrdSet::new(),
            subtypes: OrdSet::new(),
            keywords: OrdSet::new(),
            count: 1,
            tapped: false,
            mana_color: None,
        }
    }
}

// ── Zone Target ───────────────────────────────────────────────────────────────

/// A destination zone for zone-change effects (CR 400).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ZoneTarget {
    /// "enters the battlefield" — optionally tapped.
    Battlefield { tapped: bool },
    /// "put into its owner's graveyard."
    Graveyard { owner: PlayerTarget },
    /// "put into its owner's hand."
    Hand { owner: PlayerTarget },
    /// "put [on top / on bottom / at random position] of [player]'s library."
    Library {
        owner: PlayerTarget,
        position: LibraryPosition,
    },
    /// "exiled" — cards in exile do not have a specific owner in the zone reference.
    Exile,
    /// "the command zone."
    CommandZone,
}

/// Where in the library an object is placed (CR 401).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LibraryPosition {
    Top,
    Bottom,
    /// Shuffled in at random (the library is shuffled afterward).
    ShuffledIn,
}

// ── For Each Target ───────────────────────────────────────────────────────────

/// The collection `ForEach` iterates over.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ForEachTarget {
    /// Each opponent of the controller.
    EachOpponent,
    /// Every player (including the controller).
    EachPlayer,
    /// Every creature on the battlefield.
    EachCreature,
    /// Every creature the controller controls.
    EachCreatureYouControl,
    /// Every creature opponents control.
    EachOpponentsCreature,
    /// Every permanent matching a filter.
    EachPermanentMatching(TargetFilter),
}

// ── Timing Restriction ────────────────────────────────────────────────────────

/// When an activated ability can be used.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TimingRestriction {
    /// Only during your main phase when the stack is empty (sorcery speed).
    SorcerySpeed,
    /// Any time you have priority (default for activated abilities).
    AnyTime,
}

// ── Continuous Effect Definition ──────────────────────────────────────────────

/// Defines a continuous effect for use in `AbilityDefinition::Static` and `Effect::ApplyContinuousEffect`.
///
/// References layer types from `state::continuous_effect`. Static abilities
/// create these when the source is on the battlefield; some instant/sorcery effects
/// create them temporarily.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContinuousEffectDef {
    pub layer: crate::state::EffectLayer,
    pub modification: crate::state::LayerModification,
    pub filter: crate::state::EffectFilter,
    pub duration: crate::state::EffectDuration,
}
