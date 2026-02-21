//! Game events emitted by the rules engine (CR 500-514, M2+).
//!
//! Events are the single source of truth for "what happened." The network
//! layer broadcasts them; the UI consumes them; the history log records them.

use serde::{Deserialize, Serialize};

use crate::state::game_object::{ManaCost, ObjectId};
use crate::state::player::PlayerId;
use crate::state::turn::{Phase, Step};
use crate::state::types::ManaColor;
use crate::state::zone::ZoneId;

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
}
