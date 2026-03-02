//! Placeholder types for systems implemented in later milestones.
//!
//! These exist so GameState can compile with all fields from the architecture
//! doc. Each type will be fully fleshed out in its respective milestone.

use serde::{Deserialize, Serialize};

use super::game_object::{ManaCost, ObjectId};
use super::player::PlayerId;

// ContinuousEffect has moved to `state/continuous_effect.rs` (M5).

/// A delayed trigger waiting for a condition (CR 603.7). Implemented in M3.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayedTrigger {
    pub source: ObjectId,
}

// ReplacementEffect has moved to `state/replacement_effect.rs` (M8).

/// Discriminant for PendingTrigger — replaces per-trigger boolean fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingTriggerKind {
    /// Normal triggered ability — dispatched by ability_index on the source.
    Normal,
    /// CR 702.74a: Evoke sacrifice trigger.
    Evoke,
    /// CR 702.35a: Madness trigger.
    Madness,
    /// CR 702.94a: Miracle trigger.
    Miracle,
    /// CR 702.84a: Unearth delayed exile trigger.
    Unearth,
    /// CR 702.110a: Exploit ETB trigger.
    Exploit,
    /// CR 702.43a: Modular dies trigger.
    Modular,
    /// CR 702.100a: Evolve ETB trigger.
    Evolve,
    /// CR 702.116a: Myriad attack trigger.
    Myriad,
    /// CR 702.62a: Suspend upkeep counter-removal trigger.
    SuspendCounter,
    /// CR 702.62a: Suspend cast trigger (last counter removed).
    SuspendCast,
    /// CR 702.75a: Hideaway ETB trigger.
    Hideaway,
    /// CR 702.124j: Partner With ETB trigger.
    PartnerWith,
    /// CR 702.115a: Ingest combat damage trigger.
    Ingest,
    /// CR 702.25a: Flanking trigger.
    Flanking,
    /// CR 702.23a: Rampage becomes-blocked trigger.
    Rampage,
    /// CR 702.39a: Provoke attack trigger.
    Provoke,
    /// CR 702.112a: Renown combat damage trigger.
    Renown,
    /// CR 702.121a: Melee attack trigger.
    Melee,
    /// CR 702.70a: Poisonous combat damage trigger.
    Poisonous,
    /// CR 702.154a: Enlist attack trigger.
    Enlist,
    /// CR 702.141a: Encore delayed sacrifice trigger.
    EncoreSacrifice,
    /// CR 702.109a: Dash delayed return-to-hand trigger.
    DashReturn,
    // Add new trigger kinds here as abilities are implemented
}

/// A triggered ability queued to go on the stack (CR 603.3).
///
/// Collected after each event in `GameState::pending_triggers`; placed on
/// the stack in APNAP order the next time a player would receive priority.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingTrigger {
    /// The source object of the triggered ability.
    pub source: ObjectId,
    /// Index into `source.characteristics.triggered_abilities`.
    pub ability_index: usize,
    /// The player who controls this triggered ability.
    pub controller: PlayerId,
    /// Discriminant replacing all is_X_trigger boolean fields.
    ///
    /// For normal triggered abilities use `PendingTriggerKind::Normal`.
    /// For special trigger kinds, use the appropriate variant; `ability_index` is
    /// unused in those cases.
    #[serde(skip, default = "PendingTriggerKind::normal_default")]
    pub kind: PendingTriggerKind,
    /// The game event that caused this trigger to fire.
    ///
    /// Stored for Panharmonicon-style trigger doubling (CR 603.2d): the doubler
    /// needs to know what event caused the trigger to determine if it applies.
    /// `None` for triggers queued without a specific event context (e.g., delayed triggers).
    #[serde(default)]
    pub triggering_event: Option<super::game_object::TriggerEvent>,
    /// The object that entered the battlefield and caused this trigger to fire, if applicable.
    ///
    /// Populated for `AnyPermanentEntersBattlefield` triggers so that
    /// `TriggerDoublerFilter::ArtifactOrCreatureETB` can verify the entering
    /// object's card types (CR 603.2d — Panharmonicon doubles only when an
    /// artifact or creature enters, not any permanent).
    /// `None` for triggers that are not caused by a specific permanent entering.
    #[serde(default)]
    pub entering_object_id: Option<ObjectId>,
    /// CR 702.21a: The stack object that targeted this permanent (for Ward).
    ///
    /// Populated when a `SelfBecomesTargetByOpponent` trigger fires. At flush
    /// time, this ID is used to set the Ward triggered ability's target so the
    /// resolution can counter the correct spell or ability. `None` for all
    /// other trigger types.
    #[serde(default)]
    pub targeting_stack_id: Option<ObjectId>,
    /// CR 603.2 / CR 102.2: The player whose action triggered this ability.
    ///
    /// Populated when an `OpponentCastsSpell` trigger fires. At flush time,
    /// this is converted to `Target::Player(triggering_player)` at target
    /// index 0 so `DeclaredTarget { index: 0 }` can resolve to the specific
    /// opponent who cast the spell (e.g. Rhystic Study's "that player pays {1}").
    /// `None` for all other trigger types.
    #[serde(default)]
    pub triggering_player: Option<PlayerId>,
    /// CR 702.83a: The lone attacker's ObjectId for Exalted triggers.
    ///
    /// Populated when a `ControllerCreatureAttacksAlone` trigger fires. At flush
    /// time, this ID is set as `Target::Object(attacker_id)` at index 0 so the
    /// effect's `CEFilter::DeclaredTarget { index: 0 }` resolves to the correct
    /// creature (the lone attacker, not the exalted source).
    /// `None` for all other trigger types.
    #[serde(default)]
    pub exalted_attacker_id: Option<ObjectId>,
    /// CR 508.5 / CR 702.86a: The defending player for SelfAttacks triggers.
    ///
    /// Populated when a `SelfAttacks` trigger fires. At flush time, this PlayerId
    /// is set as `Target::Player` at index 0 so the annihilator effect's
    /// `PlayerTarget::DeclaredTarget { index: 0 }` resolves to the correct
    /// defending player. Also usable by any future "whenever this attacks,
    /// [effect on defending player]" trigger. `None` for all other trigger types.
    #[serde(default)]
    pub defending_player_id: Option<PlayerId>,
    /// CR 702.35a: ObjectId of the card in exile (new ID after the discard replacement).
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Madness`.
    #[serde(default)]
    pub madness_exiled_card: Option<ObjectId>,
    /// CR 702.35a: The madness alternative cost captured at trigger time.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Madness`.
    #[serde(default)]
    pub madness_cost: Option<ManaCost>,
    /// CR 702.94a: ObjectId of the revealed card in hand.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Miracle`.
    #[serde(default)]
    pub miracle_revealed_card: Option<ObjectId>,
    /// CR 702.94a: The miracle alternative cost captured at trigger time.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Miracle`.
    #[serde(default)]
    pub miracle_cost: Option<ManaCost>,
    /// CR 702.43a: Number of +1/+1 counters on the creature at death time.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Modular`. Captured from
    /// `pre_death_counters[PlusOnePlusOne]` at trigger-check time (last-known
    /// information per Arcbound Worker ruling 2006-09-25).
    #[serde(default)]
    pub modular_counter_count: Option<u32>,
    /// CR 702.100a: ObjectId of the creature that entered the battlefield.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Evolve`. Used at resolution
    /// time for the intervening-if re-check (P/T comparison, CR 603.4).
    /// If this creature left the battlefield, use last-known information.
    #[serde(default)]
    pub evolve_entering_creature: Option<ObjectId>,
    /// CR 702.62a: ObjectId of the suspended card in exile.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::SuspendCounter` or
    /// `kind == PendingTriggerKind::SuspendCast`.
    #[serde(default)]
    pub suspend_card_id: Option<ObjectId>,
    /// CR 702.75a: Number of cards to look at from the top of the library.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Hideaway`.
    #[serde(default)]
    pub hideaway_count: Option<u32>,
    /// CR 702.124j: The exact name of the partner card to search for.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::PartnerWith`.
    #[serde(default)]
    pub partner_with_name: Option<String>,
    /// CR 702.115a: The player dealt combat damage (whose library top card is exiled).
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Ingest`.
    #[serde(default)]
    pub ingest_target_player: Option<PlayerId>,
    /// CR 702.25a: The blocking creature that gets -1/-1 until end of turn.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Flanking`.
    #[serde(default)]
    pub flanking_blocker_id: Option<ObjectId>,
    /// CR 702.23a: The N value of the Rampage keyword (e.g., 2 for Rampage 2).
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Rampage`.
    #[serde(default)]
    pub rampage_n: Option<u32>,
    /// CR 702.39a: The ObjectId of the creature that must block "if able".
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Provoke`. This is the target
    /// creature the defending player controls. Set at trigger-collection time
    /// in the AttackersDeclared handler in `abilities.rs`.
    ///
    /// If `None` (no eligible target exists), the trigger is not placed on the stack (CR 603.3d).
    #[serde(default)]
    pub provoke_target_creature: Option<ObjectId>,
    /// CR 702.112a: The N value from "Renown N" -- how many +1/+1 counters
    /// to place on the creature when the trigger resolves.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Renown`.
    #[serde(default)]
    pub renown_n: Option<u32>,
    /// CR 702.70a: The N value from "Poisonous N" -- how many poison counters
    /// to give the damaged player when the trigger resolves.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Poisonous`.
    #[serde(default)]
    pub poisonous_n: Option<u32>,
    /// CR 702.70a: The player dealt combat damage (who receives poison counters).
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Poisonous`.
    #[serde(default)]
    pub poisonous_target_player: Option<PlayerId>,
    /// CR 702.154a: The ObjectId of the creature tapped for the enlist cost.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Enlist`. Used at resolution
    /// time to read the enlisted creature's power for the +X/+0 bonus.
    #[serde(default)]
    pub enlist_enlisted_creature: Option<ObjectId>,
    /// CR 702.141a: The player who activated the encore ability.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::EncoreSacrifice`. Used at
    /// resolution time to verify the token is still under this player's control
    /// before sacrificing.
    #[serde(default)]
    pub encore_activator: Option<PlayerId>,
}

impl PendingTriggerKind {
    /// Default constructor for serde skip fields.
    pub fn normal_default() -> PendingTriggerKind {
        PendingTriggerKind::Normal
    }
}

// StackObject has moved to `state/stack.rs` (M3-A).

// CombatState has moved to `state/combat.rs` (M6).

// GameEvent has moved to crate::rules::events (M2).

/// Which triggers are doubled by a `TriggerDoubler` (CR 603.2d).
///
/// Used to filter which pending triggers get additional copies queued when
/// `flush_pending_triggers` processes them. Adding new filter variants here
/// enables new Panharmonicon-style cards without touching `flush_pending_triggers`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerDoublerFilter {
    /// "Whenever an artifact or creature enters the battlefield" — Panharmonicon.
    ///
    /// Doubles ETB triggered abilities from artifacts and creatures (CR 603.2d).
    /// Specifically: any triggered ability on a permanent that would trigger from
    /// a creature or artifact permanent entering the battlefield.
    ArtifactOrCreatureETB,
}

/// A Panharmonicon-style trigger-doubling effect (CR 603.2d).
///
/// When a trigger that matches the filter would be queued, it is queued an
/// additional `additional_triggers` times instead. Each instance resolves
/// independently on the stack.
///
/// Registered when a permanent with the appropriate ability enters the
/// battlefield; unregistered (by source ObjectId) when it leaves.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerDoubler {
    /// ObjectId of the permanent generating this effect (Panharmonicon, etc.).
    pub source: ObjectId,
    /// The player who controls the source permanent.
    pub controller: PlayerId,
    /// Which ETB triggers are doubled by this effect.
    pub filter: TriggerDoublerFilter,
    /// How many additional times the trigger fires (usually 1).
    pub additional_triggers: u32,
}
