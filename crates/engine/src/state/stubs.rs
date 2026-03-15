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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PendingTriggerKind {
    /// Normal triggered ability — dispatched by ability_index on the source.
    Normal,
    /// Card-definition ETB trigger — `ability_index` is into the card registry's
    /// `CardDefinition::abilities` Vec (NOT into the runtime `triggered_abilities`).
    ///
    /// Created by `queue_carddef_etb_triggers`. At resolution, the effect and
    /// intervening_if are looked up from the card registry (never runtime triggered_abilities).
    /// This avoids index-namespace collisions when the card also has runtime triggers
    /// added by `enrich_spec_from_def` that happen to use the same index.
    CardDefETB,
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
    /// CR 702.152a: Blitz delayed sacrifice trigger.
    BlitzSacrifice,
    // ImpendingCounter: migrated to KeywordTrigger
    // VanishingCounter and VanishingSacrifice: migrated to KeywordTrigger
    // FadingUpkeep: migrated to KeywordTrigger
    // EchoUpkeep: migrated to KeywordTrigger
    // CumulativeUpkeep: migrated to KeywordTrigger
    /// CR 702.59a: Recover trigger -- fired when a creature enters the card owner's
    /// graveyard from the battlefield. Carries recover_cost and recover_card for
    /// flush_pending_triggers to build the RecoverTrigger stack entry.
    Recover,
    /// CR 702.58a: Graft trigger -- fired when another creature enters the
    /// battlefield while this permanent has a +1/+1 counter on it.
    /// Carries graft_entering_creature for flush_pending_triggers to build the
    /// GraftTrigger stack entry.
    Graft,
    /// CR 702.165a: Backup trigger -- fired when this creature enters the battlefield.
    /// Carries backup_abilities (keyword abilities to grant, locked at trigger time per
    /// CR 702.165d) and backup_n (number of +1/+1 counters to place).
    Backup,
    /// CR 702.72a: Champion ETB trigger -- "sacrifice it unless you exile
    /// another [object] you control."
    ChampionETB,
    /// CR 702.72a: Champion LTB trigger -- "return the exiled card to the
    /// battlefield under its owner's control."
    ChampionLTB,
    /// CR 702.95a: Soulbond self-ETB trigger -- fired when the soulbond creature
    /// enters the battlefield (first sentence of CR 702.95a). The soulbond creature
    /// is the source; soulbond_pair_target carries the auto-selected partner.
    SoulbondSelfETB,
    /// CR 702.95a: Soulbond other-ETB trigger -- fired when another creature enters
    /// while an unpaired soulbond creature is already on the battlefield (second
    /// sentence of CR 702.95a). The soulbond creature is the source;
    /// soulbond_pair_target carries the entering creature.
    SoulbondOtherETB,
    /// CR 702.156a: Ravenous draw trigger -- "When this permanent enters, if X is 5
    /// or more, draw a card." X is the value chosen at cast time (CR 107.3m).
    /// Intervening-if: checked at trigger time (in resolution.rs) and at resolution
    /// (in the RavenousDrawTrigger SOK arm). Since X is immutable after cast, the
    /// re-check always passes if it triggered.
    RavenousDraw,
    /// CR 702.157a: Squad ETB trigger -- fires when the creature with Squad enters
    /// the battlefield and its squad cost was paid at least once.
    /// Intervening-if (CR 603.4): only queued when `squad_count > 0` AND the permanent
    /// has `KeywordAbility::Squad` in layer-resolved characteristics.
    /// Resolved to create `squad_count` token copies of the source creature.
    SquadETB,
    /// CR 702.175a: Offspring ETB trigger -- fires when the creature with Offspring enters
    /// the battlefield and its offspring cost was paid.
    /// Intervening-if (CR 603.4): only queued when `offspring_paid == true` AND the permanent
    /// has `KeywordAbility::Offspring` in layer-resolved characteristics.
    /// Resolved to create 1 token copy (except 1/1) of the source creature.
    OffspringETB,
    /// CR 702.174b: Gift ETB trigger -- fires when the permanent with Gift enters
    /// the battlefield and its gift cost was paid.
    /// Intervening-if (CR 603.4): only queued when `gift_was_given == true` AND the permanent
    /// has `KeywordAbility::Gift` in layer-resolved characteristics.
    /// Resolved to give the chosen opponent the gift defined by `AbilityDefinition::Gift`.
    GiftETB,
    /// CR 702.99a: Cipher combat damage trigger -- "Whenever [encoded creature] deals
    /// combat damage to a player, you may copy the encoded card and you may cast the
    /// copy without paying its mana cost."
    ///
    /// Fired in the CombatDamageDealt handler for each creature with non-empty
    /// `encoded_cards` that dealt > 0 combat damage to a player.
    /// Carries the encoded card information through to `flush_pending_triggers`
    /// to build the `CipherTrigger` SOK.
    CipherCombatDamage,
    /// CR 702.55a: Haunt exile trigger -- "When this creature dies / this spell is
    /// put into a graveyard during its resolution, exile it haunting target creature."
    ///
    /// Fired in the CreatureDied handler when the dead creature had KeywordAbility::Haunt.
    /// Also fired during instant/sorcery resolution for spell Haunt.
    /// Carries `haunt_source_card_id` for the HauntExileTrigger SOK.
    HauntExile,
    /// CR 702.55c: Haunted creature dies trigger -- fires the haunt card's effect
    /// from exile when the creature it haunts dies.
    ///
    /// Fired in the CreatureDied handler when an exiled card has a matching haunting_target.
    /// Carries `haunt_source_object_id` for the HauntedCreatureDiesTrigger SOK.
    HauntedCreatureDies,
    /// CR 708.8 / CR 702.37e: "When this permanent is turned face up" triggered ability.
    ///
    /// Fired in `check_triggers` when `GameEvent::PermanentTurnedFaceUp` fires and the
    /// permanent has `TriggerCondition::WhenTurnedFaceUp` in its CardDefinition.
    /// The source is the permanent itself; `source_card_id` is looked up at flush time.
    /// Resolves as `TurnFaceUpTrigger` SOK.
    TurnFaceUp,
    /// Consolidated keyword trigger (replaces many one-off trigger variants).
    KeywordTrigger {
        keyword: crate::state::types::KeywordAbility,
        data: crate::state::stack::TriggerData,
    },
    /// CR 701.54c (ring level >= 2): Ring-bearer attack loot trigger.
    ///
    /// Fired in AttackersDeclared handler when the attacker is the controller's ring-bearer
    /// and ring_level >= 2. At flush time, creates a RingAbility SOK with a Sequence effect
    /// (DrawCards(1) then DiscardCards(1)).
    RingLoot,
    /// CR 701.54c (ring level >= 3): Ring-bearer becomes-blocked sacrifice trigger.
    ///
    /// Fired in BlockersDeclared handler when a creature blocks the ring-bearer and
    /// ring_level >= 3. At flush time, creates a RingAbility SOK with a SacrificePermanents
    /// effect targeting the blocking creature's controller.
    /// The `source` of the PendingTrigger is the blocker ObjectId.
    RingBlockSacrifice,
    /// CR 701.54c (ring level >= 4): Ring-bearer combat damage trigger.
    ///
    /// Fired in CombatDamageDealt handler when the ring-bearer deals combat damage to a
    /// player and ring_level >= 4. At flush time, creates a RingAbility SOK with a
    /// LoseLife(EachOpponent, 3) effect.
    RingCombatDamage,
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
    /// CR 702.30a: The echo cost to pay (from KeywordAbility::Echo(cost)).
    ///
    /// Only meaningful when `kind == PendingTriggerKind::EchoUpkeep`.
    /// Carries the echo cost from trigger queueing to stack object creation.
    #[serde(default)]
    pub echo_cost: Option<ManaCost>,
    /// CR 702.24a: The per-counter cumulative upkeep cost.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::CumulativeUpkeep`.
    /// Carries the cost from trigger queueing to stack object creation.
    #[serde(default)]
    pub cumulative_upkeep_cost: Option<crate::state::types::CumulativeUpkeepCost>,
    /// CR 702.59a: The recover cost to pay (from AbilityDefinition::Recover { cost }).
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Recover`.
    /// Carries the recover cost from trigger queueing to stack object creation.
    #[serde(default)]
    pub recover_cost: Option<ManaCost>,
    /// CR 702.59a: The ObjectId of the Recover card in the graveyard.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Recover`.
    /// Carries the recover card id from trigger queueing to stack object creation.
    #[serde(default)]
    pub recover_card: Option<ObjectId>,
    /// CR 702.58a: The ObjectId of the creature that entered the battlefield.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Graft`. Used at resolution
    /// time for the intervening-if re-check (CR 603.4): source must still have a
    /// +1/+1 counter and the entering creature must still be on the battlefield.
    #[serde(default)]
    pub graft_entering_creature: Option<ObjectId>,
    /// CR 702.165d: The keyword abilities to grant to the target creature.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Backup`. Determined at
    /// trigger time from the card definition's abilities printed below the Backup
    /// keyword (CR 702.165a). Non-Backup keywords only (CR 702.165c).
    #[serde(default)]
    pub backup_abilities: Option<Vec<super::types::KeywordAbility>>,
    /// CR 702.165a: The N value from Backup N -- how many +1/+1 counters to place.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::Backup`.
    #[serde(default)]
    pub backup_n: Option<u32>,
    /// CR 702.72a: The champion filter (creature, Faerie, etc.) for the ETB trigger.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::ChampionETB`.
    /// Looked up from the card registry at trigger-collection time.
    #[serde(default)]
    pub champion_filter: Option<super::types::ChampionFilter>,
    /// CR 702.72a: The ObjectId of the card exiled by the champion ETB trigger.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::ChampionLTB`.
    /// Captured from `champion_exiled_card` on the post-move object at trigger time.
    #[serde(default)]
    pub champion_exiled_card: Option<ObjectId>,
    /// CR 702.95a: The ObjectId of the creature selected as the pairing target.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::SoulbondSelfETB` or
    /// `SoulbondOtherETB`. Auto-selected at trigger-collection time; carried
    /// through to `flush_pending_triggers` to build the `SoulbondTrigger` SOK.
    #[serde(default)]
    pub soulbond_pair_target: Option<ObjectId>,
    /// CR 702.157a: Number of times the squad cost was paid at cast time.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::SquadETB`.
    /// Carried from resolution.rs (where it's read from the permanent's squad_count)
    /// through to `flush_pending_triggers` to build the `SquadTrigger` SOK.
    #[serde(default)]
    pub squad_count: Option<u32>,
    /// CR 702.174a: The opponent chosen to receive the gift at cast time.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::GiftETB`.
    /// Carried from resolution.rs (where it's read from the permanent's gift_opponent)
    /// through to `flush_pending_triggers` to build the `GiftETBTrigger` SOK.
    #[serde(default)]
    pub gift_opponent: Option<crate::state::PlayerId>,
    /// CR 702.99a: The CardId of the encoded cipher card.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::CipherCombatDamage`.
    /// Carried through to `flush_pending_triggers` to build the `CipherTrigger` SOK.
    #[serde(default)]
    pub cipher_encoded_card_id: Option<crate::state::player::CardId>,
    /// CR 702.99a: The ObjectId of the exiled cipher card.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::CipherCombatDamage`.
    /// Used at resolution to verify the encoded card still exists in exile (CR 702.99c).
    #[serde(default)]
    pub cipher_encoded_object_id: Option<ObjectId>,
    /// CR 702.55a/c: The ObjectId of the haunt card (for HauntExile: in graveyard;
    /// for HauntedCreatureDies: in exile). Used to build the SOK at flush time.
    ///
    /// Only meaningful when `kind == PendingTriggerKind::HauntExile` or
    /// `kind == PendingTriggerKind::HauntedCreatureDies`.
    #[serde(default)]
    pub haunt_source_object_id: Option<ObjectId>,
    /// CR 702.55a: The CardId of the haunt card (for registry lookup).
    ///
    /// Only meaningful when `kind == PendingTriggerKind::HauntExile` or
    /// `kind == PendingTriggerKind::HauntedCreatureDies`.
    #[serde(default)]
    pub haunt_source_card_id: Option<crate::state::player::CardId>,
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
    /// "If a creature dying causes a triggered ability to trigger" — Teysa Karlov.
    ///
    /// Doubles death-triggered abilities (triggered by CreatureDied events).
    /// CR 603.2d: the trigger fires an additional time when a creature dying
    /// is the triggering event.
    CreatureDeath,
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

/// Which permanents have their ETB triggered abilities suppressed by an `ETBSuppressor`.
///
/// CR 614.16a: A "creatures entering the battlefield don't cause abilities to trigger"
/// effect prevents triggered abilities from triggering — the trigger never fires, rather
/// than being countered after firing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ETBSuppressFilter {
    /// Suppresses ETB triggered abilities on creatures only (Torpor Orb / Hushbringer pattern).
    ///
    /// Non-creature permanents entering the battlefield are unaffected.
    CreaturesOnly,
    /// Suppresses ETB triggered abilities on all permanents.
    AllPermanents,
}

/// A Torpor Orb-style ETB trigger suppressor (CR 614.16a).
///
/// When a permanent matching the filter enters the battlefield, its ETB triggered
/// abilities are suppressed — they do not trigger at all (not just countered).
///
/// Registered when a permanent with `AbilityDefinition::SuppressCreatureETBTriggers`
/// enters the battlefield; cleaned up when that permanent leaves.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ETBSuppressor {
    /// ObjectId of the permanent generating this suppression (Torpor Orb, etc.).
    pub source: ObjectId,
    /// Which entering permanents are affected.
    pub filter: ETBSuppressFilter,
}
