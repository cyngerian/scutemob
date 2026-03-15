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
use crate::state::types::{AdditionalCost, AltCostKind, FaceDownKind, TurnFaceUpMethod};

/// A player action submitted to the engine.
///
/// All player actions are Commands. There is no way to change game state
/// except through the Command enum. This enables networking, replay, and
/// deterministic testing.
// CastSpell has many optional fields (additional costs, alt costs, splice, etc.)
// making it large by design; the size difference vs. PassPriority is expected.
#[allow(clippy::large_enum_variant)]
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
        convoke_creatures: Vec<ObjectId>,
        /// CR 702.126: Artifacts to tap for improvise cost reduction.
        #[serde(default)]
        improvise_artifacts: Vec<ObjectId>,
        /// CR 702.66: Cards in the caster's graveyard to exile for delve cost reduction.
        delve_cards: Vec<ObjectId>,
        /// CR 702.33d: Number of times to pay the kicker cost.
        #[serde(default)]
        kicker_times: u32,
        /// CR 118.9 / CR 702: Which alternative casting cost is being used, if any.
        #[serde(default)]
        alt_cost: Option<AltCostKind>,
        /// CR 702.160 / CR 718.3: If true, the spell is cast as a prototyped spell.
        /// IMPORTANT: Prototype is NOT an alternative cost (CR 118.9, ruling 2022-10-14).
        #[serde(default)]
        prototype: bool,
        /// CR 700.2a / 601.2b: Mode indices chosen for a modal spell.
        #[serde(default)]
        modes_chosen: Vec<usize>,
        /// CR 107.3m: The value chosen for X in the spell's mana cost. 0 for non-X spells.
        #[serde(default)]
        x_value: u32,
        /// CR 107.4e: For each hybrid pip in the resolved cost, how it was paid.
        /// Length must match total hybrid pips after cost calculation.
        /// Empty = default to first color option for each hybrid pip.
        #[serde(default)]
        hybrid_choices: Vec<crate::state::game_object::HybridManaPayment>,
        /// CR 107.4f: For each Phyrexian pip, true = pay 2 life; false = pay mana.
        /// Length must match total Phyrexian pips after cost calculation.
        /// Empty = default to paying with mana for each pip.
        #[serde(default)]
        phyrexian_life_payments: Vec<bool>,
        /// CR 702.37c / 702.168b: Which face-down variant is being used when casting face-down.
        #[serde(default)]
        face_down_kind: Option<FaceDownKind>,
        /// Consolidated additional costs (RC-1 type consolidation).
        /// All additional-cost data (sacrifice, discard, exile-from-zone, assist,
        /// replicate, squad, escalate, splice, entwine, fuse, offspring, gift, mutate).
        #[serde(default)]
        additional_costs: Vec<AdditionalCost>,
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
        /// CR 111.10g / CR 602.2: If the ability's cost requires discarding a card,
        /// this is the ObjectId of the card in the player's hand to discard as cost.
        /// `None` for abilities that don't require a discard cost.
        #[serde(default)]
        discard_card: Option<ObjectId>,
        /// CR 602.2: If the ability's cost requires sacrificing another permanent
        /// (not self — that's `sacrifice_self`), this is the ObjectId of the permanent
        /// to sacrifice. Must match the `sacrifice_filter` on the `ActivationCost`.
        /// `None` for abilities that don't require sacrificing another permanent.
        #[serde(default)]
        sacrifice_target: Option<ObjectId>,
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
        /// CR 702.154a / CR 508.1g: Optional enlist cost payments.
        ///
        /// Each entry is (enlisting_attacker_id, enlisted_creature_id).
        /// The enlisted creature will be tapped as a cost during the
        /// declare-attackers step. The attacker must have Enlist; the
        /// enlisted creature must be untapped, non-attacking, controlled
        /// by the player, a creature, and not have summoning sickness
        /// (or have haste).
        ///
        /// Empty vec for no enlist choices. At most one entry per Enlist
        /// keyword instance on a given attacker. A creature can only be
        /// enlisted once across all attackers (ruling 2022-09-09).
        ///
        /// Validated in handle_declare_attackers.
        #[serde(default)]
        enlist_choices: Vec<(ObjectId, ObjectId)>,
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

    // ── Forecast (CR 702.57) ─────────────────────────────────────────────
    /// Activate a forecast ability from hand (CR 702.57a).
    ///
    /// Forecast is an activated ability that can only be activated from a player's
    /// hand, only during the upkeep step of the card's owner, and only once per turn.
    /// The card is revealed as part of activation but stays in hand (unlike Cycling
    /// which discards as cost). The effect uses the stack and can be responded to.
    ///
    /// Unlike `ActivateAbility` (which requires the source on the battlefield),
    /// this command works from the hand zone.
    ActivateForecast {
        player: PlayerId,
        card: ObjectId,
        targets: Vec<crate::state::targeting::Target>,
    },

    // ── Bloodrush (CR 207.2c) ─────────────────────────────────────────────
    /// CR 207.2c: Activate a bloodrush ability from hand.
    ///
    /// Bloodrush is an activated ability that functions only while the card is in
    /// the player's hand. The activation cost is the mana cost plus discarding the
    /// card itself. The effect pumps a target attacking creature until end of turn.
    ///
    /// Unlike `ActivateAbility` (which requires the source on the battlefield),
    /// this command works from the hand zone. The card is discarded as cost (CR 602.2b).
    ActivateBloodrush {
        player: PlayerId,
        card: ObjectId,
        target: ObjectId,
    },

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

    // ── Miracle (CR 702.94) ──────────────────────────────────────────────
    /// Choose whether to reveal a miracle card drawn as the first card this turn (CR 702.94a).
    ///
    /// Sent in response to a `MiracleRevealChoiceRequired` event. If `reveal` is `true`,
    /// the card is revealed and a miracle trigger is placed on the stack. When that
    /// trigger resolves, the card stays in hand (the player may cast it from hand while
    /// the trigger is on the stack using `CastSpell` with `cast_with_miracle: true`).
    /// If `reveal` is `false`, the card stays in hand as a normal draw.
    ///
    /// Validation: the card must be in the player's hand with `KeywordAbility::Miracle`,
    /// and `cards_drawn_this_turn` must be 1 (first draw).
    ChooseMiracle {
        player: PlayerId,
        /// The drawn card in hand. If `reveal` is false, this field is ignored but should
        /// match the `card_object_id` from the `MiracleRevealChoiceRequired` event.
        card: ObjectId,
        /// True = reveal and put miracle trigger on stack. False = decline (normal draw).
        reveal: bool,
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

    // ── Saddle (CR 702.171) ──────────────────────────────────────────────────
    /// Saddle a Mount by tapping creatures (CR 702.171a).
    ///
    /// Tap any number of untapped creatures you control with total power >= N
    /// to activate the Mount's saddle ability. The ability goes on the stack;
    /// when it resolves, the Mount becomes saddled until end of turn.
    /// Activate only as a sorcery (CR 702.171a).
    ///
    /// Unlike `CrewVehicle`, the Mount is already a creature (no type change). Instead,
    /// a boolean designation `is_saddled` is set when the saddle ability resolves.
    SaddleMount {
        player: PlayerId,
        /// The Mount to saddle.
        mount: ObjectId,
        /// Creatures to tap as the saddle cost. Must be untapped creatures you control
        /// with total power >= the Mount's Saddle N value. The Mount itself cannot
        /// be in this list ("other untapped creatures" per CR 702.171a).
        saddle_creatures: Vec<ObjectId>,
    },

    // -- Foretell (CR 702.143) -----------------------------------------------
    /// Foretell a card from hand (CR 702.143a / CR 116.2h).
    ///
    /// Special action: pay {2}, exile a card with foretell from your hand face down.
    /// This does not use the stack. Legal any time you have priority during your turn.
    /// The card can be cast for its foretell cost on a future turn.
    ForetellCard { player: PlayerId, card: ObjectId },

    // -- Plot (CR 702.170) -----------------------------------------------
    /// Plot a card from hand (CR 702.170a / CR 116.2k).
    ///
    /// Special action: pay [plot cost], exile a card with plot from your hand face up.
    /// This does not use the stack. Legal during your main phase while stack is empty.
    /// The card can be cast without paying its mana cost on a future turn.
    PlotCard { player: PlayerId, card: ObjectId },

    // -- Suspend (CR 702.62) -----------------------------------------------
    /// Suspend a card from hand (CR 702.62a / CR 116.2f).
    ///
    /// Special action: pay the suspend cost, exile the card from hand face up with
    /// N time counters. This does not use the stack. Legal any time the player has
    /// priority and could begin to cast the card (timing check depends on card type).
    ///
    /// At the beginning of the owner's upkeep, a time counter is removed (triggered
    /// ability that uses the stack). When the last counter is removed, the owner may
    /// cast it without paying its mana cost (also a triggered ability). If cast via
    /// suspend and the card is a creature spell, it gains haste.
    SuspendCard { player: PlayerId, card: ObjectId },

    // -- Unearth (CR 702.84) -----------------------------------------------
    /// Activate a card's unearth ability from the graveyard (CR 702.84a).
    ///
    /// The card must be in the player's graveyard with `KeywordAbility::Unearth`.
    /// The unearth cost is paid, and the unearth ability is placed on the stack.
    /// When it resolves, the card returns to the battlefield with haste,
    /// a delayed exile trigger (beginning of next end step), and a replacement
    /// effect (if it would leave battlefield for non-exile, exile instead).
    ///
    /// "Activate only as a sorcery" -- main phase, stack empty, active player.
    ///
    /// Unlike `CastSpell`, this is an activated ability, not a spell cast.
    /// No "cast" triggers fire.
    UnearthCard { player: PlayerId, card: ObjectId },

    // -- Embalm (CR 702.128) ------------------------------------------------
    /// Activate a card's embalm ability from the graveyard (CR 702.128a).
    ///
    /// The card must be in the player's graveyard with `KeywordAbility::Embalm`.
    /// The embalm cost is paid, the card is exiled (as part of the cost), and
    /// the embalm ability is placed on the stack. When it resolves, a token copy
    /// of the card is created (white, no mana cost, Zombie added to types).
    ///
    /// Unlike Unearth, the card is exiled as part of the activation cost, NOT
    /// when the ability resolves.
    EmbalmCard { player: PlayerId, card: ObjectId },

    // -- Eternalize (CR 702.129) ---------------------------------------------
    /// Activate a card's eternalize ability from the graveyard (CR 702.129a).
    ///
    /// The card must be in the player's graveyard with `KeywordAbility::Eternalize`.
    /// The eternalize cost is paid, the card is exiled (as part of the cost), and
    /// the eternalize ability is placed on the stack. When it resolves, a token copy
    /// of the card is created (black, 4/4, no mana cost, Zombie added to types).
    ///
    /// Unlike Unearth, the card is exiled as part of the activation cost, NOT
    /// when the ability resolves.
    EternalizeCard { player: PlayerId, card: ObjectId },

    // -- Encore (CR 702.141) ------------------------------------------------
    /// Activate a card's encore ability from the graveyard (CR 702.141a).
    ///
    /// The card must be in the player's graveyard with `KeywordAbility::Encore`.
    /// The card is exiled as a cost, the encore cost is paid, and the encore
    /// ability is placed on the stack. When it resolves, for each opponent,
    /// a token copy of the exiled card is created with haste, tagged for
    /// attack this turn and sacrifice at the next end step.
    ///
    /// "Activate only as a sorcery" -- main phase, stack empty, active player.
    ///
    /// Unlike `UnearthCard`, the card is exiled as part of the cost (before
    /// the ability goes on the stack), not moved to the battlefield.
    EncoreCard { player: PlayerId, card: ObjectId },

    // -- Scavenge (CR 702.97) -----------------------------------------------
    /// Activate a card's scavenge ability from the graveyard (CR 702.97a).
    ///
    /// The card must be in the player's graveyard with `KeywordAbility::Scavenge`.
    /// The card is exiled as part of the activation cost. The ability is placed
    /// on the stack targeting `target_creature`. When it resolves, +1/+1 counters
    /// equal to the card's power (as it last existed in the graveyard) are placed
    /// on the target creature.
    ///
    /// "Activate only as a sorcery" -- main phase, stack empty, active player.
    ScavengeCard {
        player: PlayerId,
        card: ObjectId,
        target_creature: ObjectId,
    },

    // -- Ninjutsu (CR 702.49) -----------------------------------------------
    /// Activate a card's ninjutsu ability from hand (or command zone for
    /// commander ninjutsu).
    ///
    /// CR 702.49a: The player pays the ninjutsu cost, returns an unblocked
    /// attacking creature they control to its owner's hand, and the ninjutsu
    /// card is put onto the battlefield tapped and attacking the same target.
    ///
    /// This is an activated ability, NOT a spell cast. No "cast" triggers fire.
    /// Commander ninjutsu (CR 702.49d) bypasses commander tax entirely.
    ///
    /// The ability goes on the stack. The attacker is returned to hand as a
    /// cost (immediately). The ninja enters the battlefield when the ability
    /// resolves.
    ActivateNinjutsu {
        player: PlayerId,
        /// The card with ninjutsu in the player's hand (or command zone).
        ninja_card: ObjectId,
        /// The unblocked attacking creature to return to its owner's hand.
        attacker_to_return: ObjectId,
    },

    // ── Echo (CR 702.30) ─────────────────────────────────────────────────
    /// Choose whether to pay the echo cost for a permanent (CR 702.30a).
    ///
    /// Sent in response to an `EchoPaymentRequired` event. If `pay` is true,
    /// the player pays the echo cost (mana is deducted) and the permanent stays.
    /// If `pay` is false (or the player cannot afford the cost), the permanent
    /// is sacrificed.
    PayEcho {
        player: PlayerId,
        permanent: ObjectId,
        /// True = pay the echo cost. False = sacrifice the permanent.
        pay: bool,
    },

    // ── Cumulative Upkeep (CR 702.24) ─────────────────────────────────────────
    /// Choose whether to pay the cumulative upkeep cost for a permanent (CR 702.24a).
    ///
    /// Sent in response to a `CumulativeUpkeepPaymentRequired` event. If `pay`
    /// is true, the player pays the total cost (per-counter cost x age_count).
    /// If `pay` is false, the permanent is sacrificed.
    PayCumulativeUpkeep {
        player: PlayerId,
        permanent: ObjectId,
        /// True = pay the total cumulative upkeep cost. False = sacrifice.
        pay: bool,
    },

    // ── Recover (CR 702.59) ───────────────────────────────────────────────────
    /// Choose whether to pay the recover cost for a card in the graveyard (CR 702.59a).
    ///
    /// Sent in response to a `RecoverPaymentRequired` event. If `pay` is true,
    /// the player pays the recover cost (mana is deducted) and the card is returned
    /// from the graveyard to the player's hand. If `pay` is false, the card is
    /// exiled (CR 702.59a: "Otherwise, exile this card.").
    PayRecover {
        player: PlayerId,
        recover_card: ObjectId,
        /// True = pay the recover cost and return to hand. False = exile the card.
        pay: bool,
    },

    // ── Transform (CR 701.27 / CR 712) ───────────────────────────────────────
    /// Transform a double-faced permanent (CR 701.27a).
    ///
    /// Flips the permanent to its other face. No new object is created (CR 712.18).
    /// Counters, damage, auras, and continuous effects all persist.
    ///
    /// Validation:
    /// - Permanent must be on battlefield and controlled by the player.
    /// - Permanent must have a back_face in the card registry (CR 701.27c).
    /// - Back face cannot be instant or sorcery (CR 701.27d).
    /// - Permanents with daybound/nightbound can only transform via their keyword
    ///   enforcement system, not via direct transform commands (CR 702.145b/e).
    Transform {
        player: PlayerId,
        /// The permanent to transform.
        permanent: ObjectId,
    },

    // ── Craft (CR 702.167) ────────────────────────────────────────────────────
    /// Activate a permanent's craft ability (CR 702.167a).
    ///
    /// Cost: [craft cost] + exile this permanent + exile [materials] from
    /// permanents you control and/or cards in your graveyard.
    ///
    /// When this ability resolves: return the exiled source card to the battlefield
    /// transformed under its owner's control. If the card isn't a DFC, it stays in exile.
    ///
    /// "Activate only as a sorcery" (CR 702.167a).
    ActivateCraft {
        player: PlayerId,
        /// The permanent with the craft ability (will be exiled as cost).
        source: ObjectId,
        /// Cards/permanents to exile as the material cost.
        material_ids: Vec<ObjectId>,
    },

    // ── The Ring Tempts You (CR 701.54) ──────────────────────────────────────
    /// The ring tempts the given player (CR 701.54a).
    ///
    /// Advances the player's ring level by 1 (capped at 4) and lets the player
    /// choose a creature as their ring-bearer. In the deterministic engine the
    /// creature with the lowest ObjectId is chosen automatically.
    ///
    /// This command is used by scripts and triggered abilities that say "the Ring
    /// tempts you" as a keyword action (CR 701.54a). It is also the fallback used
    /// internally by `Effect::TheRingTemptsYou`.
    TheRingTemptsYou { player: PlayerId },

    // ── Dungeon / Venture (CR 701.49, CR 725) ────────────────────────────────
    /// Trigger a venture-into-the-dungeon action for the given player (CR 701.49).
    ///
    /// The engine applies deterministic dungeon/room selection:
    /// - If the player has no dungeon in the command zone: enter LostMineOfPhandelver
    ///   (or TheUndercity if `force_undercity` is set by the effect handler).
    /// - If the player is not on the bottommost room: advance to the first exit.
    /// - If the player is on the bottommost room: complete the dungeon and start a new one.
    ///
    /// After advancing the marker, a `StackObjectKind::RoomAbility` is pushed onto the
    /// stack for the room the marker moved into (CR 309.4c).
    VentureIntoDungeon { player: PlayerId },

    /// Choose which room to advance to when a dungeon has branching paths (CR 309.5a).
    ///
    /// In the current deterministic implementation, this command is accepted but the
    /// engine always advances to the first exit (index 0). Full interactive branching
    /// is deferred to M10+.
    ChooseDungeonRoom {
        player: PlayerId,
        room: crate::state::dungeon::RoomIndex,
    },

    // ── Morph / Manifest / Cloak (CR 702.37, 701.40, 701.58) ─────────────────
    /// Turn a face-down permanent face up (CR 702.37e, 702.168d, 701.40b, 701.58b).
    ///
    /// This is a special action (CR 116.2b) — it does NOT use the stack and cannot
    /// be responded to. The engine validates:
    /// 1. The permanent is on the battlefield, face-down, controlled by the player.
    /// 2. The permanent can be turned face up via the chosen method.
    /// 3. The player can pay the appropriate cost.
    ///
    /// On success: the cost is paid, face_down is set to false, face_down_as is cleared,
    /// the permanent regains its real characteristics, and a PermanentTurnedFaceUp event
    /// is emitted. ETB abilities do NOT fire (CR 708.8). "When turned face up" triggered
    /// abilities DO fire and go on the stack (CR 708.8).
    ///
    /// If the permanent is a Megamorph and the method is MorphCost, a +1/+1 counter
    /// is placed on it as it turns face up (CR 702.37b).
    TurnFaceUp {
        player: PlayerId,
        /// The face-down permanent to turn face up.
        permanent: ObjectId,
        /// Which turn-face-up method to use.
        ///
        /// A manifested card with morph may use either MorphCost or ManaCost (CR 701.40c).
        /// A manifested card with disguise may use either DisguiseCost or ManaCost (CR 701.40d).
        method: TurnFaceUpMethod,
    },

    /// CR 606: Activate a loyalty ability on a planeswalker (CR 306.5d).
    ///
    /// Special timing rules (CR 606.3): main phase, stack empty, once per permanent per turn.
    /// The loyalty cost (add/remove counters) is paid as part of activation (CR 606.4).
    /// CR 606.6: Negative costs require sufficient loyalty counters.
    ActivateLoyaltyAbility {
        player: PlayerId,
        /// The planeswalker permanent whose loyalty ability is being activated.
        source: ObjectId,
        /// Index into the card's loyalty abilities (filtered from `abilities` vec).
        ability_index: usize,
        /// Targets for the loyalty ability (may be empty).
        targets: Vec<Target>,
        /// For −X abilities: the chosen X value. `None` for fixed-cost abilities.
        #[serde(default)]
        x_value: Option<u32>,
    },
    /// CR 716.2a: Level up a Class enchantment.
    ///
    /// "[Cost]: This Class's level becomes N. Activate only if this Class is
    /// level N-1 and only as a sorcery."
    LevelUpClass {
        player: PlayerId,
        /// The Class permanent to level up.
        source: ObjectId,
        /// The target level (N). The Class must currently be at level N-1.
        target_level: u32,
    },
}
