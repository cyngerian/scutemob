//! Game events emitted by the rules engine (CR 500-514, M2+).
//!
//! Events are the single source of truth for "what happened." The network
//! layer broadcasts them; the UI consumes them; the history log records them.

use im::OrdMap;
use serde::{Deserialize, Serialize};

use crate::state::combat::AttackTarget;
use crate::state::game_object::{ManaCost, ObjectId};
use crate::state::player::{CardId, PlayerId};
use crate::state::replacement_effect::ReplacementId;
use crate::state::turn::{Phase, Step};
use crate::state::types::{CounterType, ManaColor};
use crate::state::zone::{ZoneId, ZoneType};

/// The target of a combat damage assignment: a creature, player, or planeswalker.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatDamageTarget {
    /// Damage dealt to a creature (blocker or attacker).
    Creature(ObjectId),
    /// Damage dealt directly to a player.
    Player(PlayerId),
    /// Damage dealt to a planeswalker permanent.
    Planeswalker(ObjectId),
}

/// A single source-to-target combat damage assignment (CR 510.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatDamageAssignment {
    /// The creature dealing damage.
    pub source: ObjectId,
    /// Where the damage is directed.
    pub target: CombatDamageTarget,
    /// Amount of damage assigned.
    pub amount: u32,
}

/// Why a player lost the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossReason {
    /// CR 104.3a: life total 0 or less (checked by SBAs, M4)
    LifeTotal,
    /// CR 104.3b: attempted to draw from empty library
    LibraryEmpty,
    /// CR 104.3c: 10+ poison counters (checked by SBAs, M4)
    PoisonCounters,
    /// CR 104.3d: 21+ commander damage from a single commander (M4)
    CommanderDamage,
    /// CR 104.3a: player conceded
    Conceded,
}

/// A game event describing a state change.
///
/// Every state transition produces one or more events. Events are appended
/// to `GameState::history` and can be used by triggers and the UI.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    /// A new turn has started for the given player.
    TurnStarted { player: PlayerId, turn_number: u32 },

    /// The game has moved to a new step (and implicitly a new phase).
    StepChanged { step: Step, phase: Phase },

    /// A player has been granted priority.
    PriorityGiven { player: PlayerId },

    /// A player passed priority.
    PriorityPassed { player: PlayerId },

    /// All players have passed priority in succession (stack empty).
    AllPlayersPassed,

    /// Active player's permanents were untapped (CR 502.2).
    PermanentsUntapped {
        player: PlayerId,
        objects: Vec<ObjectId>,
    },

    /// A card was drawn (moved from library to hand).
    CardDrawn {
        player: PlayerId,
        /// The new ObjectId in hand (per CR 400.7 zone-change identity).
        new_object_id: ObjectId,
    },

    /// Mana pools were emptied at step transition (CR 500.4).
    ManaPoolsEmptied,

    /// Cleanup step actions were performed (CR 514).
    CleanupPerformed,

    /// A card was discarded to meet hand size limit (CR 514.1).
    DiscardedToHandSize {
        player: PlayerId,
        object_id: ObjectId,
        zone_from: ZoneId,
        zone_to: ZoneId,
    },

    /// Damage was cleared from all permanents.
    DamageCleared,

    /// A player has lost the game.
    PlayerLost {
        player: PlayerId,
        reason: LossReason,
    },

    /// A player has conceded.
    PlayerConceded { player: PlayerId },

    /// The game is over. Winner is None if it's a draw.
    GameOver { winner: Option<PlayerId> },

    /// An extra turn has been added to the queue.
    ExtraTurnAdded { player: PlayerId },

    /// A land was played from hand to battlefield (CR 305.1).
    LandPlayed {
        player: PlayerId,
        /// ObjectId of the land on the battlefield (new per CR 400.7).
        new_land_id: ObjectId,
    },

    /// Mana was added to a player's mana pool (CR 605).
    ManaAdded {
        player: PlayerId,
        color: ManaColor,
        amount: u32,
    },

    /// A permanent became tapped (CR 701.21).
    PermanentTapped {
        player: PlayerId,
        object_id: ObjectId,
    },

    /// A spell was cast and entered the stack (CR 601.2).
    ///
    /// `stack_object_id` is the ID of the `StackObject` entry.
    /// `source_object_id` is the ID of the card now in the Stack zone (new
    /// per CR 400.7 zone-change identity).
    SpellCast {
        player: PlayerId,
        stack_object_id: ObjectId,
        source_object_id: ObjectId,
    },

    /// A spell or ability on the stack resolved (CR 608.2n, 608.3).
    ///
    /// For instant/sorcery spells, `source_object_id` is the card's new ID in
    /// the owner's graveyard. For permanent spells, it's the new ID on the
    /// battlefield (see also `PermanentEnteredBattlefield`).
    SpellResolved {
        player: PlayerId,
        stack_object_id: ObjectId,
        source_object_id: ObjectId,
    },

    /// A permanent spell resolved and the card entered the battlefield (CR 608.3a).
    ///
    /// `object_id` is the permanent's new ObjectId on the battlefield (new per
    /// CR 400.7 zone-change identity).
    PermanentEnteredBattlefield {
        player: PlayerId,
        object_id: ObjectId,
    },

    /// A spell was countered without resolving (CR 608.2b, 701.5).
    ///
    /// The card is put into its owner's graveyard. `source_object_id` is the
    /// card's new ID in the graveyard.
    SpellCountered {
        player: PlayerId,
        stack_object_id: ObjectId,
        source_object_id: ObjectId,
    },

    /// A spell fizzled because all of its targets became illegal (CR 608.2b).
    ///
    /// Distinct from `SpellCountered`: fizzle is caused by illegal targets, not an
    /// explicit counter effect. The card is put into its owner's graveyard without
    /// resolving. Triggers that care about "countered" (e.g., storm) do NOT trigger
    /// on fizzle (M3-E+).
    SpellFizzled {
        player: PlayerId,
        stack_object_id: ObjectId,
        source_object_id: ObjectId,
    },

    /// A player paid a mana cost to cast a spell (CR 601.2f-h).
    ///
    /// Emitted when a spell with a non-zero mana cost is cast and the cost is
    /// deducted from the player's mana pool.
    ManaCostPaid { player: PlayerId, cost: ManaCost },

    /// A non-mana activated ability was activated and placed on the stack (CR 602).
    ///
    /// `source_object_id` is the source object (remains in its current zone;
    /// the source does NOT move to the stack). `stack_object_id` is the new
    /// `StackObject` ID representing this ability instance on the stack.
    AbilityActivated {
        player: PlayerId,
        source_object_id: ObjectId,
        stack_object_id: ObjectId,
    },

    /// A triggered ability was placed on the stack (CR 603.3).
    ///
    /// Emitted when pending triggers are flushed to the stack in APNAP order.
    /// `source_object_id` is the source permanent. `stack_object_id` is the
    /// new `StackObject` ID on the stack.
    AbilityTriggered {
        controller: PlayerId,
        source_object_id: ObjectId,
        stack_object_id: ObjectId,
    },

    /// An activated or triggered ability resolved (CR 608.3b).
    ///
    /// The ability was popped from the stack. Effects (M7+) would execute here.
    /// For triggered abilities with an intervening-if clause (CR 603.4), the
    /// condition is checked at resolution; if false, this event is still emitted
    /// but no effects occur (M7+).
    AbilityResolved {
        controller: PlayerId,
        stack_object_id: ObjectId,
    },

    // ── M4: State-Based Action events ──────────────────────────────────────
    /// A creature was put into its owner's graveyard as a state-based action
    /// (CR 704.5f: toughness ≤ 0; CR 704.5g: lethal damage; CR 704.5h: deathtouch damage).
    CreatureDied {
        /// ObjectId of the creature on the battlefield (now retired).
        object_id: ObjectId,
        /// New ObjectId of the card in the graveyard (CR 400.7).
        new_grave_id: ObjectId,
        /// CR 603.3a: controller of the creature at the moment of death (before
        /// zone change). `move_object_to_zone` resets controller to owner, so
        /// the controller must be captured before the move and carried through
        /// this event to the trigger dispatch in `check_triggers`.
        controller: PlayerId,
        /// CR 702.79a / CR 702.93a: counters on the creature just before it left
        /// the battlefield (last known information). Used by persist (checks -1/-1)
        /// and undying (checks +1/+1) to evaluate the intervening-if condition at
        /// trigger time. Captured before `move_object_to_zone` resets counters.
        pre_death_counters: OrdMap<CounterType, u32>,
    },

    /// A planeswalker was put into its owner's graveyard because its loyalty
    /// reached 0 (CR 704.5i).
    PlaneswalkerDied {
        object_id: ObjectId,
        new_grave_id: ObjectId,
    },

    /// An aura was put into its owner's graveyard because it became attached to
    /// an illegal or non-existent object (CR 704.5m).
    AuraFellOff {
        object_id: ObjectId,
        new_grave_id: ObjectId,
    },

    /// CR 702.103f: A bestowed Aura became unattached and reverted to an
    /// enchantment creature. Unlike normal Auras (AuraFellOff), the permanent
    /// stays on the battlefield.
    BestowReverted { object_id: ObjectId },

    /// An equipment (or fortification) became unattached because the object it
    /// was attached to is no longer a legal attachment target (CR 704.5n).
    EquipmentUnattached { object_id: ObjectId },

    /// An Equipment was attached to a creature via the Equip ability (CR 702.6a).
    /// Emitted when the equip effect resolves and the attachment state changes.
    EquipmentAttached {
        /// The Equipment that was attached.
        equipment_id: ObjectId,
        /// The creature it was attached to.
        target_id: ObjectId,
        /// The player who activated the equip ability.
        controller: PlayerId,
    },

    /// A Fortification was attached to a land via the Fortify ability (CR 702.67a).
    /// Emitted when the fortify effect resolves and the attachment state changes.
    FortificationAttached {
        /// The Fortification that was attached.
        fortification_id: ObjectId,
        /// The land it was attached to.
        target_id: ObjectId,
        /// The player who activated the fortify ability.
        controller: PlayerId,
    },

    /// An Aura resolved and was attached to its target (CR 303.4a, 303.4b).
    ///
    /// Emitted when an Aura permanent spell resolves and enters the battlefield
    /// attached to the declared target.
    AuraAttached {
        /// The Aura that was attached.
        aura_id: ObjectId,
        /// The object the Aura is now enchanting.
        target_id: ObjectId,
        /// The controller of the Aura.
        controller: PlayerId,
    },

    /// A token in a non-battlefield zone ceased to exist (CR 704.5d).
    TokenCeasedToExist { object_id: ObjectId },

    /// +1/+1 and -1/-1 counters were annihilated on a permanent (CR 704.5q).
    /// `amount` is how many pairs were removed.
    CountersAnnihilated { object_id: ObjectId, amount: u32 },

    /// The legendary rule was applied: multiple legendary permanents with the
    /// same name were controlled by the same player; all but one went to the
    /// owners' graveyards (CR 704.5j).
    ///
    /// `kept_id` is the ObjectId that was kept on the battlefield.
    /// `put_to_graveyard` is a list of (old battlefield ObjectId, new graveyard ObjectId).
    ///
    /// Note: in a real game the controller chooses which to keep. For M4, the
    /// implementation auto-keeps the permanent with the highest ObjectId (most
    /// recently entered). Player choice is deferred to M7.
    LegendaryRuleApplied {
        kept_id: ObjectId,
        put_to_graveyard: Vec<(ObjectId, ObjectId)>,
    },

    // ── M6: Combat events ──────────────────────────────────────────────────
    /// The active player declared attacking creatures (CR 508.1).
    ///
    /// Each attacker is paired with its attack target (player or planeswalker).
    /// Non-Vigilance attackers are tapped as a side effect (separate PermanentTapped events).
    AttackersDeclared {
        attacking_player: PlayerId,
        /// (attacker ObjectId, attack target) pairs.
        attackers: Vec<(ObjectId, AttackTarget)>,
    },

    /// A defending player declared blocking creatures (CR 509.1).
    ///
    /// In multiplayer, each defending player declares independently.
    BlockersDeclared {
        defending_player: PlayerId,
        /// (blocker ObjectId, attacker ObjectId being blocked) pairs.
        blockers: Vec<(ObjectId, ObjectId)>,
    },

    /// Combat damage was dealt simultaneously (CR 510.2).
    ///
    /// All damage is assigned and dealt in a single batch. After this event,
    /// state-based actions fire and may cause creatures to die.
    CombatDamageDealt {
        assignments: Vec<CombatDamageAssignment>,
    },

    /// The combat phase ended and combat state was cleared (CR 511.1).
    CombatEnded,

    /// A player gained life (CR 118.4). Generated by lifelink (CR 702.15a),
    /// GainLife effects, and other life-gain sources.
    LifeGained { player: PlayerId, amount: u32 },

    /// A player lost life outside of damage (CR 118.4).
    LifeLost { player: PlayerId, amount: u32 },

    /// A player received poison counters from an infect source (CR 120.3b, CR 702.90b).
    ///
    /// Emitted when infect damage is dealt to a player, replacing the normal LifeLost
    /// event. The DamageDealt event is still emitted alongside this one (damage IS dealt,
    /// just with a different result per CR 702.90b).
    PoisonCountersGiven {
        /// The player receiving the poison counters.
        player: PlayerId,
        /// Number of poison counters given (equals the infect damage dealt).
        amount: u32,
        /// The source of the infect damage.
        source: ObjectId,
    },

    // ── M7: Effect execution events ────────────────────────────────────────
    /// Non-combat damage was dealt by a spell or ability (CR 120).
    ///
    /// For combat damage, see `CombatDamageDealt` instead.
    DamageDealt {
        /// The object dealing the damage (the spell's source card).
        source: ObjectId,
        /// The target that received the damage.
        target: CombatDamageTarget,
        /// Amount of damage dealt.
        amount: u32,
    },

    /// An object was exiled by a spell or ability (CR 701.5).
    ObjectExiled {
        /// Player whose spell/ability caused the exile.
        player: PlayerId,
        /// ObjectId of the object before exile (now retired).
        object_id: ObjectId,
        /// New ObjectId of the object in the exile zone (CR 400.7).
        new_exile_id: ObjectId,
    },

    /// A non-creature permanent was destroyed by a spell or ability (CR 701.7).
    ///
    /// Creature destruction emits `CreatureDied` instead (via SBA).
    PermanentDestroyed {
        /// ObjectId on the battlefield (now retired).
        object_id: ObjectId,
        /// New ObjectId in the owner's graveyard (CR 400.7).
        new_grave_id: ObjectId,
    },

    /// A permanent was untapped by a spell or ability (CR 701.17).
    PermanentUntapped {
        player: PlayerId,
        object_id: ObjectId,
    },

    /// A card was discarded from a player's hand (CR 701.8).
    CardDiscarded {
        player: PlayerId,
        /// ObjectId in hand (now retired).
        object_id: ObjectId,
        /// New ObjectId in the graveyard (CR 400.7).
        new_id: ObjectId,
    },

    /// A card was cycled from a player's hand (CR 702.29a).
    ///
    /// This event fires IN ADDITION TO `CardDiscarded` (CR 702.29d: "cycles or discards"
    /// triggers fire once). The `CardCycled` event enables "when you cycle" triggers
    /// (CR 702.29c) that are distinct from generic discard triggers.
    CardCycled {
        player: PlayerId,
        /// ObjectId of the card in hand (now retired — CR 400.7).
        object_id: ObjectId,
        /// New ObjectId in the graveyard.
        new_id: ObjectId,
    },

    /// A card was put into a graveyard from the top of a library (mill, CR 701.13).
    CardMilled {
        player: PlayerId,
        /// New ObjectId of the card in the graveyard (CR 400.7).
        new_id: ObjectId,
    },

    /// A token permanent was created on the battlefield (CR 701.6).
    TokenCreated {
        /// The controlling player.
        player: PlayerId,
        /// ObjectId of the token on the battlefield.
        object_id: ObjectId,
    },

    /// A player's library was shuffled (CR 701.20).
    LibraryShuffled { player: PlayerId },

    /// A counter was placed on a permanent or player (CR 122.1).
    CounterAdded {
        /// ObjectId of the permanent (or sentinel for player counters).
        object_id: ObjectId,
        counter: crate::state::types::CounterType,
        count: u32,
    },

    /// A counter was removed from a permanent or player (CR 122.1).
    CounterRemoved {
        object_id: ObjectId,
        counter: crate::state::types::CounterType,
        count: u32,
    },

    // ── MR-M7-01: Correct zone-move events ─────────────────────────────────
    /// An object was returned to a player's hand by a spell or ability (CR 400).
    ///
    /// Distinct from `CardDrawn` (library→hand) and `DiscardedToHandSize` (reverse).
    ObjectReturnedToHand {
        /// Controlling player of the effect.
        player: PlayerId,
        /// ObjectId before zone change (now retired).
        object_id: ObjectId,
        /// New ObjectId in hand (CR 400.7).
        new_hand_id: ObjectId,
    },

    /// An object was put into a graveyard by a spell or ability, not via
    /// destruction or death (CR 400).
    ///
    /// Creature/permanent death emits `CreatureDied`/`PermanentDestroyed`.
    /// This event covers direct zone moves (e.g., Entomb, Buried Alive).
    ObjectPutInGraveyard {
        /// Controlling player of the effect.
        player: PlayerId,
        /// ObjectId before zone change (now retired).
        object_id: ObjectId,
        /// New ObjectId in the graveyard (CR 400.7).
        new_grave_id: ObjectId,
    },

    /// An object was put onto a player's library by a spell or ability (CR 400).
    ObjectPutOnLibrary {
        /// Whose library the object was placed in.
        player: PlayerId,
        /// ObjectId before zone change (now retired).
        object_id: ObjectId,
        /// New ObjectId in the library (CR 400.7).
        new_lib_id: ObjectId,
    },

    // ── M8: Replacement/prevention effect events ────────────────────────
    /// A commander was sent to its owner's command zone instead of changing zones
    /// (CR 903.9a — owner may choose command zone when commander would change zones).
    ///
    /// Emitted when a commander redirect replacement effect resolves via
    /// `resolve_pending_zone_change`. Distinct from `ReplacementEffectApplied` so
    /// the UI can show a targeted "went to command zone" notification.
    CommanderZoneRedirect {
        /// The commander's ObjectId on the battlefield (now retired).
        object_id: ObjectId,
        /// New ObjectId in the command zone (CR 400.7).
        new_command_id: ObjectId,
        /// The commander's owner.
        owner: PlayerId,
    },

    /// A replacement effect was applied to an event (CR 614).
    ///
    /// Emitted whenever a replacement effect intercepts and modifies an event
    /// before it occurs. The `description` field provides a human-readable
    /// summary of what was replaced.
    ReplacementEffectApplied {
        effect_id: ReplacementId,
        description: String,
    },

    /// Multiple replacement effects apply to the same event and the affected
    /// player must choose the order (CR 616.1).
    ///
    /// The engine pauses until a `Command::OrderReplacements` is received from
    /// the choosing player. `choices` lists the applicable replacement effect
    /// IDs in no particular order.
    ReplacementChoiceRequired {
        player: PlayerId,
        event_description: String,
        choices: Vec<ReplacementId>,
    },

    /// Damage was prevented by a prevention effect (CR 615).
    ///
    /// `prevented` is the amount actually prevented; `remaining` is the damage
    /// that still gets through (original − prevented).
    DamagePrevented {
        source: ObjectId,
        target: CombatDamageTarget,
        prevented: u32,
        remaining: u32,
    },

    // ── M9: Commander casting events (discriminant 57) ────────────────────
    /// A commander was cast from the command zone (CR 903.8).
    ///
    /// Distinct from `SpellCast` for UI clarity — allows the frontend to display
    /// a dedicated "commander entered from command zone" notification. Both
    /// `SpellCast` and `CommanderCastFromCommandZone` are emitted for the same cast.
    ///
    /// `tax_paid` is the number of times previously cast (the additional cost
    /// was `tax_paid * 2` generic mana).
    CommanderCastFromCommandZone {
        player: PlayerId,
        card_id: CardId,
        tax_paid: u32,
    },

    // ── M9: Commander zone return SBA (discriminant 58) ──────────────────
    /// A commander was returned to its owner's command zone following the owner's
    /// explicit choice via `ReturnCommanderToCommandZone` (CR 903.9a / CR 704.6d).
    ///
    /// Emitted by `handle_return_commander_to_command_zone` when the player chooses
    /// to return their commander. `from_zone` indicates which zone it was moved from.
    CommanderReturnedToCommandZone {
        card_id: CardId,
        owner: PlayerId,
        from_zone: ZoneType,
    },

    // ── M9 fix: Commander zone return player choice (discriminant 62) ─────
    /// The SBA detected a commander in graveyard or exile and is awaiting the
    /// owner's choice: return it to the command zone or leave it where it is
    /// (CR 903.9a — "may put it into the command zone").
    ///
    /// The engine pauses until the owner sends either:
    /// - `Command::ReturnCommanderToCommandZone { player, object_id }` to move it
    /// - `Command::LeaveCommanderInZone { player, object_id }` to leave it
    ///
    /// `object_id` identifies the commander object in its current zone.
    CommanderZoneReturnChoiceRequired {
        owner: PlayerId,
        card_id: CardId,
        object_id: ObjectId,
        from_zone: ZoneType,
    },

    // ── M9: Mulligan events (discriminants 59-60) ─────────────────────────
    /// A player took a mulligan (CR 103.5 / CR 103.5c).
    ///
    /// `mulligan_number` is 1-based (1 = first mulligan). `is_free` is true
    /// for the first mulligan in multiplayer where the player draws back to 7
    /// with no cards required to go to the bottom (CR 103.5c).
    MulliganTaken {
        player: PlayerId,
        mulligan_number: u32,
        is_free: bool,
    },

    /// A player kept their hand (or mulliganed hand) (CR 103.5).
    ///
    /// `cards_to_bottom` lists the ObjectIds of cards put on the bottom of
    /// the player's library as a result of the mulligan. Empty for no-mulligan
    /// or the free mulligan.
    MulliganKept {
        player: PlayerId,
        cards_to_bottom: Vec<ObjectId>,
    },

    // ── M9: Companion event (discriminant 61) ─────────────────────────────
    /// A player paid {3} to bring their companion from the sideboard into hand
    /// (CR 702.139a).
    ///
    /// Emitted when `handle_bring_companion` successfully completes the special
    /// action. The companion card is now in the player's hand.
    CompanionBroughtToHand { player: PlayerId, card_id: CardId },

    // ── M9.4: Cascade events ─────────────────────────────────────────────────
    /// Cards were exiled from the top of a library during cascade resolution
    /// (CR 702.85b). Emitted once per cascade trigger listing all exiled card IDs.
    CascadeExiled {
        /// The player whose library was searched.
        player: PlayerId,
        /// ObjectIds of all cards exiled during the cascade search (in order
        /// exiled; last = the card that will be cast or put on the bottom).
        cards_exiled: Vec<ObjectId>,
    },

    /// A card was cast without paying its mana cost as a result of cascade
    /// (CR 702.85b). Emitted after `CascadeExiled` when the player casts the
    /// found card. The remaining exiled cards are placed on the bottom of the
    /// library in a random order (deterministic: ObjectId order).
    CascadeCast {
        /// The player who cast the cascade spell.
        player: PlayerId,
        /// ObjectId of the card that was cast (now in the Stack zone).
        card_id: ObjectId,
    },

    // ── B13: Discover events (CR 701.57) ─────────────────────────────────
    /// Cards were exiled from the top of a library during discover resolution
    /// (CR 701.57a). Emitted once per discover action listing all exiled card IDs
    /// in order exiled, NOT including the discovered card.
    ///
    /// The discovered card (if any) is removed from the exiled list before this
    /// event is emitted — it is tracked separately by `DiscoverCast` (when cast)
    /// or `DiscoverToHand` (when the player declines to cast). This matches the
    /// behavior of `CascadeExiled`, which also excludes the cast card.
    DiscoverExiled {
        /// The player performing the discover action.
        player: PlayerId,
        /// ObjectIds of all cards exiled during the discover search (in order
        /// exiled), excluding the discovered card (which appears in DiscoverCast
        /// or DiscoverToHand). These are the cards placed on the library bottom.
        cards_exiled: Vec<ObjectId>,
    },

    /// A card was cast without paying its mana cost as a result of discover
    /// (CR 701.57a). Emitted after `DiscoverExiled` when the player casts the
    /// found card. The remaining exiled cards are placed on the bottom of the
    /// library in a random order (deterministic: ObjectId order).
    DiscoverCast {
        /// The player who performed the discover action.
        player: PlayerId,
        /// ObjectId of the card that was cast (now in the Stack zone).
        card_id: ObjectId,
    },

    /// The discovered card was put into the player's hand instead of being cast
    /// (CR 701.57a). Emitted when the player declines to cast the discovered card,
    /// or when the card cannot be legally cast. Unlike Cascade, Discover puts the
    /// card into hand rather than the library bottom.
    DiscoverToHand {
        /// The player who performed the discover action.
        player: PlayerId,
        /// ObjectId of the card that was put into the player's hand.
        card_id: ObjectId,
    },

    // ── M9.4: Storm / spell copy events ──────────────────────────────────
    /// A spell was copied on the stack (CR 707.10, CR 702.40a).
    ///
    /// Emitted by `copy::copy_spell_on_stack` when storm creates copies or when
    /// any other copy-spell effect creates a stack copy. The copy is NOT cast —
    /// it does not trigger "whenever you cast a spell" abilities (CR 707.10c).
    SpellCopied {
        /// The original stack object that was copied.
        original_stack_id: ObjectId,
        /// The ID of the new copy on the stack.
        copy_stack_id: ObjectId,
        /// The controller of the copy (usually the storm spell's caster).
        controller: PlayerId,
    },

    // ── Ward targeting events (CR 702.21a) ───────────────────────────────
    /// A battlefield permanent became the target of a spell or ability (CR 702.21a).
    ///
    /// Emitted after a spell is cast or ability activated with targets. Used to
    /// fire Ward triggered abilities. `target_id` is the ObjectId of the targeted
    /// permanent. `targeting_stack_id` is the ObjectId of the stack object (spell
    /// or ability) that is targeting it. `targeting_controller` is the player
    /// who controls the targeting spell or ability.
    PermanentTargeted {
        target_id: ObjectId,
        targeting_stack_id: ObjectId,
        targeting_controller: PlayerId,
    },

    // ── M9.4: Infinite loop detection (CR 104.4b) ────────────────────────
    /// The engine detected a mandatory infinite loop and the game is a draw.
    ///
    /// CR 104.4b: if the game situation is such that the game cannot proceed,
    /// and all remaining choices are mandatory (no player can choose to break
    /// the loop), the game is a draw.
    ///
    /// Emitted when the same board state hash has been observed N times (threshold 3)
    /// during a mandatory-action sequence (SBA + trigger cycles with no player choices).
    LoopDetected { description: String },

    // ── M9.4: Scry event ──────────────────────────────────────────────────
    /// A player performed a scry action (CR 701.18).
    ///
    /// Emitted by `Effect::Scry` when the player looks at the top N cards
    /// of their library and rearranges them (deterministic fallback: top cards
    /// moved to bottom in ObjectId order).
    Scried { player: PlayerId, count: u32 },

    // ── Surveil event ─────────────────────────────────────────────────────
    /// A player performed a surveil action (CR 701.25).
    ///
    /// Emitted by `Effect::Surveil` when the player looks at the top N cards
    /// of their library and puts some into the graveyard, rest on top.
    /// CR 701.25c: NOT emitted when surveilling 0.
    Surveilled { player: PlayerId, count: u32 },

    // ── Investigate event ─────────────────────────────────────────────────
    /// A player performed an investigate action (CR 701.16a).
    ///
    /// Emitted by `Effect::Investigate` when the player creates Clue tokens.
    /// Enables "whenever you investigate" triggers (future cards like
    /// Lonis, Cryptozoologist). NOT emitted when investigating 0.
    Investigated { player: PlayerId, count: u32 },

    // ── Amass event ───────────────────────────────────────────────────────
    /// A player performed an amass action (CR 701.47a).
    ///
    /// Emitted by `Effect::Amass` after the Army creature has received counters
    /// and the subtype has been added. CR 701.47b: always emitted even if some
    /// or all actions were impossible. Enables "whenever you amass" triggers.
    Amassed {
        player: PlayerId,
        /// The Army creature that was chosen (or created).
        army_id: ObjectId,
        /// Number of +1/+1 counters placed.
        count: u32,
    },

    // ── M9.4: Goaded event ────────────────────────────────────────────────
    /// A permanent was goaded (CR 701.38).
    ///
    /// Emitted by `Effect::Goad` when a creature is marked as goaded.
    /// The goaded creature must attack each combat if able, and must attack
    /// a player other than the goading player if able.
    Goaded {
        /// The creature that was goaded.
        object_id: ObjectId,
        /// The player who goaded the creature.
        goading_player: PlayerId,
    },

    // ── Suspect events (CR 701.60) ────────────────────────────────────────
    /// A permanent was suspected (CR 701.60a).
    ///
    /// Emitted by `Effect::Suspect` when a creature is marked as suspected.
    /// The suspected permanent has menace and "This creature can't block"
    /// for as long as it's suspected (CR 701.60c).
    CreatureSuspected {
        /// The permanent that was suspected.
        object_id: ObjectId,
        /// The controller of the effect that caused the suspicion.
        controller: PlayerId,
    },
    /// A permanent is no longer suspected (CR 701.60a).
    ///
    /// Emitted by `Effect::Unsuspect` when the suspected designation is removed.
    CreatureUnsuspected {
        /// The permanent whose suspected designation was removed.
        object_id: ObjectId,
        /// The controller of the effect that removed the suspicion.
        controller: PlayerId,
    },

    // ── Miracle events (CR 702.94) ────────────────────────────────────────
    /// CR 702.94a: A card with miracle was drawn as the first card this turn.
    /// The player may choose to reveal it and trigger the miracle ability.
    ///
    /// The engine pauses until a `Command::ChooseMiracle` is received.
    /// `card_object_id` is the drawn card's new ObjectId in hand.
    /// `miracle_cost` is the miracle alternative cost from the card definition.
    MiracleRevealChoiceRequired {
        player: PlayerId,
        card_object_id: ObjectId,
        miracle_cost: crate::state::game_object::ManaCost,
    },

    // ── Dredge events (CR 702.52) ─────────────────────────────────────────
    /// One or more dredge cards are available in the player's graveyard and
    /// the player must choose whether to dredge one or draw normally (CR 702.52a).
    ///
    /// The engine pauses until a `Command::ChooseDredge` is received.
    /// `options` lists `(ObjectId, u32)` pairs of (dredge card, dredge amount).
    DredgeChoiceRequired {
        player: PlayerId,
        options: Vec<(ObjectId, u32)>,
    },

    /// CR 702.52: A player dredged a card — milled N cards and returned the
    /// dredge card from graveyard to hand instead of drawing.
    ///
    /// Dredge does NOT count as drawing (CR 702.52a — it's a replacement effect).
    /// `cards_drawn_this_turn` is NOT incremented.
    Dredged {
        player: PlayerId,
        /// The dredge card's new ObjectId in hand (CR 400.7 — zone change creates new id).
        card_new_id: ObjectId,
        /// Number of cards milled as part of the dredge.
        milled: u32,
    },

    // ── Foretell events (CR 702.143) ─────────────────────────────────────
    /// A card was foretold -- exiled face-down from hand via the foretell special
    /// action (CR 702.143a / CR 116.2h). The {2} cost was paid.
    ///
    /// `new_exile_id` is the ObjectId of the card in the exile zone (new per CR 400.7).
    /// This event reveals hidden info (the card identity is private to the owner;
    /// opponents only see that a card was exiled face-down).
    CardForetold {
        player: PlayerId,
        /// The card's ObjectId before exile (now retired).
        object_id: ObjectId,
        /// New ObjectId in the exile zone.
        new_exile_id: ObjectId,
    },

    // ── Plot events (CR 702.170) ──────────────────────────────────────────
    /// CR 702.170a / CR 116.2k: A card was plotted -- exiled face-up from hand via
    /// the plot special action. The plot cost was paid.
    ///
    /// `new_exile_id` is the ObjectId of the card in the exile zone (new per CR 400.7).
    /// Unlike foretell (face-down), plotted cards are face-up (public information),
    /// so this event does NOT reveal hidden info to the owner differently than opponents.
    CardPlotted {
        player: PlayerId,
        /// The card's ObjectId before exile (now retired per CR 400.7).
        object_id: ObjectId,
        /// New ObjectId in the exile zone.
        new_exile_id: ObjectId,
    },

    // ── Suspend events (CR 702.62) ────────────────────────────────────────
    /// CR 702.62a / CR 116.2f: A card was exiled from hand via the suspend special
    /// action. The suspend cost was paid and the card was exiled with N time counters.
    /// This is a special action -- it does not use the stack.
    ///
    /// Unlike foretell, suspended cards are exiled FACE UP (they are public information).
    CardSuspended {
        player: PlayerId,
        /// The card's ObjectId before exile (now retired per CR 400.7).
        object_id: ObjectId,
        /// New ObjectId in the exile zone.
        new_exile_id: ObjectId,
        /// Number of time counters placed on the card.
        time_counters: u32,
    },

    // ── Ascend event (CR 702.131) ─────────────────────────────────────────
    /// CR 702.131: A player gained the city's blessing designation.
    ///
    /// Emitted when ascend (on a permanent or resolving spell) determines that
    /// the player controls 10+ permanents. The city's blessing is permanent --
    /// once gained, it is never removed (CR 702.131c).
    CitysBlessingGained { player: PlayerId },

    // ── Connive event (CR 701.50) ─────────────────────────────────────────
    /// CR 701.50b: A permanent connived. Emitted after the draw/discard/counter
    /// sequence completes, even if some or all of those actions were impossible.
    /// Used by "whenever [this creature] connives" triggers.
    Connived {
        /// The ObjectId of the conniving permanent (may no longer be on battlefield).
        object_id: ObjectId,
        /// The controller who performed the draw/discard.
        player: PlayerId,
        /// Number of +1/+1 counters placed (0 if only lands discarded, creature
        /// left the battlefield, or draw/discard was impossible).
        counters_placed: u32,
    },

    // ── Regeneration events (CR 701.19) ──────────────────────────────────
    /// A regeneration shield was created on a permanent (CR 701.19a).
    ///
    /// Emitted when `Effect::Regenerate` resolves, creating a one-shot
    /// replacement effect that will intercept the next destruction event.
    RegenerationShieldCreated {
        /// The permanent the shield protects.
        object_id: ObjectId,
        /// The ReplacementId of the shield (for tracking consumption).
        shield_id: ReplacementId,
        /// The controller who created the shield.
        controller: PlayerId,
    },

    /// A permanent was regenerated -- destruction was replaced (CR 701.19a/614.8).
    ///
    /// Emitted when a regeneration shield intercepts destruction. The permanent
    /// remains on the battlefield with all damage removed, tapped, and removed
    /// from combat (if applicable).
    Regenerated {
        /// The permanent that was regenerated (still on the battlefield).
        object_id: ObjectId,
        /// The shield that was consumed.
        shield_id: ReplacementId,
    },

    // ── Umbra Armor event (CR 702.89a) ─────────────────────────────────────
    /// Umbra armor replacement applied -- an Aura was destroyed to save the enchanted permanent.
    ///
    /// Emitted when a permanent that would be destroyed is instead saved by an Aura with
    /// the umbra armor ability. The Aura moves to its owner's graveyard; all damage on the
    /// protected permanent is cleared. Unlike regeneration, the permanent is NOT tapped
    /// and NOT removed from combat.
    UmbraArmorApplied {
        /// The permanent that was saved (still on the battlefield, damage cleared).
        protected_id: ObjectId,
        /// The Aura that was destroyed to save it (now in graveyard).
        aura_id: ObjectId,
    },

    // ── Hideaway event (CR 702.75a) ────────────────────────────────────────
    /// A card was exiled face-down by a Hideaway ETB trigger (CR 702.75a).
    ///
    /// Emitted when `StackObjectKind::KeywordTrigger` (Hideaway) resolves and one card
    /// from the top N is exiled face-down.  The exiled card's `exiled_by_hideaway`
    /// is set to `source`.  Opponents cannot see the identity of the exiled card
    /// (it is face-down per CR 406.3); this event is private to the controller.
    HideawayExiled {
        /// The player whose library was searched (the Hideaway permanent's controller).
        player: PlayerId,
        /// ObjectId of the Hideaway permanent on the battlefield (the trigger source).
        source: ObjectId,
        /// New ObjectId of the card now in the exile zone (CR 400.7).
        exiled_card: ObjectId,
        /// Number of cards put back on the bottom of the library.
        remaining_count: u32,
    },

    // ── Echo events (CR 702.30) ───────────────────────────────────────────
    /// CR 702.30a: An echo trigger has resolved. The controller must choose
    /// whether to pay the echo cost or sacrifice the permanent.
    ///
    /// The engine pauses until a `Command::PayEcho` is received.
    EchoPaymentRequired {
        player: PlayerId,
        permanent: ObjectId,
        cost: crate::state::game_object::ManaCost,
    },

    /// CR 702.30a: A player paid the echo cost for a permanent.
    ///
    /// Emitted when the player sends `Command::PayEcho { pay: true }` and
    /// the mana has been deducted. The permanent stays on the battlefield
    /// and `echo_pending` is cleared.
    EchoPaid {
        player: PlayerId,
        permanent: ObjectId,
    },

    // ── Cumulative Upkeep events (CR 702.24) ─────────────────────────────────
    /// CR 702.24a: A cumulative upkeep trigger has resolved. The age counter
    /// has been added. The controller must choose whether to pay the total
    /// cost or sacrifice the permanent.
    ///
    /// The engine pauses until a `Command::PayCumulativeUpkeep` is received.
    CumulativeUpkeepPaymentRequired {
        player: PlayerId,
        permanent: ObjectId,
        /// The per-counter cost.
        per_counter_cost: crate::state::types::CumulativeUpkeepCost,
        /// Number of age counters currently on the permanent (after adding one).
        age_counter_count: u32,
    },

    /// CR 702.24a: A player paid the cumulative upkeep cost for a permanent.
    CumulativeUpkeepPaid {
        player: PlayerId,
        permanent: ObjectId,
        age_counter_count: u32,
    },

    // ── Recover events (CR 702.59) ────────────────────────────────────────────
    /// CR 702.59a: A recover trigger has resolved. The controller must choose
    /// whether to pay the recover cost or exile the card.
    ///
    /// The engine pauses until a `Command::PayRecover` is received.
    RecoverPaymentRequired {
        player: PlayerId,
        recover_card: ObjectId,
        cost: crate::state::game_object::ManaCost,
    },

    /// CR 702.59a: A player paid the recover cost for a card and it was
    /// returned from the graveyard to the player's hand.
    RecoverPaid {
        player: PlayerId,
        recover_card: ObjectId,
        new_hand_id: ObjectId,
    },

    /// CR 702.59a: A player declined to pay the recover cost. The card
    /// was exiled from the graveyard.
    RecoverDeclined {
        player: PlayerId,
        recover_card: ObjectId,
        new_exile_id: ObjectId,
    },

    // -- Proliferate event (CR 701.34) ------------------------------------------
    /// CR 701.34a: A player proliferated.
    ///
    /// Emitted by `Effect::Proliferate` after all counters have been added.
    /// Always emitted, even if no permanents or players were chosen (ruling
    /// 2023-02-04: "triggers even if you chose no permanents or players").
    /// Enables "whenever you proliferate" triggers.
    Proliferated {
        /// The player who performed the proliferate action.
        controller: PlayerId,
        /// Number of permanents that received additional counters.
        permanents_affected: u32,
        /// Number of players that received additional counters.
        players_affected: u32,
    },

    // -- Phasing events (CR 702.26) -------------------------------------------
    /// CR 702.26a: Permanents phased out during the untap step.
    ///
    /// Includes both directly-phased (Phasing keyword) and indirectly-phased
    /// (attached Auras/Equipment that phase out with their host).
    /// Emitted by `untap_active_player_permanents` before the untap loop.
    PermanentsPhasedOut {
        /// The player whose untap step caused the phase-out.
        player: PlayerId,
        /// ObjectIds of permanents that phased out (direct + indirect).
        objects: Vec<ObjectId>,
    },

    /// CR 702.26a: Permanents phased in during the untap step.
    ///
    /// Phasing in is NOT a zone change (CR 702.26d) -- no ETB triggers fire.
    /// Emitted by `untap_active_player_permanents` before phasing-out.
    PermanentsPhasedIn {
        /// The player whose untap step caused the phase-in.
        player: PlayerId,
        /// ObjectIds of permanents that phased in.
        objects: Vec<ObjectId>,
    },

    // ── Cipher events (CR 702.99) ─────────────────────────────────────────────
    /// CR 702.99a: A cipher card was exiled and encoded on a creature.
    ///
    /// Emitted when a cipher instant/sorcery resolves and its controller
    /// chooses to encode the card on a creature they control. The card moves
    /// directly from the stack to exile (never entering the graveyard).
    ///
    /// Ruling 2013-04-15: encoding happens after the spell's effects resolve.
    CipherEncoded {
        /// The player who controls the cipher spell.
        player: PlayerId,
        /// The ObjectId of the exiled cipher card (new ID after zone move, CR 400.7).
        exiled_card: ObjectId,
        /// The ObjectId of the creature the card is encoded on.
        creature: ObjectId,
    },

    // ── Mutate events (CR 702.140) ────────────────────────────────────────────
    /// CR 702.140d: A mutating creature spell successfully merged with its target.
    ///
    /// Emitted when a `MutatingCreatureSpell` resolves with a legal target and
    /// the spell's card data is absorbed into `target.merged_components`.
    /// The `object_id` is the merged permanent's ObjectId (the target's id is preserved,
    /// per CR 729.2c: "same object"). No ETB triggers fire — the merged permanent
    /// is not considered to have just entered the battlefield (CR 729.2c).
    ///
    /// This event is used by `TriggerEvent::SelfMutates` trigger dispatch in
    /// `abilities.rs` to fire "whenever this creature mutates" abilities.
    CreatureMutated {
        /// The merged permanent's ObjectId (the target's id, preserved per CR 400.7 exception).
        object_id: ObjectId,
        /// The controller of the mutating spell (and the merged permanent's controller).
        player: PlayerId,
    },

    // ── Haunt events (CR 702.55) ──────────────────────────────────────────────
    /// CR 702.55a: A haunt card was exiled haunting a creature.
    ///
    /// Emitted when a HauntExileTrigger resolves: the haunt card moves from the
    /// graveyard to exile and the haunting relationship is established.
    ///
    /// Discriminant 107.
    HauntExiled {
        /// The player who controls the haunt card.
        controller: PlayerId,
        /// The ObjectId of the exiled haunt card (new ID after zone move, CR 400.7).
        exiled_card: ObjectId,
        /// The ObjectId of the creature being haunted (on the battlefield).
        haunted_creature: ObjectId,
    },

    // ── Transform events (CR 701.27 / CR 712) ────────────────────────────────
    /// CR 701.27a: A double-faced permanent transformed (its other face is now up).
    ///
    /// No new object is created (CR 712.18). The ObjectId is unchanged. Counters,
    /// continuous effects, attached permanents, and damage persist.
    ///
    /// Discriminant: 108.
    PermanentTransformed {
        /// The ObjectId of the permanent that transformed.
        object_id: ObjectId,
        /// True if the permanent is now face-back (back face up / is_transformed == true).
        /// False if it transformed back to its front face.
        to_back_face: bool,
    },

    /// CR 730.1: The game's day/night designation changed.
    ///
    /// Emitted when it becomes day or night due to CR 730.2, or when a Daybound/Nightbound
    /// permanent establishes the initial day/night designation (CR 702.145d/g).
    ///
    /// Discriminant: 109.
    DayNightChanged {
        /// The new day/night designation.
        now: crate::state::DayNight,
    },

    /// CR 702.167a: A permanent's craft ability was activated and the materials exiled.
    ///
    /// Emitted after the cost is paid (source + materials exiled). The craft ability
    /// is on the stack. When it resolves, PermanentEnteredBattlefield is emitted.
    ///
    /// Discriminant: 110.
    CraftActivated {
        /// The player who activated the craft ability.
        player: PlayerId,
        /// The ObjectId of the exiled source (new ID in exile after move, CR 400.7).
        exiled_source: ObjectId,
        /// ObjectIds of the exiled material cards/permanents.
        exiled_materials: Vec<ObjectId>,
    },

    /// CR 702.37e / 702.168d / 701.40b / 701.58b: A face-down permanent was turned face up.
    ///
    /// The special action has resolved: the cost was paid, `face_down` is now false,
    /// `face_down_as` is cleared, and the permanent's real characteristics are visible.
    /// ETB abilities were NOT fired (CR 708.8). Any "when turned face up" triggered
    /// abilities have been queued as `TurnFaceUpTrigger` stack objects.
    ///
    /// Discriminant: 111.
    PermanentTurnedFaceUp {
        /// The player who turned the permanent face up.
        player: PlayerId,
        /// The permanent that was turned face up.
        permanent: ObjectId,
    },

    /// CR 708.9: A face-down permanent left the battlefield and must be revealed.
    ///
    /// All players must be shown the card's real identity when a face-down permanent
    /// leaves the battlefield for any zone (graveyard, exile, hand, library).
    ///
    /// The network layer uses this event to broadcast the card's true identity to
    /// all players (including opponents who could not see it while it was face-down).
    ///
    /// Discriminant: 112.
    FaceDownRevealed {
        /// The player who controlled the face-down permanent.
        player: PlayerId,
        /// The permanent's last ObjectId before leaving the battlefield.
        permanent: ObjectId,
        /// The real name of the card revealed to all players.
        card_name: String,
    },

    // ── Dungeon / Venture events (CR 309, CR 701.49, CR 725) ─────────────────
    /// CR 309.5a / CR 701.49: A player's venture marker moved into a dungeon room.
    ///
    /// Emitted when a player ventures into the dungeon and the marker is placed on
    /// a room (either entering a new dungeon at room 0, or advancing to the next room).
    ///
    /// Discriminant: 114.
    VenturedIntoDungeon {
        /// The player who ventured.
        player: PlayerId,
        /// Which dungeon the venture marker is now in.
        dungeon: crate::state::dungeon::DungeonId,
        /// The room index the venture marker moved to.
        room: crate::state::dungeon::RoomIndex,
    },

    /// CR 309.7: A player completed a dungeon (the dungeon was removed from the game).
    ///
    /// Emitted when the venture marker was on the bottommost room, the dungeon is removed,
    /// and `dungeons_completed` is incremented on the player's state.
    ///
    /// Discriminant: 115.
    DungeonCompleted {
        /// The player who completed the dungeon.
        player: PlayerId,
        /// Which dungeon was completed and removed.
        dungeon: crate::state::dungeon::DungeonId,
    },

    /// CR 725.2: A player took the initiative.
    ///
    /// Emitted when `Effect::TakeTheInitiative` resolves. The player who took
    /// the initiative also ventures into the Undercity (CR 725.2 inherent trigger).
    ///
    /// Discriminant: 116.
    InitiativeTaken {
        /// The player who now has the initiative.
        player: PlayerId,
    },

    /// CR 701.54a-c: The Ring tempted a player.
    ///
    /// Emitted after the ring level advances. Fires even when the player has no
    /// creatures (CR 701.54a — the temptation still occurs). Triggers abilities
    /// with `TriggerCondition::WheneverRingTemptsYou` (CR 701.54d).
    ///
    /// Discriminant: 117.
    RingTempted {
        /// The player who was tempted.
        player: PlayerId,
        /// The new ring level (1-4) after this temptation.
        new_level: u8,
    },

    /// CR 701.54a: A creature was chosen as a player's ring-bearer.
    ///
    /// Emitted after `RingTempted` when a creature is available to be chosen.
    /// Also fires when re-choosing the same creature (ruling 2023-06-16).
    ///
    /// Discriminant: 118.
    RingBearerChosen {
        /// The player who chose the ring-bearer.
        player: PlayerId,
        /// The creature that became the ring-bearer.
        creature: crate::state::game_object::ObjectId,
    },

    /// CR 724.1/724.3: A player became the monarch.
    ///
    /// Emitted when a player becomes the monarch (from an effect or from
    /// combat damage to the previous monarch — CR 724.2).
    ///
    /// Discriminant: 119.
    PlayerBecameMonarch {
        /// The player who is now the monarch.
        player: PlayerId,
    },
    // ── Coin flip / dice roll events (CR 705 / CR 706) ─────────────────────
    /// CR 705.1: A coin was flipped.
    ///
    /// Emitted by `Effect::CoinFlip` before executing the win/lose branch.
    /// `result` is true for heads (win), false for tails (lose).
    ///
    /// Discriminant: 121.
    CoinFlipped {
        /// The player who flipped the coin.
        player: PlayerId,
        /// True = heads (win), false = tails (lose).
        result: bool,
    },

    /// CR 706.2: A die was rolled.
    ///
    /// Emitted by `Effect::RollDice` before executing the matched branch.
    ///
    /// Discriminant: 122.
    DiceRolled {
        /// The player who rolled the die.
        player: PlayerId,
        /// Number of sides on the die.
        sides: u32,
        /// The result of the roll (1..=sides).
        result: u32,
    },

    /// An additional combat phase was added to the turn (CR 500.8).
    AdditionalCombatPhaseCreated {
        /// The player whose turn is gaining the extra phase.
        controller: PlayerId,
        /// Whether an additional main phase follows the extra combat.
        followed_by_main: bool,
    },

    /// CR 707.2: A permanent became a copy of another permanent.
    ///
    /// Emitted by `Effect::BecomeCopyOf` when the Layer 1 copy effect is registered.
    ///
    /// Discriminant: 123.
    BecameCopyOf {
        /// The permanent that became a copy.
        copier: crate::state::game_object::ObjectId,
        /// The permanent it became a copy of.
        source: crate::state::game_object::ObjectId,
    },
}

impl GameEvent {
    /// Returns `true` if this event reveals or commits to hidden information.
    ///
    /// Used by the M10 network layer to identify safe rewind checkpoints:
    /// once hidden information has been revealed, replaying from before that
    /// point would allow cheating. Events that return `true` include drawing
    /// cards, discarding cards (reveals the card's identity to all players),
    /// and any future scry/peek/face-down reveal events.
    ///
    /// All other events (priority, turn structure, combat, SBAs, etc.) return
    /// `false` — they involve only public information and are safe to rewind.
    pub fn reveals_hidden_info(&self) -> bool {
        match self {
            // Drawing a card reveals which card was drawn (the card moves from
            // hidden zone to the player's hand; other players learn it exists).
            GameEvent::CardDrawn { .. } => true,
            // Discarding reveals the card's identity to all players.
            GameEvent::CardDiscarded { .. } => true,
            // Cycling reveals the card's identity to all players (CR 702.29a — discard as cost).
            GameEvent::CardCycled { .. } => true,
            // CR 702.143a: Foretelling reveals the card's identity to the owner only.
            // Opponents see that a card was exiled face-down but not its identity.
            GameEvent::CardForetold { .. } => true,
            // All other events involve only public information.
            _ => false,
        }
    }
}
