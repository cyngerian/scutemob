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
    /// CR 702.116a: If true, this pending trigger is a Myriad attack trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::MyriadTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The
    /// `defending_player_id` field carries the defending player (the one NOT
    /// to create copies for). The `ability_index` field is unused when this
    /// is true.
    #[serde(default)]
    pub is_myriad_trigger: bool,
    /// CR 702.62a: If true, this pending trigger is a suspend upkeep
    /// counter-removal trigger ("at the beginning of your upkeep, remove a
    /// time counter from this suspended card").
    ///
    /// When flushed to the stack, creates a `StackObjectKind::SuspendCounterTrigger`.
    /// When this trigger resolves, one time counter is removed from the suspended
    /// card. If that was the last counter, a SuspendCastTrigger is immediately queued.
    #[serde(default)]
    pub is_suspend_counter_trigger: bool,
    /// CR 702.62a: If true, this pending trigger is the suspend cast trigger
    /// ("when the last time counter is removed, you may cast it without paying
    /// its mana cost").
    ///
    /// When flushed to the stack, creates a `StackObjectKind::SuspendCastTrigger`.
    /// When this trigger resolves, the card is cast for free from exile.
    #[serde(default)]
    pub is_suspend_cast_trigger: bool,
    /// CR 702.62a: ObjectId of the suspended card in exile.
    ///
    /// Only meaningful when `is_suspend_counter_trigger` or `is_suspend_cast_trigger`
    /// is true.
    #[serde(default)]
    pub suspend_card_id: Option<ObjectId>,
    /// CR 702.75a: If true, this pending trigger is a Hideaway ETB trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::HideawayTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`.  The
    /// `hideaway_count` field carries the N parameter.  The `ability_index`
    /// field is unused when this is true.
    #[serde(default)]
    pub is_hideaway_trigger: bool,
    /// CR 702.75a: Number of cards to look at from the top of the library.
    ///
    /// Only meaningful when `is_hideaway_trigger` is true.
    #[serde(default)]
    pub hideaway_count: Option<u32>,
    /// CR 702.124j: If true, this pending trigger is a Partner With ETB trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::PartnerWithTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The
    /// `partner_with_name` carries the partner's exact card name. The
    /// `ability_index` field is unused when this is true.
    #[serde(default)]
    pub is_partner_with_trigger: bool,
    /// CR 702.124j: The exact name of the partner card to search for.
    ///
    /// Only meaningful when `is_partner_with_trigger` is true.
    #[serde(default)]
    pub partner_with_name: Option<String>,
    /// CR 702.115a: If true, this pending trigger is an Ingest trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::IngestTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`.
    /// The `ingest_target_player` carries the damaged player's ID so the
    /// resolution knows whose library to exile from.
    #[serde(default)]
    pub is_ingest_trigger: bool,
    /// CR 702.115a: The player dealt combat damage (whose library top card is exiled).
    ///
    /// Only meaningful when `is_ingest_trigger` is true.
    #[serde(default)]
    pub ingest_target_player: Option<PlayerId>,
    /// CR 702.25a: If true, this pending trigger is a Flanking trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::FlankingTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`.
    /// The `flanking_blocker_id` carries the blocking creature's ObjectId so
    /// the resolution knows which creature to apply -1/-1 to.
    #[serde(default)]
    pub is_flanking_trigger: bool,
    /// CR 702.25a: The blocking creature that gets -1/-1 until end of turn.
    ///
    /// Only meaningful when `is_flanking_trigger` is true.
    #[serde(default)]
    pub flanking_blocker_id: Option<ObjectId>,
    /// CR 702.23a: If true, this pending trigger is a Rampage N trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::RampageTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The
    /// `rampage_n` field carries the N parameter from the Rampage keyword.
    /// CR 702.23b: Bonus is calculated at resolution time from `state.combat`.
    #[serde(default)]
    pub is_rampage_trigger: bool,
    /// CR 702.23a: The N value of the Rampage keyword (e.g., 2 for Rampage 2).
    ///
    /// Only meaningful when `is_rampage_trigger` is true.
    #[serde(default)]
    pub rampage_n: Option<u32>,
    /// CR 702.39a: If true, this pending trigger is a Provoke trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::ProvokeTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The
    /// `provoke_target_creature` carries the ObjectId of the creature to be
    /// provoked. The `ability_index` field is unused when this is true.
    ///
    /// If `provoke_target_creature` is `None` (no eligible target exists),
    /// the trigger is not placed on the stack (CR 603.3d).
    #[serde(default)]
    pub is_provoke_trigger: bool,
    /// CR 702.39a: The ObjectId of the creature that must block "if able".
    ///
    /// Only meaningful when `is_provoke_trigger` is true. This is the target
    /// creature the defending player controls. Set at trigger-collection time
    /// in the AttackersDeclared handler in `abilities.rs`.
    #[serde(default)]
    pub provoke_target_creature: Option<ObjectId>,
    /// CR 702.112a: If true, this pending trigger is a Renown trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::RenownTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`.
    /// The `renown_n` carries the N value (number of +1/+1 counters).
    #[serde(default)]
    pub is_renown_trigger: bool,
    /// CR 702.112a: The N value from "Renown N" -- how many +1/+1 counters
    /// to place on the creature when the trigger resolves.
    ///
    /// Only meaningful when `is_renown_trigger` is true.
    #[serde(default)]
    pub renown_n: Option<u32>,
    /// CR 702.121a: If true, this pending trigger is a Melee trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::MeleeTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`.
    /// The bonus is computed at resolution time from `state.combat`.
    #[serde(default)]
    pub is_melee_trigger: bool,
    /// CR 702.70a: If true, this pending trigger is a Poisonous trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::PoisonousTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`.
    /// The `poisonous_n` carries the N value (number of poison counters).
    /// The `poisonous_target_player` carries the damaged player's ID.
    #[serde(default)]
    pub is_poisonous_trigger: bool,
    /// CR 702.70a: The N value from "Poisonous N" -- how many poison counters
    /// to give the damaged player when the trigger resolves.
    ///
    /// Only meaningful when `is_poisonous_trigger` is true.
    #[serde(default)]
    pub poisonous_n: Option<u32>,
    /// CR 702.70a: The player dealt combat damage (who receives poison counters).
    ///
    /// Only meaningful when `is_poisonous_trigger` is true.
    #[serde(default)]
    pub poisonous_target_player: Option<PlayerId>,
    /// CR 702.154a: If true, this pending trigger is an Enlist trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::EnlistTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The
    /// `enlist_enlisted_creature` carries the ObjectId of the creature that
    /// was tapped as the enlist cost.
    #[serde(default)]
    pub is_enlist_trigger: bool,
    /// CR 702.154a: The ObjectId of the creature tapped for the enlist cost.
    ///
    /// Only meaningful when `is_enlist_trigger` is true. Used at resolution
    /// time to read the enlisted creature's power for the +X/+0 bonus.
    #[serde(default)]
    pub enlist_enlisted_creature: Option<ObjectId>,
    /// CR 702.141a: If true, this pending trigger is an Encore sacrifice trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::EncoreSacrificeTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The
    /// `encore_activator` field carries the player who activated encore (to verify
    /// control at resolution). The `ability_index` field is unused when this is true.
    #[serde(default)]
    pub is_encore_sacrifice_trigger: bool,
    /// CR 702.141a: The player who activated the encore ability.
    ///
    /// Only meaningful when `is_encore_sacrifice_trigger` is true. Used at
    /// resolution time to verify the token is still under this player's control
    /// before sacrificing.
    #[serde(default)]
    pub encore_activator: Option<PlayerId>,
    /// CR 702.109a: If true, this pending trigger is a dash return-to-hand trigger.
    ///
    /// When flushed to the stack, creates a `StackObjectKind::DashReturnTrigger`
    /// instead of the normal `StackObjectKind::TriggeredAbility`. The `ability_index`
    /// field is unused when this is true.
    #[serde(default)]
    pub is_dash_return_trigger: bool,
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
