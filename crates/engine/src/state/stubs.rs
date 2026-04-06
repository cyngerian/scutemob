//! Placeholder types for systems implemented in later milestones.
//!
//! These exist so GameState can compile with all fields from the architecture
//! doc. Each type will be fully fleshed out in its respective milestone.
use super::game_object::{ManaCost, ObjectId};
use super::player::PlayerId;
use super::stack::TriggerData;
use serde::{Deserialize, Serialize};
// ContinuousEffect has moved to `state/continuous_effect.rs` (M5).
/// A delayed trigger waiting for a condition (CR 603.7).
///
/// Created when an effect like "exile until end of turn" or "exile until this
/// leaves the battlefield" resolves. The trigger fires at the appropriate time
/// and executes the stored action on the target object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayedTrigger {
    /// The source that created this delayed trigger (for tracking).
    pub source: ObjectId,
    /// The controller of the delayed trigger (CR 603.7d/e).
    pub controller: PlayerId,
    /// The object this delayed trigger acts upon (usually in exile or on battlefield).
    pub target_object: ObjectId,
    /// What this delayed trigger does when it fires.
    pub action: DelayedTriggerAction,
    /// When this delayed trigger fires.
    pub timing: DelayedTriggerTiming,
    /// Whether this trigger has already fired (CR 603.7b: fires only once).
    pub fired: bool,
}

/// What a delayed trigger does when it fires.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DelayedTriggerAction {
    /// Return the target object from exile to the battlefield under its owner's control.
    /// CR 610.3c: returns under owner's control unless otherwise specified.
    ReturnFromExileToBattlefield { tapped: bool },
    /// Return the target object from exile to its owner's hand.
    ReturnFromExileToHand,
    /// Return the target object from the graveyard to its owner's hand.
    ReturnFromGraveyardToHand,
    /// Sacrifice the target object (must still be on the battlefield).
    SacrificeObject,
    /// Exile the target object (must still be on the battlefield).
    ExileObject,
}

/// When a delayed trigger fires.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DelayedTriggerTiming {
    /// At the beginning of the next end step (any player's).
    AtNextEndStep,
    /// At the beginning of the target object's owner's next end step.
    AtOwnersNextEndStep,
    /// When the source permanent leaves the battlefield (CR 610.3).
    WhenSourceLeavesBattlefield,
    /// At the beginning of the next end of combat step.
    AtEndOfCombat,
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
    /// CR 603.7: A delayed triggered ability fires, executing a stored action
    /// on a target object (return from exile, sacrifice, exile, etc.).
    /// The action and target are carried in `PendingTrigger.data` as
    /// `TriggerData::DelayedAction { action, target }`.
    DelayedAction,
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
    // echo_cost: REMOVED — echo cost is read from KeywordAbility::Echo in the object's abilities
    // at KeywordTrigger (Echo) resolution time; no need to carry it in PendingTrigger.
    // cumulative_upkeep_cost: REMOVED — cumulative upkeep cost is read from KeywordAbility at
    // KeywordTrigger (CumulativeUpkeep) resolution time; no need to carry it in PendingTrigger.
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
    /// CR 510.3a: The player dealt combat damage (for combat damage triggers).
    /// Used at flush/resolution time to populate EffectContext::damaged_player.
    /// None for all other trigger types.
    #[serde(default)]
    pub damaged_player: Option<crate::state::player::PlayerId>,
    /// CR 510.3a: The amount of combat damage dealt (for damage-amount-dependent effects).
    /// Used at resolution time to populate EffectContext::combat_damage_amount.
    /// 0 for all other trigger types.
    #[serde(default)]
    pub combat_damage_amount: u32,
    /// Unified per-trigger payload. Replaces per-variant Option fields for
    /// trigger kinds that carry structured data. When `Some(TriggerData::X)`,
    /// `flush_pending_triggers` reads this field instead of the legacy per-field Options.
    ///
    /// Not serialized (same as `kind`) — triggers are transient within a turn.
    #[serde(skip)]
    pub data: Option<TriggerData>,
}
impl PendingTrigger {
    /// Construct a PendingTrigger with all Option fields as `None` and default values.
    ///
    /// Use struct update syntax to override specific fields:
    /// ```ignore
    /// PendingTrigger {
    ///     triggering_event: Some(TriggerEvent::SelfDies),
    ///     data: Some(TriggerData::DeathModular { counter_count: n }),
    ///     ..PendingTrigger::blank(source_id, controller_id, PendingTriggerKind::Modular)
    /// }
    /// ```
    pub fn blank(
        source: ObjectId,
        controller: PlayerId,
        kind: PendingTriggerKind,
    ) -> PendingTrigger {
        PendingTrigger {
            source,
            ability_index: 0,
            controller,
            kind,
            triggering_event: None,
            entering_object_id: None,
            targeting_stack_id: None,
            triggering_player: None,
            exalted_attacker_id: None,
            defending_player_id: None,
            ingest_target_player: None,
            flanking_blocker_id: None,
            rampage_n: None,
            renown_n: None,
            poisonous_n: None,
            poisonous_target_player: None,
            enlist_enlisted_creature: None,
            recover_cost: None,
            recover_card: None,
            cipher_encoded_card_id: None,
            cipher_encoded_object_id: None,
            haunt_source_object_id: None,
            haunt_source_card_id: None,
            damaged_player: None,
            combat_damage_amount: 0,
            data: None,
        }
    }
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
// ── Game Restrictions (PB-18: Stax / Action Restrictions) ────────────────────
/// What kind of restriction is imposed on the game (CR 604).
///
/// Restrictions are static abilities that prevent players from taking certain actions.
/// They are NOT continuous effects (they don't modify characteristics through the layer system).
/// Instead, they are checked at action-legality time in casting.rs, combat.rs, and
/// the simulator's legal_actions.rs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameRestriction {
    /// "Each player can't cast more than one spell each turn."
    /// (Rule of Law, Archon of Emeria, Eidolon of Rhetoric)
    /// CR 101.2: restriction overrides permission.
    MaxSpellsPerTurn { max: u32 },
    /// "Your opponents can't cast spells during your turn."
    /// (Dragonlord Dromoka, Grand Abolisher, Myrel)
    /// The controller's opponents are restricted during the controller's turn.
    OpponentsCantCastDuringYourTurn,
    /// "Your opponents can't cast spells or activate abilities of artifacts,
    /// creatures, or enchantments [during your turn]."
    /// (Grand Abolisher, Myrel — superset of OpponentsCantCastDuringYourTurn)
    OpponentsCantCastOrActivateDuringYourTurn,
    /// "Your opponents can't cast spells from anywhere other than their hands."
    /// (Drannith Magistrate)
    OpponentsCantCastFromNonHand,
    /// "Creatures can't attack you unless their controller pays {N} for each."
    /// (Propaganda, Ghostly Prison)
    CantAttackYouUnlessPay {
        cost_per_creature: super::game_object::ManaCost,
    },
    /// "Activated abilities of artifacts can't be activated."
    /// (Collector Ouphe, Stony Silence)
    /// Mana abilities of artifacts are also restricted (CR 605.3b).
    ArtifactAbilitiesCantBeActivated,
    /// "Each player can't cast more than one noncreature spell each turn."
    /// (Deafening Silence)
    /// CR 101.2: restriction overrides permission. Only restricts noncreature spells.
    MaxNoncreatureSpellsPerTurn { max: u32 },
    /// "Each player who has cast a nonartifact spell this turn can't cast additional
    /// nonartifact spells." (Ethersworn Canonist)
    /// CR 101.2: effectively max 1 nonartifact spell per turn per player.
    MaxNonartifactSpellsPerTurn { max: u32 },
    /// "Each opponent can cast spells only any time they could cast a sorcery."
    /// (Teferi, Time Raveler)
    /// CR 307.5: "only as a sorcery" = must have priority, main phase, empty stack.
    /// CR 101.2: restriction overrides permission — this beats flash grants.
    OpponentsCanOnlyCastAtSorcerySpeed,
}
/// An active restriction in the game, registered from a static ability of a
/// permanent on the battlefield.
///
/// Follows the same pattern as `TriggerDoubler` and `ETBSuppressor`:
/// - `source: ObjectId` for cleanup when the source leaves the battlefield
/// - `controller: PlayerId` to determine who "you" is in "your opponents"
/// - Registered in `register_static_continuous_effects` from
///   `AbilityDefinition::StaticRestriction`
/// - Automatically cleaned up when the source leaves (checked via `state.objects`)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveRestriction {
    /// ObjectId of the permanent generating this restriction.
    pub source: ObjectId,
    /// The player who controls the source permanent.
    pub controller: PlayerId,
    /// The restriction being imposed.
    pub restriction: GameRestriction,
}
/// Filter for which spells a flash grant applies to.
/// CR 601.3b: "a spell with certain qualities"
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlashGrantFilter {
    /// All spells (Borne Upon a Wind: "You may cast spells...")
    AllSpells,
    /// Sorcery spells only (Complete the Circuit, Teferi +1: "sorcery spells")
    Sorceries,
    /// Green creature spells only (Yeva: "green creature spells")
    GreenCreatures,
}
/// An active flash grant allowing a player to cast certain spells at instant speed.
///
/// Follows the same pattern as `ActiveRestriction`:
/// - `source: Option<ObjectId>` for cleanup when the source leaves the battlefield
///   (None for spell-based grants that expire by duration)
/// - Registered by `Effect::GrantFlash` or `AbilityDefinition::StaticFlashGrant`
/// - Checked in `casting.rs` timing validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlashGrant {
    /// ObjectId of the source (permanent for static grants, None for one-shot spell effects).
    pub source: Option<ObjectId>,
    /// The player who receives the flash permission.
    pub player: PlayerId,
    /// Which spells this grant applies to.
    pub filter: FlashGrantFilter,
    /// How long the grant lasts.
    pub duration: crate::state::continuous_effect::EffectDuration,
}
/// CR 305.2: A static "additional land play" source registered from a permanent
/// on the battlefield with `AbilityDefinition::AdditionalLandPlays`.
///
/// At the start of each turn, all sources whose controller matches the active player
/// contribute their `count` to `land_plays_remaining`. Cleaned up when the source
/// permanent leaves the battlefield.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdditionalLandPlaySource {
    /// ObjectId of the permanent generating the extra land plays.
    pub source: ObjectId,
    /// The player who controls the source permanent (beneficiary).
    pub controller: PlayerId,
    /// Number of extra land plays granted per turn.
    pub count: u32,
}
