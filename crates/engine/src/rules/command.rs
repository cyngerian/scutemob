//! Player commands — the only way to change game state.
//!
//! All player actions are Commands. There is no way to change game state
//! except through the Command enum. This enables networking, replay, and
//! deterministic testing.

use serde::{Deserialize, Serialize};

use crate::state::combat::AttackTarget;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::replacement_effect::ReplacementId;
use crate::state::targeting::Target;

/// A player action submitted to the engine.
///
/// All player actions are Commands. There is no way to change game state
/// except through the Command enum. This enables networking, replay, and
/// deterministic testing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// The player passes priority (CR 117.3d).
    PassPriority { player: PlayerId },

    /// The player concedes the game (CR 104.3a).
    Concede { player: PlayerId },

    /// Activate a mana ability by tapping a permanent (CR 605).
    ///
    /// Mana abilities do not use the stack and resolve immediately.
    /// `ability_index` is the index into `characteristics.mana_abilities`
    /// on the source object.
    TapForMana {
        player: PlayerId,
        source: ObjectId,
        ability_index: usize,
    },

    /// Play a land from hand to battlefield (CR 305.1).
    ///
    /// Playing a land is a special action — it does not use the stack.
    /// Legal only during the active player's main phase with an empty stack.
    PlayLand { player: PlayerId, card: ObjectId },

    /// Cast a spell from hand (CR 601).
    ///
    /// `targets` contains the targets announced at cast time (CR 601.2c). Pass an
    /// empty vec for non-targeting spells. Each target is validated at cast time and
    /// again at resolution for the fizzle rule (CR 608.2b).
    ///
    /// Mana cost is paid from the player's mana pool (CR 601.2f-h). Spells with no
    /// mana cost (e.g., adventure spells cast for free via effects) pass cost via
    /// card definitions in M7.
    ///
    /// Casting speed:
    /// - Instants and spells with Flash may be cast any time the player has priority.
    /// - All other spells require sorcery speed (active player, main phase, empty stack).
    CastSpell {
        player: PlayerId,
        card: ObjectId,
        /// Targets announced at cast time (CR 601.2c). Empty for non-targeting spells.
        targets: Vec<Target>,
    },
    /// Activate a non-mana activated ability (CR 602).
    ///
    /// Unlike `TapForMana` (which resolves immediately), activated abilities
    /// use the stack. The ability is validated, its cost paid, and a
    /// `StackObject` is pushed. `ability_index` indexes into
    /// `characteristics.activated_abilities` on the source object.
    ///
    /// `targets` contains any targets for the ability (M3-E: stored but
    /// not type-validated; full validation in M7).
    ActivateAbility {
        player: PlayerId,
        source: ObjectId,
        ability_index: usize,
        targets: Vec<Target>,
    },

    // ── M6: Combat commands ───────────────────────────────────────────────
    /// Declare attacking creatures and their targets (CR 508.1).
    ///
    /// Legal only in the DeclareAttackers step for the active player, who must
    /// have priority. Non-Vigilance attackers become tapped as a side effect.
    /// Pass an empty vec to attack with no creatures (legal — ends the attack step).
    DeclareAttackers {
        player: PlayerId,
        /// (attacker ObjectId, attack target) pairs.
        attackers: Vec<(ObjectId, AttackTarget)>,
    },

    /// Declare blocking creatures (CR 509.1).
    ///
    /// Legal only in the DeclareBlockers step. Each defending player may declare
    /// independently; priority is not required. Pass an empty vec to block with
    /// no creatures.
    DeclareBlockers {
        player: PlayerId,
        /// (blocker ObjectId, attacker ObjectId being blocked) pairs.
        blockers: Vec<(ObjectId, ObjectId)>,
    },

    /// Set the damage assignment order for an attacker with multiple blockers (CR 509.2).
    ///
    /// The attacking player chooses the order in which their attacker's damage
    /// is assigned when multiple creatures are blocking. `order` lists blocker
    /// ObjectIds front-to-back (front receives damage first; must receive lethal
    /// before damage flows to the next).
    OrderBlockers {
        player: PlayerId,
        attacker: ObjectId,
        order: Vec<ObjectId>,
    },

    // ── M8: Replacement effect commands ─────────────────────────────────
    /// Choose the order in which multiple replacement effects apply (CR 616.1).
    ///
    /// When multiple replacement effects would apply to the same event, the
    /// affected player chooses the order. `ids` must be a permutation of the
    /// IDs from the corresponding `ReplacementChoiceRequired` event.
    OrderReplacements {
        player: PlayerId,
        ids: Vec<ReplacementId>,
    },

    // ── M9: Commander zone return command ────────────────────────────────
    /// Return a commander from graveyard or exile to its owner's command zone
    /// as a state-based action choice (CR 903.9a / CR 704.6d).
    ///
    /// In M9 this command is handled automatically by the SBA (no explicit
    /// player action required). The command variant is retained for future
    /// M10+ player opt-out support.
    ReturnCommanderToCommandZone {
        player: PlayerId,
        object_id: ObjectId,
    },

    // ── M9: Mulligan commands (CR 103.5 / CR 103.5c) ─────────────────────
    /// Take a mulligan: shuffle hand into library, draw 7, then put N cards
    /// on the bottom where N = mulligan number (0 for the free mulligan).
    ///
    /// CR 103.5: Mulligan procedure. CR 103.5c: First mulligan in multiplayer
    /// is free (draw back to 7 with no cards to bottom).
    TakeMulligan { player: PlayerId },

    /// Keep hand (with optional cards to put on the bottom of library).
    ///
    /// CR 103.5: After deciding to keep, a player with N mulligans taken puts
    /// N cards from their hand on the bottom of their library in any order.
    /// For the free mulligan (N=0 effectively), no cards go to the bottom.
    KeepHand {
        player: PlayerId,
        /// Cards to put on the bottom of library (in order, bottom-most last).
        /// Length must equal the number of mulligans taken by this player.
        cards_to_bottom: Vec<ObjectId>,
    },

    // ── M9: Companion command (CR 702.139a) ───────────────────────────────
    /// Pay {3} to put companion from the sideboard into hand (CR 702.139a).
    ///
    /// Special action: costs {3}, requires main phase, stack empty, priority,
    /// and that the player has not already used this action.
    BringCompanion { player: PlayerId },
}
