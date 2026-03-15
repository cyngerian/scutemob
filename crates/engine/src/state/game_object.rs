//! Game object types: ObjectId, characteristics, status, and the GameObject struct.

use bitflags::bitflags;
use im::{OrdMap, OrdSet, Vector};
use serde::{Deserialize, Serialize};

use super::player::{CardId, PlayerId};
use super::types::{
    AltCostKind, CardType, Color, CounterType, FaceDownKind, KeywordAbility, ManaColor, SubType,
    SuperType,
};
use super::zone::ZoneId;

bitflags! {
    /// Packed boolean designations on a GameObject. Replaces individual `is_*` fields
    /// to reduce struct size and simplify initialization.
    ///
    /// Each flag is a non-copiable, non-ability designation that persists until the
    /// permanent leaves the battlefield (CR 400.7 resets all).
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Designations: u16 {
        /// CR 702.112b: Renowned designation (set by Renown trigger).
        const RENOWNED       = 1 << 0;
        /// CR 701.60b: Suspected designation (grants Menace + can't block).
        const SUSPECTED      = 1 << 1;
        /// CR 702.171b: Saddled designation (set by Saddle activation, until EOT).
        const SADDLED        = 1 << 2;
        /// CR 702.30a: Echo payment pending (ETB flag, cleared on trigger resolution).
        const ECHO_PENDING   = 1 << 3;
        /// CR 702.103b: Currently bestowed (Aura, not creature).
        const BESTOWED       = 1 << 4;
        /// CR 702.143a: Foretold (exiled face-down via foretell action).
        const FORETOLD       = 1 << 5;
        /// CR 702.62b: Suspended (in exile with time counters).
        const SUSPENDED      = 1 << 6;
        /// CR 702.151b: Reconfigured (attached Equipment is not a creature).
        const RECONFIGURED   = 1 << 7;
        /// CR 701.54b: Ring-bearer designation. Not a copiable value (CR 701.54b).
        ///
        /// Set by `handle_ring_tempts_you` when a player chooses this creature as their
        /// ring-bearer. Cleared when the creature leaves the battlefield or changes control
        /// (CR 701.54a). Grants the Legendary supertype via the layer system (ring level >= 1)
        /// and enables ring-bearer blocking restriction (ring level >= 1).
        const RING_BEARER    = 1 << 8;
    }
}

/// Identifies a game object instance. Per CR 400.7, when an object changes
/// zones it becomes a new object with a new ObjectId.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u64);

/// A single hybrid mana symbol (CR 107.4e).
///
/// Hybrid mana symbols represent a cost that can be paid in one of two ways.
/// Each hybrid symbol is all of its component colors (CR 202.2d).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HybridMana {
    /// {W/U}, {B/R}, etc. — can be paid with either color.
    ColorColor(ManaColor, ManaColor),
    /// {2/W}, {2/U}, etc. — can be paid with the color OR 2 generic mana.
    GenericColor(ManaColor),
}

/// A single Phyrexian mana symbol (CR 107.4f).
///
/// Each Phyrexian symbol represents a cost payable with one mana of its color
/// or by paying 2 life.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PhyrexianMana {
    /// {W/P}, {U/P}, etc. — pay with the color OR 2 life.
    Single(ManaColor),
    /// {G/W/P}, {U/B/P}, etc. — pay with either color OR 2 life.
    Hybrid(ManaColor, ManaColor),
}

/// How a hybrid mana pip was paid (deterministic choice encoding).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HybridManaPayment {
    /// Pay with a specific color (for ColorColor or GenericColor hybrids).
    Color(ManaColor),
    /// Pay with 2 generic mana (only valid for GenericColor hybrids).
    Generic,
}

/// Mana cost of a card or ability (CR 202).
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ManaCost {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
    pub generic: u32,
    /// Hybrid mana symbols (CR 107.4e). Each entry is one hybrid pip.
    #[serde(default)]
    pub hybrid: Vec<HybridMana>,
    /// Phyrexian mana symbols (CR 107.4f). Each entry is one Phyrexian pip.
    #[serde(default)]
    pub phyrexian: Vec<PhyrexianMana>,
    /// Number of {X} symbols in the cost (CR 107.3). Usually 1, sometimes 2
    /// (e.g., Treasure Vault {X}{X}). The actual value of X is chosen at cast
    /// time and stored on CastSpell/StackObject.x_value.
    #[serde(default)]
    pub x_count: u32,
}

impl ManaCost {
    /// Mana value (formerly "converted mana cost") per CR 202.3.
    ///
    /// CR 202.3f: Hybrid symbols use the largest component ({W/U}=1, {2/W}=2).
    /// CR 202.3g: Each Phyrexian symbol contributes 1.
    /// CR 202.3e: X is 0 off the stack (x_count is structural; actual value on StackObject).
    pub fn mana_value(&self) -> u32 {
        let base = self.white
            + self.blue
            + self.black
            + self.red
            + self.green
            + self.colorless
            + self.generic;
        // CR 202.3f: hybrid — use largest component
        let hybrid_mv: u32 = self
            .hybrid
            .iter()
            .map(|h| match h {
                HybridMana::ColorColor(_, _) => 1, // max(1, 1) = 1
                HybridMana::GenericColor(_) => 2,  // max(2, 1) = 2
            })
            .sum();
        // CR 202.3g: each Phyrexian symbol contributes 1
        let phyrexian_mv = self.phyrexian.len() as u32;
        // CR 202.3e: X is 0 off stack (x_count not counted here)
        base + hybrid_mv + phyrexian_mv
    }
}

/// A mana ability: an activated ability that produces mana (CR 605).
///
/// Mana abilities do not use the stack and resolve immediately. They can be
/// activated any time a player has priority or is paying a cost (CR 605.3b).
///
/// For M3-A, only tap-activated mana abilities are supported (the most common
/// case: basic lands, dual lands, etc.). Future milestones will add additional
/// cost components (pay life, sacrifice a permanent, etc.).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaAbility {
    /// The mana produced when this ability resolves, keyed by color.
    /// e.g., `{Green: 1}` for a Forest's "{T}: Add {G}".
    pub produces: OrdMap<ManaColor, u32>,
    /// True if activating this ability requires tapping the source permanent.
    /// Most land mana abilities require tapping. Some do not (future milestone).
    pub requires_tap: bool,
    /// CR 111.10a / CR 602.2c: If true, the source permanent is sacrificed as
    /// part of this ability's activation cost (moved to the graveyard before
    /// mana is added). Used by Treasure tokens: "{T}, Sacrifice this artifact:
    /// Add one mana of any color."
    #[serde(default)]
    pub sacrifice_self: bool,
    /// CR 111.10a: If true, this ability produces 1 mana of any color (player's
    /// choice). Overrides `produces`. Simplified: defaults to colorless until
    /// interactive color choice is implemented.
    #[serde(default)]
    pub any_color: bool,
    /// Pain land damage: if > 0, this mana ability deals this much damage to the
    /// controller when activated. Used by pain lands (e.g., Battlefield Forge:
    /// "{T}: Add {R} or {W}. This land deals 1 damage to you.").
    #[serde(default)]
    pub damage_to_controller: u32,
}

impl ManaAbility {
    /// Convenience constructor: tap this permanent to add one mana of `color`.
    pub fn tap_for(color: ManaColor) -> Self {
        let mut produces = OrdMap::new();
        produces.insert(color, 1);
        Self {
            produces,
            requires_tap: true,
            sacrifice_self: false,
            any_color: false,
            damage_to_controller: 0,
        }
    }

    /// CR 111.10a: Treasure token mana ability — "{T}, Sacrifice this artifact:
    /// Add one mana of any color."
    pub fn treasure() -> Self {
        Self {
            produces: OrdMap::new(),
            requires_tap: true,
            sacrifice_self: true,
            any_color: true,
            damage_to_controller: 0,
        }
    }
}

/// Filter for "sacrifice a [type]" activation costs (CR 602.2).
///
/// Used when an activated ability requires sacrificing another permanent (not self)
/// as part of its cost. E.g., Phyrexian Tower: "{T}, Sacrifice a creature: Add {B}{B}."
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SacrificeFilter {
    /// "Sacrifice a creature" — any creature you control (other than the source if needed).
    Creature,
    /// "Sacrifice a land" — any land you control.
    Land,
    /// "Sacrifice an artifact" — any artifact you control.
    Artifact,
    /// "Sacrifice an artifact or creature" — either type.
    ArtifactOrCreature,
    /// "Sacrifice a [subtype]" — e.g., "Sacrifice a Desert", "Sacrifice a Food".
    Subtype(super::types::SubType),
}

/// Cost to activate an activated ability (CR 602.2).
///
/// For M3-E, activation costs can include tapping and paying mana.
/// Sacrifice-as-cost is also supported (CR 602.2c).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationCost {
    /// True if activating requires tapping the source (CR 602.2).
    pub requires_tap: bool,
    /// Mana cost component of the activation cost (if any).
    pub mana_cost: Option<ManaCost>,
    /// True if this ability requires sacrificing the source permanent as a cost.
    /// CR 602.2: sacrifice is paid at activation time, before the ability is on the stack.
    #[serde(default)]
    pub sacrifice_self: bool,
    /// CR 602.2 / CR 111.10g: True if activating this ability requires discarding a card as a cost.
    /// The discard happens at activation time (before the ability goes on the stack).
    /// Used by Blood tokens: "{1}, {T}, Discard a card, Sacrifice this token: Draw a card."
    #[serde(default)]
    pub discard_card: bool,
    /// CR 702.34: True if activating this ability requires discarding this card from hand.
    /// Channel abilities are activated from hand (not the battlefield).
    #[serde(default)]
    pub discard_self: bool,
    /// CR 701.61a: True if activating this ability requires performing the forage action.
    /// "Forage" means: exile three cards from your graveyard OR sacrifice a Food artifact.
    #[serde(default)]
    pub forage: bool,
    /// CR 602.2: Optional filter for sacrificing another permanent as part of the cost.
    /// E.g., "Sacrifice a creature" = `Some(SacrificeFilter::Creature)`.
    /// The caller must supply the ObjectId of the permanent to sacrifice in the command.
    #[serde(default)]
    pub sacrifice_filter: Option<SacrificeFilter>,
}

/// A non-mana activated ability that uses the stack (CR 602).
///
/// Written as "Cost: Effect." Distinct from `ManaAbility` (CR 605) which
/// resolves immediately without the stack.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivatedAbility {
    /// The cost to activate this ability.
    pub cost: ActivationCost,
    /// Human-readable description of the effect (CR-compatible text).
    pub description: String,
    /// The structured effect executed on resolution (M7+). None for abilities
    /// that have no automated effect (e.g., abilities that rely on player choice in M9+).
    #[serde(default)]
    pub effect: Option<crate::cards::card_definition::Effect>,
    /// CR 602.5d: If true, this ability can only be activated at sorcery speed
    /// (main phase, stack empty, active player only).
    #[serde(default)]
    pub sorcery_speed: bool,
    /// Target requirements for this activated ability (CR 601.2c).
    /// Empty = no targets required.
    #[serde(default)]
    pub targets: Vec<crate::cards::card_definition::TargetRequirement>,
}

/// Trigger event patterns for triggered abilities (CR 603).
///
/// Describes what game event causes a triggered ability to trigger.
/// Only common patterns are enumerated; M7+ will add full card definition triggers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerEvent {
    /// Triggers when the source permanent enters the battlefield (CR 603.5).
    SelfEntersBattlefield,
    /// Triggers whenever any permanent enters the battlefield (CR 603.5).
    AnyPermanentEntersBattlefield,
    /// Triggers whenever a spell is cast (CR 603.5).
    AnySpellCast,
    /// Triggers when the source permanent becomes tapped (CR 603.5).
    SelfBecomesTapped,
    /// Triggers when this creature attacks (CR 603.5, CR 508.1).
    SelfAttacks,
    /// Triggers when this creature blocks (CR 603.5, CR 509.1).
    SelfBlocks,
    /// CR 702.21a: Triggers when this permanent becomes the target of a spell or
    /// ability controlled by an opponent. Used exclusively by the Ward keyword.
    /// The opponent check is done at trigger-collection time in `rules/abilities.rs`.
    SelfBecomesTargetByOpponent,
    /// CR 702.108a: Triggers when the controller of this permanent casts a
    /// noncreature spell. Used by the Prowess keyword. The noncreature check
    /// and controller-match are verified at trigger-collection time in
    /// `rules/abilities.rs`.
    ControllerCastsNoncreatureSpell,
    /// CR 603.6c / CR 700.4: Triggers when this permanent is put into a
    /// graveyard from the battlefield ("dies"). This is a leaves-the-battlefield
    /// trigger that "looks back in time" (CR 603.10a).
    SelfDies,
    /// CR 603.2 / CR 510.3a: Triggers when this creature deals combat damage
    /// to a player. The creature must still be on the battlefield after damage
    /// is dealt (CR 603.10 — combat damage triggers do NOT look back in time).
    /// Fires only when damage > 0 (CR 603.2g: prevented damage does not trigger).
    SelfDealsCombatDamageToPlayer,
    /// CR 603.2 / CR 102.2: Triggers when an opponent of the source's controller
    /// casts a spell. "Opponent" = any player other than the source's controller
    /// (CR 102.2 two-player, CR 102.3 multiplayer FFA = Commander default).
    /// The opponent check is done at trigger-collection time in `rules/abilities.rs`.
    OpponentCastsSpell,
    /// CR 702.83a: Triggers on each permanent controlled by the attacking player
    /// when exactly one creature is declared as an attacker ("attacks alone").
    /// Used by the Exalted keyword. The "attacks alone" check is done at
    /// trigger-collection time in `rules/abilities.rs`. The +1/+1 effect targets
    /// the lone attacker (not the source), resolved via DeclaredTarget { index: 0 }.
    ControllerCreatureAttacksAlone,
    /// CR 701.25d: Triggers when the controller of this permanent surveils.
    /// Used by Dimir Spybug and similar "whenever you surveil" cards.
    /// The controller match is done at trigger-collection time in `rules/abilities.rs`.
    ControllerSurveils,
    /// CR 702.101a: Triggers when the controller of this permanent casts a spell
    /// (any spell, including creature spells). Used by the Extort keyword.
    /// The controller-match is verified at trigger-collection time in
    /// `rules/abilities.rs`.
    ControllerCastsSpell,
    /// CR 702.105a: Triggers when this creature attacks a player who has the
    /// most life or is tied for the most life among all players.
    /// Only fires for AttackTarget::Player (not planeswalker/battle).
    /// The "most life" check is done at trigger-collection time in
    /// `rules/abilities.rs` AttackersDeclared handler.
    SelfAttacksPlayerWithMostLife,
    /// CR 701.50b: Triggers when this permanent connives.
    /// Used by "whenever [this creature] connives" abilities (e.g., Ledger Shredder).
    /// The conniving-object match is done at trigger-collection time in
    /// `rules/abilities.rs`.
    SourceConnives,
    /// CR 701.16a: Triggers when the controller of this permanent investigates.
    /// Used by "whenever you investigate" cards (e.g., Lonis, Cryptozoologist).
    /// The controller match is done at trigger-collection time in `rules/abilities.rs`.
    ControllerInvestigates,
    /// CR 701.34: Triggers when the controller of this permanent proliferates.
    /// Used by "whenever you proliferate" cards (e.g., Core Prowler, Vat Emergence).
    /// The controller match is done at trigger-collection time in `rules/abilities.rs`.
    ControllerProliferates,
    /// CR 509.1h / CR 702.45a / CR 702.23a: Triggers when this attacking creature becomes
    /// blocked (has one or more blockers declared against it). Used by the Bushido keyword
    /// (CR 702.45a) and the Rampage keyword (CR 702.23a).
    /// The "becomes blocked" check is done at trigger-collection time in
    /// `rules/abilities.rs` when processing `BlockersDeclared` events.
    /// Triggers once per attacker regardless of how many creatures block it (CR 509.3c).
    SelfBecomesBlocked,
    /// CR 702.149a: Triggers when this creature attacks alongside at least one
    /// other attacking creature with strictly greater power.
    /// The power comparison is done at trigger-collection time in
    /// `rules/abilities.rs` AttackersDeclared handler.
    /// Used by the Training keyword.
    SelfAttacksWithGreaterPowerAlly,
    /// CR 207.2c / CR 120.3: Triggers when this creature is dealt damage (> 0
    /// after prevention). Used by the Enrage ability word. Fires once per
    /// simultaneous damage event, regardless of how many sources dealt damage.
    SelfIsDealtDamage,
    /// CR 702.55c: Triggers when the creature this exiled haunt card haunts dies.
    /// Only meaningful for triggered abilities on cards in the exile zone that
    /// have a haunting relationship (haunting_target is set). Fires from exile
    /// when the creature with that ObjectId dies.
    HauntedCreatureDies,
    /// CR 702.140d: Triggers on the merged permanent itself when a mutating creature
    /// spell successfully merges with it. Used by "whenever this creature mutates"
    /// abilities (e.g., Gemrazer, Nethroi, Brokkos). Fired from `check_triggers`
    /// on `GameEvent::CreatureMutated`.
    SelfMutates,
    /// CR 708.8: "When this permanent is turned face up" -- triggered ability that fires
    /// when a face-down permanent is turned face up via any method (morph cost, disguise
    /// cost, manifest/cloak mana cost). Goes on the stack and can be responded to.
    ///
    /// Note: turning face up does NOT fire ETB abilities (CR 708.8). This trigger
    /// is distinct from ETB -- it is specifically for "when turned face up" abilities
    /// (e.g., Willbender, Den Protector).
    ///
    /// Fired from `check_triggers` on `GameEvent::PermanentTurnedFaceUp`.
    SelfTurnedFaceUp,
}

/// Intervening-if clause for conditional triggered abilities (CR 603.4).
///
/// The condition is checked at trigger time (ability only triggers if true)
/// and again at resolution (ability only resolves if still true).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterveningIf {
    /// "if your life total is [N] or more" — for testing conditional triggers.
    ControllerLifeAtLeast(u32),
    /// "if it had no [counter type] counters on it" — checked against pre-death
    /// counter state for persist/undying (CR 702.79a, CR 702.93a).
    /// At trigger time: checks `CreatureDied.pre_death_counters`.
    /// At resolution time: the condition passes unconditionally (the source is
    /// in the graveyard with no counters; MoveZone will simply find nothing if
    /// the source has since left the graveyard).
    SourceHadNoCounterOfType(crate::state::types::CounterType),
}

/// Filter applied to ETB triggers to restrict which entering permanents cause
/// the trigger to fire. All `true` fields must be satisfied (AND logic).
///
/// Used by Alliance ("another creature you control"), Constellation
/// ("enchantment you control"), Landfall ("land"), etc.
/// CR 207.2c / CR 603.2
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ETBTriggerFilter {
    /// If true, the entering permanent must be a creature.
    pub creature_only: bool,
    /// If true, the entering permanent must be controlled by the trigger source's controller.
    pub controller_you: bool,
    /// If true, the entering permanent must NOT be the trigger source itself ("another").
    pub exclude_self: bool,
}

/// A triggered ability definition on a game object (CR 603).
///
/// When the trigger event occurs, this ability is queued into
/// `GameState::pending_triggers` for APNAP ordering and placement on the stack.
///
/// Effects are described textually for M3-E; full implementation is M7+.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggeredAbilityDef {
    /// The event that causes this ability to trigger.
    pub trigger_on: TriggerEvent,
    /// Optional intervening-if condition (CR 603.4). Checked at trigger time
    /// AND at resolution time.
    pub intervening_if: Option<InterveningIf>,
    /// Human-readable description of the effect (CR-compatible text).
    pub description: String,
    /// The structured effect executed on resolution (M7+). None for abilities
    /// that have no automated effect yet.
    #[serde(default)]
    pub effect: Option<crate::cards::card_definition::Effect>,
    /// Optional ETB filter for "whenever [another] [creature] [you control] enters"
    /// triggers. When present, the trigger only fires if the entering permanent
    /// matches all specified criteria. Used by Alliance, Constellation, Landfall.
    /// CR 207.2c / CR 603.2
    #[serde(default)]
    pub etb_filter: Option<ETBTriggerFilter>,
    /// Target requirements for this triggered ability (CR 601.2c).
    /// Empty = no targets required.
    #[serde(default)]
    pub targets: Vec<crate::cards::card_definition::TargetRequirement>,
}

/// The observable characteristics of a game object (CR 109.3).
///
/// These are the copiable values of an object — what a copy effect copies.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Characteristics {
    pub name: String,
    pub mana_cost: Option<ManaCost>,
    pub colors: OrdSet<Color>,
    pub color_indicator: Option<OrdSet<Color>>,
    pub supertypes: OrdSet<SuperType>,
    pub card_types: OrdSet<CardType>,
    pub subtypes: OrdSet<SubType>,
    pub rules_text: String,
    pub abilities: Vector<AbilityInstance>,
    /// Keyword abilities (CR 702).
    pub keywords: OrdSet<KeywordAbility>,
    /// Mana abilities on this object (CR 605). Activated in-place without the stack.
    pub mana_abilities: Vector<ManaAbility>,
    /// Non-mana activated abilities that use the stack (CR 602).
    pub activated_abilities: Vec<ActivatedAbility>,
    /// Triggered abilities (CR 603). Queued and put on the stack in APNAP order.
    pub triggered_abilities: Vec<TriggeredAbilityDef>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub loyalty: Option<i32>,
    pub defense: Option<i32>,
}

/// Status bits for a permanent on the battlefield (CR 110.5).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectStatus {
    pub tapped: bool,
    pub flipped: bool,
    pub face_down: bool,
    pub phased_out: bool,
}

/// An instance of an ability on a game object.
/// Placeholder — will be fully defined in M3/M7.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityInstance {
    pub id: u64,
    pub description: String,
}

/// A game object — a card, token, copy, or ability on the stack (CR 109).
///
/// Every card and token in the game is represented as a GameObject with a
/// unique ObjectId. When an object changes zones, it gets a new ObjectId
/// per CR 400.7 ("an object that moves from one zone to another becomes a
/// new object with no memory of, or relation to, its previous existence").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameObject {
    pub id: ObjectId,
    /// Links back to the physical card identity (survives zone changes).
    pub card_id: Option<CardId>,
    pub characteristics: Characteristics,
    pub controller: PlayerId,
    pub owner: PlayerId,
    pub zone: ZoneId,
    pub status: ObjectStatus,
    pub counters: OrdMap<CounterType, u32>,
    pub attachments: Vector<ObjectId>,
    pub attached_to: Option<ObjectId>,
    pub damage_marked: u32,
    /// Packed boolean designations (Renowned, Suspected, Saddled, etc.).
    /// See `Designations` bitflags type for individual flags.
    #[serde(default)]
    pub designations: Designations,
    /// True if any damage dealt to this permanent was from a source with deathtouch (CR 704.5h).
    /// Set during combat damage assignment (M6+). Cleared with other damage in cleanup (CR 514.1).
    pub deathtouch_damage: bool,
    pub is_token: bool,
    /// Timestamp for continuous effect ordering (CR 613.7).
    pub timestamp: u64,
    /// True if this permanent has summoning sickness (CR 302.6).
    ///
    /// Set to `true` whenever a permanent enters the battlefield. Cleared at the
    /// beginning of each player's untap step for all permanents they control.
    /// A creature with summoning sickness cannot attack or have its activated
    /// abilities with {T} in the cost used, unless it has Haste (CR 702.10).
    pub has_summoning_sickness: bool,
    /// CR 701.15a: Players who have goaded this creature.
    ///
    /// Non-empty when a Goad effect has been applied to this permanent.
    /// A goaded creature must attack each combat if able (CR 701.15b) and
    /// must attack a player other than the goading player if able (CR 701.15b).
    /// This list is cleared when the creature's controller's next turn begins.
    #[serde(default)]
    pub goaded_by: Vector<PlayerId>,
    /// CR 702.33d: If this permanent was kicked when cast, records how many times
    /// the kicker cost was paid.
    ///
    /// 0 = not kicked (or permanent entered without being cast).
    /// 1 = kicked once (standard kicker). N = multikicked N times (CR 702.33c).
    /// Set during spell resolution when the permanent enters the battlefield.
    /// Never set for permanents that entered without being cast (CR ruling:
    /// "If you put a permanent onto the battlefield without casting it, you can't kick it.").
    #[serde(default)]
    pub kicker_times_paid: u32,
    /// CR 702.74a / CR 702.138b / CR 702.109a: Which alternative cost was paid when this
    /// permanent was cast, if any.
    ///
    /// - `Some(AltCostKind::Evoke)` — cast by paying the evoke cost. The evoke sacrifice
    ///   trigger checks this at ETB time.
    /// - `Some(AltCostKind::Escape)` — cast via escape from the graveyard. Used by
    ///   "escapes with [counter]" (CR 702.138c) and "escapes with [ability]" (CR 702.138d).
    /// - `Some(AltCostKind::Dash)` — cast by paying the dash cost. The permanent gains
    ///   haste and a delayed trigger returns it to hand at end step.
    /// - `None` — not cast with any tracked alternative cost (or entered without being cast).
    ///
    /// Set during spell resolution when the permanent enters the battlefield.
    /// Reset to `None` on zone changes (CR 400.7).
    #[serde(default)]
    pub cast_alt_cost: Option<AltCostKind>,
    /// CR 702.143a: The turn number when this card was foretold.
    ///
    /// The card can only be cast for its foretell cost "after the current turn
    /// has ended" -- i.e., on any turn where `state.turn.turn_number > foretold_turn`.
    /// Zero means not foretold. Set alongside `is_foretold`.
    #[serde(default)]
    pub foretold_turn: u32,
    /// CR 702.84a: If true, this permanent was returned to the battlefield via
    /// an unearth ability. Two effects track this:
    /// 1. Replacement effect: if this permanent would leave the battlefield for
    ///    any zone other than exile, it is exiled instead.
    /// 2. Delayed triggered ability: at the beginning of the next end step,
    ///    exile this permanent.
    ///
    /// These effects are NOT abilities on the creature -- they persist even if
    /// the creature loses all abilities (e.g., Humility).
    ///
    /// Set when the UnearthAbility resolves. Reset on zone changes (CR 400.7).
    #[serde(default)]
    pub was_unearthed: bool,
    /// CR 702.116a: If true, this token was created by a myriad ability and must
    /// be exiled at end of combat.
    ///
    /// Set when a MyriadTrigger resolves and creates the token. Checked in
    /// `end_combat()` in turn_actions.rs. Reset on zone changes (CR 400.7).
    #[serde(default)]
    pub myriad_exile_at_eoc: bool,
    /// CR 702.147a: If true, this creature attacked with decayed and must be
    /// sacrificed at end of combat. Set in `handle_declare_attackers` in combat.rs
    /// when an attacker has the Decayed keyword.
    ///
    /// Ruling 2021-09-24: "Once a creature with decayed attacks, it will be
    /// sacrificed at end of combat, even if it no longer has decayed at that time."
    /// This flag ensures the sacrifice happens even if decayed is removed after
    /// the attack declaration.
    #[serde(default)]
    pub decayed_sacrifice_at_eoc: bool,
    /// CR 701.54c (ring level >= 3): If true, this creature blocked the ring-bearer
    /// and must be sacrificed at end of combat by its controller.
    ///
    /// Set in `check_triggers` (abilities.rs) in the `BlockersDeclared` handler
    /// when the attacker is a ring-bearer with ring_level >= 3.
    ///
    /// CR 701.54c: "that creature's controller sacrifices it at end of combat."
    /// The sacrifice targets the specific blocking creature (not a generic
    /// SacrificePermanents choice). Checked in `end_combat()` in turn_actions.rs.
    #[serde(default)]
    pub ring_block_sacrifice_at_eoc: bool,
    /// CR 702.75a / CR 607.2a: If set, this object in exile was exiled face-down
    /// by a Hideaway ETB trigger from the permanent with this ObjectId.
    ///
    /// Used by the linked "play the exiled card" ability (`Effect::PlayExiledCard`)
    /// to identify which exiled card belongs to which Hideaway source.  Set when
    /// the HideawayTrigger resolves.  Cleared when the card is played out of
    /// exile or leaves exile for any other reason.
    ///
    /// The ObjectId stored here is the Hideaway permanent's ObjectId on the
    /// battlefield at the time the trigger resolved (CR 607.2a).
    #[serde(default)]
    pub exiled_by_hideaway: Option<ObjectId>,
    /// CR 702.141a: If true, this token was created by an encore ability and
    /// must be sacrificed at the beginning of the next end step.
    ///
    /// Unlike Unearth (which exiles and uses a replacement effect), encore
    /// tokens are simply sacrificed -- no replacement effect is involved.
    ///
    /// Set when the EncoreAbility resolves and creates the token. Checked in
    /// `end_step_actions()` in turn_actions.rs. Reset on zone changes (CR 400.7).
    ///
    /// Ruling 2020-11-10: "If one of the tokens is under another player's
    /// control as the delayed triggered ability resolves, you can't sacrifice
    /// that token." -- sacrifice only if controller == encore activator.
    #[serde(default)]
    pub encore_sacrifice_at_end_step: bool,
    /// CR 702.141a: If set, this token was created by encore and must attack
    /// the specified player this turn if able.
    ///
    /// Enforced during declare-attackers validation in `combat.rs`. The token
    /// must attack this player if able; if it can't attack that player (tapped,
    /// Propaganda cost not paid, etc.), it can attack any player or not attack.
    ///
    /// Cleared at end of turn. Also cleared on zone changes (CR 400.7).
    #[serde(default)]
    pub encore_must_attack: Option<crate::state::player::PlayerId>,
    /// CR 702.141a / Ruling 2020-11-10: The player who originally activated the
    /// encore ability that created this token.
    ///
    /// Used by the end-step sacrifice trigger to verify that the current
    /// controller of the token is still the original activator. Per the ruling:
    /// "If one of the tokens is under another player's control as the delayed
    /// triggered ability resolves, you can't sacrifice that token."
    ///
    /// Set during `EncoreAbility` resolution in `resolution.rs`. Cleared on
    /// zone changes (CR 400.7).
    #[serde(default)]
    pub encore_activated_by: Option<crate::state::player::PlayerId>,
    /// CR 702.170a: If true, this card was plotted -- exiled face-up via the plot
    /// special action. The card can be cast from exile without paying its mana cost
    /// on any later turn (CR 702.170d).
    ///
    /// Unlike `is_foretold`, the card is face-up in exile (public information).
    /// Zone changes (CR 400.7) clear this -- since plotted cards are in exile,
    /// any zone change from exile clears this.
    #[serde(default)]
    pub is_plotted: bool,
    /// CR 702.170d: The turn number when this card was plotted.
    ///
    /// The card can only be cast "during any turn after the turn in which it became
    /// plotted" -- i.e., on any turn where `state.turn.turn_number > plotted_turn`.
    /// Zero means not plotted. Set alongside `is_plotted`.
    #[serde(default)]
    pub plotted_turn: u32,
    /// CR 718.3b: If true, this permanent was cast as a prototyped spell.
    ///
    /// The permanent uses its alternative power, toughness, mana cost, and
    /// color (derived from the prototype mana cost, CR 718.3b / CR 105.2)
    /// instead of its normal values. These values are written into the
    /// permanent's base characteristics at resolution time (CR 718.2a).
    ///
    /// Reset to false on zone changes (CR 400.7 / CR 718.4). In every zone
    /// except the stack or the battlefield when cast as a prototyped spell,
    /// the card has only its normal characteristics.
    ///
    /// Part of copiable values (CR 718.2a, 718.3c, 718.3d): copies of a
    /// prototyped permanent retain the prototype characteristics.
    #[serde(default)]
    pub is_prototyped: bool,
    /// CR 702.166b: If true, this permanent was cast with its bargain cost paid.
    /// Used by ETB triggers that check "if this permanent was bargained" (e.g.,
    /// Hylda's Crown of Winter). Propagated from `StackObject.was_bargained`
    /// at resolution time when the permanent enters the battlefield.
    #[serde(default)]
    pub was_bargained: bool,
    /// CR 701.59c: If true, this permanent was cast with its collect evidence cost paid
    /// (exiled cards from graveyard with total mana value >= N as an additional cost).
    /// Propagated from `StackObject.evidence_collected` at resolution time.
    /// Used by ETB triggers that check `Condition::EvidenceWasCollected`.
    #[serde(default)]
    pub evidence_collected: bool,
    /// CR 702.26g: If true, this permanent phased out indirectly -- it was attached
    /// to another permanent that phased out directly. Indirectly-phased permanents
    /// do NOT phase in independently; they phase in only when their host phases in.
    ///
    /// Set during the phasing-out step in `turn_actions.rs`. Reset when the host
    /// phases back in. Reset to false on zone changes (CR 400.7).
    #[serde(default)]
    pub phased_out_indirectly: bool,
    /// CR 702.26a: The player who controlled this permanent when it phased out.
    /// Used to determine which player's untap step phases it back in.
    ///
    /// Only meaningful when `status.phased_out` is true. For directly-phased
    /// permanents, set to the controller at phase-out time. For indirectly-phased
    /// permanents, set to the same player as the host (their controller).
    ///
    /// Reset to None when the permanent phases back in or on zone changes (CR 400.7).
    #[serde(default)]
    pub phased_out_controller: Option<PlayerId>,
    /// CR 702.82b: The number of creatures this permanent devoured as it entered
    /// the battlefield. "It devoured" means "sacrificed as a result of its devour
    /// ability as it entered the battlefield."
    ///
    /// Used by abilities that reference the number of creatures devoured.
    /// Reset to 0 on zone changes (CR 400.7).
    #[serde(default)]
    pub creatures_devoured: u32,
    /// CR 702.72a / CR 607.2a: The ObjectId of the card exiled by this
    /// permanent's champion ETB trigger. Used by the linked LTB trigger
    /// to return the correct card to the battlefield.
    ///
    /// Set when the ChampionETBTrigger resolves and a card is exiled.
    /// Preserved across zone changes so the LTB trigger can read it from the
    /// post-move object (look-back information, CR 603.10a).
    /// If `None`, no card was championed (champion was sacrificed instead
    /// or the ETB trigger hasn't resolved yet).
    #[serde(default)]
    pub champion_exiled_card: Option<ObjectId>,
    /// CR 702.95b: The ObjectId of the creature this permanent is currently
    /// paired with via soulbond. Both paired creatures point to each other.
    ///
    /// Set when a SoulbondTrigger resolves. Cleared to `None` on zone changes
    /// (CR 400.7 + CR 702.95e) and by the soulbond SBA when unpairing conditions
    /// are met (CR 702.95e: control change or stops being a creature).
    #[serde(default)]
    pub paired_with: Option<ObjectId>,
    /// CR 702.104b: Whether tribute was paid for this permanent. When true,
    /// the chosen opponent placed N +1/+1 counters on it as it entered.
    /// Used by "if tribute wasn't paid" intervening-if trigger conditions.
    /// Reset to false on zone changes (CR 400.7).
    #[serde(default)]
    pub tribute_was_paid: bool,
    /// CR 107.3m: The value of X chosen when the spell that became this permanent was cast.
    /// Used by ETB replacement effects and triggers that reference X (e.g., Ravenous CR 702.156a).
    /// The permanent's own X is 0 per CR 107.3i, but ETB abilities use this stored value.
    /// Reset to 0 on zone changes (CR 400.7). Copied from StackObject.x_value at resolution.
    #[serde(default)]
    pub x_value: u32,
    /// CR 702.157a: Number of times the squad cost was paid when this permanent was cast.
    /// Used by the SquadETB trigger to create N token copies at resolution.
    /// Reset to 0 on zone changes (CR 400.7). Copied from StackObject.squad_count at resolution.
    /// Tokens created by Squad themselves have squad_count: 0 (not cast).
    #[serde(default)]
    pub squad_count: u32,
    /// CR 702.175a: Whether the offspring cost was paid when this permanent was cast.
    /// Used by the OffspringETB trigger to create 1 token copy (except 1/1) at resolution.
    /// Reset to false on zone changes (CR 400.7). Copied from StackObject.offspring_paid at resolution.
    /// Tokens created by Offspring themselves have offspring_paid: false (not cast).
    #[serde(default)]
    pub offspring_paid: bool,
    /// CR 702.174a: Whether the gift cost was paid when this permanent was cast.
    /// Used by the GiftETB trigger at resolution. Reset on zone changes (CR 400.7).
    #[serde(default)]
    pub gift_was_given: bool,
    /// CR 702.174a: The opponent chosen to receive the gift when this permanent was cast.
    /// Used by the GiftETB trigger to determine which player gets the gift.
    /// Reset to None on zone changes (CR 400.7).
    #[serde(default)]
    pub gift_opponent: Option<crate::state::PlayerId>,
    /// CR 702.99b: Cipher encoded cards.
    ///
    /// Each entry is `(exiled_object_id, card_id)` where `exiled_object_id` is the
    /// ObjectId of the exiled cipher card and `card_id` is its CardId (for creating
    /// copies). Multiple cipher spells can be encoded on the same creature.
    ///
    /// CR 702.99c: Encoding persists even if the creature stops being a creature or
    /// changes controller, as long as it remains on the battlefield.
    ///
    /// CR 400.7: Reset to empty on zone changes (new object has no encoding).
    /// CR 702.99c: If the encoded card leaves exile, verify at trigger resolution
    /// time that the card still exists in exile (fizzle if not -- no SBA needed).
    #[serde(default)]
    pub encoded_cards: im::Vector<(ObjectId, crate::state::player::CardId)>,
    /// CR 702.55b: The ObjectId of the creature this exiled card is haunting.
    ///
    /// Set when a HauntExileTrigger resolves: the haunt card is moved from the
    /// graveyard to exile, and this field is set to the ObjectId of the target
    /// creature on the battlefield. The creature's battlefield ObjectId is stored
    /// here for matching when the haunted creature dies.
    ///
    /// CR 400.7: Reset to None on zone changes (new object has no haunting relationship).
    /// CR 702.55c: When the creature with this ObjectId dies (via CreatureDied.object_id),
    /// the engine scans exile for haunt cards with a matching haunting_target.
    #[serde(default)]
    pub haunting_target: Option<ObjectId>,
    /// CR 729.2: Components of a merged permanent (Mutate, CR 702.140).
    ///
    /// Empty for unmerged permanents (the common case — NOT a vec of one).
    /// When non-empty, `merged_components[0]` is always the topmost component;
    /// the merged permanent uses the topmost component's characteristics as its
    /// base copiable values (CR 729.2a) and has ALL abilities from ALL components
    /// (CR 702.140e).
    ///
    /// When the merged permanent leaves the battlefield, each component becomes
    /// a separate `GameObject` in the destination zone (CR 729.3).
    ///
    /// CR 400.7: Reset to empty on zone changes (new objects start unmerged).
    #[serde(default)]
    pub merged_components: im::Vector<MergedComponent>,
    /// CR 712.8d/e: If true, this permanent has its back face up (is transformed).
    ///
    /// When true, the layer system replaces this permanent's base characteristics
    /// with the back face's characteristics from the card registry (CardDefinition::back_face).
    /// When false (default), the front face characteristics are used normally.
    ///
    /// CR 712.18: Transforming does NOT create a new object — all counters, damage,
    /// auras, and continuous effects continue to apply.
    ///
    /// CR 400.7: Reset to false on zone changes (new object starts with front face up),
    /// EXCEPT when a permanent enters the battlefield via Disturb (enters transformed) or
    /// via "enters transformed" effects (e.g., Daybound ETB during night, Craft return).
    ///
    /// CR 712.8a: DFCs in non-battlefield zones use only front face characteristics.
    /// This is enforced by resetting to false on zone changes (CR 400.7).
    #[serde(default)]
    pub is_transformed: bool,
    /// CR 701.27f: Timestamp of the last time this permanent transformed or converted.
    ///
    /// Used to enforce CR 701.27f: if a non-delayed triggered ability tries to transform
    /// a permanent that already transformed since the ability was put on the stack,
    /// the instruction is ignored. Compare against the ability's creation timestamp:
    /// if `last_transform_timestamp >= ability_timestamp`, ignore the transform.
    ///
    /// Set to `state.timestamp_counter` whenever the permanent transforms.
    /// Reset to 0 on zone changes (CR 400.7).
    #[serde(default)]
    pub last_transform_timestamp: u64,
    /// Ruling (CR 702.146): If true, this permanent was cast via its disturb ability.
    ///
    /// Enables the exile-on-graveyard replacement effect: "If this permanent would be
    /// put into a graveyard from anywhere, exile it instead."
    ///
    /// This effect persists even if the permanent loses all abilities (CR 702.146 ruling).
    ///
    /// Set when a disturb-cast spell resolves. Reset on zone changes (CR 400.7).
    /// (A disturbed permanent that goes to exile then re-enters the battlefield starts fresh.)
    #[serde(default)]
    pub was_cast_disturbed: bool,
    /// CR 702.167c: ObjectIds of cards exiled as craft materials.
    ///
    /// When a Craft ability resolves, the source permanent and material permanents/cards
    /// are exiled as cost. The material ObjectIds (from before exile zone change) are
    /// stored here for card abilities that reference "cards exiled to craft this" (CR 702.167c).
    ///
    /// Note: because zone changes create new ObjectIds (CR 400.7), these ObjectIds will
    /// be stale by the time the permanent enters the battlefield. They are preserved for
    /// informational lookup via last-known-information.
    ///
    /// Reset to empty on zone changes (CR 400.7).
    #[serde(default)]
    pub craft_exiled_cards: im::Vector<ObjectId>,
    /// CR 106.12: The creature type chosen as this permanent entered the battlefield.
    ///
    /// Used by cards like Cavern of Souls, Secluded Courtyard, Unclaimed Territory:
    /// "As this enters, choose a creature type." The chosen type is referenced by
    /// `ManaRestriction::ChosenTypeCreaturesOnly` / `ChosenTypeSpellsOnly` to restrict
    /// what the produced mana can be spent on.
    ///
    /// Set by `Effect::ChooseCreatureType`. Reset to `None` on zone changes (CR 400.7).
    #[serde(default)]
    pub chosen_creature_type: Option<SubType>,
    /// CR 606.3: True if a loyalty ability of this permanent has been activated this turn.
    ///
    /// Only one loyalty ability per permanent per turn. Reset to false at the beginning
    /// of each player's turn (in `reset_turn_state`).
    #[serde(default)]
    pub loyalty_ability_activated_this_turn: bool,
    /// CR 716.2b: Class level designation. 0 means "no level set" — treated as level 1
    /// when checking Class abilities (CR 716.2d). Set to 1 when a Class enters the battlefield.
    /// Levels are not a copiable characteristic.
    #[serde(default)]
    pub class_level: u32,
    /// CR 702.37 / 701.40 / 701.58 / 702.168: Why this object is face-down (if at all).
    ///
    /// `None` means the object is either face-up, or face-down for reasons unrelated to
    /// morph/manifest/cloak (e.g., Foretell exile). `Some(kind)` means this object was
    /// specifically turned face-down via morph, megamorph, disguise, manifest, or cloak.
    ///
    /// Used by the layer system to apply face-down characteristic overrides (CR 708.2a)
    /// and to determine valid turn-face-up methods (CR 702.37e / 702.168d / 701.40b).
    ///
    /// Reset to `None` on zone changes (CR 400.7). Cleared when turned face up.
    #[serde(default)]
    pub face_down_as: Option<FaceDownKind>,
}

/// CR 729.2: A single component in a merged permanent.
///
/// When a mutating creature spell resolves, it merges with the target permanent.
/// Each card involved in the merge is represented as a `MergedComponent`.
/// `merged_components[0]` is always the topmost component (CR 729.2a).
///
/// An unmerged permanent has `merged_components` empty — not a vec of one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergedComponent {
    /// The CardId of this component (for looking up definitions and zone-change reconstruction).
    pub card_id: Option<crate::state::player::CardId>,
    /// The base characteristics of this component, frozen at merge time.
    /// Used to reconstruct individual GameObjects when the merged permanent leaves the battlefield (CR 729.3).
    pub characteristics: Characteristics,
    /// True if this component is a token.
    pub is_token: bool,
}

impl GameObject {
    /// CR 702.26b: Returns true if this permanent is visible on the battlefield.
    /// Phased-out permanents are "treated as though they do not exist" for all
    /// game purposes except rules that specifically mention phased-out permanents.
    pub fn is_phased_in(&self) -> bool {
        !self.status.phased_out
    }
}
