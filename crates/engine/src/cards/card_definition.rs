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

use crate::state::continuous_effect::{EffectLayer, LayerModification};
use crate::state::game_object::{ActivatedAbility, ManaAbility};
use crate::state::replacement_effect::{ReplacementModification, ReplacementTrigger};
use crate::state::types::AltCostKind;
use crate::state::{
    CardId, CardType, ChampionFilter, Color, CounterType, CumulativeUpkeepCost, KeywordAbility,
    ManaColor, ManaCost, ManaPool, SubType, SuperType,
};

// ── Card Definition ───────────────────────────────────────────────────────────

/// The back face of a double-faced card (CR 712).
///
/// Holds the back face's characteristics and abilities. The front face data
/// remains in the parent `CardDefinition` struct. When `is_transformed` is true
/// on a `GameObject`, the engine uses this struct for base characteristics.
///
/// CR 712.8a: While a double-faced card is outside the game or in a zone other
/// than the battlefield or stack, it has only the characteristics of its front face.
/// CR 712.8e: The back face's mana value is calculated using the front face's mana cost.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardFace {
    pub name: String,
    /// The back face's own mana cost (if any). None for most back faces.
    /// Note: mana VALUE uses the front face's mana cost (CR 712.8e).
    pub mana_cost: Option<ManaCost>,
    pub types: TypeLine,
    pub oracle_text: String,
    pub abilities: Vec<AbilityDefinition>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    /// Color indicator (CR 204) — used by back faces that have no mana cost
    /// but need a color identity (e.g., Insectile Aberration is blue via indicator).
    #[serde(default)]
    pub color_indicator: Option<Vec<crate::state::Color>>,
}

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
    /// CR 204: Color indicator (colored dot on type line). Overrides mana-cost-derived
    /// colors. Used by cards like Dryad Arbor (green with no mana cost).
    #[serde(default)]
    pub color_indicator: Option<Vec<crate::state::Color>>,
    /// CR 712: The back face of a double-faced card.
    ///
    /// `None` for single-faced cards. `Some(face)` for DFCs — Transform,
    /// Disturb, Daybound/Nightbound, Craft, etc. When `GameObject.is_transformed`
    /// is true, the layer system uses this face's characteristics as the base.
    #[serde(default)]
    pub back_face: Option<CardFace>,
    /// Static cost modifiers this permanent applies to spells being cast (CR 601.2f).
    ///
    /// Example: Thalia, Guardian of Thraben — noncreature spells cost {1} more.
    /// Example: Goblin Warchief — Goblin spells you cast cost {1} less.
    #[serde(default)]
    pub spell_cost_modifiers: Vec<SpellCostModifier>,
    /// Self-cost-reduction for this spell at cast time (CR 601.2f).
    ///
    /// Example: Blasphemous Act — costs {1} less for each creature on the battlefield.
    #[serde(default)]
    pub self_cost_reduction: Option<SelfCostReduction>,
    /// CR 306.5a: Printed loyalty number (planeswalkers only). None for non-planeswalkers.
    /// CR 306.5b: A planeswalker enters with this many loyalty counters.
    #[serde(default)]
    pub starting_loyalty: Option<u32>,
    /// CR 712.4: Meld pair information. Present on both cards that form a meld pair.
    ///
    /// One card in each pair has an ability that exiles both cards and melds them
    /// (CR 712.4a). The other card has `(Melds with <partner>.)` reminder text.
    /// Both cards share the same `back_face` — the combined melded permanent's
    /// characteristics. When melded, the permanent uses `back_face` characteristics
    /// (CR 712.8g) and its mana value is the sum of both front face mana values.
    #[serde(default)]
    pub meld_pair: Option<MeldPair>,
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
            color_indicator: None,
            back_face: None,
            spell_cost_modifiers: vec![],
            self_cost_reduction: None,
            starting_loyalty: None,
            meld_pair: None,
        }
    }
}

/// CR 712.4: Meld pair information for a card that participates in a meld.
///
/// Both cards in a meld pair carry this struct. The `pair_card_id` identifies
/// the other card. The `melded_card_id` identifies the combined permanent's
/// CardDefinition (whose `back_face` holds the melded characteristics).
///
/// CR 712.5: There are seven specific meld pairs in Magic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeldPair {
    /// The CardId of the other card in this meld pair.
    pub pair_card_id: CardId,
    /// The CardId of the combined melded permanent's definition.
    /// This definition's `back_face` holds the melded face characteristics.
    /// Both cards in the pair reference the same melded_card_id.
    pub melded_card_id: CardId,
}

/// Type line of a card: supertypes, card types, and subtypes (CR 205).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeLine {
    pub supertypes: OrdSet<SuperType>,
    pub card_types: OrdSet<CardType>,
    pub subtypes: OrdSet<SubType>,
}

// ── Ability Definitions ───────────────────────────────────────────────────────

/// One ability on a card (CR 112). Encodes behavior the engine can execute.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbilityDefinition {
    /// Activated ability: "[Cost]: [Effect]" (CR 602).
    Activated {
        cost: Cost,
        effect: Effect,
        /// If Some, restricts when the ability can be activated (e.g., sorcery speed).
        timing_restriction: Option<TimingRestriction>,
        /// Target requirements for this activated ability (CR 601.2c).
        /// Empty = no targets required.
        #[serde(default)]
        targets: Vec<TargetRequirement>,
    },
    /// Triggered ability: "When/Whenever/At [event], [Effect]" (CR 603).
    Triggered {
        trigger_condition: TriggerCondition,
        effect: Effect,
        /// Intervening-if condition checked at trigger time and resolution (CR 603.4).
        intervening_if: Option<Condition>,
        /// Target requirements for this triggered ability (CR 601.2c).
        /// Empty = no targets required.
        #[serde(default)]
        targets: Vec<TargetRequirement>,
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
        /// CR 101.6: If true, this spell can't be countered by spells or abilities.
        #[serde(default)]
        cant_be_countered: bool,
    },
    /// A replacement/prevention static ability (CR 614-615).
    ///
    /// The effect modifies an event before it occurs. Unlike triggers, replacement
    /// effects don't use the stack — they happen inline.
    ///
    /// When `is_self` is true (CR 614.15), this effect applies to the object itself
    /// and is applied before non-self replacements on the same event.
    Replacement {
        trigger: ReplacementTrigger,
        modification: ReplacementModification,
        /// CR 614.15: if true, this is a self-replacement (applies before global effects).
        #[serde(default)]
        is_self: bool,
        /// CR 614.1c: "enters tapped unless [condition]" — if the condition is met,
        /// the replacement is skipped (permanent enters untapped). If not met, the
        /// replacement applies normally. Used by check-lands, fast-lands, bond-lands, etc.
        #[serde(default)]
        unless_condition: Option<Condition>,
    },
    /// CR 113.6b: Opening-hand static ability — "If ~ is in your opening hand, you may
    /// begin the game with it on the battlefield."
    ///
    /// Cards with this ability are placed on the battlefield by `start_game` before
    /// the first turn begins. The card is moved from the player's hand to the
    /// battlefield as a pre-game action (not cast, not resolved, no ETB trigger).
    OpeningHand,
    /// A Panharmonicon-style trigger-doubling static ability (CR 603.2d).
    ///
    /// "Whenever an artifact or creature enters the battlefield under your control,
    /// if a triggered ability of a permanent you control would trigger from that
    /// event, that ability triggers an additional time."
    ///
    /// When a permanent with this ability enters the battlefield, a `TriggerDoubler`
    /// entry is registered in `GameState::trigger_doublers`. When it leaves, the
    /// entry is cleaned up lazily (check at use: source must still be on battlefield).
    TriggerDoubling {
        filter: crate::state::stubs::TriggerDoublerFilter,
        additional_triggers: u32,
    },
    /// CR 614.16a: A Torpor Orb-style static ability that prevents ETB triggered abilities
    /// from triggering on entering permanents.
    ///
    /// "Creatures entering the battlefield don't cause abilities to trigger."
    ///
    /// This is a replacement effect (CR 614.16a): the trigger never fires, rather than
    /// being countered after firing. Applies to CardDef `AbilityDefinition::Triggered` with
    /// `WhenEntersBattlefield` condition. Does NOT suppress keyword ETB effects like Landfall
    /// or ETB replacements — only CardDef triggered ability queueing.
    ///
    /// When a permanent with this ability enters the battlefield, an `ETBSuppressor` entry
    /// is registered in `GameState::etb_suppressors`. Cleaned up when that permanent leaves.
    SuppressCreatureETBTriggers {
        filter: crate::state::stubs::ETBSuppressFilter,
    },
    /// Consolidated alternative-cost/graveyard-cast ability (RC-3 consolidation).
    /// Replaces individual Flashback/Embalm/Eternalize/Encore/Unearth/Dash/Blitz/Plot/Escape/Prototype variants.
    /// The `kind` discriminant (AltCostKind) determines resolution behavior.
    ///
    /// Cards with this ability should also include the corresponding
    /// `AbilityDefinition::Keyword(KeywordAbility::*)` for quick presence-checking.
    AltCastAbility {
        kind: AltCostKind,
        cost: ManaCost,
        details: Option<AltCastDetails>,
    },
    /// CR 702.29: Cycling [cost]. The card may be activated from hand by paying
    /// [cost] and discarding itself. The effect is "draw a card."
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Cycling)` for quick
    /// presence-checking without scanning all abilities.
    Cycling { cost: ManaCost },
    /// CR 702.33: Kicker [cost]. Optional additional cost that can be paid
    /// when casting this spell. If paid, the spell is "kicked" and may have
    /// enhanced effects.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Kicker)` for quick
    /// presence-checking without scanning all abilities.
    ///
    /// `is_multikicker` indicates multikicker (CR 702.33c) — the cost can
    /// be paid any number of times instead of at most once.
    Kicker {
        cost: ManaCost,
        #[serde(default)]
        is_multikicker: bool,
    },
    /// CR 702.74: Evoke [cost]. The card may be cast by paying this cost instead of
    /// its mana cost (alternative cost, CR 118.9). When the permanent enters the
    /// battlefield, if evoke was paid, its controller sacrifices it.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Evoke)` for quick
    /// presence-checking without scanning all abilities.
    Evoke { cost: ManaCost },
    /// CR 702.103: Bestow [cost]. The card may be cast by paying this cost instead
    /// of its mana cost (alternative cost, CR 118.9). When cast bestowed, the spell
    /// becomes an Aura enchantment with enchant creature (CR 702.103b).
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Bestow)` for quick
    /// presence-checking without scanning all abilities.
    Bestow { cost: ManaCost },
    /// CR 606: Loyalty ability on a planeswalker.
    ///
    /// Loyalty abilities are activated abilities with loyalty symbols in their costs
    /// (CR 606.2). They follow special timing rules (CR 606.3): sorcery-speed,
    /// empty stack, main phase, and only one loyalty ability per permanent per turn.
    ///
    /// The cost is adding or removing loyalty counters (CR 606.4). The effect is
    /// any Effect, and may have target requirements.
    LoyaltyAbility {
        cost: LoyaltyCost,
        effect: Effect,
        #[serde(default)]
        targets: Vec<TargetRequirement>,
    },
    /// CR 714.2: Saga chapter ability. "{rN}—[Effect]" means "When one or more lore
    /// counters are put onto this Saga, if the number of lore counters on it was less
    /// than N and became at least N, [effect]."
    ///
    /// Chapter abilities are keyword abilities that represent triggered abilities.
    /// They fire when lore counters cross the chapter threshold (CR 714.2b).
    SagaChapter {
        chapter: u32,
        effect: Effect,
        #[serde(default)]
        targets: Vec<TargetRequirement>,
    },
    /// CR 716.2: Class level bar. "[Cost]: Level N — [Abilities]" means
    /// "[Cost]: This Class's level becomes N. Activate only if this Class is level N-1
    /// and only as a sorcery" and "As long as this Class is level N or greater, it has
    /// [abilities]."
    ///
    /// The `cost` is the activation cost of the level-up ability. The `abilities` are
    /// the static/triggered abilities gained at that level.
    ClassLevel {
        level: u32,
        cost: ManaCost,
        abilities: Vec<AbilityDefinition>,
    },
    /// CR 702.35: Madness [cost]. When this card is discarded, it is exiled instead
    /// of going to the graveyard. Then a triggered ability fires: the owner may cast
    /// it by paying [cost] (an alternative cost, CR 118.9). If they decline, it goes
    /// to the graveyard.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Madness)` for quick
    /// presence-checking without scanning all abilities.
    Madness { cost: ManaCost },
    /// CR 702.94: Miracle [cost]. When this card is drawn as the first card of
    /// the turn, the player may reveal it and trigger a triggered ability:
    /// "you may cast it by paying [cost] instead of its mana cost" (alternative
    /// cost, CR 118.9).
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Miracle)` for quick
    /// presence-checking without scanning all abilities.
    Miracle { cost: ManaCost },
    /// CR 702.138c: "This permanent escapes with [N] [counter type] counter(s) on it."
    /// If the permanent escaped, it enters the battlefield with the specified counters.
    /// This is a replacement effect on the ETB event.
    EscapeWithCounter {
        counter_type: CounterType,
        count: u32,
    },
    /// CR 702.143: Foretell [cost]. During your turn, pay {2} and exile this card
    /// from your hand face down. Cast it on a later turn for [cost] rather than
    /// its mana cost (alternative cost, CR 118.9).
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Foretell)` for quick
    /// presence-checking without scanning all abilities.
    Foretell { cost: ManaCost },
    /// CR 702.27: Buyback [cost]. You may pay an additional [cost] as you cast
    /// this spell. If you do, put this spell into its owner's hand instead of
    /// into that player's graveyard as it resolves.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Buyback)` for quick
    /// presence-checking without scanning all abilities.
    Buyback { cost: ManaCost },
    /// CR 702.62: Suspend N -- [cost]. Exile this card from your hand with N
    /// time counters on it by paying [cost]. At the beginning of your upkeep,
    /// remove a time counter. When the last is removed, you may cast it without
    /// paying its mana cost. If it's a creature, it gains haste.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Suspend)` for quick
    /// presence-checking without scanning all abilities.
    Suspend { cost: ManaCost, time_counters: u32 },
    /// CR 702.96: Overload [cost]. The card may be cast by paying this cost instead
    /// of its mana cost (alternative cost, CR 118.9). When overloaded, the spell's
    /// text replaces all instances of "target" with "each" -- modeled as conditional
    /// effect dispatch via `Condition::WasOverloaded`.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Overload)` for quick
    /// presence-checking without scanning all abilities.
    Overload { cost: ManaCost },
    /// CR 702.49: Ninjutsu [cost]. Activated from hand: pay cost, return an
    /// unblocked attacker to its owner's hand, put this card onto battlefield
    /// tapped and attacking the same target.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Ninjutsu)` for quick
    /// presence-checking without scanning all abilities.
    Ninjutsu { cost: ManaCost },
    /// CR 702.49d: Commander Ninjutsu [cost]. Same as ninjutsu but can also
    /// be activated from the command zone. Bypasses commander tax entirely.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::CommanderNinjutsu)` for
    /// quick presence-checking.
    CommanderNinjutsu { cost: ManaCost },
    /// CR 702.127: Aftermath. The second half of a split card. Can only be cast
    /// from the graveyard. When it leaves the stack after being cast from graveyard,
    /// it is exiled instead of going anywhere else.
    ///
    /// The aftermath half is a complete spell: it has its own name, mana cost,
    /// card type(s), spell effect, and targets. The card definition's top-level
    /// fields (name, mana_cost, types) describe the first (hand-castable) half.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Aftermath)` for quick
    /// presence-checking without scanning all abilities.
    ///
    /// At cast time from graveyard, the engine uses the aftermath half's mana_cost
    /// as the spell cost (alternative cost per CR 118.9). The aftermath half's
    /// effect is resolved instead of the card's first-half Spell effect.
    Aftermath {
        /// Name of the aftermath half (e.g., "Ribbons" for "Cut // Ribbons").
        name: String,
        /// Mana cost of the aftermath half (paid when casting from graveyard).
        cost: ManaCost,
        /// Card type of the aftermath half (Sorcery, Instant, etc.).
        card_type: CardType,
        /// The spell effect of the aftermath half.
        effect: Effect,
        /// Target requirements for the aftermath half's spell.
        targets: Vec<TargetRequirement>,
    },
    /// CR 702.176: Impending N--[cost]. You may cast this spell by paying [cost]
    /// rather than its mana cost. If you do, it enters with N time counters and
    /// isn't a creature while it has time counters. At the beginning of your end
    /// step, remove a time counter from it.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Impending)` for quick
    /// presence-checking without scanning all abilities.
    Impending { cost: ManaCost, count: u32 },
    /// CR 702.119: Emerge [cost]. The card may be cast by paying this cost and
    /// sacrificing a creature (alternative cost, CR 118.9). The total cost is
    /// reduced by the sacrificed creature's mana value.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Emerge)` for quick
    /// presence-checking without scanning all abilities.
    Emerge { cost: ManaCost },
    /// CR 702.137: Spectacle [cost]. The card may be cast by paying this cost
    /// instead of its mana cost if an opponent lost life this turn.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Spectacle)` for quick
    /// presence-checking without scanning all abilities.
    Spectacle { cost: ManaCost },
    /// CR 702.117: Surge [cost]. The card may be cast by paying this cost
    /// instead of its mana cost if you or a teammate has cast another spell this turn.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Surge)` for quick
    /// presence-checking without scanning all abilities.
    Surge { cost: ManaCost },
    /// CR 702.56a: Replicate [cost] -- optional additional cost paid any number of
    /// times when casting this spell. Each payment adds [cost] to the total mana cost.
    /// When you cast this spell, if a replicate cost was paid, copy it for each time
    /// the replicate cost was paid. Paying the replicate cost follows CR 601.2b and
    /// CR 601.2f-h.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Replicate)` for quick
    /// presence-checking without scanning all abilities.
    Replicate { cost: ManaCost },
    /// CR 702.148a: Cleave [cost]. The card may be cast by paying this cost instead
    /// of its mana cost (alternative cost, CR 118.9). When cleaved, the spell's
    /// square-bracketed text is removed -- modeled as conditional effect dispatch via
    /// `Condition::WasCleaved`. Cards with cleave should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Cleave)` for quick
    /// presence-checking without scanning all abilities.
    Cleave { cost: ManaCost },
    /// CR 702.47a: Splice onto [subtype] [cost]. When declared while casting a spell
    /// of the matching subtype (e.g., Arcane), the player pays [cost] as an additional
    /// cost and the target spell gains this card's rules text (`effect`). The spliced
    /// card stays in the player's hand after resolution (CR 702.47a). The spell only
    /// gains the rules text (CR 702.47c) -- not the name, mana cost, types, or other
    /// characteristics of the spliced card.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Splice)` for quick
    /// presence-checking without scanning all abilities.
    Splice {
        cost: ManaCost,
        onto_subtype: SubType,
        effect: Box<Effect>,
    },
    /// CR 702.42: Entwine [cost]. Optional additional cost that allows the caster to
    /// choose all modes of this modal spell instead of just one.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Entwine)` for quick
    /// presence-checking without scanning all abilities.
    /// `AbilityDefinition::Spell.modes` must be `Some(...)` for entwine to be meaningful.
    Entwine { cost: ManaCost },
    /// CR 702.120: Escalate [cost]. Additional cost paid for each mode chosen beyond
    /// the first. For N extra modes chosen, the escalate cost is paid N times.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Escalate)` for quick
    /// presence-checking without scanning all abilities.
    /// `AbilityDefinition::Spell.modes` must be `Some(...)` with `min_modes: 1,
    /// max_modes: <mode_count>` for escalate to be meaningful.
    Escalate { cost: ManaCost },
    /// CR 702.63a: Vanishing N -- "This permanent enters with N time counters on it."
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Vanishing(N))` for quick
    /// presence-checking without scanning all abilities.
    ///
    /// `count` is N (the number of time counters placed on ETB). When N=0 (CR 702.63b,
    /// Vanishing without a number), no counters are placed at ETB.
    Vanishing { count: u32 },
    /// CR 702.32a: Fading N -- "This permanent enters with N fade counters on it."
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Fading(N))` for quick
    /// presence-checking without scanning all abilities.
    ///
    /// `count` is N (the number of fade counters placed on ETB).
    /// Fading always has N >= 1 (unlike Vanishing which can be 0).
    Fading { count: u32 },
    /// CR 702.30a: Echo [cost] -- triggered ability that fires on the controller's
    /// first upkeep after the permanent enters.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Echo(cost))` for quick
    /// presence-checking without scanning all abilities.
    ///
    /// `cost` is the echo cost (ManaCost). For Urza block cards, this equals
    /// the card's mana cost (CR 702.30b).
    Echo { cost: ManaCost },
    /// CR 702.24a: Cumulative upkeep [cost] -- triggered ability that fires on
    /// the controller's upkeep. Adds an age counter, then requires payment of
    /// cost x age_count or sacrifice.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::CumulativeUpkeep(cost))` for
    /// quick presence-checking.
    CumulativeUpkeep { cost: CumulativeUpkeepCost },
    /// CR 702.59a: Recover [cost]. Triggered ability from the graveyard. When a
    /// creature is put into your graveyard from the battlefield, you may pay [cost].
    /// If you do, return this card from your graveyard to your hand. Otherwise,
    /// exile this card.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Recover)` for quick
    /// presence-checking without scanning all abilities. Discriminant 45.
    Recover { cost: ManaCost },
    /// CR 702.57: Forecast [cost], Reveal this card from your hand: [Effect].
    /// Activated ability from hand, only during owner's upkeep, once per turn.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Forecast)` for quick
    /// presence-checking without scanning all abilities.
    ///
    /// Discriminant 46.
    Forecast { cost: ManaCost, effect: Effect },
    /// CR 702.97: Scavenge [cost]. The card's scavenge ability can be activated
    /// from its owner's graveyard by paying this cost plus exiling the card. When
    /// the ability resolves, put +1/+1 counters equal to the card's power on
    /// target creature.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Scavenge)` for quick
    /// presence-checking without scanning all abilities. Discriminant 47.
    Scavenge { cost: ManaCost },
    /// CR 702.107: Outlast [cost]. A convenience variant that enrich_spec_from_def
    /// expands into an ActivatedAbility with: requires_tap=true, mana_cost=cost,
    /// sorcery_speed=true, effect=AddCounter(Source, PlusOnePlusOne, 1).
    /// Cards should also include AbilityDefinition::Keyword(KeywordAbility::Outlast)
    /// for quick presence-checking. Discriminant 48.
    Outlast { cost: ManaCost },
    /// CR 702.72: Champion an [object]. Two linked triggered abilities (CR 607.2k):
    /// 1. ETB: "sacrifice it unless you exile another [object] you control"
    /// 2. LTB: "return the exiled card to the battlefield under its owner's control"
    ///
    /// The filter specifies what can be championed (creature, Faerie, etc.).
    /// `enrich_spec_from_def` adds `KeywordAbility::Champion` to the keywords.
    ///
    /// Discriminant 49.
    Champion { filter: ChampionFilter },
    /// CR 702.95a: Soulbond -- two ETB triggered abilities plus static "as long as
    /// paired" grants. `enrich_spec_from_def` adds `KeywordAbility::Soulbond`.
    ///
    /// `grants` specifies continuous effects applied to BOTH paired creatures via
    /// `EffectDuration::WhilePaired` CEs registered at SoulbondTrigger resolution.
    ///
    /// Example (Wolfir Silverheart): `grants: [SoulbondGrant { layer: PtModify, modification: ModifyBoth(4) }]`
    ///
    /// Discriminant 50.
    Soulbond { grants: Vec<SoulbondGrant> },

    /// CR 702.102: Fuse. The second (right) half of a split card with fuse.
    /// When fused, both halves' effects execute at resolution (left first,
    /// then right — CR 702.102d).
    ///
    /// The card definition's top-level `name`, `mana_cost`, `types`, and
    /// `AbilityDefinition::Spell` describe the left half. This variant
    /// stores the right half's data.
    ///
    /// Cards with this ability should also include
    /// `AbilityDefinition::Keyword(KeywordAbility::Fuse)` for quick
    /// presence-checking without scanning all abilities.
    ///
    /// Discriminant 51.
    Fuse {
        /// Name of the right half (e.g., "Tear" for "Wear // Tear").
        name: String,
        /// Mana cost of the right half (added to left half's cost when fused — CR 702.102c).
        cost: ManaCost,
        /// Card type of the right half (Instant, Sorcery, etc.).
        card_type: CardType,
        /// The spell effect of the right half.
        effect: Effect,
        /// Target requirements for the right half's spell.
        targets: Vec<TargetRequirement>,
    },

    /// CR 701.59a: Collect evidence N — keyword action as additional cost.
    ///
    /// "As an additional cost to cast this spell, you may collect evidence N."
    /// (Or mandatory: "collect evidence N" without "you may".)
    ///
    /// The player exiles cards from their graveyard with total mana value >= N
    /// to pay this additional cost (CR 701.59a). Unlike Delve, the exiled cards
    /// do NOT reduce the mana cost — the full mana cost is still paid.
    ///
    /// `threshold`: the minimum total mana value of exiled cards (N).
    /// `mandatory`: if true, player MUST collect evidence; if false, it is optional.
    ///
    /// At resolution, `Condition::EvidenceWasCollected` checks whether the cost
    /// was paid, enabling "if evidence was collected" linked ability effects (CR 701.59c).
    ///
    /// Discriminant 53.
    CollectEvidence { threshold: u32, mandatory: bool },

    /// CR 702.157a: Squad -- the cost data for the squad additional cost.
    ///
    /// Pairs with `KeywordAbility::Squad` (presence marker). This variant carries
    /// the squad cost itself (e.g., `{2}` for "Squad {2}").
    ///
    /// At cast time: the player pays `cost` N times as an additional cost (CR 601.2b).
    /// N is stored in `CastSpell.squad_count` and `StackObject.squad_count`.
    /// At ETB: a trigger creates N token copies if N > 0 and the permanent still has Squad.
    ///
    /// Discriminant 54.
    Squad { cost: ManaCost },

    /// CR 207.2c: Bloodrush — ability word. Activated ability from hand.
    ///
    /// "{cost}, Discard this card: Target attacking creature gets +N/+M
    /// [and gains {keyword}] until end of turn."
    ///
    /// Bloodrush is an ability word (CR 207.2c), not a keyword — it has no
    /// individual CR entry. The underlying rules are CR 602 (activated abilities).
    ///
    /// The card is discarded as cost before the ability goes on the stack (CR 602.2b).
    /// If the ability is countered (e.g., by Stifle), the card remains in the
    /// graveyard — it was already consumed as cost.
    ///
    /// `cost`: mana cost of the bloodrush activation.
    /// `power_boost`: the +N to power until end of turn.
    /// `toughness_boost`: the +M to toughness until end of turn.
    /// `grants_keyword`: optional keyword granted to the target until end of turn.
    ///
    /// Discriminant 52.
    Bloodrush {
        cost: ManaCost,
        power_boost: i32,
        toughness_boost: i32,
        grants_keyword: Option<KeywordAbility>,
    },
    /// CR 702.175a: Offspring -- the cost data for the offspring additional cost.
    ///
    /// Pairs with `KeywordAbility::Offspring` (presence marker). This variant carries
    /// the offspring cost itself (e.g., `{2}` for "Offspring {2}").
    ///
    /// At cast time: the player optionally pays `cost` once as an additional cost (CR 601.2b).
    /// Binary: paid or not paid. If paid, `CastSpell.offspring_paid` is true.
    /// At ETB: a trigger creates 1 token copy (except 1/1) if paid and permanent still has Offspring.
    ///
    /// Discriminant 55.
    Offspring { cost: ManaCost },
    /// CR 702.174a: Gift a [something] -- two linked abilities.
    ///
    /// First ability: "As an additional cost to cast this spell, you may choose an opponent."
    /// Second ability (permanent): "When this enters, if its gift cost was paid, [effect]."
    /// Second ability (instant/sorcery): "If this spell's gift cost was paid, [effect]."
    ///
    /// The `gift_type` determines what the chosen opponent receives (CR 702.174d-i).
    ///
    /// Discriminant 56.
    Gift { gift_type: GiftType },
    /// CR 702.99a: Cipher -- marks an instant or sorcery as having cipher.
    ///
    /// Cipher is two linked abilities: at resolution the controller may exile the
    /// card encoded on a creature they control; while encoded that creature has
    /// "Whenever this creature deals combat damage to a player, you may copy the
    /// encoded card and cast the copy without paying its mana cost."
    ///
    /// This variant is a marker -- the actual encoding logic lives in resolution.rs
    /// and the trigger dispatch lives in abilities.rs.
    ///
    /// Discriminant 57.
    Cipher,
    /// CR 702.151a: Reconfigure [cost] -- the cost for both attach and unattach abilities.
    ///
    /// `enrich_spec_from_def` expands this into TWO `ActivatedAbility` entries:
    /// 1. Attach: "[Cost]: Attach this permanent to another target creature you control.
    ///    Activate only as a sorcery."
    /// 2. Unattach: "[Cost]: Unattach this permanent. Activate only as a sorcery."
    ///
    /// Cards should also include `AbilityDefinition::Keyword(KeywordAbility::Reconfigure)`
    /// for quick presence-checking.
    ///
    /// Discriminant 58.
    Reconfigure { cost: ManaCost },
    /// CR 702.140a: Mutate [cost] — the alternative casting cost for a mutate spell.
    ///
    /// When casting a spell with mutate, the player may pay this cost instead of the
    /// spell's mana cost. Doing so requires choosing a target non-Human creature the
    /// caster owns on the battlefield (CR 702.140a).
    ///
    /// Cards should also include `AbilityDefinition::Keyword(KeywordAbility::Mutate)`
    /// for quick presence-checking.
    ///
    /// Discriminant 59.
    MutateCost { cost: ManaCost },
    /// CR 702.146a: Disturb [cost] — cast this card transformed from your graveyard
    /// by paying [cost] rather than its mana cost.
    ///
    /// A resolving spell that was cast using its disturb ability enters the battlefield
    /// with its back face up (CR 702.146b). The back face has an ability that instructs
    /// its controller to exile if it would be put into a graveyard from anywhere (ruling).
    ///
    /// Cards should also include `AbilityDefinition::Keyword(KeywordAbility::Disturb)`
    /// for quick presence-checking.
    ///
    /// Discriminant 60.
    Disturb { cost: ManaCost },
    /// CR 702.167a: Craft with [materials] [cost] — "[Cost], Exile this permanent,
    /// Exile [materials] from among permanents you control and/or cards in your graveyard:
    /// Return this card to the battlefield transformed under its owner's control.
    /// Activate only as a sorcery."
    ///
    /// Cards should also include `AbilityDefinition::Keyword(KeywordAbility::Craft)`
    /// for quick presence-checking.
    ///
    /// Discriminant 61.
    Craft {
        cost: ManaCost,
        materials: CraftMaterials,
    },
    /// CR 702.37a: Morph [cost] — you may cast this card face-down as a 2/2 creature
    /// for {3} instead of paying its mana cost. At any time you have priority, you may
    /// turn this face-down permanent face up by paying [cost].
    ///
    /// This variant carries the turn-face-up cost. The presence marker is
    /// `AbilityDefinition::Keyword(KeywordAbility::Morph)`.
    ///
    /// Discriminant 62.
    Morph { cost: ManaCost },
    /// CR 702.37b: Megamorph [cost] — variant of morph. When turned face up via its
    /// megamorph cost, the permanent also gets a +1/+1 counter.
    ///
    /// This variant carries the turn-face-up cost. The presence marker is
    /// `AbilityDefinition::Keyword(KeywordAbility::Megamorph)`.
    ///
    /// Discriminant 63.
    Megamorph { cost: ManaCost },
    /// CR 702.168a: Disguise [cost] — like morph but the face-down permanent has
    /// ward {2} while face-down. Turn face up by paying the disguise cost.
    ///
    /// This variant carries the turn-face-up cost. The presence marker is
    /// `AbilityDefinition::Keyword(KeywordAbility::Disguise)`.
    ///
    /// Discriminant 64.
    Disguise { cost: ManaCost },
}

/// Extra data for `AltCastAbility` variants that need more than just a `ManaCost`.
///
/// Most alt-cast abilities (Flashback, Dash, etc.) only need a cost. Escape additionally
/// needs an exile count, and Prototype additionally needs power/toughness overrides.
/// This enum captures those extra fields without bloating the common case.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AltCastDetails {
    /// CR 702.138: Escape requires exiling N other cards from graveyard.
    Escape { exile_count: u32 },
    /// CR 702.160 / CR 718: Prototype overrides power and toughness.
    Prototype { power: i32, toughness: i32 },
}

/// CR 702.167b: Describes what can be exiled as materials for a Craft activated ability.
///
/// "If an object in the [materials] is described using only a card type or subtype
/// without 'card,' it refers to either a permanent on the battlefield or a card in a
/// graveyard of that type."
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftMaterials {
    /// Craft with N artifacts — exile N artifacts from battlefield/graveyard.
    Artifacts(u32),
    /// Craft with N creatures — exile N creatures from battlefield/graveyard.
    Creatures(u32),
    /// Craft with N lands — exile N lands from battlefield/graveyard.
    Lands(u32),
    /// Craft with N cards of any type — exile N permanents/cards.
    AnyCards(u32),
}

/// CR 702.174d-i: The specific gift given to the chosen opponent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GiftType {
    /// CR 702.174d: "The chosen player creates a Food token."
    Food,
    /// CR 702.174e: "The chosen player draws a card."
    Card,
    /// CR 702.174f: "The chosen player creates a tapped 1/1 blue Fish creature token."
    TappedFish,
    /// CR 702.174h: "The chosen player creates a Treasure token."
    Treasure,
    /// CR 702.174i: "The chosen player creates an 8/8 blue Octopus creature token."
    Octopus,
    /// CR 702.174g: "The chosen player takes an extra turn after this one."
    ExtraTurn,
}

/// A continuous effect granted by soulbond to both paired creatures (CR 702.95a).
///
/// Registered as a `ContinuousEffect` with `EffectDuration::WhilePaired` when the
/// SoulbondTrigger resolves. Active as long as both creatures remain paired.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoulbondGrant {
    /// Which layer the modification applies in (CR 613.1).
    pub layer: EffectLayer,
    /// What the modification does (e.g., ModifyBoth(4) for +4/+4).
    pub modification: LayerModification,
}

// ── Cost ─────────────────────────────────────────────────────────────────────

/// The cost to activate an ability or cast a spell (CR 118).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cost {
    /// Pay a mana cost (CR 202).
    Mana(ManaCost),
    /// Tap the source permanent (CR 602.2).
    Tap,
    /// Sacrifice this permanent (CR 602.2).
    SacrificeSelf,
    /// Sacrifice another permanent matching the filter (CR 602.2).
    Sacrifice(TargetFilter),
    /// Pay N life (CR 119).
    PayLife(u32),
    /// Discard a card.
    DiscardCard,
    /// Discard this card from hand as a cost (Channel — CR 702.34).
    /// Implies the ability is activated from hand, not the battlefield.
    DiscardSelf,
    /// CR 701.61: Sacrifice a Food you control OR exile three cards from your graveyard.
    Forage,
    /// Multiple costs, all paid simultaneously (CR 601.2g).
    Sequence(Vec<Cost>),
}

/// CR 606.4: The cost to activate a loyalty ability — add or remove loyalty counters.
///
/// CR 606.5: Multiple loyalty costs are combined into a single add/remove.
/// CR 606.6: Negative costs can't be activated unless the permanent has enough counters.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LoyaltyCost {
    /// "+N" — add N loyalty counters (CR 606.4).
    Plus(u32),
    /// "−N" — remove N loyalty counters (CR 606.4).
    Minus(u32),
    /// "0" — no loyalty counters added or removed.
    Zero,
    /// "−X" — remove X loyalty counters, where X is chosen by the player.
    /// X must be at least 0; the permanent must have at least X counters (CR 606.6).
    MinusX,
}

// ── Effect ────────────────────────────────────────────────────────────────────

/// CR 106.12: Restriction on what a mana payment can be spent on.
///
/// Mana produced with a restriction can only be used to pay costs that match
/// the restriction. If the mana would be spent on something that doesn't match,
/// it cannot be used for that payment.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ManaRestriction {
    /// "Spend this mana only to cast creature spells."
    CreatureSpellsOnly,
    /// "Spend this mana only to cast [subtype] spells." (e.g., Dragon, Elf)
    SubtypeOnly(SubType),
    /// "Spend this mana only to cast [subtype] or [subtype] spells." (e.g., Dragon or Omen)
    SubtypeOrSubtype(SubType, SubType),
    /// "Spend this mana only to cast creature spells of the chosen type."
    /// Uses `chosen_creature_type` from the source permanent.
    ChosenTypeCreaturesOnly,
    /// "Spend this mana only to cast spells of the chosen type."
    /// Uses `chosen_creature_type` from the source permanent.
    ChosenTypeSpellsOnly,
}

/// A recursive effect primitive: the engine's internal DSL for card behavior.
///
/// Effects are executed by `effects::execute_effect`. Every effect that changes
/// game state emits `GameEvent`s. Effects are composed with `Sequence`, `Conditional`,
/// `Choose`, and `ForEach`.
///
/// See architecture doc Section 3.7 for the full list of primitives.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// CR 702.101a: Each opponent of the controller loses `amount` life, and the
    /// controller gains life equal to the total life actually lost by all opponents.
    ///
    /// This is NOT the same as LoseLife + GainLife because the gain depends on
    /// the actual life change, not the intended loss (relevant when an opponent's
    /// life total can't change, e.g., Platinum Emperion).
    ///
    /// The "controller" is the controller of the spell or ability that created
    /// this effect (from EffectContext).
    DrainLife { amount: EffectAmount },

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
    /// CR 701.16a: "Investigate" means "Create a Clue token."
    ///
    /// Creates `count` Clue tokens sequentially (ruling 2024-06-07:
    /// "If you're instructed to investigate multiple times, those actions
    /// are sequential, meaning you'll create that many Clue tokens one
    /// at a time."). Does nothing when count resolves to 0.
    Investigate { count: EffectAmount },
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
    /// Add N mana of a specific color, where N is dynamic.
    /// Used for "Add {G} for each creature you control" (Gaea's Cradle),
    /// "Add {B} for each Swamp you control" (Cabal Coffers), etc.
    AddManaScaled {
        player: PlayerTarget,
        color: ManaColor,
        count: EffectAmount,
    },
    /// CR 106.12: Add mana with a spending restriction.
    ///
    /// Restricted mana can only be used to pay costs matching the restriction.
    /// Used by Cavern of Souls, Haven of the Spirit Dragon, Gnarlroot Trapper, etc.
    AddManaRestricted {
        player: PlayerTarget,
        mana: ManaPool,
        restriction: ManaRestriction,
    },
    /// CR 106.12: Add one mana of any color with a spending restriction.
    AddManaAnyColorRestricted {
        player: PlayerTarget,
        restriction: ManaRestriction,
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

    /// CR 701.39: Bolster N -- "Choose a creature you control with the least
    /// toughness or tied for least toughness among creatures you control. Put
    /// N +1/+1 counters on that creature."
    ///
    /// Bolster does NOT target (ruling 2014-11-24). The creature is chosen at
    /// resolution time using layer-aware toughness. If the controller has no
    /// creatures, nothing happens.
    /// Deterministic fallback for tied toughness: choose smallest ObjectId.
    Bolster {
        /// The player who controls the bolster effect (determines which
        /// creatures are eligible).
        player: PlayerTarget,
        /// Number of +1/+1 counters to place.
        count: EffectAmount,
    },

    /// CR 701.47a: Amass [subtype] N -- If you don't control an Army creature,
    /// create a 0/0 black [subtype] Army creature token. Choose an Army creature
    /// you control. Put N +1/+1 counters on that creature. If it isn't a
    /// [subtype], it becomes a [subtype] in addition to its other types.
    ///
    /// CR 701.47b: Always completes even if some or all actions were impossible.
    /// CR 701.47d: Older cards without a subtype use "Zombie" per Oracle errata.
    /// Deterministic fallback for multiple Armies: choose smallest ObjectId.
    Amass {
        /// The creature subtype to add (e.g., "Zombie", "Orc").
        subtype: String,
        /// Number of +1/+1 counters to place.
        count: EffectAmount,
    },

    // ── Zone ─────────────────────────────────────────────────────────────────
    /// Move an object to a zone (CR 400).
    MoveZone {
        target: EffectTarget,
        to: ZoneTarget,
    },

    // ── Library ─────────────────────────────────────────────────────────────
    /// CR 701.18: Scry N — look at top N cards of your library, then put any
    /// number on the bottom and the rest on top in any order.
    ///
    /// M9.4 deterministic fallback: looks at top N cards of the library and
    /// puts them on the bottom in ObjectId ascending order (interactive
    /// ordering deferred to M10+).
    Scry {
        player: PlayerTarget,
        count: EffectAmount,
    },
    /// CR 701.25: Surveil N -- look at the top N cards of your library, then put
    /// any number of them into your graveyard and the rest on top in any order.
    ///
    /// Deterministic fallback: puts ALL top N cards into the graveyard
    /// (interactive ordering deferred to M10+). This mirrors the Scry fallback
    /// but sends cards to the graveyard instead of the bottom of the library.
    /// CR 701.25c: Surveil 0 produces no event.
    Surveil {
        player: PlayerTarget,
        count: EffectAmount,
    },
    /// CR 701.50: Connive -- a permanent's controller draws a card, then discards
    /// a card. If a nonland card is discarded this way, put a +1/+1 counter on
    /// the conniving permanent.
    ///
    /// Deterministic fallback: discards the first card in hand (alphabetically).
    /// CR 701.50e: Connive N draws N and discards N, placing counters equal to
    /// the number of nonland cards discarded.
    Connive {
        target: EffectTarget,
        count: EffectAmount,
    },
    /// CR 701.20: Put N cards from a zone onto the top of a player's library.
    ///
    /// M7: Deterministic — moves the first N objects (by ObjectId ascending) from
    /// the source zone. M9+: interactive (player chooses which cards to put back).
    PutOnLibrary {
        player: PlayerTarget,
        count: EffectAmount,
        /// The zone to take cards from (typically the player's hand).
        from: ZoneTarget,
    },
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
    /// CR 701.17a: The specified player sacrifices `count` permanents they control.
    ///
    /// If the player controls fewer than `count` permanents, they sacrifice all
    /// permanents they control. The player chooses which permanents to sacrifice
    /// (deterministic fallback: sacrifice in ObjectId ascending order). Sacrifice
    /// ignores indestructible (CR 701.17a). Used by Annihilator (CR 702.86a) and
    /// other "target player sacrifices N permanents" effects.
    SacrificePermanents {
        player: PlayerTarget,
        count: EffectAmount,
    },
    /// CR 701.38: Goad — target creature must attack each combat if able, and
    /// must attack a player other than the goading player if able.
    ///
    /// M9.4: marks the creature as goaded until the start of the goaded creature
    /// controller's next turn. Enforcement of attack requirements is deferred
    /// to a future session.
    Goad { target: EffectTarget },
    /// CR 701.60a: Suspect -- set the suspected designation on the target permanent.
    /// A suspected permanent has menace and "This creature can't block" for as long
    /// as it's suspected (CR 701.60c). The designation is NOT a copiable value
    /// (CR 701.60b). Suspecting an already-suspected permanent is a no-op (CR 701.60d).
    Suspect { target: EffectTarget },
    /// CR 701.60a: Unsuspect -- remove the suspected designation from the target
    /// permanent. Clears `is_suspected`, removing the menace grant and unblocking
    /// the can't-block restriction.
    Unsuspect { target: EffectTarget },
    /// CR 724.1/724.3: Target player becomes the monarch.
    ///
    /// Sets `state.monarch` to the target player. If another player was the monarch,
    /// they cease to be the monarch (CR 724.3). Emits `GameEvent::PlayerBecameMonarch`.
    /// Inherent triggers (EOT draw, combat damage steal) are handled in turn_actions.rs
    /// and combat.rs respectively.
    BecomeMonarch { player: PlayerTarget },
    /// Set `chosen_creature_type` on the source permanent (CR 106.12 support).
    ///
    /// Used by lands like Cavern of Souls: "As this enters, choose a creature type."
    /// The chosen type is stored on the `GameObject` and referenced by
    /// `ManaRestriction::ChosenTypeCreaturesOnly` / `ChosenTypeSpellsOnly`.
    ///
    /// In the deterministic engine, the choice is made automatically: picks the most
    /// common creature subtype among creatures the controller controls, or defaults
    /// to the `default` field if no creatures are on the battlefield.
    ChooseCreatureType {
        /// Default creature type if no creatures are controlled (deterministic fallback).
        default: SubType,
    },
    /// CR 701.19a: Regenerate -- create a one-shot regeneration shield on the target
    /// permanent. The next time that permanent would be destroyed this turn, instead
    /// remove all damage marked on it, tap it, and remove it from combat (if in combat).
    /// The shield lasts until used or until end of turn (cleanup step).
    Regenerate { target: EffectTarget },
    /// CR 702.6a / CR 701.3a: Attach the source Equipment to the target creature.
    ///
    /// Used as the effect of the Equip activated ability. On resolution:
    /// 1. Detach Equipment from any previously equipped creature (CR 301.5c).
    /// 2. Set `source.attached_to = target` and add source to `target.attachments`.
    /// 3. Update Equipment timestamp (CR 701.3c, CR 613.7e).
    ///
    /// If the target is no longer legal at resolution (left battlefield, no longer
    /// a creature, no longer controlled by the activating player), the ability
    /// fizzles via the standard target legality check in resolution.rs.
    AttachEquipment {
        /// The equipment to attach. Should be `EffectTarget::Source`.
        equipment: EffectTarget,
        /// The creature to attach to. Should be `EffectTarget::DeclaredTarget { index: 0 }`.
        target: EffectTarget,
    },
    /// CR 702.67a / CR 701.3a: Attach the source Fortification to the target land.
    ///
    /// Used as the effect of the Fortify activated ability. On resolution:
    /// 1. Detach Fortification from any previously fortified land (CR 301.6 via 301.5c analog).
    /// 2. Set `source.attached_to = target` and add source to `target.attachments`.
    /// 3. Update Fortification timestamp (CR 701.3c, CR 613.7e).
    ///
    /// If the target is no longer legal at resolution (left battlefield, no longer
    /// a land, no longer controlled by the activating player), the ability fizzles.
    AttachFortification {
        /// The fortification to attach. Should be `EffectTarget::Source`.
        fortification: EffectTarget,
        /// The land to attach to. Should be `EffectTarget::DeclaredTarget { index: 0 }`.
        target: EffectTarget,
    },
    /// CR 702.151a: Unattach an Equipment from its currently equipped creature.
    ///
    /// Used as the effect of the Reconfigure unattach activated ability. On resolution:
    /// 1. Verify the equipment is on the battlefield and has `attached_to` set.
    /// 2. Clear `attached_to` on the equipment.
    /// 3. Remove equipment from the target's `attachments`.
    /// 4. Clear `is_reconfigured` flag (CR 702.151b: creature type is restored).
    ///
    /// The equipment remains on the battlefield as an unattached permanent.
    DetachEquipment {
        /// The equipment to unattach. Should be `EffectTarget::Source`.
        equipment: EffectTarget,
    },
    /// CR 702.92a: Create a token and immediately attach the source Equipment to it.
    ///
    /// Used by Living Weapon. The token creation and attachment happen as a single
    /// atomic operation -- SBAs are not checked between token creation and attachment
    /// (ruling: "The Germ token enters the battlefield as a 0/0 creature and the
    /// Equipment becomes attached to it before state-based actions would cause the
    /// token to die.").
    ///
    /// If multiple tokens would be created (e.g., Doubling Season), the Equipment
    /// attaches to the first one. The others are subject to SBAs normally.
    CreateTokenAndAttachSource { spec: TokenSpec },
    /// CR 701.34a: Proliferate -- choose any number of permanents and/or players
    /// that have a counter, then give each one additional counter of each kind
    /// that permanent or player already has.
    ///
    /// Simplified implementation: auto-selects all eligible permanents on the
    /// battlefield and all players with counters (controller "chooses all").
    /// Interactive selection deferred to M10+.
    ///
    /// Always emits a Proliferated event (even with 0 eligible targets) to
    /// support "whenever you proliferate" triggers (ruling 2023-02-04).
    Proliferate,
    /// CR 701.57a: Discover N — exile cards from the top of the specified player's
    /// library until you exile a nonland card with mana value N or less. You may
    /// cast that card without paying its mana cost. If you don't cast it, put that
    /// card into your hand. Put the remaining exiled cards on the bottom of your
    /// library in a random order.
    ///
    /// Key differences from Cascade (CR 702.85):
    /// - MV threshold is <= N (Cascade uses < spell_MV)
    /// - Declined card goes to hand (Cascade puts it on library bottom)
    /// - Uses a fixed N, not the spell's own MV
    ///
    /// CR 701.57b: A player has "discovered" even if some or all actions were
    /// impossible (e.g., empty library).
    ///
    /// Deterministic fallback: always casts the discovered card (interactive choice
    /// "may cast" deferred to M10+). If the card cannot be cast, it goes to hand.
    Discover {
        /// The player who performs the discover action (usually the controller).
        player: PlayerTarget,
        /// The discover value N — qualifying cards have mana value <= N.
        n: u32,
    },
    /// CR 702.75a / CR 607.2a: Play the card exiled face-down by this
    /// permanent's Hideaway ETB trigger without paying its mana cost.
    ///
    /// At resolution: find the card in the exile zone where
    /// `exiled_by_hideaway == Some(source_id)` and `status.face_down == true`,
    /// turn it face-up, then move it to the battlefield (if a permanent) or
    /// handle it as a cast spell.
    ///
    /// Deterministic fallback: always plays the card (does not decline).
    /// If no matching exiled card is found, the ability does nothing.
    ///
    /// CR 118.9: Playing without paying the mana cost is an alternative cost.
    PlayExiledCard,
    /// CR 701.40a: Manifest the top card of a player's library. The card is placed
    /// onto the battlefield face-down as a 2/2 creature with no text, no name, no
    /// subtypes, and no mana cost. ETB abilities do not trigger (CR 708.3).
    ///
    /// If the library is empty, the effect does nothing (CR 701.40f).
    /// If the card cannot enter the battlefield for any reason, it isn't manifested.
    Manifest {
        /// The player whose top library card is manifested (usually the controller).
        player: PlayerTarget,
    },
    /// CR 701.58a: Cloak the top card of a player's library. Like Manifest (CR 701.40a),
    /// but the face-down creature also has ward {2} (CR 701.58a) while it is face-down.
    ///
    /// If the library is empty, the effect does nothing (CR 701.58f via 701.40f).
    Cloak {
        /// The player whose top library card is cloaked (usually the controller).
        player: PlayerTarget,
    },
    /// No effect (used in Conditional branches, or for keyword-only cards).
    Nothing,

    /// CR 701.49: Venture into the dungeon.
    ///
    /// The player ventures into the dungeon (CR 701.49a-c). Uses the standard
    /// three-case logic: no dungeon in command zone (enter new dungeon), not on
    /// bottommost room (advance marker), or on bottommost room (complete dungeon,
    /// start new one). Deterministic fallback: enter LostMineOfPhandelver when
    /// choosing a new dungeon. Room abilities push a RoomAbility SOK onto the stack.
    VentureIntoDungeon,

    /// CR 725.2: Take the initiative.
    ///
    /// Sets `has_initiative = Some(controller)` on GameState, emits `InitiativeTaken`,
    /// and immediately ventures into the Undercity (CR 725.2: "that player ventures
    /// into the Undercity" as an inherent triggered ability of taking the initiative).
    TakeTheInitiative,

    /// CR 701.54a-c: "The Ring tempts you."
    ///
    /// Advances the controller's ring level (cap at 4), emits `RingTempted`, then
    /// the controller chooses a creature they control as their ring-bearer.
    /// Deterministic fallback: choose the creature with the lowest ObjectId.
    /// If no creature is available, ring level still advances but no ring-bearer is chosen.
    TheRingTemptsYou,

    /// CR 701.42a: Meld the source permanent with its meld pair partner.
    ///
    /// Exile this permanent and the named partner permanent (must both be on the
    /// battlefield, owned and controlled by the same player). Then put them onto
    /// the battlefield combined as the melded permanent. The melded permanent's
    /// characteristics come from the meld pair's back_face (CR 712.8g).
    ///
    /// CR 701.42c: If the pair cannot be melded (partner not present, different
    /// controllers, etc.), nothing happens — both stay in their current zone.
    Meld,
}

// ── Effect Targets ────────────────────────────────────────────────────────────

/// How an effect identifies its primary target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Count permanents on the battlefield matching a filter.
    /// Used for "number of creatures you control", "number of lands you control", etc.
    PermanentCount {
        filter: TargetFilter,
        controller: PlayerTarget,
    },
    /// CR 700.5: Devotion to a color — count mana symbols of that color in the mana costs
    /// of permanents you control.
    DevotionTo(Color),
    /// Count counters of a given type on a target permanent.
    /// Used for "draw cards equal to the number of +1/+1 counters on this creature", etc.
    CounterCount {
        target: EffectTarget,
        counter: CounterType,
    },
}

// ── Target Requirements ───────────────────────────────────────────────────────

/// A legal target type for a spell or ability (CR 601.2c, CR 115).
///
/// This is declared on the spell/ability at definition time and used to validate
/// targets when the spell or ability is put on the stack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// "target noncreature spell" — must be on the stack and match the filter.
    TargetSpellWithFilter(TargetFilter),
    /// "target [type] card from your graveyard" — card in controller's graveyard (CR 115.1).
    TargetCardInYourGraveyard(TargetFilter),
    /// "target [type] card from a graveyard" — card in any player's graveyard (CR 115.1).
    TargetCardInGraveyard(TargetFilter),
}

/// A filter on game objects, used for target requirements and `SearchLibrary`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetFilter {
    /// Max power (inclusive). None = no restriction.
    pub max_power: Option<i32>,
    /// Min power (inclusive). None = no restriction.
    pub min_power: Option<i32>,
    /// Exactly one of these card types. None = no restriction.
    pub has_card_type: Option<CardType>,
    /// Must have all these keywords.
    pub has_keywords: OrdSet<KeywordAbility>,
    /// Must be one of these colors (inclusion — object must have at least one). None = no restriction.
    pub colors: Option<OrdSet<Color>>,
    /// Must NOT be any of these colors (exclusion — object must share none). None = no restriction.
    /// Used for cards like Doom Blade ("target non-black creature").
    pub exclude_colors: Option<OrdSet<Color>>,
    /// Must not be a creature.
    pub non_creature: bool,
    /// Must not be a land.
    pub non_land: bool,
    /// Must be basic.
    pub basic: bool,
    /// Controller constraint.
    pub controller: TargetController,
    /// Subtype constraint (single — must have this subtype).
    pub has_subtype: Option<SubType>,
    /// Subtype constraint (OR semantics — must have at least one of these subtypes).
    /// Used for "Vampire or Wizard creature card" (Bloodline Necromancer).
    #[serde(default)]
    pub has_subtypes: Vec<SubType>,
    /// Must have exactly this name (exact match). None = no restriction.
    /// Used by "Partner with" ETB search (CR 702.124j) and similar
    /// "search for a card named [name]" effects.
    #[serde(default)]
    pub has_name: Option<String>,
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    ///
    /// If `during_opponent_turn` is true, the trigger only fires when it is NOT
    /// the controller's own turn (CR 603.1 — condition checked at trigger time).
    /// Used for Alela, Cunning Conqueror's "first spell during each opponent's turn".
    WheneverYouCastSpell {
        /// If true, only fires during opponents' turns (not controller's own turn).
        during_opponent_turn: bool,
    },
    /// "Whenever you gain life."
    WheneverYouGainLife,
    /// "Whenever you draw a card."
    WheneverYouDrawACard,
    /// CR 702.21a: "Whenever this permanent becomes the target of a spell or ability
    /// an opponent controls." Used by the Ward keyword.
    WhenBecomesTargetByOpponent,
    /// CR 701.25d: "Whenever you surveil."
    ///
    /// Fires after the surveil action is complete (CR 701.25d), once per surveil
    /// action regardless of how many cards were looked at. Does NOT fire when
    /// surveilling 0 (CR 701.25c — no surveil event occurs).
    WheneverYouSurveil,
    /// CR 701.50b: "Whenever this creature connives."
    ///
    /// Fires after the connive action completes (CR 701.50b), even if some or all
    /// actions were impossible. Fires even if the creature has left the battlefield
    /// before the event is processed (Psychic Pickpocket ruling, 2022-04-29).
    WhenConnives,
    /// CR 701.16a: "Whenever you investigate."
    ///
    /// Fires after the investigate action completes (CR 701.16a), once per
    /// investigate action. Does NOT fire when investigating 0 (no Investigated
    /// event is emitted in that case).
    WheneverYouInvestigate,
    /// CR 702.104b: "When ~ enters, if tribute wasn't paid, ..."
    ///
    /// Fires inline at ETB time (via fire_when_enters_triggered_effects) only if
    /// the entering permanent has `tribute_was_paid == false`. This is an
    /// intervening-if condition: the trigger checks the condition at fire time
    /// and again at resolution (CR 603.4).
    TributeNotPaid,
    /// CR 207.2c / CR 120.3: "Whenever this creature is dealt damage" -- Enrage ability word.
    ///
    /// Fires when the source creature receives > 0 damage from any source (combat
    /// or non-combat). Per CR 603.2g, if all damage is prevented (final amount = 0),
    /// the trigger does not fire. Per ruling 2018-01-19, if multiple sources deal
    /// damage simultaneously (e.g., combat), triggers only once per damage event.
    WhenDealtDamage,
    /// CR 702.55c: "When the creature [this card] haunts dies."
    ///
    /// Fires from exile when the creature this haunt card is haunting dies.
    /// The trigger condition is matched at CreatureDied time by scanning exiled objects
    /// with haunting_target == dying creature's pre-death ObjectId.
    /// The HauntedCreatureDiesTrigger SOK resolves this effect.
    HauntedCreatureDies,
    /// CR 702.140d: "Whenever this creature mutates."
    ///
    /// Fires on the merged permanent itself (same ObjectId as the target before merging,
    /// per CR 729.2c) after a successful mutate merge. Converted by `enrich_spec_from_def`
    /// to `TriggerEvent::SelfMutates` so `check_triggers` can dispatch it via
    /// `GameEvent::CreatureMutated`.
    WhenMutates,
    /// CR 708.8: "When this permanent is turned face up."
    ///
    /// Fires when a face-down permanent (morph, megamorph, disguise, manifest, or cloak)
    /// is turned face up via `Command::TurnFaceUp`. Unlike ETB triggers, these DO fire
    /// when a permanent is turned face up — the permanent already entered the battlefield
    /// face-down, so ETB was suppressed at that time (CR 708.3).
    ///
    /// Dispatched via `GameEvent::PermanentTurnedFaceUp` → `TriggerEvent::SelfTurnedFaceUp`
    /// in `check_triggers`. Resolves as a `TurnFaceUpTrigger` stack object.
    WhenTurnedFaceUp,
    /// CR 701.54d: "Whenever the Ring tempts you."
    ///
    /// Fires when `GameEvent::RingTempted` is emitted for the ability controller.
    /// The ring tempts a player even when no creature is available (CR 701.54d), so
    /// this trigger fires regardless of whether a ring-bearer was chosen.
    WheneverRingTemptsYou,
    /// "Whenever this permanent becomes tapped" — fires on ANY tap (mana ability,
    /// combat, opponent's effect, etc.). Used by City of Brass.
    ///
    /// Dispatched via `GameEvent::PermanentTapped` → `TriggerEvent::SelfBecomesTapped`
    /// in `check_triggers`.
    WhenSelfBecomesTapped,
}

// ── Conditions ────────────────────────────────────────────────────────────────

/// A boolean condition checked at trigger time or in Conditional effects (CR 603.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// "if ~ has no [counter] counters on it" — negation of SourceHasCounters.
    ///
    /// Used by Adapt (CR 701.46a): the condition check happens at resolution time,
    /// not at activation time (ruling 2019-01-25). If the source has zero counters
    /// of the given type, the condition is true and counters are placed.
    /// If the source has any counters of the given type, the condition is false
    /// and the counters are NOT placed (but the ability still resolved; mana was spent).
    SourceHasNoCountersOfType { counter: CounterType },
    /// Always true (for Conditional branches that always fire).
    Always,
    /// CR 702.33d: "if this spell was kicked" — true when `kicker_times_paid > 0`.
    ///
    /// Checked at resolution time by reading the `kicker_times_paid` field on the
    /// `EffectContext` (for spells) or on the `GameObject` (for ETB triggers on
    /// permanents that entered kicked).
    WasKicked,
    /// CR 702.96a: "if this spell's overload cost was paid" — true when
    /// `was_overloaded` is set on the EffectContext or StackObject.
    ///
    /// Checked at resolution time. Used in card definitions to branch between
    /// single-target and all-matching-permanents effects (analogous to WasKicked).
    WasOverloaded,
    /// CR 702.166b: "if this spell was bargained" — true when
    /// `was_bargained` is set on the EffectContext or StackObject.
    ///
    /// Checked at resolution time. Used in card definitions to branch between
    /// base and enhanced effects (analogous to WasKicked).
    WasBargained,
    /// CR 702.148a: "if this spell's cleave cost was paid" — true when
    /// `was_cleaved` is set on the EffectContext. Checked at resolution time.
    /// Used in card definitions to branch between restricted (normal cast) and
    /// broadened (cleaved cast) effects.
    WasCleaved,
    /// CR 207.2c (Corrupted ability word): "if an opponent has N or more poison counters."
    ///
    /// Checked at the current game state. In multiplayer Commander, true if ANY living
    /// opponent of the source object's controller has >= N poison counters.
    /// Eliminated opponents (has_lost == true) are ignored.
    OpponentHasPoisonCounters(u32),
    /// CR 701.59c: "if evidence was collected" — true when the collect evidence
    /// additional cost was paid for this spell (CR 701.59a).
    ///
    /// This is a linked ability check (CR 607): only the specific spell that paid
    /// the collect evidence cost will have `evidence_collected == true`.
    /// Checked at resolution time via `EffectContext.evidence_collected`.
    EvidenceWasCollected,
    /// CR 702.174b: "if this spell's gift cost was paid" / "if its gift cost was paid"
    /// True when gift_opponent was chosen at cast time. Checked at resolution time
    /// for instants/sorceries; at ETB trigger resolution for permanents.
    GiftWasGiven,

    /// CR 309.7: "as long as you've completed a dungeon" / "if you've completed a dungeon"
    ///
    /// True when the effect controller's `dungeons_completed > 0`. Used for permanents
    /// like Nadaar, Selfless Paladin that gain abilities after completing any dungeon.
    CompletedADungeon,

    /// CR 309.7 (specific dungeon variant): "if you haven't completed [dungeon]"
    ///
    /// True when the controller has completed the specified dungeon. Used for
    /// Acererak's intervening-if check ("if you haven't completed Tomb of Annihilation").
    /// Note: the condition evaluates to "has NOT completed", so use negation in the
    /// card definition (i.e., `Condition::Not(Box::new(CompletedSpecificDungeon(...)))`)
    /// when the oracle text says "haven't".
    CompletedSpecificDungeon(crate::state::dungeon::DungeonId),

    /// CR 701.54c: "if the Ring has tempted you N or more times."
    ///
    /// True when the controller's `ring_level >= n`. Used for cards that check how
    /// many times the Ring has tempted you (e.g., Frodo, Sauron's Bane at level 4).
    RingHasTemptedYou(u8),

    /// Logical negation of another condition.
    ///
    /// Used for Acererak's "if you haven't completed Tomb of Annihilation":
    /// `Condition::Not(Box::new(Condition::CompletedSpecificDungeon(DungeonId::TombOfAnnihilation)))`.
    Not(Box<Condition>),

    /// Logical disjunction of two conditions. True if either is true.
    ///
    /// Used for Temple of the Dragon Queen: "unless you revealed a Dragon card
    /// this way or you control a Dragon."
    Or(Box<Condition>, Box<Condition>),

    // ── ETB condition variants (PB-2) ────────────────────────────────────────
    /// "unless you control a [Plains/Island/etc.]" — check-lands, castles.
    /// True if the controller controls a land on the battlefield with ANY of the
    /// listed subtypes. Used with `unless_condition` on `AbilityDefinition::Replacement`.
    ControlLandWithSubtypes(Vec<SubType>),

    /// "unless you control N or fewer other lands" — fast-lands (e.g., N=2).
    /// True if the controller controls N or fewer OTHER lands on the battlefield
    /// (excluding the entering land itself).
    ControlAtMostNOtherLands(u32),

    /// "unless you have two or more opponents" — bond-lands.
    /// True if the controller has >= 2 opponents still in the game.
    HaveTwoOrMoreOpponents,

    /// "you may reveal a [type] card from your hand" — reveal-lands.
    /// Deterministic fallback: auto-reveal if hand contains a card with ANY of the
    /// listed subtypes. True if a matching card is found.
    CanRevealFromHandWithSubtype(Vec<SubType>),

    /// "unless you control N or more basic lands" — battle-lands (e.g., N=2).
    /// True if the controller controls >= N basic lands on the battlefield.
    ControlBasicLandsAtLeast(u32),

    /// "unless you control N or more other lands" — slow-lands (e.g., N=2).
    /// True if the controller controls >= N OTHER lands on the battlefield
    /// (excluding the entering land itself).
    ControlAtLeastNOtherLands(u32),

    /// "unless you control N or more other [subtype]s" — Mystic Sanctuary, Witch's Cottage.
    /// True if the controller controls >= N OTHER lands with the given subtype on the
    /// battlefield (excluding the entering land itself).
    ControlAtLeastNOtherLandsWithSubtype { count: u32, subtype: SubType },

    /// "unless you control a legendary creature" — Minas Tirith.
    /// True if the controller controls a legendary creature on the battlefield.
    ControlLegendaryCreature,

    /// "unless you control a creature with subtype X" — Temple of the Dragon Queen.
    /// True if the controller controls a creature with the given subtype.
    ControlCreatureWithSubtype(SubType),

    /// CR 702.131c: "if you have the city's blessing"
    ///
    /// True when the controller has the city's blessing designation (permanent,
    /// never removed once gained). Used by Ascend cards to gate abilities.
    HasCitysBlessing,
}

// ── Mode Selection ────────────────────────────────────────────────────────────

/// Modal spells/abilities: choose N of M modes (CR 700.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeSelection {
    /// Minimum number of modes the caster must choose.
    pub min_modes: usize,
    /// Maximum number of modes the caster may choose.
    pub max_modes: usize,
    /// The available modes (effects). Indexed by the `ChooseOption` command.
    pub modes: Vec<Effect>,
    /// CR 700.2d: If true, the same mode may be chosen more than once.
    /// Default is false (standard modal behavior — no duplicate modes).
    #[serde(default)]
    pub allow_duplicate_modes: bool,
    /// CR 700.2h / 702.172a: Per-mode additional costs for spree spells.
    /// When `Some`, `mode_costs[i]` is the additional mana cost that must be
    /// paid when mode `i` is chosen. Must have the same length as `modes`.
    /// `None` for standard modal spells (no per-mode costs).
    #[serde(default)]
    pub mode_costs: Option<Vec<ManaCost>>,
}

// ── Token Specification ───────────────────────────────────────────────────────

/// Everything needed to create a token (CR 111).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenSpec {
    pub name: String,
    pub power: i32,
    pub toughness: i32,
    pub colors: OrdSet<Color>,
    /// Supertypes (e.g., Legendary for The Atropal). CR 205.4.
    #[serde(default)]
    pub supertypes: OrdSet<SuperType>,
    pub card_types: OrdSet<CardType>,
    pub subtypes: OrdSet<SubType>,
    pub keywords: OrdSet<KeywordAbility>,
    /// How many tokens to create.
    pub count: u32,
    /// True if the tokens enter the battlefield tapped.
    pub tapped: bool,
    /// Color override: all created under the controller's control.
    pub mana_color: Option<ManaColor>,
    /// Mana abilities to populate on each created token (CR 605).
    /// Used by Treasure tokens (CR 111.10a), Gold tokens (CR 111.10c), etc.
    #[serde(default)]
    pub mana_abilities: Vec<ManaAbility>,
    /// Non-mana activated abilities on the token (CR 602).
    /// Used by Food tokens (CR 111.10b), Clue tokens (CR 111.10f),
    /// Shard tokens (CR 111.10e), etc.
    #[serde(default)]
    pub activated_abilities: Vec<ActivatedAbility>,
}

impl Default for TokenSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            power: 1,
            toughness: 1,
            colors: OrdSet::new(),
            supertypes: OrdSet::new(),
            card_types: OrdSet::new(),
            subtypes: OrdSet::new(),
            keywords: OrdSet::new(),
            count: 1,
            tapped: false,
            mana_color: None,
            mana_abilities: Vec::new(),
            activated_abilities: Vec::new(),
        }
    }
}

/// CR 111.10a: Predefined Treasure token specification.
///
/// A colorless Treasure artifact token with "{T}, Sacrifice this artifact:
/// Add one mana of any color."
pub fn treasure_token_spec(count: u32) -> TokenSpec {
    TokenSpec {
        name: "Treasure".to_string(),
        power: 0,
        toughness: 0,
        colors: OrdSet::new(),
        supertypes: OrdSet::new(),
        card_types: [CardType::Artifact].into_iter().collect(),
        subtypes: [SubType("Treasure".to_string())].into_iter().collect(),
        keywords: OrdSet::new(),
        mana_abilities: vec![ManaAbility::treasure()],
        activated_abilities: vec![],
        count,
        tapped: false,
        mana_color: None,
    }
}

/// CR 111.10b: Predefined Food token specification.
///
/// A colorless Food artifact token with "{2}, {T}, Sacrifice this token:
/// You gain 3 life."
pub fn food_token_spec(count: u32) -> TokenSpec {
    TokenSpec {
        name: "Food".to_string(),
        power: 0,
        toughness: 0,
        colors: OrdSet::new(),
        supertypes: OrdSet::new(),
        card_types: [CardType::Artifact].into_iter().collect(),
        subtypes: [SubType("Food".to_string())].into_iter().collect(),
        keywords: OrdSet::new(),
        mana_abilities: vec![],
        activated_abilities: vec![ActivatedAbility {
            targets: vec![],
            cost: crate::state::game_object::ActivationCost {
                requires_tap: true,
                mana_cost: Some(ManaCost {
                    generic: 2,
                    ..ManaCost::default()
                }),
                sacrifice_self: true,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
            },
            description: "{2}, {T}, Sacrifice this token: You gain 3 life.".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(3),
            }),
            sorcery_speed: false,
        }],
        count,
        tapped: false,
        mana_color: None,
    }
}

/// CR 111.10f: Predefined Clue token specification.
///
/// A colorless Clue artifact token with "{2}, Sacrifice this token: Draw a card."
/// Note: Unlike Food tokens, the Clue ability does NOT require {T} — a tapped Clue
/// can still have its ability activated.
pub fn clue_token_spec(count: u32) -> TokenSpec {
    TokenSpec {
        name: "Clue".to_string(),
        power: 0,
        toughness: 0,
        colors: OrdSet::new(),
        supertypes: OrdSet::new(),
        card_types: [CardType::Artifact].into_iter().collect(),
        subtypes: [SubType("Clue".to_string())].into_iter().collect(),
        keywords: OrdSet::new(),
        mana_abilities: vec![],
        activated_abilities: vec![ActivatedAbility {
            targets: vec![],
            cost: crate::state::game_object::ActivationCost {
                requires_tap: false,
                mana_cost: Some(ManaCost {
                    generic: 2,
                    ..ManaCost::default()
                }),
                sacrifice_self: true,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
            },
            description: "{2}, Sacrifice this token: Draw a card.".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
        }],
        count,
        tapped: false,
        mana_color: None,
    }
}

/// CR 111.10g: Predefined Blood token specification.
///
/// A colorless Blood artifact token with "{1}, {T}, Discard a card, Sacrifice this
/// token: Draw a card." All four costs — mana, tap, discard, sacrifice — are paid
/// simultaneously at activation time (CR 602.2).
///
/// Unlike Food ({2},{T},Sacrifice) and Clue ({2},Sacrifice), Blood requires
/// BOTH tap AND discard AND sacrifice AND {1} mana.
pub fn blood_token_spec(count: u32) -> TokenSpec {
    TokenSpec {
        name: "Blood".to_string(),
        power: 0,
        toughness: 0,
        colors: OrdSet::new(),
        supertypes: OrdSet::new(),
        card_types: [CardType::Artifact].into_iter().collect(),
        subtypes: [SubType("Blood".to_string())].into_iter().collect(),
        keywords: OrdSet::new(),
        mana_abilities: vec![],
        activated_abilities: vec![ActivatedAbility {
            targets: vec![],
            cost: crate::state::game_object::ActivationCost {
                requires_tap: true,
                mana_cost: Some(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                }),
                sacrifice_self: true,
                discard_card: true,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
            },
            description: "{1}, {T}, Discard a card, Sacrifice this token: Draw a card.".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
        }],
        count,
        tapped: false,
        mana_color: None,
    }
}

/// CR 701.47a: Token spec for an Army creature token.
///
/// Creates a 0/0 black [subtype] Army creature token. The `subtype` parameter
/// determines the creature subtype (e.g., "Zombie" for "amass Zombies N").
/// If no subtype is provided, defaults to "Zombie" per CR 701.47d Oracle errata.
pub fn army_token_spec(subtype: &str) -> TokenSpec {
    TokenSpec {
        name: format!("{} Army", subtype),
        power: 0,
        toughness: 0,
        colors: [Color::Black].into_iter().collect(),
        supertypes: OrdSet::new(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType(subtype.to_string()), SubType("Army".to_string())]
            .into_iter()
            .collect(),
        keywords: OrdSet::new(),
        count: 1,
        tapped: false,
        mana_color: None,
        mana_abilities: vec![],
        activated_abilities: vec![],
    }
}

/// CR 702.147a: Predefined Zombie Decayed token specification.
///
/// Creates a 2/2 black Zombie creature token with Decayed.
/// Used by Jadar, Wilhelt, Ghoulish Procession, Tainted Adversary, and other
/// Midnight Hunt / Crimson Vow cards that produce Decayed tokens.
pub fn zombie_decayed_token_spec(count: u32) -> TokenSpec {
    TokenSpec {
        name: "Zombie".to_string(),
        power: 2,
        toughness: 2,
        colors: [Color::Black].into_iter().collect(),
        supertypes: OrdSet::new(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Zombie".to_string())].into_iter().collect(),
        keywords: [KeywordAbility::Decayed].into_iter().collect(),
        count,
        tapped: false,
        mana_color: None,
        mana_abilities: vec![],
        activated_abilities: vec![],
    }
}

// ── Zone Target ───────────────────────────────────────────────────────────────

/// A destination zone for zone-change effects (CR 400).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryPosition {
    Top,
    Bottom,
    /// Shuffled in at random (the library is shuffled afterward).
    ShuffledIn,
}

// ── For Each Target ───────────────────────────────────────────────────────────

/// The collection `ForEach` iterates over.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Every card in every graveyard (all players).
    ///
    /// Used by Rest in Peace ETB: "exile all cards from all graveyards" (CR 614.1).
    EachCardInAllGraveyards,
    /// Every other attacking creature (excluding the source of the effect).
    ///
    /// Used by Battle Cry (CR 702.91a): "each other attacking creature gets
    /// +1/+0 until end of turn." Queries `state.combat.attackers` and
    /// excludes `ctx.source`.
    EachOtherAttackingCreature,
}

// ── Timing Restriction ────────────────────────────────────────────────────────

/// When an activated ability can be used.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuousEffectDef {
    pub layer: crate::state::EffectLayer,
    pub modification: crate::state::LayerModification,
    pub filter: crate::state::EffectFilter,
    pub duration: crate::state::EffectDuration,
}

// ── Spell Cost Modification ─────────────────────────────────────────────────

/// A static cost modifier from a permanent on the battlefield (or command zone for Eminence).
///
/// CR 601.2f: The total cost is the mana cost (or alternative cost) plus any cost
/// increases minus any cost reductions. Cost increases and reductions are applied
/// after the base cost is determined and before optional cost payments (convoke, etc.).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellCostModifier {
    /// The generic mana change: positive = increase (Thalia), negative = reduction (Warchief).
    pub change: i32,
    /// Which spells this modifier applies to.
    pub filter: SpellCostFilter,
    /// Who is affected — all players or just the controller.
    pub scope: CostModifierScope,
    /// If true, this modifier applies from the command zone as well as the battlefield (Eminence).
    #[serde(default)]
    pub eminence: bool,
}

/// Filter for which spells a cost modifier applies to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellCostFilter {
    /// Noncreature spells (Thalia, Guardian of Thraben).
    NonCreature,
    /// Spells with a specific creature subtype (Goblin Warchief: "Goblin spells").
    HasSubtype(SubType),
    /// Historic spells: artifacts, legendaries, and Sagas (Jhoira's Familiar).
    Historic,
    /// Spells with a specific card type (e.g., Aura spells, Equipment spells).
    HasCardType(CardType),
    /// Aura or Equipment spells (Danitha Capashen, Paragon).
    AuraOrEquipment,
}

/// Who is affected by a spell cost modifier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostModifierScope {
    /// All players (Thalia: "noncreature spells cost {1} more").
    AllPlayers,
    /// Only the controller of the source permanent (Warchief: "Goblin spells YOU cast").
    Controller,
}

/// A self-cost-reduction on a spell — the spell itself is cheaper based on game state at cast time.
///
/// CR 601.2f: Cost reductions are applied during total cost calculation. The generic
/// component cannot be reduced below 0.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelfCostReduction {
    /// "{1} less for each <permanent matching filter> on the battlefield" (Blasphemous Act).
    PerPermanent {
        per: i32,
        filter: TargetFilter,
        controller: PlayerTarget,
    },
    /// "costs {X} less where X is the total power of creatures you control" (Ghalta).
    TotalPowerOfCreatures,
    /// "{1} less for each card type among cards in your graveyard" (Emrakul, the Promised End).
    CardTypesInGraveyard,
    /// "{N} less for each basic land type among lands you control" (Scion of Draco — Domain).
    BasicLandTypes { per: i32 },
    /// "costs {X} less where X is the total mana value of <permanents matching filter> you control"
    /// (Earthquake Dragon).
    TotalManaValue { filter: TargetFilter },
}
