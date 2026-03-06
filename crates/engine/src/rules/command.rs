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
use crate::state::types::AltCostKind;

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
        /// CR 118.9 / CR 702: Which alternative casting cost is being used, if any.
        ///
        /// - `None` — pay the printed mana cost (normal cast).
        /// - `Some(AltCostKind::Evoke)` — CR 702.74a: pay evoke cost instead of mana cost.
        /// - `Some(AltCostKind::Bestow)` — CR 702.103a: pay bestow cost instead of mana cost.
        /// - `Some(AltCostKind::Miracle)` — CR 702.94a: pay miracle cost; card must be in hand
        ///   with MiracleTrigger on the stack.
        /// - `Some(AltCostKind::Escape)` — CR 702.138a: pay escape cost; `escape_exile_cards`
        ///   must contain the required number of cards to exile.
        /// - `Some(AltCostKind::Foretell)` — CR 702.143a: pay foretell cost; card must be in
        ///   exile with `is_foretold == true` and foretold on a prior turn.
        /// - `Some(AltCostKind::Buyback)` — CR 702.27a: pay buyback as an ADDITIONAL cost;
        ///   card returns to hand on resolution instead of graveyard.
        /// - `Some(AltCostKind::Overload)` — CR 702.96a: pay overload cost; spell affects all
        ///   valid objects. Do NOT pass any `targets` when overloading.
        /// - `Some(AltCostKind::Retrace)` — CR 702.81a: cast from graveyard; `retrace_discard_land`
        ///   must identify a land in hand to discard as an additional cost.
        /// - `Some(AltCostKind::JumpStart)` — CR 702.133a: cast from graveyard; `jump_start_discard`
        ///   must identify a card in hand to discard. Spell exiled on stack departure.
        /// - `Some(AltCostKind::Aftermath)` — CR 702.127a: cast aftermath half from graveyard;
        ///   uses aftermath half's cost and effect. Spell exiled on stack departure.
        /// - `Some(AltCostKind::Dash)` — CR 702.109a: pay dash cost; permanent enters with haste
        ///   and returns to hand at beginning of next end step.
        ///
        /// Only one alternative cost may be applied (CR 118.9a: cannot combine two alternative costs).
        /// Buyback and Retrace are additional costs (CR 118.8); Flashback is always auto-detected
        /// from zone + keyword and does NOT appear here.
        #[serde(default)]
        alt_cost: Option<AltCostKind>,
        /// CR 702.138a: Cards in the caster's graveyard to exile as part of the
        /// escape cost. Must be exactly `exile_count` cards (from AbilityDefinition::Escape).
        /// Each card must be in the caster's graveyard, must not be the card being
        /// cast (the spell itself), and must not be duplicated.
        ///
        /// Empty vec when `alt_cost != Some(AltCostKind::Escape)`.
        #[serde(default)]
        escape_exile_cards: Vec<ObjectId>,
        /// CR 702.81a: If using retrace, the ObjectId of a land card in the player's
        /// hand to discard as an additional cost. Must be a card with `CardType::Land`
        /// in the player's hand. `None` if not using retrace.
        ///
        /// Required when `alt_cost == Some(AltCostKind::Retrace)`.
        #[serde(default)]
        retrace_discard_land: Option<ObjectId>,
        /// CR 702.133a: The card to discard as the jump-start additional cost.
        /// Must be a card in the caster's hand (not the jump-start card itself,
        /// which is in the graveyard). Required when `alt_cost == Some(AltCostKind::JumpStart)`.
        #[serde(default)]
        jump_start_discard: Option<ObjectId>,
        /// CR 702.160 / CR 718.3: If true, the spell is cast as a prototyped spell.
        ///
        /// When prototyped, the prototype mana cost (from `AbilityDefinition::Prototype`)
        /// is used for payment instead of the card's normal mana cost, and the
        /// spell and the permanent it becomes have only the alternative P/T, color,
        /// and mana cost (CR 718.3b).
        ///
        /// IMPORTANT: Prototype is NOT an alternative cost (CR 118.9, ruling 2022-10-14).
        /// It can be combined with `alt_cost` (e.g., prototype + flashback). The `prototype`
        /// flag is orthogonal to `alt_cost`.
        ///
        /// Validation: card must have `AbilityDefinition::Prototype` in its definition.
        #[serde(default)]
        prototype: bool,
        /// CR 702.166a: The ObjectId of an artifact, enchantment, or token on the
        /// battlefield to sacrifice as the bargain additional cost.
        /// `None` means the player chose not to bargain (bargain is optional).
        ///
        /// When `Some`, the identified permanent must be:
        /// - On the battlefield, controlled by the caster
        /// - An artifact OR an enchantment OR a token
        ///
        /// Validated in `handle_cast_spell`. Ignored for spells without bargain.
        #[serde(default)]
        bargain_sacrifice: Option<ObjectId>,
        /// CR 702.119a: The ObjectId of a creature on the battlefield to sacrifice
        /// as part of the emerge alternative cost. `None` means not using emerge.
        ///
        /// When `Some`, the identified creature must be:
        /// - On the battlefield, controlled by the caster
        /// - A creature (by current characteristics)
        ///
        /// The spell's total cost is reduced by the sacrificed creature's mana value.
        /// `alt_cost` must be `Some(AltCostKind::Emerge)` when this is `Some`.
        #[serde(default)]
        emerge_sacrifice: Option<ObjectId>,
        /// CR 702.153a: The ObjectId of a creature on the battlefield to sacrifice
        /// as the casualty additional cost. `None` means the player chose not to pay
        /// the casualty cost (casualty is optional).
        ///
        /// When `Some`, the identified permanent must be:
        /// - On the battlefield, controlled by the caster
        /// - A creature (by current characteristics)
        /// - Have power >= N (where N is from `KeywordAbility::Casualty(N)`)
        ///
        /// Unlike Emerge, Casualty is an additional cost — the normal mana cost is
        /// still paid. Validated in `handle_cast_spell`.
        #[serde(default)]
        casualty_sacrifice: Option<ObjectId>,
        /// CR 702.132a: The player who assists with the generic mana cost.
        /// `None` means no assist (either the spell lacks Assist or the caster
        /// chose not to use it). Must be a non-eliminated player other than
        /// the caster (CR 702.132a: "another player").
        #[serde(default)]
        assist_player: Option<PlayerId>,
        /// CR 702.132a: The amount of generic mana the assisting player pays.
        /// Must be <= the generic component of the spell's total cost (after all
        /// cost modifications: commander tax, kicker, affinity, undaunted, convoke,
        /// improvise, delve). 0 is valid (no-op assist). Ignored when
        /// `assist_player` is `None`.
        #[serde(default)]
        assist_amount: u32,
        /// CR 702.56a: Number of times the replicate cost was paid as an additional
        /// cost during casting.
        /// 0 = not paid (no copies). N = paid N times → N copies created by trigger.
        /// Each payment adds the replicate cost to the total mana cost paid for the spell
        /// (CR 601.2f-h). Validated against the spell having `KeywordAbility::Replicate`.
        /// Ignored for spells without replicate.
        #[serde(default)]
        replicate_count: u32,
        /// CR 702.47a: Cards in the caster's hand to splice onto this spell.
        /// Each ObjectId must be a card in the caster's hand that has
        /// `KeywordAbility::Splice` and whose `AbilityDefinition::Splice.onto_subtype`
        /// matches a subtype of the spell being cast (e.g., "Arcane").
        /// Each splice adds its cost as an additional cost (CR 601.2f-h) and appends
        /// its effect to the spell's resolution (CR 702.47b).
        /// Empty vec = no splice. CR 702.47b: each card can only be spliced once per spell.
        #[serde(default)]
        splice_cards: Vec<ObjectId>,
        /// CR 702.42a: If true, the entwine additional cost was paid. When true, all modes
        /// of the modal spell are chosen instead of just one.
        ///
        /// Validated in `handle_cast_spell`: spell must have `KeywordAbility::Entwine` and
        /// `AbilityDefinition::Spell.modes` must be `Some(...)`. Ignored for non-modal spells.
        #[serde(default)]
        entwine_paid: bool,
        /// CR 702.120a: Number of additional modes beyond the first for which the escalate
        /// cost is paid. 0 = single mode (no extra cost). N = pay escalate cost N times and
        /// execute modes 0..=N at resolution.
        ///
        /// Validated in `handle_cast_spell`: spell must have `KeywordAbility::Escalate` and
        /// `AbilityDefinition::Spell.modes` must be `Some(...)`. N must be < modes.len().
        #[serde(default)]
        escalate_modes: u32,
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
}
