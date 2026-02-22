//! Game events emitted by the rules engine (CR 500-514, M2+).
//!
//! Events are the single source of truth for "what happened." The network
//! layer broadcasts them; the UI consumes them; the history log records them.

use serde::{Deserialize, Serialize};

use crate::state::combat::AttackTarget;
use crate::state::game_object::{ManaCost, ObjectId};
use crate::state::player::PlayerId;
use crate::state::turn::{Phase, Step};
use crate::state::types::ManaColor;
use crate::state::zone::ZoneId;

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

    /// An equipment (or fortification) became unattached because the object it
    /// was attached to is no longer a legal attachment target (CR 704.5n).
    EquipmentUnattached { object_id: ObjectId },

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
}
