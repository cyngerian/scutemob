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
    /// CR 702.74a: If true, this pending trigger is the evoke sacrifice trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::EvokeSacrificeTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The `ability_index`
    /// field is unused when this is true.
    #[serde(default)]
    pub is_evoke_sacrifice: bool,
    /// CR 508.5 / CR 702.86a: The defending player for SelfAttacks triggers.
    ///
    /// Populated when a `SelfAttacks` trigger fires. At flush time, this PlayerId
    /// is set as `Target::Player` at index 0 so the annihilator effect's
    /// `PlayerTarget::DeclaredTarget { index: 0 }` resolves to the correct
    /// defending player. Also usable by any future "whenever this attacks,
    /// [effect on defending player]" trigger. `None` for all other trigger types.
    #[serde(default)]
    pub defending_player_id: Option<PlayerId>,
    /// CR 702.35a: If true, this pending trigger is a Madness trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::MadnessTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The fields
    /// `madness_exiled_card` and `madness_cost` carry the madness-specific data.
    /// The `ability_index` field is unused when this is true.
    #[serde(default)]
    pub is_madness_trigger: bool,
    /// CR 702.35a: ObjectId of the card in exile (new ID after the discard replacement).
    ///
    /// Only meaningful when `is_madness_trigger` is true.
    #[serde(default)]
    pub madness_exiled_card: Option<ObjectId>,
    /// CR 702.35a: The madness alternative cost captured at trigger time.
    ///
    /// Only meaningful when `is_madness_trigger` is true.
    #[serde(default)]
    pub madness_cost: Option<ManaCost>,
    /// CR 702.94a: If true, this pending trigger is a Miracle trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::MiracleTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The fields
    /// `miracle_revealed_card` and `miracle_cost` carry the miracle-specific data.
    /// The `ability_index` field is unused when this is true.
    #[serde(default)]
    pub is_miracle_trigger: bool,
    /// CR 702.94a: ObjectId of the revealed card in hand.
    ///
    /// Only meaningful when `is_miracle_trigger` is true.
    #[serde(default)]
    pub miracle_revealed_card: Option<ObjectId>,
    /// CR 702.94a: The miracle alternative cost captured at trigger time.
    ///
    /// Only meaningful when `is_miracle_trigger` is true.
    #[serde(default)]
    pub miracle_cost: Option<ManaCost>,
    /// CR 702.84a: If true, this pending trigger is the unearth delayed exile trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::UnearthTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The `ability_index`
    /// field is unused when this is true.
    #[serde(default)]
    pub is_unearth_trigger: bool,
    /// CR 702.110a: If true, this pending trigger is an Exploit trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::ExploitTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The `ability_index`
    /// field is unused when this is true.
    #[serde(default)]
    pub is_exploit_trigger: bool,
    /// CR 702.43a: If true, this pending trigger is a Modular trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::ModularTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The
    /// `modular_counter_count` carries the +1/+1 counter count from last-known
    /// information (pre_death_counters). The `ability_index` field is unused
    /// when this is true.
    #[serde(default)]
    pub is_modular_trigger: bool,
    /// CR 702.43a: Number of +1/+1 counters on the creature at death time.
    ///
    /// Only meaningful when `is_modular_trigger` is true. Captured from
    /// `pre_death_counters[PlusOnePlusOne]` at trigger-check time (last-known
    /// information per Arcbound Worker ruling 2006-09-25).
    #[serde(default)]
    pub modular_counter_count: Option<u32>,
    /// CR 702.100a: If true, this pending trigger is an Evolve trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::EvolveTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The
    /// `evolve_entering_creature` carries the ObjectId of the creature that
    /// entered the battlefield and triggered evolve.
    #[serde(default)]
    pub is_evolve_trigger: bool,
    /// CR 702.100a: ObjectId of the creature that entered the battlefield.
    ///
    /// Only meaningful when `is_evolve_trigger` is true. Used at resolution
    /// time for the intervening-if re-check (P/T comparison, CR 603.4).
    /// If this creature left the battlefield, use last-known information.
    #[serde(default)]
    pub evolve_entering_creature: Option<ObjectId>,
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
