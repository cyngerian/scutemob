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
        /// CR 702.51: Creatures to tap for convoke cost reduction.
        /// Empty vec for non-convoke spells. Each creature must be:
        /// - Untapped, on the battlefield, controlled by the caster
        /// - A creature (by current characteristics)
        /// - Not duplicated (no ObjectId appears twice)
        ///
        /// Colored creatures pay for one colored mana of their color;
        /// any creature pays for {1} generic. Validated in handle_cast_spell.
        convoke_creatures: Vec<ObjectId>,
        /// CR 702.126: Artifacts to tap for improvise cost reduction.
        /// Empty vec for non-improvise spells. Each artifact must be:
        /// - Untapped, on the battlefield, controlled by the caster
        /// - An artifact (by current characteristics)
        /// - Not duplicated (no ObjectId appears twice)
        ///
        /// Each artifact pays for {1} generic mana. Cannot exceed the generic
        /// mana component of the spell's total cost (after convoke reduction).
        /// Validated in handle_cast_spell -> apply_improvise_reduction.
        #[serde(default)]
        improvise_artifacts: Vec<ObjectId>,
        /// CR 702.66: Cards in the caster's graveyard to exile for delve cost reduction.
        /// Empty vec for non-delve spells. Each card must be:
        /// - In the caster's graveyard (not opponent's)
        /// - Not duplicated (no ObjectId appears twice)
        ///
        /// Each exiled card pays for {1} generic mana. Cannot exceed the generic
        /// mana component of the spell's total cost.
        /// Validated in handle_cast_spell -> apply_delve_reduction.
        delve_cards: Vec<ObjectId>,
        /// CR 702.33d: Number of times to pay the kicker cost.
        ///
        /// 0 = not kicked. 1 = kicked once (standard kicker). N > 1 = multikicker
        /// paid N times (CR 702.33c). Validated against the spell's kicker definition:
        /// standard kicker rejects values > 1; multikicker accepts any N >= 0.
        /// Ignored for spells without kicker.
        #[serde(default)]
        kicker_times: u32,
        /// CR 702.74a: If true, cast this spell by paying its evoke cost instead
        /// of its mana cost. This is an alternative cost (CR 118.9) -- cannot
        /// combine with flashback or other alternative costs.
        /// Ignored for spells without evoke.
        #[serde(default)]
        cast_with_evoke: bool,
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

    // ── M9: Commander zone return commands ───────────────────────────────
    /// Return a commander from graveyard or exile to its owner's command zone
    /// (CR 903.9a / CR 704.6d).
    ///
    /// Sent in response to a `CommanderZoneReturnChoiceRequired` event. The
    /// commander is moved to the command zone and the pending choice is cleared.
    ReturnCommanderToCommandZone {
        player: PlayerId,
        object_id: ObjectId,
    },

    /// Leave a commander in its current zone (graveyard or exile) instead of
    /// returning it to the command zone (CR 903.9a — owner "may" return).
    ///
    /// Sent in response to a `CommanderZoneReturnChoiceRequired` event. The
    /// pending choice is cleared; the commander stays where it is.
    LeaveCommanderInZone {
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

    // ── Cycling (CR 702.29) ───────────────────────────────────────────────
    /// Cycle a card from hand (CR 702.29a).
    ///
    /// Cycling is an activated ability that functions only while the card is in
    /// the player's hand. The activation cost is the cycling cost (mana) plus
    /// discarding the card itself. The effect is "draw a card" and uses the stack.
    ///
    /// Unlike `ActivateAbility` (which requires the source on the battlefield),
    /// `CycleCard` works from the hand zone. The card is discarded as cost
    /// (immediately), and a cycling ability is placed on the stack. When it
    /// resolves, the controller draws a card.
    CycleCard { player: PlayerId, card: ObjectId },

    // ── Dredge (CR 702.52) ───────────────────────────────────────────────
    /// Choose whether to dredge a card from the graveyard instead of drawing (CR 702.52a).
    ///
    /// Sent in response to a `DredgeChoiceRequired` event. If `card` is `Some(id)`,
    /// the player dredges that card (mills N, returns card to hand). If `card` is `None`,
    /// the player draws normally.
    ///
    /// Validation: the card must be in the player's graveyard with `KeywordAbility::Dredge(n)`,
    /// and the player must have >= n cards in their library (CR 702.52b).
    ChooseDredge {
        player: PlayerId,
        /// The dredge card to return from graveyard to hand, or None to draw normally.
        card: Option<ObjectId>,
    },

    // ── Crew (CR 702.122) ────────────────────────────────────────────────
    /// Crew a Vehicle by tapping creatures (CR 702.122a).
    ///
    /// Tap any number of untapped creatures you control with total power >= N
    /// to activate the Vehicle's crew ability. The ability goes on the stack;
    /// when it resolves, the Vehicle becomes an artifact creature until end of turn.
    ///
    /// Unlike `ActivateAbility`, this command explicitly names the creatures tapped
    /// as part of the crew cost (similar to how `CastSpell` names `convoke_creatures`).
    /// The `ActivationCost` struct cannot express a multi-creature tap cost, so Crew
    /// uses a dedicated command.
    CrewVehicle {
        player: PlayerId,
        /// The Vehicle to crew.
        vehicle: ObjectId,
        /// Creatures to tap as the crew cost. Must be untapped creatures you control
        /// with total power >= the Vehicle's Crew N value. The Vehicle itself cannot
        /// be in this list ("other untapped creatures" per CR 702.122a).
        crew_creatures: Vec<ObjectId>,
    },
}
