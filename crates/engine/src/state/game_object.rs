//! Game object types: ObjectId, characteristics, status, and the GameObject struct.

use im::{OrdMap, OrdSet, Vector};
use serde::{Deserialize, Serialize};

use super::player::{CardId, PlayerId};
use super::types::{
    AltCostKind, CardType, Color, CounterType, KeywordAbility, ManaColor, SubType, SuperType,
};
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
    /// CR 602.5d: If true, this ability can only be activated at sorcery speed
    /// (main phase, stack empty, active player only).
    #[serde(default)]
    pub sorcery_speed: bool,
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
    /// CR 702.103b: If true, this permanent is currently bestowed. While bestowed,
    /// it is an Aura enchantment (NOT a creature) with enchant creature.
    /// CR 702.103f: When it becomes unattached, it ceases to be bestowed and
    /// reverts to an enchantment creature -- this is an exception to CR 704.5m
    /// (normal Auras go to graveyard when unattached; bestowed Auras become creatures).
    ///
    /// Set during spell resolution when the permanent enters the battlefield
    /// as a bestowed Aura. Reset to false when unattached (SBA) or on zone
    /// changes (CR 400.7).
    #[serde(default)]
    pub is_bestowed: bool,
    /// CR 702.143a: If true, this object in exile was foretold (exiled face-down
    /// via the foretell special action). Used to determine whether the card can be
    /// cast from exile for its foretell cost.
    ///
    /// Set when the ForetellCard command is processed. Reset to false on zone
    /// changes (CR 400.7) -- but since foretold cards are already in exile,
    /// any zone change from exile clears this.
    #[serde(default)]
    pub is_foretold: bool,
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
    /// CR 702.62b: If true, this object in exile was suspended (exiled via the
    /// suspend special action from hand with time counters). Used to identify
    /// suspended cards for the upkeep counter-removal trigger.
    ///
    /// A card is "suspended" (CR 702.62b) if it's in exile, has suspend, AND
    /// has a time counter on it. This flag is set when the suspend special action
    /// exiles the card. Unlike foretell, suspended cards are exiled face up.
    #[serde(default)]
    pub is_suspended: bool,
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
    /// CR 702.112b: Renowned designation. Tracked as a boolean flag on the
    /// permanent. Once set by a resolved Renown trigger, stays true until
    /// the permanent leaves the battlefield (CR 400.7 resets it).
    ///
    /// NOT a copiable value (CR 702.112b) -- copies start non-renowned.
    /// NOT an ability -- persists even if abilities are removed (e.g., Humility).
    #[serde(default)]
    pub is_renowned: bool,
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
}
