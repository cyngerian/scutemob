//! Game object types: ObjectId, characteristics, status, and the GameObject struct.

use im::{OrdMap, OrdSet, Vector};
use serde::{Deserialize, Serialize};

use super::player::{CardId, PlayerId};
use super::types::{CardType, Color, CounterType, KeywordAbility, ManaColor, SubType, SuperType};
use super::zone::ZoneId;

/// Identifies a game object instance. Per CR 400.7, when an object changes
/// zones it becomes a new object with a new ObjectId.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u64);

/// Mana cost of a card or ability (CR 202).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaCost {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
    pub generic: u32,
}

impl ManaCost {
    /// Mana value (formerly "converted mana cost") per CR 202.3.
    pub fn mana_value(&self) -> u32 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless + self.generic
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
}

impl ManaAbility {
    /// Convenience constructor: tap this permanent to add one mana of `color`.
    pub fn tap_for(color: ManaColor) -> Self {
        let mut produces = OrdMap::new();
        produces.insert(color, 1);
        Self {
            produces,
            requires_tap: true,
        }
    }
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
}

/// Intervening-if clause for conditional triggered abilities (CR 603.4).
///
/// The condition is checked at trigger time (ability only triggers if true)
/// and again at resolution (ability only resolves if still true).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterveningIf {
    /// "if your life total is [N] or more" — for testing conditional triggers.
    ControllerLifeAtLeast(u32),
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
    /// True if this Aura has an "Enchant creature" restriction (CR 702.5b, 704.5m).
    ///
    /// When set, the SBA for Aura legality (CR 704.5m) additionally checks that the
    /// attached permanent is a creature using the layer-computed characteristics. If
    /// the target stops being a creature (e.g., a type-change animation expires), the
    /// aura falls off.
    ///
    /// Default is `false` (no type restriction on attachment target beyond "on battlefield").
    #[serde(default)]
    pub enchants_creatures: bool,
    /// CR 701.15a: Players who have goaded this creature.
    ///
    /// Non-empty when a Goad effect has been applied to this permanent.
    /// A goaded creature must attack each combat if able (CR 701.15b) and
    /// must attack a player other than the goading player if able (CR 701.15b).
    /// This list is cleared when the creature's controller's next turn begins.
    #[serde(default)]
    pub goaded_by: Vector<PlayerId>,
}
