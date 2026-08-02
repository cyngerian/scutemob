//! Legal action enumeration for the game simulator.
//!
//! Defines the `LegalAction` enum (all possible player actions) and the
//! `LegalActionProvider` trait. `StubProvider` implements basic checks
//! without deep engine knowledge — enough to play games, but misses edge
//! cases that a full engine implementation would catch.

use mtg_engine::state::game_object::SacrificeFilter;
use mtg_engine::{
    apply_commander_tax, AbilityDefinition, ActivatedAbility, AdditionalCost, AttackTarget,
    CardType, CounterType, EffectChoiceAnswer, EffectChoiceQuestion, EffectDuration, FaceDownKind,
    FlashGrantFilter, GameObject, GameRestriction, GameState, HybridMana, HybridManaPayment,
    KeywordAbility, ManaColor, ManaCost, ObjectId, PhyrexianMana, PlayerId, SpellAdditionalCost,
    Step, Target, TriggerTargetOption, TurnFaceUpMethod, ZoneId,
};

/// CR 118.8 / CR 601.2b (UI-2): the additional costs a `CastSpell` offer must or may
/// pay. Built by the provider, consumed by `params.rs` (for the bot default) and by
/// `tools/play-server` (to render a picker).
#[derive(Clone, Debug, Default)]
pub struct AdditionalCostPlan {
    /// CR 118.8: the spell's REQUIRED sacrifice, when its `CardDefinition` declares
    /// one. `None` for every other spell.
    pub sacrifice: Option<SacrificeCostOption>,
    /// CR 702.157: the OPTIONAL squad cost, when the spell has
    /// `AbilityDefinition::Squad { cost }`. `None` for every other spell.
    pub squad: Option<SquadCostOption>,
}

/// CR 118.8 (UI-2 §1.1): the offer's required-sacrifice descriptor.
#[derive(Clone, Debug)]
pub struct SacrificeCostOption {
    /// The engine's own requirement, verbatim -- for labelling only.
    pub requirement: SpellAdditionalCost,
    /// Battlefield permanents this player controls that `casting.rs`'s own gate will
    /// accept. **Never empty**: an empty set suppresses the whole offer (§1.3) --
    /// see `StubProvider::legal_actions`'s two cast loops.
    pub eligible: Vec<ObjectId>,
    /// The deterministic default a bot submits: `eligible[0]` (lowest `ObjectId`,
    /// since `objects_in_zone` yields the engine's own order). Kept as its own field
    /// rather than re-derived at the two consumer sites, so the two cannot drift.
    /// A sentinel (`ObjectId::SENTINEL`) when `eligible` is empty -- never read in
    /// that case, since the whole offer is suppressed before this value reaches a
    /// consumer.
    pub default: ObjectId,
}

/// CR 602.2 (SIM-6, triage G4): the **non-mana** activation costs an
/// `ActivateAbility` offer has to pay, described well enough that a client can
/// render a picker and a bot can submit an answer the engine will accept.
///
/// The sibling of [`AdditionalCostPlan`], one command over. UI-2 built that one for
/// `CastSpell` (CR 118.8) and stopped there; `Command::ActivateAbility` carries
/// `sacrifice_target` / `discard_card` on the wire and has since PB-EF1, but nothing
/// in this crate ever filled them, so `handle_activate_ability` refused every
/// sacrifice- or discard-cost activation with an `InvalidCommand` (CR 602.2) — a 422
/// in the browser, and ~135 of the 166 bot command refusals SIM-5 recorded.
///
/// **Mana is deliberately absent.** The mana/hybrid/Phyrexian/life components of an
/// activation cost are already gated (and, for pips, *planned*) by the offer loop in
/// [`StubProvider::legal_actions`]; this type covers exactly the components that
/// require the activating player to NAME an object.
#[derive(Clone, Debug, Default)]
pub struct ActivationCostPlan {
    /// CR 602.2: the "Sacrifice a/another <thing>" component, when the ability's
    /// `ActivationCost` declares a `sacrifice_filter`. `None` for every other
    /// ability — including one that sacrifices ITSELF (`sacrifice_self`), which
    /// names no object and so needs no channel.
    pub sacrifice: Option<ActivationSacrificeOption>,
    /// CR 602.2 / CR 111.10g: the "Discard a card" component, when the ability's
    /// `ActivationCost` sets `discard_card`. `None` for every other ability —
    /// including a Channel ability (`discard_self`), which names no object.
    pub discard: Option<ActivationDiscardOption>,
}

/// CR 602.2 (SIM-6): the offer's sacrifice descriptor.
#[derive(Clone, Debug)]
pub struct ActivationSacrificeOption {
    /// The engine's own `ActivationCost::sacrifice_filter`, verbatim — for
    /// labelling only. The CANDIDATES already carry the judgment of who is
    /// eligible, so a wrong label here cannot become a wrong game state.
    pub filter: mtg_engine::state::game_object::SacrificeFilter,
    /// CR 109.1 / PB-EF1: the ability's `sacrifice_exclude_self` bit, carried so a
    /// client can say *why* the source is missing from `eligible` rather than
    /// leaving a human to wonder. Already applied to `eligible`.
    pub exclude_self: bool,
    /// Battlefield permanents this player controls that `handle_activate_ability`'s
    /// own sacrifice gate will accept. **Never empty**: an empty set suppresses the
    /// whole `ActivateAbility` offer, the same SR-38 rule
    /// [`offerable_cast_plan`] applies one command over.
    pub eligible: Vec<ObjectId>,
    /// The deterministic default a bot submits: `eligible[0]` (lowest `ObjectId`,
    /// since `objects_in_zone` yields the engine's own order). Its own field rather
    /// than re-derived at each consumer, so the consumers cannot drift.
    /// `ObjectId::SENTINEL` when `eligible` is empty — never read in that case,
    /// because the whole offer is suppressed before this value reaches a consumer.
    pub default: ObjectId,
}

/// CR 602.2 / CR 111.10g (SIM-6): the offer's discard descriptor.
#[derive(Clone, Debug)]
pub struct ActivationDiscardOption {
    /// The cards in this player's hand. `handle_activate_ability` accepts any card
    /// in the activating player's hand (`abilities.rs`' discard-cost block checks
    /// the zone and nothing else), so this is that zone verbatim — there is no
    /// filter to mirror, which is why this option has no `filter` field.
    ///
    /// **Never empty**, for the same SR-38 reason as
    /// [`ActivationSacrificeOption::eligible`].
    pub eligible: Vec<ObjectId>,
    /// The deterministic default a bot submits: `eligible[0]`. Same contract as
    /// [`ActivationSacrificeOption::default`].
    pub default: ObjectId,
}

/// CR 702.157a (UI-2 §1.1): the offer's optional Squad descriptor.
#[derive(Clone, Debug)]
pub struct SquadCostOption {
    /// The per-copy cost from `AbilityDefinition::Squad { cost }`.
    pub cost: ManaCost,
    /// The largest N this player can currently afford on top of the spell's own cost.
    /// 0 means "offerable but not payable right now" -- the client must still be
    /// able to cast the spell, declining is always legal (CR 702.157a "any number of
    /// times", including zero).
    pub max_count: u32,
}

/// A legal action a player may take at this moment.
#[derive(Clone, Debug)]
pub enum LegalAction {
    PassPriority,
    Concede,
    PlayLand {
        card: ObjectId,
    },
    CastSpell {
        card: ObjectId,
        from_zone: ZoneId,
        /// CR 118.8 / CR 702.157 (UI-2): `AdditionalCostPlan::default()` -- both
        /// fields `None` -- for the overwhelming majority of spells.
        additional_costs: AdditionalCostPlan,
    },
    TapForMana {
        source: ObjectId,
        ability_index: usize,
        /// PB-EF12 (CR 605.3b/106.1b/111.10a): the colour to choose if the ability is
        /// `any_color: true`. `None` for a fixed-colour ability. When `Some`, always a
        /// concrete legal colour (never `Colorless`) — a bot must never suggest a colour
        /// the engine rejects (SR-38 precedent).
        chosen_color: Option<ManaColor>,
        /// PB-RS2 (CR 107.4e via CR 605.1a, SR-38 precedent): a fully-payable hybrid
        /// payment plan for the ability's cost, if it has one. Empty when the cost has
        /// no hybrid pips. When non-empty, always a plan the engine will accept — the
        /// provider only offers this action at all if some plan is fully payable
        /// (`resolve_hybrid_phyrexian_plan`).
        hybrid_choices: Vec<HybridManaPayment>,
        /// PB-RS2 (CR 107.4f via CR 605.1a, CR 104.3b): a fully-payable, non-suicidal
        /// Phyrexian payment plan for the ability's cost, if it has one. Empty when the
        /// cost has no Phyrexian pips.
        phyrexian_life_payments: Vec<bool>,
    },
    ActivateAbility {
        source: ObjectId,
        ability_index: usize,
        /// PB-RS2 (CR 107.4e via CR 602.2b, SR-38 precedent): mirrors
        /// `TapForMana::hybrid_choices`.
        hybrid_choices: Vec<HybridManaPayment>,
        /// PB-RS2 (CR 107.4f via CR 602.2b, CR 104.3b): mirrors
        /// `TapForMana::phyrexian_life_payments`.
        phyrexian_life_payments: Vec<bool>,
        /// CR 602.2 (SIM-6): the ability's non-mana, object-naming cost components
        /// — the sacrifice and the discard. `ActivationCostPlan::default()` (both
        /// fields `None`) for the overwhelming majority of abilities, exactly as
        /// `CastSpell::additional_costs` is for spells.
        activation_costs: ActivationCostPlan,
    },
    DeclareAttackers {
        eligible: Vec<ObjectId>,
        targets: Vec<AttackTarget>,
    },
    DeclareBlockers {
        eligible: Vec<ObjectId>,
        attackers: Vec<ObjectId>,
    },
    TakeMulligan,
    KeepHand,
    ReturnCommanderToCommandZone {
        object_id: ObjectId,
    },
    LeaveCommanderInZone {
        object_id: ObjectId,
    },
    /// CR 207.2c / CR 602: Bloodrush -- activated ability from hand targeting an attacking
    /// creature. Discards the card as cost, grants a P/T boost to the target until end of turn.
    ActivateBloodrush {
        /// The card with bloodrush in the player's hand.
        card: ObjectId,
        /// An attacking creature to target.
        target: ObjectId,
    },
    /// CR 702.171: Saddle a Mount by tapping creatures with total power >= N.
    /// Sorcery-speed only (active player, main phase, empty stack).
    SaddleMount {
        /// The Mount permanent to saddle.
        mount: ObjectId,
        /// Creatures to tap as the saddle cost (total power >= N).
        saddle_creatures: Vec<ObjectId>,
    },
    /// CR 702.140: Cast a card with Mutate using its mutate alternative cost,
    /// merging it with a target non-Human creature the caster owns.
    CastWithMutate {
        /// The card with Mutate in the player's hand.
        card: ObjectId,
        /// A non-Human creature the caster owns on the battlefield.
        mutate_target: ObjectId,
    },
    /// CR 702.37e / CR 702.168b / CR 701.40b / CR 701.58b: Turn a face-down permanent
    /// face up. This is a special action (no stack, no priority needed beyond having it).
    /// Valid at any time the player has priority.
    TurnFaceUp {
        /// The face-down permanent to turn face up.
        permanent: ObjectId,
        /// The method to use for paying the face-up cost.
        method: TurnFaceUpMethod,
        /// PB-DX6 (CR 107.4e via CR 701.40b/702.37e/702.168d, SR-38 precedent):
        /// mirrors `ActivateAbility::hybrid_choices` — a fully-payable hybrid
        /// payment plan for whichever cost `method` resolves to, if it has one.
        /// Empty when that cost has no hybrid pips. When non-empty, always a plan
        /// the engine will accept — the provider only offers this action at all if
        /// some plan is fully payable (`resolve_hybrid_phyrexian_plan`).
        hybrid_choices: Vec<HybridManaPayment>,
        /// PB-DX6 (CR 107.4f via CR 701.40b/702.37e/702.168d, CR 104.3b): mirrors
        /// `ActivateAbility::phyrexian_life_payments`.
        phyrexian_life_payments: Vec<bool>,
    },
    /// CR 606: Activate a loyalty ability on a planeswalker.
    /// Sorcery-speed, empty stack, once per permanent per turn.
    ActivateLoyaltyAbility {
        /// The planeswalker permanent.
        source: ObjectId,
        /// Which loyalty ability (filtered index).
        ability_index: usize,
    },
    /// CR 702.37a / CR 702.37b / CR 702.168b: Cast a card with Morph/Megamorph/Disguise
    /// face-down for {3} (or disguise cost) as a 2/2 creature with no name/text/subtypes.
    CastMorphFaceDown {
        /// The card in hand to cast face-down.
        card: ObjectId,
        /// The alt-cost kind to use (always AltCostKind::Morph).
        face_down_kind: FaceDownKind,
    },
    /// CR 702.30a (PB-DP4 / DP-11): answer an outstanding echo payment. `pay: false` is
    /// always offered (declining is always legal -- CR 118.12a); `pay: true` is offered
    /// only when `casting::can_pay_cost` says the engine will accept it, mirroring
    /// `handle_pay_echo`'s own check (SR-38 precedent: never offer a payment the engine
    /// rejects).
    PayEcho {
        permanent: ObjectId,
        pay: bool,
    },
    /// CR 702.24a (PB-DP4 / DP-11): answer an outstanding cumulative upkeep payment. The
    /// total is `per_counter_cost` x the permanent's age counters (CR 702.24b counts ALL
    /// age counters on the permanent, not per-ability). `pay: true` is gated on the mana
    /// pool for `CumulativeUpkeepCost::Mana` and on the life total for `::Life`
    /// (CR 119.4).
    PayCumulativeUpkeep {
        permanent: ObjectId,
        pay: bool,
    },
    /// CR 702.59a (PB-DP4 / DP-11): answer an outstanding recover payment. `pay: false`
    /// exiles the card; declining is always legal.
    PayRecover {
        recover_card: ObjectId,
        pay: bool,
    },
    /// CR 509.2 (M11-local S8, plan item 2): set the damage-assignment order for
    /// one attacker that is blocked by two or more creatures.
    ///
    /// **Never emitted by [`StubProvider`], and that is deliberate.** It is offered
    /// only to a *human*-occupied seat, appended by
    /// `LocalGame::human_only_actions` (`local_game.rs`). Two reasons, both load-bearing:
    ///
    /// 1. `Command::OrderBlockers` is **optional** — `combat.rs::apply_combat_damage`
    ///    falls back to `combat.blockers`' `OrdMap` order when no order was set — so a
    ///    bot that never issues it plays a legal game.
    /// 2. Adding an action to the provider's list shifts every `RandomBot` RNG draw
    ///    downstream of it, which would change what every recorded fuzz seed
    ///    reproduces (plan §8 R11). Keeping it out of the provider is what lets S8
    ///    claim the fuzzer is unperturbed and *check* it.
    ///
    /// `blockers` is the candidate set — every creature currently blocking
    /// `attacker`, in the engine's own `OrdMap` order, which is also the order the
    /// engine would use by default. `ActionParams::blocker_order` carries the
    /// human's chosen permutation; empty means "accept the default", and
    /// `params.rs` then submits `blockers` verbatim (a no-op that the engine
    /// accepts, per `handle_order_blockers`' completeness check).
    OrderBlockers {
        attacker: ObjectId,
        blockers: Vec<ObjectId>,
    },
    /// CR 514.1 / CR 701.9b (PB-DP7 / DP-3): answer the outstanding cleanup
    /// discard. `count` is how many must go and `hand` is the full candidate
    /// set, so a human client can render a real subset picker. `cards` is the
    /// deterministic default
    /// (`mtg_engine::rules::turn_actions::default_cleanup_discard`) -- exactly
    /// `count` distinct ids from `hand`, so a bot that submits it verbatim is
    /// always accepted (SR-38: never offer an action the engine rejects).
    DiscardToHandSize {
        count: u32,
        hand: Vec<ObjectId>,
        cards: Vec<ObjectId>,
    },
    /// CR 603.3d / CR 601.2c (PB-DP8 / DP-6): announce the targets of a triggered
    /// ability being put on the stack. `slots` is the full per-slot candidate set
    /// so a human client can render a real picker; `targets` is the deterministic
    /// default (`mtg_engine::rules::abilities::default_trigger_targets`), which
    /// the engine is guaranteed to accept (SR-38: never offer an action the engine
    /// rejects).
    ChooseTriggerTargets {
        choice_id: u64,
        source: ObjectId,
        slots: Vec<TriggerTargetOption>,
        targets: Vec<Vec<Target>>,
    },
    /// CR 608.2d (PB-DP9 / DP-7/8/9): answer an outstanding resolution-time
    /// choice — a library search, a scry or a surveil. `question` carries the
    /// full legal answer space so a human client can render a picker without a
    /// second query; `answer` is the engine's own deterministic default
    /// (`mtg_engine::effects::default_effect_choice_answer`), which the engine
    /// is guaranteed to accept (SR-38: never offer an action the engine
    /// rejects).
    ///
    /// Note the scry/surveil defaults are the IDENTITY (keep everything on
    /// top), not the pre-PB-DP9 bottom-everything/mill-everything behaviour —
    /// see the default helpers' docs. A bot that submits it verbatim therefore
    /// plays a scry as a no-op, which is un-strategic but neutral; seed
    /// OOS-DP9-1.
    AnswerEffectChoice {
        choice_id: u64,
        source: ObjectId,
        question: EffectChoiceQuestion,
        answer: EffectChoiceAnswer,
    },
}

/// Trait for enumerating legal actions from a game state.
///
/// **Obligation (PB-DP7 / PB-DP8):** a provider MUST offer an answer for every
/// `mtg_engine::rules::engine::BlockingDecision` variant. While one is
/// outstanding the engine's admission gate rejects every other command, so a
/// provider that offers nothing (or offers something else) converts a
/// recoverable state into a dead game -- `LocalGame`'s bot-command-rejected
/// fallback issues `PassPriority`, which the gate refuses, yielding
/// `Halted(EngineError)`. See OOS-DP7-12.
pub trait LegalActionProvider: Send + Sync {
    fn legal_actions(&self, state: &GameState, player: PlayerId) -> Vec<LegalAction>;
}

/// Basic legal action enumeration — enough to play games, but misses
/// edge cases (flashback, escape, foretell, activated abilities on
/// permanents, etc.) that the full engine implementation will catch.
///
/// **B6 update (2026-03-05)**: Batch 6 alt-cost keywords (Bargain, Emerge, Spectacle,
/// Surge, Casualty, Assist) are fully implemented in the engine. The random_bot passes
/// `alt_cost: None` and `bargain_sacrifice/emerge_sacrifice/etc: None` — it never
/// attempts alt-cost casts. Full behavioral support (bot deciding to use alt costs based
/// on game state) is a W2 TUI task; see `docs/workstream-coordination.md` Phase 2.
///
/// **B10 update (2026-03-07)**: Batch 10 ETB/dies pattern keywords (Devour, Backup,
/// Champion, Umbra Armor, Living Metal, Soulbond, Fortify) are fully implemented in the
/// engine. StubProvider handles these automatically via engine resolution — no LegalAction
/// changes needed. Soulbond pairing choice and Champion target choice are auto-selected
/// by the engine during trigger resolution. Fortify activation is handled by
/// `LegalAction::ActivateAbility` (already emitted). No bot behavioral changes needed.
///
/// **B12–B14 + Mutate update (2026-03-08)**:
/// - Bloodrush (B12): `LegalAction::ActivateBloodrush` emitted when a hand card has
///   `AbilityDefinition::Bloodrush` and there is at least one attacking creature.
/// - Saddle (B13): `LegalAction::SaddleMount` emitted when a Mount is on the battlefield
///   and the player controls untapped creatures with total power >= the Saddle N value.
///   Sorcery-speed only. StubProvider picks the first valid greedy set.
/// - Mutate: `LegalAction::CastWithMutate` emitted when a card in hand has
///   `KeywordAbility::Mutate` (and `AbilityDefinition::MutateCost`) and the player
///   owns a non-Human creature on the battlefield. Mutate is an alternative cost —
///   random_bot casts with `alt_cost: Some(AltCostKind::Mutate)` and `mutate_on_top: true`.
/// - Enrage/Alliance (B12), Collect Evidence (B13), Blood tokens/Reconfigure (B14):
///   all passive or handled via existing `ActivateAbility`/`CastSpell` paths — no new
///   `LegalAction` variants needed.
///
/// **PB-22 S7 (2026-03-21) — Adventure gap (deferred to W2)**:
/// TODO(W2): Adventure casting paths are not offered to the bot. Two gaps:
///   (a) `CastAsAdventure { card: ObjectId }` — cast a card in hand as its Adventure half
///       (CR 715.3); requires checking `adventure_face.is_some()` on CardDefinition and
///       comparing mana against the adventure_face cost. The engine supports
///       `alt_cost: Some(AltCostKind::Adventure)` on CastSpell; bot never sets it.
///   (b) `CastFromAdventureExile { card: ObjectId }` — cast a creature from adventure exile
///       (CR 715.3d); requires checking `adventure_exiled_by == Some(player)` on GameObject.
///   Both gaps are consistent with other alt-cost keywords (Spectacle, Surge, etc.) where
///   the bot always uses `alt_cost: None`. Deferred to W2 TUI/simulator improvements.
pub struct StubProvider;

impl LegalActionProvider for StubProvider {
    fn legal_actions(&self, state: &GameState, player: PlayerId) -> Vec<LegalAction> {
        let mut actions = Vec::new();

        // PB-DP7 / DP-3 (CR 514.1): answer the outstanding cleanup discard
        // first -- nothing else is legal while it is pending (CR 514.3: no
        // player has priority in cleanup). Must be checked BEFORE the
        // commander-zone block below: the engine's admission gate
        // (`rules::engine::process_command`) rejects
        // `ReturnCommanderToCommandZone` while blocked, so offering it first
        // would offer a command the engine refuses. Also must be checked
        // before the `priority_holder != Some(player)` early return further
        // down, which would otherwise return an empty list for the blocked
        // player too (nobody holds priority during cleanup).
        // Fix-cycle Finding 4 (MEDIUM): read the liveness-filtered predicate,
        // not the raw `pending_cleanup_discard()` field -- a dead active
        // player's stale entry must not make this provider offer (or gate on)
        // an action for a player who can never answer it.
        if let Some(decision) = state.blocking_decision() {
            match decision {
                mtg_engine::rules::engine::BlockingDecision::CleanupDiscard {
                    player: entry_player,
                    count,
                } => {
                    if entry_player == player {
                        let cards =
                            mtg_engine::rules::turn_actions::default_cleanup_discard(state, player);
                        let hand: Vec<ObjectId> = state
                            .zones()
                            .get(&ZoneId::Hand(player))
                            .map(|z| z.object_ids())
                            .unwrap_or_default();
                        actions.push(LegalAction::DiscardToHandSize { count, hand, cards });
                    }
                }
                // PB-DP8 / DP-6 (CR 603.3d): announce the trigger's targets. The
                // raw accessor is correct here -- `blocking_decision()` already
                // applied the liveness filter to decide we are blocked at all.
                mtg_engine::rules::engine::BlockingDecision::TriggerTargets {
                    player: entry_player,
                    choice_id,
                    source,
                } => {
                    if entry_player == player {
                        if let Some(entry) = state.pending_trigger_targets() {
                            let slots: Vec<TriggerTargetOption> =
                                entry.slots.iter().cloned().collect();
                            let targets =
                                mtg_engine::rules::abilities::default_trigger_targets(&slots);
                            actions.push(LegalAction::ChooseTriggerTargets {
                                choice_id,
                                source,
                                slots,
                                targets,
                            });
                        }
                    }
                }
                // PB-DP9 / DP-7/8/9 (CR 608.2d): answer the resolution-time
                // choice. The raw accessor is correct here for the same reason
                // as above -- `blocking_decision()` already applied the liveness
                // filter to decide we are blocked at all.
                mtg_engine::rules::engine::BlockingDecision::EffectChoice {
                    player: entry_player,
                    choice_id,
                    source,
                } => {
                    if entry_player == player {
                        if let Some(entry) = state.pending_effect_choice() {
                            let question = entry.question.clone();
                            let answer =
                                mtg_engine::effects::default_effect_choice_answer(&question);
                            actions.push(LegalAction::AnswerEffectChoice {
                                choice_id,
                                source,
                                question,
                                answer,
                            });
                        }
                    }
                }
            }
            // Every other player (and the entry's own player, once the action
            // above is pushed) gets exactly this and nothing else.
            return actions;
        }

        // Handle pending commander zone choices first
        if let Some((_pending_player, obj_id)) = state
            .pending_commander_zone_choices()
            .iter()
            .find(|(p, _)| *p == player)
        {
            actions.push(LegalAction::ReturnCommanderToCommandZone { object_id: *obj_id });
            actions.push(LegalAction::LeaveCommanderInZone { object_id: *obj_id });
            return actions;
        }

        // Mulligan phase
        if state.turn().is_first_turn_of_game && state.turn().turn_number == 0 {
            actions.push(LegalAction::TakeMulligan);
            actions.push(LegalAction::KeepHand);
            return actions;
        }

        // Check if this player has priority
        if state.turn().priority_holder != Some(player) {
            return actions;
        }

        // Always available: pass priority
        // (Concede is intentionally omitted — bots should never auto-concede.
        // The human player can still quit via 'q'.)
        actions.push(LegalAction::PassPriority);

        // SG-1 (SR-38, CR 118.3 / CR 119.4): the activating player's life total, used to
        // gate life-cost activations below. SR-34 gave `ManaAbility` a `life_cost` (horizon
        // lands, Mana Confluence) and SR-36 gave `ActivationCost` a `life_cost` (fetchlands,
        // Doom Whisperer) — and made both real, so `handle_tap_for_mana` / `handle_activate_ability`
        // now reject an unpayable life cost with `GameStateError::InsufficientLife`. Before this
        // the provider would offer a bot at low life an activation the engine rejects (or, worse,
        // a "lethal" fetch that would put it below 0). We mirror the engine's check here so the
        // bot's legal-action list stays a subset of what the engine will accept. CR 119.4b makes
        // a cost of 0 always payable, so every check short-circuits on `life_cost > 0`.
        let life_total = state.player(player).map(|p| p.life_total).unwrap_or(0);

        // PB-DP4 / DP-11: an outstanding pay-or-lose-it payment. Offered as ordinary
        // priority-window actions rather than a separate blocking decision, because the
        // engine's deadline is the end of this priority round
        // (rules/engine.rs::force_resolve_overdue_payments) and because CR 608.2g lets the
        // player activate mana abilities first -- so TapForMana must stay available
        // alongside these. Appending (not early-returning) is deliberate: the commander-zone
        // and mulligan blocks above early-return because those decisions genuinely exclude
        // everything else, but a payment must not -- the engine's payment path reads only
        // the pool (it never auto-taps), so early-returning would make `pay: true` reachable
        // only when the pool happens to already be funded.
        //
        // Not answering is a legal decline (CR 118.12a); the engine applies it at the
        // boundary. `pay: false` is always offered; `pay: true` is gated on affordability
        // (SR-38: never offer a payment the engine will reject).
        let pool = state
            .player(player)
            .map(|p| p.mana_pool.clone())
            .unwrap_or_default();
        for (owing, permanent, cost) in state.pending_echo_payments().iter() {
            if *owing != player {
                continue;
            }
            actions.push(LegalAction::PayEcho {
                permanent: *permanent,
                pay: false,
            });
            if mtg_engine::rules::casting::can_pay_cost(&pool, cost) {
                actions.push(LegalAction::PayEcho {
                    permanent: *permanent,
                    pay: true,
                });
            }
        }
        for (owing, permanent, per_counter_cost) in
            state.pending_cumulative_upkeep_payments().iter()
        {
            if *owing != player {
                continue;
            }
            actions.push(LegalAction::PayCumulativeUpkeep {
                permanent: *permanent,
                pay: false,
            });
            // CR 702.24b: the total is per_counter_cost x ALL age counters currently on
            // the permanent (not per-ability).
            let age_count = state
                .object(*permanent)
                .ok()
                .and_then(|obj| obj.counters.get(&CounterType::Age).copied())
                .unwrap_or(0);
            let affordable = match per_counter_cost {
                mtg_engine::CumulativeUpkeepCost::Mana(mc) => {
                    mtg_engine::rules::casting::can_pay_cost(
                        &pool,
                        &multiply_mana_cost(mc, age_count),
                    )
                }
                // CR 119.4 / 119.4b: mirrors engine.rs Change 2e's affordability gate.
                // Fix cycle (T7): short-circuit on a total of 0 BEFORE comparing against
                // life_total, matching engine.rs's `if total_life > 0 { check }` guard
                // exactly. Without this, a Life(0) cost at a negative life_total (itself
                // reachable -- nothing clamps life_total at 0) was withheld here even
                // though the engine always accepts it -- a real divergence from the
                // engine this provider claims to mirror (SR-38).
                mtg_engine::CumulativeUpkeepCost::Life(amount) => {
                    let total = amount * age_count;
                    total == 0 || life_total >= total as i32
                }
            };
            if affordable {
                actions.push(LegalAction::PayCumulativeUpkeep {
                    permanent: *permanent,
                    pay: true,
                });
            }
        }
        for (owing, recover_card, cost) in state.pending_recover_payments().iter() {
            if *owing != player {
                continue;
            }
            actions.push(LegalAction::PayRecover {
                recover_card: *recover_card,
                pay: false,
            });
            if mtg_engine::rules::casting::can_pay_cost(&pool, cost) {
                actions.push(LegalAction::PayRecover {
                    recover_card: *recover_card,
                    pay: true,
                });
            }
        }

        let is_main_phase = matches!(
            state.turn().step,
            Step::PreCombatMain | Step::PostCombatMain
        );
        let stack_empty = state.stack_objects().is_empty();
        let is_active = state.turn().active_player == player;

        // Play lands: hand lands, main phase, stack empty, active player,
        // land plays remaining
        if is_main_phase && stack_empty && is_active {
            if let Ok(p) = state.player(player) {
                if p.land_plays_remaining > 0 {
                    let hand = ZoneId::Hand(player);
                    for obj in state.objects_in_zone(&hand) {
                        if obj.characteristics.card_types.contains(&CardType::Land) {
                            actions.push(LegalAction::PlayLand { card: obj.id });
                        }
                    }
                }
            }
        }

        // PB-18: Pre-compute restriction flags for this player.
        let cast_restricted = is_cast_restricted_by_stax(state, player);

        // Cast spells from hand
        if !cast_restricted {
            let hand = ZoneId::Hand(player);
            for obj in state.objects_in_zone(&hand) {
                let is_land = obj.characteristics.card_types.contains(&CardType::Land);
                if is_land {
                    continue;
                }

                // SIM-1 Step 3: timing predicate extracted to `can_cast_at_this_time`
                // so the hand loop and the new command-zone loop (below) cannot drift
                // out of sync on CR 117.1a / CR 601.3b timing.
                if can_cast_at_this_time(state, player, obj, is_main_phase, stack_empty, is_active)
                {
                    // Basic mana affordability check
                    if let Some(ref cost) = obj.characteristics.mana_cost {
                        if can_afford(state, player, cost) {
                            // UI-2 (CR 118.8 / CR 702.157, SR-38 criterion 5999): see
                            // `offerable_cast_plan`. The command-zone loop below calls
                            // the SAME helper -- the two used to carry byte-identical
                            // inline copies under a comment saying they must not
                            // diverge, which is a rule stated rather than enforced
                            // (review Issue 4).
                            if let Some(additional_costs) = offerable_cast_plan(state, player, obj)
                            {
                                actions.push(LegalAction::CastSpell {
                                    card: obj.id,
                                    from_zone: hand,
                                    additional_costs,
                                });
                            }
                        }
                    }
                }
            }
        }

        // SIM-1 (CR 903.8 / CR 601.2a, playtest triage F7): a player may cast a
        // commander they OWN from the command zone. The engine has always supported
        // this (`casting.rs` derives command-zone-ness from the object's zone, admits
        // the "not in your hand" gate, gates CR 903.8 on `CardId`, applies the tax, and
        // increments the tax counter on cast) -- the provider simply never looked in
        // the zone, so a human clicking their commander in the browser was told the
        // server offered nothing.
        //
        // Three filters, each mirroring an engine gate rather than a preference:
        //   * `ZoneId::Command(player)` only -- never another seat's zone
        //     (`casting.rs`'s `casting_from_command_zone` derivation).
        //   * `commander_ids` (CR 903.8) -- CR 408.1 makes the command zone a home for
        //     other objects too (emblems), and CR 903.9a/b can move things through it.
        //     The zone is NOT the filter; `commander_ids` is.
        //   * CR 101.2 non-hand cast restriction (`is_cast_from_nonhand_restricted`) --
        //     newly reachable now that this provider offers a non-hand cast at all; see
        //     that function's doc.
        //
        // Timing is MIRRORED, not assumed: a commander is a permanent so it is normally
        // sorcery speed (CR 117.1a), but the engine's own gate is zone-agnostic, so a
        // commander with Flash or under a CR 601.3b flash grant is legal at instant
        // speed and must be offered -- `can_cast_at_this_time` is the same predicate
        // the hand loop above uses, so the two cannot diverge.
        //
        // Placed immediately after the hand loop it mirrors, so the two stay readable
        // as a pair. **This is NOT an "append", and the difference matters**: the
        // tap-for-mana, declare-attackers and declare-blockers blocks all run below,
        // so whenever this loop pushes, every one of their indices shifts by one — and
        // `RandomBot` chooses by index into this list.
        //
        // So the reason no recorded `mtg-fuzzer` seed moves is **not** placement. It is
        // that the offer is gated on `commander_ids`, and `fuzzer.rs` builds its
        // command-zone object without ever calling `builder.player_commander(..)` —
        // so `commander_ids` is empty in every fuzzer game and this loop cannot fire
        // there at all. That is structural unreachability, filed as `OOS-SIM1-4`, and
        // it is a stronger guarantee than index arithmetic would have been. Verified
        // by A/B against the merge-base: 60 games, per-game results byte-identical.
        //
        // A future session that closes `OOS-SIM1-4` (teaching the fuzzer to register
        // commanders) re-rolls every recorded seed. That is the cost named in the seed,
        // and it is the *registration* that causes it — moving this block would not
        // help.
        if !cast_restricted && !is_cast_from_nonhand_restricted(state, player) {
            let command_zone = ZoneId::Command(player);
            for obj in state.objects_in_zone(&command_zone) {
                // CR 117.1a: a land is played, not cast. No commander is a land today;
                // the skip mirrors the hand loop so the two cannot diverge if one ever
                // is.
                if obj.characteristics.card_types.contains(&CardType::Land) {
                    continue;
                }
                // CR 903.8, keyed on CardId -- NOT ObjectId.
                let Some(cid) = obj.card_id.as_ref() else {
                    continue;
                };
                let is_commander = state
                    .player(player)
                    .map(|ps| ps.commander_ids.contains(cid))
                    .unwrap_or(false);
                if !is_commander {
                    continue;
                }
                if !can_cast_at_this_time(state, player, obj, is_main_phase, stack_empty, is_active)
                {
                    continue;
                }
                // CR 903.8 / 601.2f: the tax is a cost INCREASE folded into the total
                // cost, so the affordability gate must see it or the offer is a
                // guaranteed rejection (SR-38).
                let Some(cost) = effective_cast_cost(state, player, obj.id) else {
                    continue;
                };
                if can_afford(state, player, &cost) {
                    // UI-2: the SAME helper the hand loop above uses, so the two
                    // cannot diverge on when a required sacrifice suppresses the
                    // offer. A commander with a CR 118.8 additional cost is a shape
                    // no card in the corpus has today, which is exactly why this
                    // must be shared code rather than a second copy nobody tests.
                    if let Some(additional_costs) = offerable_cast_plan(state, player, obj) {
                        actions.push(LegalAction::CastSpell {
                            card: obj.id,
                            from_zone: command_zone,
                            additional_costs,
                        });
                    }
                }
            }
        }

        // Tap for mana: untapped permanents with mana abilities on battlefield
        // CR 613.1f: Use layer-resolved characteristics so granted mana abilities
        // (Cryptolith Rite, Chromatic Lantern) and removals (Humility) are visible.
        for obj in state.objects_in_zone(&ZoneId::Battlefield) {
            if obj.controller != player {
                continue;
            }
            if obj.status.tapped {
                continue;
            }
            let chars = mtg_engine::rules::layers::calculate_characteristics(state, obj.id)
                .unwrap_or_else(|| obj.characteristics.clone());
            for (idx, ability) in chars.mana_abilities.iter().enumerate() {
                if ability.requires_tap {
                    // SG-1 (CR 118.3 / CR 119.4b) + **OOS-CARDS2-9** (SIM-2): a mana
                    // ability whose activation `handle_tap_for_mana` would refuse for a
                    // reason knowable from the state alone must not be offered (SR-38).
                    // SG-1 covered the life component; the shared predicate covers it plus
                    // the two `tools/play-server`'s driver had been absorbing in its
                    // `KNOWN_FALSE_OFFERS` list for a batch — an unmet
                    // `activation_condition` (CR 602.5b) and a summoning-sick creature
                    // (CR 302.6) — a counter cost with too few counters (CR 118.3), and
                    // (added by SIM-2's own `/review`, which found it mirrored nowhere on
                    // this path) the CR 605.3 stax restrictions of `rules/mana.rs` step 1b.
                    // It does NOT cover what needs the activation performed to decide; see
                    // `mana_solver::plannable_tap_ability`'s doc for that bound.
                    //
                    // The SAME predicate the mana solver uses, so the offer list and the
                    // payment plan cannot drift: that identity is the fix, not the
                    // individual checks.
                    if !crate::mana_solver::tap_ability_is_activatable(
                        state, player, obj, &chars, ability,
                    ) {
                        continue;
                    }
                    // PB-EF12: an any_color ability requires a chosen_color on the
                    // activation Command (CR 605.3b — the choice is made at
                    // activation, never deferred). Deterministic WUBRG order, first
                    // legal = White, mirroring `mana_solver.rs`'s pick.
                    let chosen_color = if ability.any_color {
                        Some(ManaColor::White)
                    } else {
                        None
                    };
                    // PB-RS2 (CR 107.4e/107.4f via CR 605.1a, SR-38): if the ability's
                    // OWN mana_cost component (e.g. a filter land's {B/R}) has a hybrid
                    // or Phyrexian pip, only offer this action if a fully-payable,
                    // non-suicidal plan exists — the raw `ability.life_cost` check
                    // above only covers the ability's non-Phyrexian life component.
                    let has_pip_cost = ability
                        .mana_cost
                        .as_ref()
                        .is_some_and(|mc| !mc.hybrid.is_empty() || !mc.phyrexian.is_empty());
                    let (hybrid_choices, phyrexian_life_payments) = if has_pip_cost {
                        // Review finding #4: `let Some(...) else { continue }` instead of
                        // `.expect()` — `has_pip_cost` being true implies `mana_cost` is
                        // `Some` by construction, but this avoids asserting that at
                        // runtime when a `continue` is already the natural escape hatch
                        // in this loop.
                        let Some(mc) = ability.mana_cost.as_ref() else {
                            continue;
                        };
                        match resolve_hybrid_phyrexian_plan(state, player, mc, ability.life_cost) {
                            Some(plan) => plan,
                            None => continue,
                        }
                    } else {
                        (vec![], vec![])
                    };
                    actions.push(LegalAction::TapForMana {
                        source: obj.id,
                        ability_index: idx,
                        chosen_color,
                        hybrid_choices,
                        phyrexian_life_payments,
                    });
                }
            }
        }

        // Declare attackers: untapped creatures without summoning sickness
        // (unless haste) during DeclareAttackers step when active player
        if state.turn().step == Step::DeclareAttackers && is_active && stack_empty {
            let mut eligible = Vec::new();
            let mut targets = Vec::new();

            for obj in state.objects_in_zone(&ZoneId::Battlefield) {
                if obj.controller != player {
                    continue;
                }
                if !obj.characteristics.card_types.contains(&CardType::Creature) {
                    continue;
                }
                if obj.status.tapped {
                    continue;
                }
                if obj
                    .characteristics
                    .keywords
                    .contains(&KeywordAbility::Defender)
                {
                    continue;
                }
                if obj.has_summoning_sickness
                    && !obj
                        .characteristics
                        .keywords
                        .contains(&KeywordAbility::Haste)
                {
                    continue;
                }
                eligible.push(obj.id);
            }

            // Valid attack targets: opponents
            for p in state.active_players() {
                if p != player {
                    targets.push(AttackTarget::Player(p));
                }
            }

            if !eligible.is_empty() && !targets.is_empty() {
                actions.push(LegalAction::DeclareAttackers { eligible, targets });
            }
        }

        // Declare blockers: untapped creatures during DeclareBlockers step
        if state.turn().step == Step::DeclareBlockers && stack_empty {
            if let Some(ref combat) = state.combat() {
                if !combat.attackers.is_empty() {
                    let mut eligible = Vec::new();
                    let mut attacker_ids: Vec<ObjectId> = Vec::new();

                    // Defending player(s) can block
                    for obj in state.objects_in_zone(&ZoneId::Battlefield) {
                        if obj.controller != player {
                            continue;
                        }
                        if !obj.characteristics.card_types.contains(&CardType::Creature) {
                            continue;
                        }
                        if obj.status.tapped {
                            continue;
                        }
                        eligible.push(obj.id);
                    }

                    for (attacker_id, _) in &combat.attackers {
                        attacker_ids.push(*attacker_id);
                    }

                    if !eligible.is_empty() && !attacker_ids.is_empty() {
                        actions.push(LegalAction::DeclareBlockers {
                            eligible,
                            attackers: attacker_ids,
                        });
                    }
                }
            }
        }

        // Activate non-mana abilities on battlefield permanents
        // CR 613.1f: Use layer-resolved characteristics so granted activated abilities
        // and removals (Humility) are visible. W3-LC audit fix.
        for obj in state.objects_in_zone(&ZoneId::Battlefield) {
            if obj.controller != player {
                continue;
            }
            let act_chars = mtg_engine::rules::layers::calculate_characteristics(state, obj.id)
                .unwrap_or_else(|| obj.characteristics.clone());
            for (idx, ability) in act_chars.activated_abilities.iter().enumerate() {
                // Check tap requirement
                if ability.cost.requires_tap && obj.status.tapped {
                    continue;
                }
                // Sorcery-speed abilities
                if ability.sorcery_speed && !(is_main_phase && stack_empty && is_active) {
                    continue;
                }
                // PB-RS2 (CR 107.4e/107.4f via CR 602.2b, SR-38): if the ability's cost
                // has a hybrid or Phyrexian pip, the raw-cost `can_afford` check below
                // is WRONG in the offering direction — a pure hybrid pip's `white`/
                // `blue`/etc. fields are all 0, so an unrelated pool total can pass it
                // while the engine (which flattens first) correctly rejects the
                // activation. Route through the same plan-resolution the mana-ability
                // site uses instead, combining the life check with `ability.cost.life_cost`.
                let has_pip_cost = ability
                    .cost
                    .mana_cost
                    .as_ref()
                    .is_some_and(|mc| !mc.hybrid.is_empty() || !mc.phyrexian.is_empty());
                let (hybrid_choices, phyrexian_life_payments) = if has_pip_cost {
                    // Review finding #4: `let Some(...) else { continue }` instead of
                    // `.expect()` — see the sibling site above for the reasoning.
                    let Some(mc) = ability.cost.mana_cost.as_ref() else {
                        continue;
                    };
                    match resolve_hybrid_phyrexian_plan(state, player, mc, ability.cost.life_cost) {
                        Some(plan) => plan,
                        None => continue,
                    }
                } else {
                    // Basic mana check (no hybrid/Phyrexian pips — the raw cost's
                    // standard fields already fully describe what must be paid).
                    if let Some(ref cost) = ability.cost.mana_cost {
                        if !can_afford(state, player, cost) {
                            continue;
                        }
                    }
                    // SG-1 (CR 118.3 / CR 119.4b): a non-mana activated ability with a
                    // life component the player cannot pay (fetchlands' "Pay 1 life",
                    // Doom Whisperer's "Pay 2 life") is rejected by
                    // `handle_activate_ability` (rules/abilities.rs) — don't offer it.
                    // Mirrors the mana-ability check above. Skipped when `has_pip_cost`
                    // because `resolve_hybrid_phyrexian_plan` already combined this with
                    // the Phyrexian-life check (CR 119.4/602.2b — the components may be
                    // paid in any order, so the check must be on the combined total).
                    if ability.cost.life_cost > 0 && life_total < ability.cost.life_cost as i32 {
                        continue;
                    }
                    (vec![], vec![])
                };
                // PB-18 review Finding 4: filter abilities blocked by active restrictions.
                // Mirrors check_activate_restrictions in rules/abilities.rs.
                if is_ability_restricted_by_stax(state, player, obj.id) {
                    continue;
                }
                // CR 602.2 / SR-38 (SIM-6, triage G4): the non-mana components that
                // require NAMING an object -- the sacrifice and the discard. `None`
                // suppresses the offer entirely, exactly as `offerable_cast_plan`
                // does one command over: an activation whose required sacrifice has
                // nothing eligible to name is one `handle_activate_ability` refuses,
                // and offering it was the whole G4 defect.
                let Some(activation_costs) =
                    offerable_activation_plan(state, player, obj.id, ability)
                else {
                    continue;
                };
                actions.push(LegalAction::ActivateAbility {
                    source: obj.id,
                    ability_index: idx,
                    hybrid_choices,
                    phyrexian_life_payments,
                    activation_costs,
                });
            }
        }

        // ── Loyalty abilities (CR 606) ────────────────────────────────────────────
        // Sorcery-speed, stack empty, once per permanent per turn.
        if is_main_phase && stack_empty && is_active {
            for obj in state.objects_in_zone(&ZoneId::Battlefield) {
                if obj.controller != player {
                    continue;
                }
                if obj.loyalty_ability_activated_this_turn {
                    continue;
                }
                if !obj
                    .characteristics
                    .card_types
                    .contains(&CardType::Planeswalker)
                {
                    continue;
                }
                // Look up card definition for loyalty abilities.
                if let Some(ref cid) = obj.card_id {
                    if let Some(def) = state.card_registry().get(cid.clone()) {
                        let loyalty_count = obj
                            .counters
                            .get(&CounterType::Loyalty)
                            .copied()
                            .unwrap_or(0);
                        for (idx, ability) in def.abilities.iter().enumerate() {
                            if let mtg_engine::AbilityDefinition::LoyaltyAbility { cost, .. } =
                                ability
                            {
                                // CR 606.6: check sufficient loyalty for negative costs.
                                let can_afford_loyalty = match cost {
                                    mtg_engine::LoyaltyCost::Plus(_)
                                    | mtg_engine::LoyaltyCost::Zero => true,
                                    mtg_engine::LoyaltyCost::Minus(n) => loyalty_count >= *n,
                                    mtg_engine::LoyaltyCost::MinusX => true, // X can be 0
                                };
                                if can_afford_loyalty {
                                    // Use filtered index: count LoyaltyAbility entries up to idx.
                                    let filtered_idx = def.abilities[..=idx]
                                        .iter()
                                        .filter(|a| {
                                            matches!(
                                                a,
                                                mtg_engine::AbilityDefinition::LoyaltyAbility { .. }
                                            )
                                        })
                                        .count()
                                        - 1;
                                    actions.push(LegalAction::ActivateLoyaltyAbility {
                                        source: obj.id,
                                        ability_index: filtered_idx,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── Bloodrush (CR 207.2c / B12) ─────────────────────────────────────────
        // Bloodrush is an activated ability from hand: discard the card to grant
        // a P/T boost to an attacking creature. Legal any time the player has
        // priority and an attacking creature exists (instant speed, no stack restriction).
        {
            // Collect attacking creature IDs once.
            let attacking: Vec<ObjectId> = if let Some(ref combat) = state.combat() {
                combat.attackers.keys().copied().collect()
            } else {
                Vec::new()
            };

            if !attacking.is_empty() {
                let hand = ZoneId::Hand(player);
                for obj in state.objects_in_zone(&hand) {
                    // Check card_id is set and card definition has AbilityDefinition::Bloodrush.
                    let has_bloodrush = obj
                        .card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry().get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .any(|a| matches!(a, AbilityDefinition::Bloodrush { .. }))
                        })
                        .unwrap_or(false);

                    if !has_bloodrush {
                        continue;
                    }

                    // Check mana affordability for the bloodrush cost.
                    let bloodrush_cost = obj
                        .card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry().get(cid.clone()))
                        .and_then(|def| {
                            def.abilities.iter().find_map(|a| {
                                if let AbilityDefinition::Bloodrush { cost, .. } = a {
                                    Some(cost.clone())
                                } else {
                                    None
                                }
                            })
                        });

                    if let Some(cost) = bloodrush_cost {
                        if can_afford(state, player, &cost) {
                            // Emit one action per attacking creature target.
                            for &target in &attacking {
                                actions.push(LegalAction::ActivateBloodrush {
                                    card: obj.id,
                                    target,
                                });
                            }
                        }
                    }
                }
            }
        }

        // ── Saddle (CR 702.171 / B13) ────────────────────────────────────────────
        // Sorcery-speed (active player, main phase, empty stack). The player taps
        // untapped creatures they control (excluding the Mount itself) with total
        // power >= N to saddle the Mount.
        if is_main_phase && stack_empty && is_active {
            // Collect untapped creatures the player controls (potential saddle creatures).
            let untapped_creatures: Vec<(ObjectId, i32)> = state
                .objects_in_zone(&ZoneId::Battlefield)
                .into_iter()
                .filter(|o| {
                    o.controller == player
                        && !o.status.tapped
                        && o.characteristics.card_types.contains(&CardType::Creature)
                })
                .map(|o| (o.id, o.characteristics.power.unwrap_or(0)))
                .collect();

            // Find Mounts with Saddle(N).
            for obj in state.objects_in_zone(&ZoneId::Battlefield) {
                if obj.controller != player {
                    continue;
                }
                let saddle_n = obj.characteristics.keywords.iter().find_map(|kw| {
                    if let KeywordAbility::Saddle(n) = kw {
                        Some(*n)
                    } else {
                        None
                    }
                });
                let saddle_n = match saddle_n {
                    Some(n) => n,
                    None => continue,
                };

                // Greedy selection: pick untapped creatures (excluding the Mount itself)
                // until we meet or exceed the power threshold.
                let mut chosen: Vec<ObjectId> = Vec::new();
                let mut total_power: i32 = 0;
                for &(cid, power) in &untapped_creatures {
                    if cid == obj.id {
                        continue; // Can't use the Mount itself as a saddle creature.
                    }
                    chosen.push(cid);
                    total_power += power;
                    if total_power >= saddle_n as i32 {
                        break;
                    }
                }

                if total_power >= saddle_n as i32 {
                    actions.push(LegalAction::SaddleMount {
                        mount: obj.id,
                        saddle_creatures: chosen,
                    });
                }
            }
        }

        // ── Mutate (CR 702.140) ──────────────────────────────────────────────────
        // Mutate is an alternative cost. Cards with Mutate in hand may be cast merging
        // with a non-Human creature the caster OWNS on the battlefield (CR 702.140a).
        // Timing follows normal spell timing (instant if the card is an instant / has flash;
        // otherwise sorcery-speed). StubProvider conservatively emits at sorcery-speed only
        // for creature spells (the common case — almost all mutate cards are creatures).
        if is_main_phase && stack_empty && is_active {
            // Collect non-Human creatures the player OWNS on the battlefield.
            let non_human_own: Vec<ObjectId> = state
                .objects_in_zone(&ZoneId::Battlefield)
                .into_iter()
                .filter(|o| {
                    // Owner check (not controller — CR 702.140a says "you own").
                    o.owner == player
                        && o.characteristics.card_types.contains(&CardType::Creature)
                        && !o
                            .characteristics
                            .subtypes
                            .contains(&mtg_engine::SubType("Human".to_string()))
                })
                .map(|o| o.id)
                .collect();

            if !non_human_own.is_empty() {
                let hand = ZoneId::Hand(player);
                for obj in state.objects_in_zone(&hand) {
                    if !obj
                        .characteristics
                        .keywords
                        .contains(&KeywordAbility::Mutate)
                    {
                        continue;
                    }

                    // Look up the mutate cost from the card registry.
                    let mutate_cost = obj
                        .card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry().get(cid.clone()))
                        .and_then(|def| {
                            def.abilities.iter().find_map(|a| {
                                if let AbilityDefinition::MutateCost { cost } = a {
                                    Some(cost.clone())
                                } else {
                                    None
                                }
                            })
                        });

                    let mutate_cost = match mutate_cost {
                        Some(c) => c,
                        None => continue, // No MutateCost defined — skip.
                    };

                    if !can_afford(state, player, &mutate_cost) {
                        continue;
                    }

                    // Emit one action per valid mutate target.
                    for &target in &non_human_own {
                        actions.push(LegalAction::CastWithMutate {
                            card: obj.id,
                            mutate_target: target,
                        });
                    }
                }
            }
        }

        // ── TurnFaceUp (CR 702.37e) ──────────────────────────────────────────────
        // Special action: turn a face-down permanent face up at any time the player
        // has priority (no sorcery restriction — CR 116.2b). The player must control
        // the permanent and be able to pay the turn-face-up cost.
        for obj in state.objects_in_zone(&ZoneId::Battlefield) {
            if obj.controller != player {
                continue;
            }
            if !obj.status.face_down {
                continue;
            }
            let face_down_kind = match &obj.face_down_as {
                Some(k) => k.clone(),
                None => continue,
            };

            let card_def = obj
                .card_id
                .as_ref()
                .and_then(|cid| state.card_registry().get(cid.clone()));

            match face_down_kind {
                FaceDownKind::Morph | FaceDownKind::Megamorph => {
                    // Check for Morph or Megamorph ability in the card definition.
                    if let Some(def) = &card_def {
                        for ability in &def.abilities {
                            match ability {
                                AbilityDefinition::Morph { cost } => {
                                    if let Some((hybrid_choices, phyrexian_life_payments)) =
                                        turn_face_up_payment_plan(state, player, cost)
                                    {
                                        actions.push(LegalAction::TurnFaceUp {
                                            permanent: obj.id,
                                            method: TurnFaceUpMethod::MorphCost,
                                            hybrid_choices,
                                            phyrexian_life_payments,
                                        });
                                    }
                                    break;
                                }
                                AbilityDefinition::Megamorph { cost } => {
                                    if let Some((hybrid_choices, phyrexian_life_payments)) =
                                        turn_face_up_payment_plan(state, player, cost)
                                    {
                                        actions.push(LegalAction::TurnFaceUp {
                                            permanent: obj.id,
                                            method: TurnFaceUpMethod::MorphCost,
                                            hybrid_choices,
                                            phyrexian_life_payments,
                                        });
                                    }
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                FaceDownKind::Disguise => {
                    // Disguise turn-face-up cost is in the Disguise ability.
                    if let Some(def) = &card_def {
                        for ability in &def.abilities {
                            if let AbilityDefinition::Disguise { cost } = ability {
                                if let Some((hybrid_choices, phyrexian_life_payments)) =
                                    turn_face_up_payment_plan(state, player, cost)
                                {
                                    actions.push(LegalAction::TurnFaceUp {
                                        permanent: obj.id,
                                        method: TurnFaceUpMethod::DisguiseCost,
                                        hybrid_choices,
                                        phyrexian_life_payments,
                                    });
                                }
                                break;
                            }
                        }
                    }
                }
                FaceDownKind::Manifest | FaceDownKind::Cloak => {
                    // Manifest/Cloak: turn face up by paying printed mana cost, but only
                    // if the card is a creature card (CR 701.40b, CR 701.58b).
                    let is_creature = obj.characteristics.card_types.contains(&CardType::Creature);
                    // For Manifest/Cloak the raw card_types reflect the real card — check
                    // the card registry for the actual type line.
                    let def_is_creature = card_def
                        .as_ref()
                        .map(|def| def.types.card_types.contains(&CardType::Creature))
                        .unwrap_or(false);

                    if is_creature || def_is_creature {
                        let mana_cost = card_def
                            .as_ref()
                            .and_then(|def| def.mana_cost.clone())
                            .or_else(|| obj.characteristics.mana_cost.clone());
                        if let Some(cost) = mana_cost {
                            if let Some((hybrid_choices, phyrexian_life_payments)) =
                                turn_face_up_payment_plan(state, player, &cost)
                            {
                                actions.push(LegalAction::TurnFaceUp {
                                    permanent: obj.id,
                                    method: TurnFaceUpMethod::ManaCost,
                                    hybrid_choices,
                                    phyrexian_life_payments,
                                });
                            }
                        }
                    }

                    // Manifested/cloaked card with Morph/Megamorph can also use morph cost
                    // (CR 701.40c, CR 701.58c).
                    if let Some(def) = &card_def {
                        for ability in &def.abilities {
                            match ability {
                                AbilityDefinition::Morph { cost }
                                | AbilityDefinition::Megamorph { cost } => {
                                    if let Some((hybrid_choices, phyrexian_life_payments)) =
                                        turn_face_up_payment_plan(state, player, cost)
                                    {
                                        actions.push(LegalAction::TurnFaceUp {
                                            permanent: obj.id,
                                            method: TurnFaceUpMethod::MorphCost,
                                            hybrid_choices,
                                            phyrexian_life_payments,
                                        });
                                    }
                                    break;
                                }
                                AbilityDefinition::Disguise { cost } => {
                                    if let Some((hybrid_choices, phyrexian_life_payments)) =
                                        turn_face_up_payment_plan(state, player, cost)
                                    {
                                        actions.push(LegalAction::TurnFaceUp {
                                            permanent: obj.id,
                                            method: TurnFaceUpMethod::DisguiseCost,
                                            hybrid_choices,
                                            phyrexian_life_payments,
                                        });
                                    }
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // ── Morph/Megamorph/Disguise face-down cast (CR 702.37a / CR 702.168b) ──
        // Sorcery-speed only: active player, main phase, empty stack. The player may
        // cast a card with Morph/Megamorph/Disguise face-down for {3} (morph) or the
        // disguise cost. StubProvider conservatively emits sorcery-speed only.
        if is_main_phase && stack_empty && is_active {
            let morph_base_cost = ManaCost {
                generic: 3,
                ..Default::default()
            };
            let hand = ZoneId::Hand(player);
            for obj in state.objects_in_zone(&hand) {
                let is_land = obj.characteristics.card_types.contains(&CardType::Land);
                if is_land {
                    continue;
                }

                let card_def = obj
                    .card_id
                    .as_ref()
                    .and_then(|cid| state.card_registry().get(cid.clone()));

                let has_morph = card_def.as_ref().map(|def| {
                    def.abilities.iter().any(|a| {
                        matches!(
                            a,
                            AbilityDefinition::Morph { .. }
                                | AbilityDefinition::Megamorph { .. }
                                | AbilityDefinition::Disguise { .. }
                        )
                    })
                });

                if has_morph != Some(true) {
                    continue;
                }

                // Morph/Megamorph cast face-down costs {3}.
                let has_morph_or_mega = card_def.as_ref().map(|def| {
                    def.abilities.iter().any(|a| {
                        matches!(
                            a,
                            AbilityDefinition::Morph { .. } | AbilityDefinition::Megamorph { .. }
                        )
                    })
                });
                if has_morph_or_mega == Some(true) && can_afford(state, player, &morph_base_cost) {
                    actions.push(LegalAction::CastMorphFaceDown {
                        card: obj.id,
                        face_down_kind: FaceDownKind::Morph,
                    });
                }

                // Disguise cast costs its disguise cost.
                if let Some(def) = &card_def {
                    for ability in &def.abilities {
                        if let AbilityDefinition::Disguise { cost } = ability {
                            if can_afford(state, player, cost) {
                                actions.push(LegalAction::CastMorphFaceDown {
                                    card: obj.id,
                                    face_down_kind: FaceDownKind::Disguise,
                                });
                            }
                            break;
                        }
                    }
                }
            }
        }

        actions
    }
}

/// PB-DX6 (CR 107.4e/107.4f via CR 701.40b/702.37e/702.168d, SR-38): resolves the
/// TurnFaceUp offer-site's payment plan for `cost`, whichever of
/// `MorphCost`/`DisguiseCost`/`ManaCost` the caller is evaluating. Mirrors the
/// `ActivateAbility`/`TapForMana` sites' `has_pip_cost` branch above -- for a
/// pip-free cost the plain `can_afford` check (unchanged) is enough and the plan is
/// the empty pair; for a pipped cost `resolve_hybrid_phyrexian_plan` is the ONLY
/// correct check (a pip's raw `white`/`blue`/etc. fields are all 0, so
/// `can_afford`'s pool-total fallback can wrongly pass a pool the engine will
/// reject). `other_life_cost` is always 0 here -- a turn-face-up cost has no OTHER
/// life component to combine with a Phyrexian pip paid with life (plan §5.1's
/// `combined_life_cost` comment in `rules/engine.rs`, mirrored here on the offer
/// side). Deliberately reuses `resolve_hybrid_phyrexian_plan` verbatim rather than a
/// second copy (plan §9.1) -- including its CR 104.3b non-suicide policy, which must
/// not be collapsed with the CR 119.4 legality check it also performs.
fn turn_face_up_payment_plan(
    state: &GameState,
    player: PlayerId,
    cost: &ManaCost,
) -> Option<(Vec<HybridManaPayment>, Vec<bool>)> {
    if cost.hybrid.is_empty() && cost.phyrexian.is_empty() {
        if can_afford(state, player, cost) {
            Some((vec![], vec![]))
        } else {
            None
        }
    } else {
        resolve_hybrid_phyrexian_plan(state, player, cost, 0)
    }
}

/// PB-RS2 (CR 107.4e/107.4f, SR-38 precedent — "a bot must never suggest a choice the
/// engine rejects"). `can_afford`/the raw affordability checks above it check a cost's
/// standard fields; a cost with hybrid/Phyrexian pips needs an actual PLAN (which half,
/// mana-or-life) before it can be checked for affordability at all — the raw cost's
/// `white`/`blue`/etc. fields don't carry that information, and `mana_value()` alone
/// (used by `can_afford`'s pool-total fallback) can accept a pool that cannot actually
/// pay the specific colors required.
///
/// Returns `Some((hybrid_choices, phyrexian_life_payments))` for a plan that is BOTH
/// fully payable (mana + combined life, CR 119.4) AND non-suicidal (CR 104.3b — never
/// offers a Phyrexian-life plan that would drop the player to 0 or below unless it is
/// the only payable plan, in which case the whole action is not offered at all).
/// Returns `None` if no such plan exists.
///
/// `other_life_cost` is any OTHER life component of the SAME activation cost (e.g.
/// `ability_cost.life_cost` / `ManaAbility::life_cost`) that must be combined with a
/// Phyrexian-life choice for the CR 119.4 legality check — mirrors the engine's own
/// combined check in `rules/abilities.rs` / `rules/mana.rs` (§5.2 of the plan).
pub(crate) fn resolve_hybrid_phyrexian_plan(
    state: &GameState,
    player: PlayerId,
    cost: &ManaCost,
    other_life_cost: u32,
) -> Option<(Vec<HybridManaPayment>, Vec<bool>)> {
    let player_state = state.player(player).ok()?;
    let pool = &player_state.mana_pool;
    let life_total = player_state.life_total;

    let phyrexian_choices = build_phyrexian_choices(cost, pool);

    // Review finding #5: `can_afford` (called inside `try_hybrid_phyrexian_plan` below)
    // consults BOTH the pool AND untapped sources via the mana solver, but the
    // pool-preference heuristic in `build_hybrid_choices` only looks at the pool. When
    // the pool covers NEITHER half of a pip, the heuristic arbitrarily defaults to the
    // first half — and if only the OTHER half is payable via an untapped source, the
    // primary plan fails `can_afford` even though a payable plan exists. This is a
    // false negative (an action not offered though payable), never a false positive
    // (no illegal/lethal action is ever offered either way) — but it degrades bot
    // completeness. Fix: try the pool-preferred plan first (deterministic, matches the
    // flattener's own default so an empty-vector caller gets the same plan), then fall
    // back to the flipped-hybrid-half plan before giving up.
    let primary_hybrid_choices = build_hybrid_choices(cost, pool, false);
    if let Some(plan) = try_hybrid_phyrexian_plan(
        state,
        player,
        cost,
        primary_hybrid_choices,
        phyrexian_choices.clone(),
        other_life_cost,
        life_total,
    ) {
        return Some(plan);
    }
    if cost.hybrid.is_empty() {
        return None;
    }
    let alt_hybrid_choices = build_hybrid_choices(cost, pool, true);
    try_hybrid_phyrexian_plan(
        state,
        player,
        cost,
        alt_hybrid_choices,
        phyrexian_choices,
        other_life_cost,
        life_total,
    )
}

/// Plan-selection policy for hybrid pips (deterministic, mirrors the flattener's own
/// defaults so a caller that leaves `hybrid_choices` empty gets the SAME plan this
/// function would choose when `flip == false`): prefer whichever half the pool can
/// actually cover; for a monocolored hybrid, prefer the 1-mana colored option over 2
/// generic. `flip == true` inverts every preference — the review finding #5 fallback,
/// tried when the pool-preferred plan turns out not to be payable via untapped sources.
fn build_hybrid_choices(
    cost: &ManaCost,
    pool: &mtg_engine::ManaPool,
    flip: bool,
) -> Vec<HybridManaPayment> {
    let mut hybrid_choices = Vec::with_capacity(cost.hybrid.len());
    for h in &cost.hybrid {
        match h {
            HybridMana::ColorColor(a, b) => {
                let prefer_a = pool.get(*a) > 0 || pool.get(*b) == 0;
                let choice = if prefer_a != flip { *a } else { *b };
                hybrid_choices.push(HybridManaPayment::Color(choice));
            }
            HybridMana::GenericColor(c) => {
                let prefer_color = pool.get(*c) > 0;
                if prefer_color != flip {
                    hybrid_choices.push(HybridManaPayment::Color(*c));
                } else {
                    hybrid_choices.push(HybridManaPayment::Generic);
                }
            }
        }
    }
    hybrid_choices
}

/// Plan-selection policy for Phyrexian pips: prefer mana over life. Unaffected by
/// review finding #5 (that finding is specific to the hybrid-half preference).
fn build_phyrexian_choices(cost: &ManaCost, pool: &mtg_engine::ManaPool) -> Vec<bool> {
    let mut phyrexian_life_payments = Vec::with_capacity(cost.phyrexian.len());
    for ph in &cost.phyrexian {
        let color = match ph {
            PhyrexianMana::Single(c) => *c,
            PhyrexianMana::Hybrid(c, _) => *c,
        };
        phyrexian_life_payments.push(pool.get(color) == 0);
    }
    phyrexian_life_payments
}

/// Check a candidate `(hybrid_choices, phyrexian_life_payments)` plan for full
/// payability (mana + combined life, CR 119.4) AND non-suicidality (CR 104.3b — never
/// a plan that would drop the player to 0 or below). Returns the plan unchanged on
/// success so the caller can hand it straight to `LegalAction`.
#[allow(clippy::too_many_arguments)]
fn try_hybrid_phyrexian_plan(
    state: &GameState,
    player: PlayerId,
    cost: &ManaCost,
    hybrid_choices: Vec<HybridManaPayment>,
    phyrexian_life_payments: Vec<bool>,
    other_life_cost: u32,
    life_total: i32,
) -> Option<(Vec<HybridManaPayment>, Vec<bool>)> {
    let any_pip_paid_with_life = phyrexian_life_payments
        .iter()
        .any(|&paid_with_life| paid_with_life);
    let (flat, phyrexian_life) = cost
        .flatten_hybrid_phyrexian(&hybrid_choices, &phyrexian_life_payments)
        .ok()?;
    if !can_afford(state, player, &flat) {
        return None;
    }
    let combined_life_cost = other_life_cost + phyrexian_life;
    if combined_life_cost > 0 {
        // CR 119.4: legality. Distinct from the CR 104.3b policy check below on
        // purpose — collapsing the two is the exact hazard §7.1 of the plan warns
        // about (a future "simplification" reintroducing bot self-kill).
        if life_total < combined_life_cost as i32 {
            return None;
        }
        // CR 104.3b: policy. At life_total == combined_life_cost the payment is
        // LEGAL (>=) but drops the player to exactly 0, which SBA converts into a
        // loss — never offer that, only offer a plan that leaves the player alive.
        if any_pip_paid_with_life && life_total - (combined_life_cost as i32) <= 0 {
            return None;
        }
    }

    Some((hybrid_choices, phyrexian_life_payments))
}

/// Multiply a mana cost by a scalar (PB-DP4 / DP-11, CR 702.24b: cumulative upkeep's total
/// is per_counter_cost x the permanent's age counter count).
///
/// Deliberately mirrors `rules/engine.rs::multiply_mana_cost` EXACTLY, including
/// hybrid/phyrexian/x_count -- that function is private to `engine.rs`, so this is a
/// necessary duplicate (seed OOS-DP4-7), not a stylistic choice. If this copy ever drifts
/// from the engine's, the SR-38 "only offer what the engine accepts" contract breaks
/// silently: a bot's `PayCumulativeUpkeep { pay: true }` would start getting rejected via
/// `driver.rs`'s `PassPriority` fallback.
fn multiply_mana_cost(cost: &ManaCost, multiplier: u32) -> ManaCost {
    ManaCost {
        white: cost.white * multiplier,
        blue: cost.blue * multiplier,
        black: cost.black * multiplier,
        red: cost.red * multiplier,
        green: cost.green * multiplier,
        colorless: cost.colorless * multiplier,
        generic: cost.generic * multiplier,
        hybrid: cost
            .hybrid
            .iter()
            .flat_map(|h| std::iter::repeat_n(h.clone(), multiplier as usize))
            .collect(),
        phyrexian: cost
            .phyrexian
            .iter()
            .flat_map(|p| std::iter::repeat_n(p.clone(), multiplier as usize))
            .collect(),
        x_count: cost.x_count * multiplier,
    }
}

/// CR 117.1a / CR 601.3b (SIM-1 Step 3): may `player` begin casting `obj` in the
/// current priority window? Shared by the hand loop and the command-zone loop in
/// `StubProvider::legal_actions` so the two enumerations cannot drift apart on
/// timing -- a pure extraction of what was previously duplicated inline in the hand
/// loop, behaviour-preserving (verified by the existing suite staying green with no
/// test edits).
///
/// Instants, Flash, and any active CR 601.3b flash grant (`state.flash_grants()`) are
/// castable any time the caller has priority; everything else is sorcery-speed only
/// (main phase, empty stack, active player). This is deliberately zone-agnostic: the
/// engine's own timing gate (`casting.rs`) knows nothing about which zone the card
/// came from, so a commander with Flash, or one under a Vedalken Orrery / Leyline of
/// Anticipation grant, is legally castable at instant speed and must be offered --
/// hard-coding sorcery speed for the command-zone loop would under-offer.
fn can_cast_at_this_time(
    state: &GameState,
    player: PlayerId,
    obj: &GameObject,
    is_main_phase: bool,
    stack_empty: bool,
    is_active: bool,
) -> bool {
    let is_instant = obj.characteristics.card_types.contains(&CardType::Instant);
    let has_flash = obj
        .characteristics
        .keywords
        .contains(&KeywordAbility::Flash);

    // CR 601.3b: Check if player has an active flash grant for this spell.
    let has_flash_grant = state.flash_grants().iter().any(|g| {
        if g.player != player {
            return false;
        }
        // CR 611.2b: WhileSourceOnBattlefield grants are only active while
        // the source object is still on the battlefield (mirrors engine's
        // has_active_flash_grant check in casting.rs).
        if matches!(g.duration, EffectDuration::WhileSourceOnBattlefield) {
            if let Some(src) = g.source {
                let on_bf = state
                    .objects()
                    .get(&src)
                    .map(|o| matches!(o.zone, ZoneId::Battlefield))
                    .unwrap_or(false);
                if !on_bf {
                    return false;
                }
            }
        }
        match &g.filter {
            FlashGrantFilter::AllSpells => true,
            FlashGrantFilter::Sorceries => {
                obj.characteristics.card_types.contains(&CardType::Sorcery)
            }
            FlashGrantFilter::GreenCreatures => {
                obj.characteristics.card_types.contains(&CardType::Creature)
                    && obj
                        .characteristics
                        .colors
                        .contains(&mtg_engine::Color::Green)
            }
        }
    });
    // Timing check: instants and flash anytime with priority;
    // sorcery-speed only main phase + stack empty + active player
    if is_instant || has_flash || has_flash_grant {
        true
    } else {
        is_main_phase && stack_empty && is_active
    }
}

/// CR 903.8 / CR 601.2f (SIM-1): the mana cost this player will actually be charged for
/// casting `card` right now -- the printed cost, plus commander tax when (and only when)
/// `card` is in `ZoneId::Command(player)` AND its `CardId` is one of `player`'s
/// `commander_ids`.
///
/// Mirrors `rules/casting.rs`'s own two-part derivation exactly:
///   * `casting.rs::process_command`'s `casting_from_command_zone` derivation -- the
///     object's zone equals `ZoneId::Command(player)`.
///   * `casting.rs`'s CR 903.8 gate -- `player_state.commander_ids.contains(card_id)`.
///   * `casting.rs`'s tax application -- `apply_commander_tax(&base, tax)`, where `tax`
///     is the count of PREVIOUS casts (`commander_tax.get(cid)`, defaulting to 0).
///
/// The tax itself is consumed from the engine (`mtg_engine::apply_commander_tax`), never
/// re-derived here -- SR-38's "only offer what the engine accepts" is only true if the
/// two arithmetics are literally the same function. Contrast `multiply_mana_cost` above,
/// which is a *necessary* duplicate because the engine's copy is private; this one is
/// not, so duplicating it would be a choice, and the wrong one.
///
/// Returns `None` when the object has no mana cost (an emblem in the command zone under
/// CR 408.1, a land, a missing object) or -- defensively, unreached by any real card --
/// a command-zone object with a mana cost but no `CardId`. Every caller treats `None`
/// as "nothing to pay for / nothing to offer".
///
/// **Identity for every non-commander cast**: for a card in hand (or any zone other than
/// this player's command zone, or a command-zone object that is not one of this
/// player's registered commanders) both guards fail and the printed cost is returned
/// unchanged, so no existing offer, plan or seed moves (T12).
pub fn effective_cast_cost(
    state: &GameState,
    player: PlayerId,
    card: ObjectId,
) -> Option<ManaCost> {
    let obj = state.object(card).ok()?;
    let printed = obj.characteristics.mana_cost.clone()?;
    if obj.zone != ZoneId::Command(player) {
        return Some(printed);
    }
    let ps = state.player(player).ok()?;
    let cid = obj.card_id.as_ref()?;
    if !ps.commander_ids.contains(cid) {
        return Some(printed);
    }
    let tax = ps.commander_tax.get(cid).copied().unwrap_or(0);
    Some(apply_commander_tax(&printed, tax))
}

/// Mana affordability check: considers both mana pool and untapped sources.
/// Uses the mana solver for precise color-aware checking.
///
/// # SIM-2: one question, asked once
///
/// This used to be two checks with a gap between them — a pool-total shortcut
/// (`pool.total() >= cost.mana_value()` with per-colour floors) OR a solve for the
/// **entire** cost from untapped sources, with nothing covering the case in between.
/// A player with `{G}` floating and one Forest up was told they could not cast a
/// `{1}{G}` spell: the pool alone did not cover it and the sources alone did not
/// either. `solve_mana_payment_with_pool` subtracts the pool and solves the residual,
/// which answers all three cases with one call and, crucially, is the **same function**
/// `LocalGame::auto_tap_commands_for` uses to build the plan — so the gate and the
/// plan cannot disagree (SR-38: never offer what the engine rejects, and its dual,
/// never withhold what the engine accepts).
fn can_afford(state: &GameState, player: PlayerId, cost: &mtg_engine::ManaCost) -> bool {
    if state.player(player).is_err() {
        return false;
    }
    crate::mana_solver::solve_mana_payment_with_pool(state, player, cost).is_some()
}

/// PB-AC8 / CR 701.21a (UI-2 §1.2): mirrors `effects::object_cant_be_sacrificed`,
/// which is `pub(crate)` to the engine and therefore not callable from this crate.
/// A NECESSARY duplicate over PUBLIC state (`state.restrictions()`,
/// `GameRestriction::CantBeSacrificed`, plus the source's own zone) -- the same
/// category as `multiply_mana_cost` above, and explicitly NOT the category of
/// `effective_cast_cost`, whose engine copy IS public and must be consumed, never
/// re-derived.
fn object_cant_be_sacrificed(state: &GameState, obj_id: ObjectId) -> bool {
    state.restrictions().iter().any(|r| {
        r.source == obj_id
            && matches!(r.restriction, GameRestriction::CantBeSacrificed)
            && state
                .objects()
                .get(&r.source)
                .map(|o| o.zone == ZoneId::Battlefield)
                .unwrap_or(false)
    })
}

/// CR 118.8 (UI-2 §1.2): the battlefield permanents `player` controls that
/// `casting.rs:3300-3369`'s own spell-additional-sacrifice gate will accept for
/// `required`, gate for gate and in the SAME order (its checks: on the battlefield,
/// controlled by `player`, not `object_cant_be_sacrificed`, matches the filter
/// against LAYER-RESOLVED characteristics).
///
/// Deliberately NOT `effects::eligible_sacrifice_targets` -- that helper also checks
/// `is_phased_in`, which the CAST gate does not. Mirroring the sacrifice-EFFECT
/// helper here would offer a different set from the one the engine will actually
/// validate.
fn eligible_spell_sacrifice_targets(
    state: &GameState,
    player: PlayerId,
    required: &SpellAdditionalCost,
) -> Vec<ObjectId> {
    state
        .objects_in_zone(&ZoneId::Battlefield)
        .into_iter()
        .filter(|obj| obj.controller == player)
        .filter(|obj| !object_cant_be_sacrificed(state, obj.id))
        .filter(|obj| {
            let chars = mtg_engine::rules::layers::calculate_characteristics(state, obj.id)
                .unwrap_or_else(|| obj.characteristics.clone());
            match required {
                SpellAdditionalCost::SacrificeCreature => {
                    chars.card_types.contains(&CardType::Creature)
                }
                SpellAdditionalCost::SacrificeLand => chars.card_types.contains(&CardType::Land),
                SpellAdditionalCost::SacrificeArtifactOrCreature => {
                    chars.card_types.contains(&CardType::Artifact)
                        || chars.card_types.contains(&CardType::Creature)
                }
                SpellAdditionalCost::SacrificeSubtype(sub) => chars.subtypes.contains(sub),
                SpellAdditionalCost::SacrificeColorPermanent(color) => chars.colors.contains(color),
            }
        })
        .map(|obj| obj.id)
        .collect()
}

/// CR 602.2 (SIM-6): the battlefield permanents `player` controls that
/// `handle_activate_ability`'s own sacrifice-cost gate (`rules/abilities.rs`, the
/// `ability_cost.sacrifice_filter` block) will accept for `filter`, **gate for gate
/// and in the same order**: on the battlefield, controlled by `player`, not the
/// source when `exclude_self` (CR 109.1 / PB-EF1), not
/// `object_cant_be_sacrificed` (CR 701.21a / PB-AC8), and matching the filter
/// against LAYER-RESOLVED characteristics (CR 613.1f).
///
/// Deliberately NOT `effects::eligible_sacrifice_targets`, and NOT
/// [`eligible_spell_sacrifice_targets`] either — for the same reason that one is not
/// the effect helper. The three gates differ: the effect helper also checks
/// `is_phased_in`; the CAST gate (CR 118.8) has no self-exclusion because a spell on
/// the stack is not a permanent; only this one has `exclude_self` and the
/// `CreatureOfChosenType` arm. Mirroring a neighbouring gate would offer a set the
/// engine does not validate.
///
/// `CreatureOfChosenType` reads the SOURCE's `chosen_creature_type`, which is why
/// `source` is a parameter and not just the exclusion subject.
fn eligible_activation_sacrifice_targets(
    state: &GameState,
    player: PlayerId,
    source: ObjectId,
    filter: &SacrificeFilter,
    exclude_self: bool,
) -> Vec<ObjectId> {
    state
        .objects_in_zone(&ZoneId::Battlefield)
        .into_iter()
        .filter(|obj| obj.controller == player)
        .filter(|obj| !(exclude_self && obj.id == source))
        .filter(|obj| !object_cant_be_sacrificed(state, obj.id))
        .filter(|obj| {
            let chars = mtg_engine::rules::layers::calculate_characteristics(state, obj.id)
                .unwrap_or_else(|| obj.characteristics.clone());
            match filter {
                SacrificeFilter::Creature => chars.card_types.contains(&CardType::Creature),
                SacrificeFilter::Land => chars.card_types.contains(&CardType::Land),
                SacrificeFilter::Artifact => chars.card_types.contains(&CardType::Artifact),
                SacrificeFilter::ArtifactOrCreature => {
                    chars.card_types.contains(&CardType::Artifact)
                        || chars.card_types.contains(&CardType::Creature)
                }
                SacrificeFilter::Subtype(sub) => chars.subtypes.contains(sub),
                // The engine's own arm, restated: a creature AND carrying the
                // ACTIVATING SOURCE's chosen creature type. A source that has
                // chosen nothing accepts nothing, which is what makes the whole
                // offer disappear rather than 422.
                SacrificeFilter::CreatureOfChosenType => {
                    chars.card_types.contains(&CardType::Creature)
                        && state
                            .objects()
                            .get(&source)
                            .and_then(|o| o.chosen_creature_type.as_ref())
                            .is_some_and(|ct| chars.subtypes.contains(ct))
                }
            }
        })
        .map(|obj| obj.id)
        .collect()
}

/// CR 602.2 / SR-38 (SIM-6): the [`ActivationCostPlan`] to offer with an
/// `ActivateAbility` for `ability` on `source`, or `None` if the activation must not
/// be offered at all.
///
/// `None` means exactly one thing, and it is [`offerable_cast_plan`]'s meaning one
/// command over: the ability declares a cost component that must NAME an object and
/// this player has no legal object to name, so `handle_activate_ability` would refuse
/// the activation outright. Offering it anyway was the G4 defect — the human clicked
/// Yahenni and got a 422.
///
/// Both components are REQUIRED when present (CR 602.2b: costs are not optional), so
/// either one being unpayable suppresses the whole offer. Nothing here is optional
/// the way Squad is, which is why this has no counterpart to
/// `offerable_cast_plan`'s "Squad never suppresses" carve-out.
fn offerable_activation_plan(
    state: &GameState,
    player: PlayerId,
    source: ObjectId,
    ability: &ActivatedAbility,
) -> Option<ActivationCostPlan> {
    let plan = build_activation_cost_plan(state, player, source, ability);
    if plan
        .sacrifice
        .as_ref()
        .is_some_and(|s| s.eligible.is_empty())
    {
        return None;
    }
    if plan.discard.as_ref().is_some_and(|d| d.eligible.is_empty()) {
        return None;
    }
    Some(plan)
}

/// CR 602.2 (SIM-6): build the non-mana activation-cost descriptor for `ability`.
/// Consumed by `params.rs` (the bot default and the human's answer) and by
/// `tools/play-server` (the picker). `ActivationCostPlan::default()` (both fields
/// `None`) for every ability whose `ActivationCost` declares neither component.
///
/// The caller ([`offerable_activation_plan`]) suppresses the whole offer when either
/// eligible set comes back empty, so the `SENTINEL` defaults below are never read in
/// that case.
fn build_activation_cost_plan(
    state: &GameState,
    player: PlayerId,
    source: ObjectId,
    ability: &ActivatedAbility,
) -> ActivationCostPlan {
    let sacrifice = ability.cost.sacrifice_filter.as_ref().map(|filter| {
        let exclude_self = ability.cost.sacrifice_exclude_self;
        let eligible =
            eligible_activation_sacrifice_targets(state, player, source, filter, exclude_self);
        ActivationSacrificeOption {
            filter: filter.clone(),
            exclude_self,
            default: eligible.first().copied().unwrap_or(ObjectId::SENTINEL),
            eligible,
        }
    });

    // CR 602.2 / CR 111.10g: `abilities.rs`' discard-cost block checks that the
    // named card is in the ACTIVATING PLAYER's hand and nothing else — no filter, no
    // count beyond one — so the eligible set is that zone verbatim.
    let discard = ability.cost.discard_card.then(|| {
        let eligible: Vec<ObjectId> = state
            .objects_in_zone(&ZoneId::Hand(player))
            .into_iter()
            .map(|obj| obj.id)
            .collect();
        ActivationDiscardOption {
            default: eligible.first().copied().unwrap_or(ObjectId::SENTINEL),
            eligible,
        }
    });

    ActivationCostPlan { sacrifice, discard }
}

/// CR 118.8 / SR-38 (UI-2 §1.3): the `AdditionalCostPlan` to offer with a
/// `CastSpell` for `obj`, or `None` if the cast must not be offered at all.
///
/// `None` means exactly one thing: the spell declares a REQUIRED sacrifice
/// (CR 118.8) and this player controls nothing eligible to pay it, so
/// `casting.rs:3311` would refuse the cast outright. Offering it anyway was the F9
/// defect -- the human clicked Life's Legacy and got a 422.
///
/// Squad never suppresses: it is optional (CR 702.157a, "any number of times",
/// including zero), so a spell with Squad and no spare mana is still cast.
///
/// **One helper, two call sites, and that is the point** (review Issue 4): the hand
/// loop and the command-zone loop used to carry byte-identical inline copies of this
/// under a comment saying the two must not diverge. A rule that lives in a comment
/// is a rule nothing enforces, and only one of the two copies had a test.
fn offerable_cast_plan(
    state: &GameState,
    player: PlayerId,
    obj: &GameObject,
) -> Option<AdditionalCostPlan> {
    let plan = build_additional_cost_plan(state, player, obj);
    if plan
        .sacrifice
        .as_ref()
        .is_some_and(|s| s.eligible.is_empty())
    {
        return None;
    }
    Some(plan)
}

/// CR 118.8 / CR 702.157 (UI-2 §1): builds the additional-cost descriptor for
/// casting `obj` -- consumed by `params.rs` (the bot default) and, in a later
/// stage, `tools/play-server` (the picker). `AdditionalCostPlan::default()` (both
/// fields `None`) for every spell whose `CardDefinition` declares neither.
fn build_additional_cost_plan(
    state: &GameState,
    player: PlayerId,
    obj: &GameObject,
) -> AdditionalCostPlan {
    let Some(def) = obj
        .card_id
        .as_ref()
        .and_then(|cid| state.card_registry().get(cid.clone()))
    else {
        return AdditionalCostPlan::default();
    };

    // §1.2: only `spell_additional_costs.first()` is read. `casting.rs` says so in
    // its own words ("For now, we support exactly one mandatory sacrifice cost")
    // and then validates `required_costs[0]` alone -- offering a second requirement
    // the engine will never check would be an offer that means nothing.
    let sacrifice = def.spell_additional_costs.first().map(|requirement| {
        let eligible = eligible_spell_sacrifice_targets(state, player, requirement);
        // The caller suppresses the WHOLE `CastSpell` offer when `eligible` is
        // empty (§1.3), so this sentinel is never actually read in that case.
        let default = eligible.first().copied().unwrap_or(ObjectId::SENTINEL);
        SacrificeCostOption {
            requirement: requirement.clone(),
            eligible,
            default,
        }
    });

    // CR 702.157a: detected via the COST-CARRYING variant, not the
    // `KeywordAbility::Squad` presence marker. A def carrying the marker but no
    // `AbilityDefinition::Squad { cost }` makes `casting.rs`'s own `get_squad_cost`
    // return `None` and the cast is refused the moment `squad_count > 0` is
    // announced (`"spell has squad keyword but no squad cost defined"`); detecting
    // on the cost variant means this provider never offers Squad on such a def in
    // the first place (SR-38), rather than offering it and having every non-zero
    // count refused.
    //
    // That is not a hypothetical shape: `galadhrim_brigade` -- the very card the
    // first human playtest tried to Squad -- shipped `Complete` with the marker
    // alone, and UI-2 repaired the def. The roster gate
    // (`core::ui2_additional_cost_roster`, R4) now pins that every Squad def carries
    // a non-zero cost, so this arm's SR-38 fallback is a floor rather than the
    // corpus's actual state.
    let squad = def
        .abilities
        .iter()
        .find_map(|a| {
            if let AbilityDefinition::Squad { cost } = a {
                Some(cost.clone())
            } else {
                None
            }
        })
        .map(|cost| {
            let max_count = squad_max_count(state, player, obj.id, &cost);
            SquadCostOption { cost, max_count }
        });

    AdditionalCostPlan { sacrifice, squad }
}

/// CR 702.157a (UI-2 §1.4): the largest N this player can currently afford on top
/// of `card`'s own effective cast cost, paying `squad_cost` N times.
///
/// A GENUINE upper bound gates the search, not an arbitrary cap: no payment plan
/// can exceed `pool.total()` plus the summed maximum output of this player's
/// untapped mana sources. The loop then walks `n = 1..` checking `can_afford` and
/// stops at the first unaffordable `n` -- the cost strictly increases with `n`, so
/// affordability cannot come back once lost.
///
/// The COST arithmetic is `effective_cast_cost_with_additional`, the same function
/// `LocalGame`'s two auto-tap sites use, so this cannot drift from what the engine
/// will actually charge.
///
/// **The AFFORDABILITY arithmetic is a different matter, and the distinction is
/// worth stating** (review Issue 1): `can_afford` answers "the pool alone covers
/// this" OR "the solver finds a plan from untapped sources alone", and never
/// "pool plus a few fresh taps". That is the surviving half of `OOS-M11-2`
/// (`mana_solver.rs` ignores the pool entirely), and it is **symmetric** rather
/// than a new asymmetry: `LocalGame::auto_tap_commands_for` takes exactly the same
/// two branches, so a count this function calls unaffordable is a count the payment
/// path could not have paid either. It is still a real under-report against what
/// the ENGINE would accept from a hand-built command, and it is the reason a human
/// with floating mana and untapped lands may be offered a smaller `max_count` than
/// CR 601.2h would allow -- filed as `OOS-UI2-3`.
///
/// Returns 0 if `squad_cost.mana_value() == 0`: the loop would be unbounded (an
/// ever-larger N stays equally "free"), and the roster gate
/// (`core::ui2_additional_cost_roster` R4) pins that no def in the corpus has one.
fn squad_max_count(
    state: &GameState,
    player: PlayerId,
    card: ObjectId,
    squad_cost: &ManaCost,
) -> u32 {
    let squad_mv = squad_cost.mana_value();
    if squad_mv == 0 {
        return 0;
    }

    let pool_total = state
        .player(player)
        .map(|p| p.mana_pool.total())
        .unwrap_or(0);
    // The bound must OVER-estimate, never under-estimate: `can_afford` below is the
    // real gate, so an over-estimate only costs a wasted loop iteration, while an
    // under-estimate silently caps what a human is allowed to pay and is then
    // enforced as a hard 400 by `validate_additional_cost_params`.
    //
    // So sum each untapped source's actual maximum output rather than counting
    // sources at one mana each (review Issue 1). Summing every ability of every
    // source over-counts a source that can only be activated once, which is the
    // safe direction.
    //
    // This bound was written while `mana_solver`'s Phase 3 still paid one generic
    // pip per source tapped (playtest triage **F4**), which capped the offer one
    // layer down and made this fix unobservable at the time. F4 closed in SIM-2
    // (`scutemob-176`) at the same collect that merged this batch, so the two
    // under-counts closed together and the test
    // `squad_max_count_counts_true_production_now_that_f4_is_closed` now pins the
    // post-fix value. Seed `OOS-UI2-3` records the history.
    let mana_source_total: u32 = state
        .objects_in_zone(&ZoneId::Battlefield)
        .into_iter()
        .filter(|o| o.controller == player && !o.status.tapped)
        .map(|o| {
            mtg_engine::rules::layers::calculate_characteristics(state, o.id)
                .unwrap_or_else(|| o.characteristics.clone())
                .mana_abilities
                .iter()
                .map(|a| a.produces.values().copied().sum::<u32>().max(1))
                .sum::<u32>()
        })
        .sum();
    let available = pool_total.saturating_add(mana_source_total);

    let base_mv = effective_cast_cost(state, player, card)
        .map(|c| c.mana_value())
        .unwrap_or(0);
    let upper_bound = available.saturating_sub(base_mv) / squad_mv;

    let mut max_count = 0;
    for n in 1..=upper_bound {
        let announced = [AdditionalCost::Squad { count: n }];
        let Some(candidate_cost) =
            effective_cast_cost_with_additional(state, player, card, &announced)
        else {
            break;
        };
        if can_afford(state, player, &candidate_cost) {
            max_count = n;
        } else {
            break;
        }
    }
    max_count
}

/// CR 702.157a / CR 601.2b,f (UI-2 §2): `effective_cast_cost` plus the mana cost of
/// any announced `AdditionalCost::Squad { count }`, added `count` times EXACTLY as
/// `casting.rs`'s own Squad arm does (CR 118.8d: additional costs don't change the
/// spell's PRINTED mana cost, but they do change what this player is actually
/// charged, which is this helper's whole purpose). Calls `effective_cast_cost`
/// rather than re-deriving it, so the commander-tax arithmetic stays
/// `mtg_engine::apply_commander_tax` in exactly one place.
///
/// `casting.rs`'s own `get_squad_cost` is private to that file, so reading
/// `AbilityDefinition::Squad { cost }` off `state.card_registry()` here is a
/// NECESSARY duplicate (same category as `object_cant_be_sacrificed` above).
///
/// Identity for every cast that announces no Squad -- which is every bot cast and
/// every cast today (T-series pin this at both call sites in `local_game.rs`).
pub fn effective_cast_cost_with_additional(
    state: &GameState,
    player: PlayerId,
    card: ObjectId,
    additional_costs: &[AdditionalCost],
) -> Option<ManaCost> {
    let mut cost = effective_cast_cost(state, player, card)?;
    // **LAST wins, not the sum** -- and the difference is a mirror correction, not a
    // preference. `casting.rs`'s own destructuring loop is
    // `AdditionalCost::Squad { count } => { squad_count = *count; }`, a plain
    // assignment, so a command carrying two `Squad` entries is charged for the LAST
    // one only. A first draft here summed them, which meant a two-entry submission
    // made this helper (and therefore the auto-tap) reach for strictly more mana
    // than the engine would charge: with only the smaller amount available the
    // solver found no plan, no taps were issued, and the engine refused the cast
    // with "player does not have enough mana to pay the cost" -- a 422 after a
    // clean offer, exactly the SR-38 shape this batch exists to delete. Found by
    // review, not by a test.
    //
    // `tools/play-server`'s `validate_additional_cost_params` additionally refuses a
    // duplicate `Squad` at the 400 boundary, because a second entry is an
    // announcement the offer never made. The two are complementary rather than
    // redundant: this one keeps the ARITHMETIC honest for every caller (a bot, the
    // TUI, a test) and that one keeps the AMBIGUITY out of the HTTP surface.
    let squad_count = additional_costs
        .iter()
        .filter_map(|ac| match ac {
            AdditionalCost::Squad { count } => Some(*count),
            _ => None,
        })
        .next_back();
    if let Some(count) = squad_count.filter(|c| *c > 0) {
        let obj = state.object(card).ok()?;
        let squad_cost = obj
            .card_id
            .as_ref()
            .and_then(|cid| state.card_registry().get(cid.clone()))
            .and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Squad { cost } = a {
                        Some(cost.clone())
                    } else {
                        None
                    }
                })
            })?;
        for _ in 0..count {
            cost.white += squad_cost.white;
            cost.blue += squad_cost.blue;
            cost.black += squad_cost.black;
            cost.red += squad_cost.red;
            cost.green += squad_cost.green;
            cost.generic += squad_cost.generic;
            cost.colorless += squad_cost.colorless;
        }
    }
    Some(cost)
}

/// PB-18 review Finding 4: Check whether any active restriction prevents this player
/// from activating an ability of a specific source object.
///
/// Mirrors check_activate_restrictions in rules/abilities.rs. Only objects on the
/// battlefield are affected (zone-scope fix from Finding 3).
///
/// # SIM-2: mana abilities are activated abilities too (CR 605.3)
///
/// `pub(crate)` and called from `mana_solver::tap_ability_is_activatable` as well.
/// `rules/mana.rs`'s step 1b enforces these same two `GameRestriction` variants on a
/// `TapForMana` — CR 605.3 says activating a mana ability follows the rules for
/// activating any other activated ability, so Stony Silence / Collector Ouphe stop a
/// Sol Ring and Grand Abolisher stops an opponent's — and until SIM-2's `/review`
/// caught it, **neither the provider's `TapForMana` loop nor the solver mirrored them**.
/// With an opponent's Collector Ouphe out, `can_afford` counted a Sol Ring, the cast was
/// offered, and the atomic tap-and-cast sequence was then refused: the exact SR-38
/// failure this batch exists to remove, one restriction class away from where it looked.
///
/// The `restrictions().is_empty()` fast path is new and load-bearing for that second
/// caller: the solver asks this per source per solve, and `calculate_characteristics` is
/// not free. Almost every board has no restrictions at all.
///
/// **Known cost, accepted**: past that fast path this recomputes `calculate_characteristics`
/// twice per source per solve, while `mana_solver::gather_sources` is already holding the
/// layer-resolved characteristics and passes them in for the summoning-sickness arm. It
/// mirrors `rules/mana.rs` step 1b, which makes the same double call, and correctness does
/// not depend on it — recorded so a stax-heavy fuzz seed running slow is a known cost
/// rather than a surprise.
pub(crate) fn is_ability_restricted_by_stax(
    state: &GameState,
    player: PlayerId,
    source: ObjectId,
) -> bool {
    if state.restrictions().is_empty() {
        return false;
    }
    let active_player = state.turn().active_player;

    // Source must be on the battlefield for restrictions to apply (Finding 3).
    let source_on_battlefield = state
        .objects()
        .get(&source)
        .map(|o| o.zone == ZoneId::Battlefield)
        .unwrap_or(false);

    if !source_on_battlefield {
        return false;
    }

    // Compute source card types once.
    let source_is_artifact = mtg_engine::rules::layers::calculate_characteristics(state, source)
        .map(|c| c.card_types.contains(&CardType::Artifact))
        .unwrap_or(false);

    let source_is_restricted_type =
        mtg_engine::rules::layers::calculate_characteristics(state, source)
            .map(|c| {
                c.card_types.contains(&CardType::Artifact)
                    || c.card_types.contains(&CardType::Creature)
                    || c.card_types.contains(&CardType::Enchantment)
            })
            .unwrap_or(false);

    for restriction in state.restrictions().iter() {
        let restriction_source_on_bf = state
            .objects()
            .get(&restriction.source)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);
        if !restriction_source_on_bf {
            continue;
        }

        let controller = restriction.controller;

        #[allow(clippy::collapsible_match)]
        match &restriction.restriction {
            GameRestriction::ArtifactAbilitiesCantBeActivated => {
                if source_is_artifact {
                    return true;
                }
            }
            GameRestriction::OpponentsCantCastOrActivateDuringYourTurn => {
                if active_player == controller && player != controller && source_is_restricted_type
                {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

/// PB-18: Check whether any active restriction prevents this player from casting
/// any spell at all (MaxSpellsPerTurn, OpponentsCantCast*).
///
/// Returns true if the player is completely restricted from casting.
/// Does NOT check per-card ZONE restrictions (like Drannith Magistrate's) --
/// those are checked separately by `is_cast_from_nonhand_restricted` (SIM-1), which
/// callers must consult alongside this function wherever a non-hand cast can be
/// offered.
fn is_cast_restricted_by_stax(state: &GameState, player: PlayerId) -> bool {
    use mtg_engine::GameRestriction;

    let active_player = state.turn().active_player;

    for restriction in state.restrictions().iter() {
        // Skip restrictions whose source is no longer on the battlefield.
        let source_on_bf = state
            .objects()
            .get(&restriction.source)
            .map(|o| matches!(o.zone, mtg_engine::ZoneId::Battlefield))
            .unwrap_or(false);
        if !source_on_bf {
            continue;
        }

        let controller = restriction.controller;

        #[allow(clippy::collapsible_match)]
        match &restriction.restriction {
            GameRestriction::MaxSpellsPerTurn { max } => {
                let spells_cast = state
                    .players()
                    .get(&player)
                    .map(|ps| ps.spells_cast_this_turn)
                    .unwrap_or(0);
                if spells_cast >= *max {
                    return true;
                }
            }
            GameRestriction::OpponentsCantCastDuringYourTurn => {
                if active_player == controller && player != controller {
                    return true;
                }
            }
            GameRestriction::OpponentsCantCastOrActivateDuringYourTurn => {
                if active_player == controller && player != controller {
                    return true;
                }
            }
            // PB-I: Teferi, Time Raveler — opponents may only cast spells at sorcery speed.
            // CR 101.2: This restriction overrides any flash permission the player has.
            // Sorcery speed = player's own main phase + empty stack + active player.
            GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed => {
                if player != controller {
                    let is_own_main = active_player == player
                        && matches!(
                            state.turn().step,
                            Step::PreCombatMain | Step::PostCombatMain
                        );
                    let stack_empty = state.stack_objects().is_empty();
                    if !is_own_main || !stack_empty {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }

    false
}

/// CR 101.2 (Drannith Magistrate) (SIM-1): mirrors the `OpponentsCantCastFromNonHand`
/// arm of `rules/casting.rs`'s `check_cast_restrictions`.
///
/// `is_cast_restricted_by_stax` deliberately does not check per-card ZONE
/// restrictions -- its own doc says so -- and that was harmless while the provider
/// offered hand casts only, because a hand cast always satisfies
/// `zone == Hand(player)`. Every command-zone cast is a non-hand cast, so without
/// this check SIM-1 would offer an action the engine rejects 100% of the time
/// whenever any opponent controls a Drannith Magistrate (`drannith_magistrate.rs`,
/// `Completeness::Complete`, deck-legal). SR-38.
///
/// Player-level, not per-card: the engine's arm reduces to `zone != Hand(player)`,
/// which is unconditionally true for the command zone.
fn is_cast_from_nonhand_restricted(state: &GameState, player: PlayerId) -> bool {
    for restriction in state.restrictions().iter() {
        // Same "source still on the battlefield" guard as `is_cast_restricted_by_stax`.
        let source_on_bf = state
            .objects()
            .get(&restriction.source)
            .map(|o| matches!(o.zone, ZoneId::Battlefield))
            .unwrap_or(false);
        if !source_on_bf {
            continue;
        }
        if matches!(
            restriction.restriction,
            GameRestriction::OpponentsCantCastFromNonHand
        ) && player != restriction.controller
        {
            return true;
        }
    }
    false
}

/// CR 601.2b / 602.2b / 700.2a (PB-DP3): the engine no longer auto-selects mode 0, so a
/// bot must announce a legal mode set. Choose the first `min_modes` distinct indices in
/// printed order — always legal (never duplicates, never out of range, never over
/// `max_modes` since `min_modes <= max_modes`). Returns empty for a non-modal object,
/// which is exactly what a non-modal cast/activation wants.
pub fn default_modes_chosen(ms: &mtg_engine::ModeSelection) -> Vec<usize> {
    (0..ms.min_modes.min(ms.modes.len())).collect()
}

/// Mirrors `casting.rs:3495-3506`'s `AbilityDefinition::Spell { modes: Some(..) }` lookup.
/// Returns `vec![]` for a non-modal card (a no-op for every non-modal cast).
pub fn spell_default_modes(state: &GameState, card: ObjectId) -> Vec<usize> {
    let Some(obj) = state.objects().get(&card) else {
        return vec![];
    };
    let Some(cid) = obj.card_id.clone() else {
        return vec![];
    };
    let Some(def) = state.card_registry().get(cid) else {
        return vec![];
    };
    def.abilities
        .iter()
        .find_map(|a| {
            if let AbilityDefinition::Spell { modes: Some(m), .. } = a {
                Some(default_modes_chosen(m))
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Mirrors `abilities.rs:313-331` — indexes the LAYER-RESOLVED `activated_abilities` list
/// (CR 613.1f), not `def.abilities`. Getting this wrong is an index-namespace bug of
/// exactly the class PB-RS4 spent a session closing.
pub fn ability_default_modes(
    state: &GameState,
    source: ObjectId,
    ability_index: usize,
) -> Vec<usize> {
    let chars = match mtg_engine::rules::layers::calculate_characteristics(state, source) {
        Some(c) => c,
        None => match state.objects().get(&source) {
            Some(o) => o.characteristics.clone(),
            None => return vec![],
        },
    };
    chars
        .activated_abilities
        .get(ability_index)
        .and_then(|ab| ab.modes.as_ref())
        .map(default_modes_chosen)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtg_engine::{
        ActivatedAbility, ActivationCost, Color, ContinuousEffect, EffectFilter, EffectId,
        EffectLayer, GameStateBuilder, LayerModification, ManaAbility, ManaPool, ObjectSpec,
        SubType,
    };

    /// Find a battlefield object's id by its printed name.
    fn id_of(state: &GameState, name: &str) -> ObjectId {
        state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == name)
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("no object named {name:?}"))
    }

    /// A `ManaAbility` that taps for one Black mana and costs `life` life to activate.
    fn tap_for_black_costing_life(life: u32) -> ManaAbility {
        let mut ma = ManaAbility::tap_for(mtg_engine::ManaColor::Black);
        ma.life_cost = life;
        ma
    }

    /// A non-mana activated ability whose only cost is `life` life (no tap, no mana).
    fn activated_costing_life(life: u32) -> ActivatedAbility {
        ActivatedAbility {
            cost: ActivationCost {
                life_cost: life,
                ..Default::default()
            },
            description: format!("Pay {life} life: do nothing"),
            modes: None,

            ..Default::default()
        }
    }

    /// SG-1 (SR-38, CR 118.3 / CR 119.4): the provider must never offer a life-cost
    /// activation the player cannot pay. Before SR-36 the life cost was silently dropped
    /// by the engine, so the provider's optimism was accidentally correct; SF-9 made the
    /// cost real, so an over-optimistic provider now hands the bot an action the engine
    /// rejects with `GameStateError::InsufficientLife`. A single board holds five sources —
    /// two payable, three not — proving both the mana-ability and the non-mana-ability path.
    #[test]
    fn provider_omits_life_costs_the_player_cannot_pay() {
        // Player 1 sits at 1 life. Payable: life_cost 0 and life_cost 1 (1 >= 1). Not
        // payable: life_cost 2 (1 < 2), on both a mana ability and a stack-using ability.
        let state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .active_player(PlayerId(1))
            .player_life(PlayerId(1), 1)
            .object(
                ObjectSpec::land(PlayerId(1), "Free Source")
                    .in_zone(ZoneId::Battlefield)
                    .with_mana_ability(tap_for_black_costing_life(0)),
            )
            .object(
                ObjectSpec::land(PlayerId(1), "Cheap Source")
                    .in_zone(ZoneId::Battlefield)
                    .with_mana_ability(tap_for_black_costing_life(1)),
            )
            .object(
                ObjectSpec::land(PlayerId(1), "Expensive Source")
                    .in_zone(ZoneId::Battlefield)
                    .with_mana_ability(tap_for_black_costing_life(2)),
            )
            .object(
                ObjectSpec::artifact(PlayerId(1), "Cheap Sacrament")
                    .in_zone(ZoneId::Battlefield)
                    .with_activated_ability(activated_costing_life(1)),
            )
            .object(
                ObjectSpec::artifact(PlayerId(1), "Suicidal Engine")
                    .in_zone(ZoneId::Battlefield)
                    .with_activated_ability(activated_costing_life(2)),
            )
            .build()
            .expect("state builds");

        let actions = StubProvider.legal_actions(&state, PlayerId(1));

        let taps_for = |name: &str| {
            let id = id_of(&state, name);
            actions
                .iter()
                .any(|a| matches!(a, LegalAction::TapForMana { source, .. } if *source == id))
        };
        let activates = |name: &str| {
            let id = id_of(&state, name);
            actions
                .iter()
                .any(|a| matches!(a, LegalAction::ActivateAbility { source, .. } if *source == id))
        };

        // CR 119.4b: a life cost of 0 is always payable — offered even though 1 life is low.
        assert!(taps_for("Free Source"), "life_cost 0 must be offered");
        // 1 >= 1: exactly affordable, offered.
        assert!(
            taps_for("Cheap Source"),
            "life_cost == life must be offered"
        );
        // 1 < 2: unpayable, must NOT be offered (would be InsufficientLife, CR 118.3).
        assert!(
            !taps_for("Expensive Source"),
            "a mana ability whose life cost exceeds life must not be offered"
        );
        assert!(
            activates("Cheap Sacrament"),
            "payable life cost must be offered"
        );
        assert!(
            !activates("Suicidal Engine"),
            "a non-mana ability whose life cost exceeds life must not be offered"
        );
    }

    /// CR 119.4b corner: a player at negative life may still activate a `life_cost: 0`
    /// ability. The short-circuit on `life_cost > 0` is what makes this hold — a bare
    /// `life_total >= life_cost as i32` would wrongly reject it at, say, -3 life.
    #[test]
    fn provider_offers_zero_life_cost_at_negative_life() {
        let state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .active_player(PlayerId(1))
            .player_life(PlayerId(1), -3)
            .object(
                ObjectSpec::land(PlayerId(1), "Free Source")
                    .in_zone(ZoneId::Battlefield)
                    .with_mana_ability(tap_for_black_costing_life(0)),
            )
            .object(
                ObjectSpec::artifact(PlayerId(1), "Free Sacrament")
                    .in_zone(ZoneId::Battlefield)
                    .with_activated_ability(activated_costing_life(0)),
            )
            .build()
            .expect("state builds");

        let actions = StubProvider.legal_actions(&state, PlayerId(1));
        let mana_id = id_of(&state, "Free Source");
        let act_id = id_of(&state, "Free Sacrament");
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, LegalAction::TapForMana { source, .. } if *source == mana_id)),
            "life_cost 0 mana ability must be offered even at negative life"
        );
        assert!(
            actions.iter().any(
                |a| matches!(a, LegalAction::ActivateAbility { source, .. } if *source == act_id)
            ),
            "life_cost 0 activated ability must be offered even at negative life"
        );
    }

    /// PB-EF12 (CR 605.3b/106.1b/111.10a, SR-38 precedent): the provider must never offer a
    /// `TapForMana` for an `any_color: true` source without a concrete legal `chosen_color`
    /// (never `None`, never `Colorless`) — the engine now rejects both. Proves the offered
    /// action is engine-legal by actually executing it via `process_command`.
    #[test]
    fn provider_offers_a_concrete_legal_chosen_color_for_any_color_sources() {
        let mut any_color_ability = mtg_engine::ManaAbility::tap_for(mtg_engine::ManaColor::White);
        any_color_ability.any_color = true;
        any_color_ability.produces = Default::default();

        let state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .active_player(PlayerId(1))
            .object(
                ObjectSpec::artifact(PlayerId(1), "Any Color Rock")
                    .in_zone(ZoneId::Battlefield)
                    .with_mana_ability(any_color_ability),
            )
            .build()
            .expect("state builds");

        let actions = StubProvider.legal_actions(&state, PlayerId(1));
        let rock_id = id_of(&state, "Any Color Rock");
        let tap_action = actions
            .iter()
            .find(|a| matches!(a, LegalAction::TapForMana { source, .. } if *source == rock_id))
            .expect("any_color source must be offered");

        let LegalAction::TapForMana { chosen_color, .. } = tap_action else {
            unreachable!("matched by discriminant above");
        };
        assert!(
            chosen_color.is_some(),
            "an any_color ability must be offered with Some(chosen_color), never None (the \
             engine rejects None for an any_color ability)"
        );
        assert_ne!(
            *chosen_color,
            Some(mtg_engine::ManaColor::Colorless),
            "CR 106.1b: colorless is not a legal choice for 'any color' — the engine rejects it"
        );

        // Prove it end-to-end: the emitted Command must actually be accepted by the engine.
        let cmd = mtg_engine::Command::TapForMana {
            player: PlayerId(1),
            source: rock_id,
            ability_index: 0,
            chosen_color: *chosen_color,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        };
        let result = mtg_engine::process_command(state, cmd);
        assert!(
            result.is_ok(),
            "the provider's offered TapForMana must be engine-legal: {:?}",
            result.err()
        );
    }

    /// A `ManaAbility` that taps for one colorless mana but COSTS a `{a/b}` hybrid pip
    /// to activate (a synthetic filter-land-shaped source for isolated testing).
    fn tap_for_colorless_costing_hybrid(
        a: mtg_engine::ManaColor,
        b: mtg_engine::ManaColor,
    ) -> ManaAbility {
        let mut ma = ManaAbility::tap_for(mtg_engine::ManaColor::Colorless);
        ma.mana_cost = Some(ManaCost {
            hybrid: vec![HybridMana::ColorColor(a, b)],
            ..Default::default()
        });
        ma
    }

    /// A `ManaAbility` that taps for one colorless mana but COSTS a `{c/P}` Phyrexian
    /// pip to activate.
    fn tap_for_colorless_costing_phyrexian(c: mtg_engine::ManaColor) -> ManaAbility {
        let mut ma = ManaAbility::tap_for(mtg_engine::ManaColor::Colorless);
        ma.mana_cost = Some(ManaCost {
            phyrexian: vec![PhyrexianMana::Single(c)],
            ..Default::default()
        });
        ma
    }

    /// PB-RS2 §9.6 test 15 (SR-38 precedent, CR 107.4e): a `{B/R}` mana ability with an
    /// EMPTY pool must not be offered — the raw-cost `can_afford` check (before this PB)
    /// would have wrongly offered it, since a pure hybrid pip's standard fields are all
    /// zero. Sibling of `legal_actions.rs`'s existing `chosen_color` test.
    #[test]
    fn provider_never_offers_an_unpayable_pip_ability() {
        let state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .active_player(PlayerId(1))
            .object(
                ObjectSpec::land(PlayerId(1), "Test Filter Land")
                    .in_zone(ZoneId::Battlefield)
                    .with_mana_ability(tap_for_colorless_costing_hybrid(
                        mtg_engine::ManaColor::Black,
                        mtg_engine::ManaColor::Red,
                    )),
            )
            .build()
            .expect("state builds");

        let actions = StubProvider.legal_actions(&state, PlayerId(1));
        let id = id_of(&state, "Test Filter Land");
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, LegalAction::TapForMana { source, .. } if *source == id)),
            "a {{B/R}}-cost mana ability with an EMPTY pool must NOT be offered (CR 107.4e; \
             pre-PB-RS2, can_afford's raw-cost check would wrongly offer it): {actions:?}"
        );
    }

    /// PB-RS2 §9.6 test 16 (CR 104.3b, CR 119.4): a `{G/P}` mana ability with no green
    /// mana available must not be offered as a suicidal life payment. At exactly 2 life,
    /// paying 2 life is LEGAL (CR 119.4, 2 >= 2) but drops the player to 0 — the provider
    /// must not offer it. At 1 life, paying 2 life is ILLEGAL — not offered either, for a
    /// different reason (never conflate the two checks). At 5 life, the plan is safe and
    /// must be offered, engine-verified via `process_command`.
    #[test]
    fn provider_never_offers_a_suicidal_phyrexian_life_plan() {
        let build = |life: i32| {
            GameStateBuilder::new()
                .add_player(PlayerId(1))
                .add_player(PlayerId(2))
                .active_player(PlayerId(1))
                .player_life(PlayerId(1), life)
                .object(
                    ObjectSpec::land(PlayerId(1), "Test Phyrexian Land")
                        .in_zone(ZoneId::Battlefield)
                        .with_mana_ability(tap_for_colorless_costing_phyrexian(
                            mtg_engine::ManaColor::Green,
                        )),
                )
                .build()
                .expect("state builds")
        };

        let offered = |life: i32| -> bool {
            let state = build(life);
            let id = id_of(&state, "Test Phyrexian Land");
            StubProvider
                .legal_actions(&state, PlayerId(1))
                .iter()
                .any(|a| matches!(a, LegalAction::TapForMana { source, .. } if *source == id))
        };

        assert!(
            !offered(2),
            "CR 104.3b: at exactly 2 life, a {{G/P}} ability with no green mana is legal but \
             lethal (drops to 0) — the provider must never offer it"
        );
        assert!(
            !offered(1),
            "CR 119.4: at 1 life, paying 2 life for {{G/P}} is illegal — must not be offered"
        );
        assert!(
            offered(5),
            "at 5 life, a {{G/P}} ability with no green mana has a safe life-payment plan and \
             must be offered"
        );

        // Prove the offered action at 5 life is engine-legal end-to-end.
        let state = build(5);
        let id = id_of(&state, "Test Phyrexian Land");
        let action = StubProvider
            .legal_actions(&state, PlayerId(1))
            .into_iter()
            .find(|a| matches!(a, LegalAction::TapForMana { source, .. } if *source == id))
            .expect("offered at 5 life");
        let LegalAction::TapForMana {
            chosen_color,
            hybrid_choices,
            phyrexian_life_payments,
            ..
        } = action
        else {
            unreachable!("matched by discriminant above");
        };
        assert_eq!(
            phyrexian_life_payments,
            vec![true],
            "with no green mana available, the plan must pay the Phyrexian pip with life"
        );
        let cmd = mtg_engine::Command::TapForMana {
            player: PlayerId(1),
            source: id,
            ability_index: 0,
            chosen_color,
            hybrid_choices,
            phyrexian_life_payments,
        };
        let result = mtg_engine::process_command(state, cmd);
        assert!(
            result.is_ok(),
            "the provider's offered suicide-avoiding TapForMana must be engine-legal: {:?}",
            result.err()
        );
    }

    // ── PB-DP3 (DP-4): `spell_default_modes` / `ability_default_modes` ─────────────
    //
    // CR 601.2b / 602.2b / 700.2a: the engine no longer auto-selects mode 0 for a modal
    // spell or activated ability with an empty `modes_chosen` — a bot must announce a
    // legal mode set itself, or every modal cast/activation it attempts is silently
    // rejected (`driver.rs` answers a rejected command with a silent `PassPriority`, so
    // this would otherwise be an invisible regression in bot action coverage).

    fn dp3_registry() -> std::sync::Arc<mtg_engine::CardRegistry> {
        mtg_engine::CardRegistry::new(mtg_engine::all_cards())
    }

    /// CR 700.2a — `spell_default_modes` returns the first `min_modes` indices in
    /// printed order for a `min_modes: 2` modal spell (Cryptic Command).
    #[test]
    fn test_dp3_spell_default_modes_cryptic_command() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let spell = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Cryptic Command")
                .with_card_id(mtg_engine::CardId("cryptic-command".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        );
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(dp3_registry())
            .object(spell)
            .active_player(p1)
            .build()
            .expect("state builds");
        let card_id = id_of(&state, "Cryptic Command");
        assert_eq!(
            spell_default_modes(&state, card_id),
            vec![0, 1],
            "min_modes: 2 must default to the first two modes in printed order"
        );
    }

    /// CR 700.2a — `spell_default_modes` returns `[0]` for a `min_modes: 1` modal spell
    /// (Crux of Fate).
    #[test]
    fn test_dp3_spell_default_modes_min_one() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let spell = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Crux of Fate")
                .with_card_id(mtg_engine::CardId("crux-of-fate".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        );
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(dp3_registry())
            .object(spell)
            .active_player(p1)
            .build()
            .expect("state builds");
        let card_id = id_of(&state, "Crux of Fate");
        assert_eq!(
            spell_default_modes(&state, card_id),
            vec![0],
            "min_modes: 1 must default to mode 0"
        );
    }

    /// CR 601.2b — `spell_default_modes` returns `[]` for a non-modal card, so the
    /// PB-DP3 change is a no-op for every non-modal cast (Lightning Bolt has no
    /// `ModeSelection` at all).
    #[test]
    fn test_dp3_spell_default_modes_non_modal_is_empty() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let spell = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Lightning Bolt")
                .with_card_id(mtg_engine::CardId("lightning-bolt".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        );
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(dp3_registry())
            .object(spell)
            .active_player(p1)
            .build()
            .expect("state builds");
        let card_id = id_of(&state, "Lightning Bolt");
        assert_eq!(
            spell_default_modes(&state, card_id),
            Vec::<usize>::new(),
            "a non-modal card must yield an empty mode list (no-op for non-modal casts)"
        );
    }

    /// CR 613.1f / 700.2a — `ability_default_modes` reads the LAYER-RESOLVED
    /// `activated_abilities` list (via `calculate_characteristics`), not `def.abilities`
    /// directly. Umezawa's Jitte's sole activated ability is modal (`min_modes: 1`) and
    /// sits at layer-resolved index 0 (`JITTE_MODAL_ABILITY_INDEX` in
    /// `pb_os10_singleton_cleanup.rs`).
    #[test]
    fn test_dp3_ability_default_modes_uses_layer_resolved_index() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let jitte = mtg_engine::enrich_spec_from_def(
            ObjectSpec::artifact(p1, "Umezawa's Jitte")
                .with_card_id(mtg_engine::CardId("umezawas-jitte".to_string()))
                .in_zone(ZoneId::Battlefield),
            &defs,
        );
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(dp3_registry())
            .object(jitte)
            .active_player(p1)
            .build()
            .expect("state builds");
        let source = id_of(&state, "Umezawa's Jitte");
        const JITTE_MODAL_ABILITY_INDEX: usize = 0;
        assert_eq!(
            ability_default_modes(&state, source, JITTE_MODAL_ABILITY_INDEX),
            vec![0],
            "min_modes: 1 must default to mode 0, read via calculate_characteristics"
        );
    }

    /// Review finding #5: the pool-only hybrid-half heuristic in
    /// `build_hybrid_choices`/`resolve_hybrid_phyrexian_plan` must not produce a false
    /// negative when the pool covers NEITHER half of a `{B/R}` pip but an UNTAPPED
    /// source can pay one of the halves. The player controls a `{B/R}`-cost filter land
    /// (empty pool, so the primary preference defaults to Black — the pip's first
    /// listed color) and an untapped Mountain (produces Red, not Black). Before the
    /// fix, the primary plan (Black) fails `can_afford` and the whole action is
    /// withheld even though the Red half is payable via the Mountain. After the fix,
    /// the fallback (flipped) plan tries Red and the action IS offered.
    #[test]
    fn provider_offers_the_payable_hybrid_half_when_only_the_other_is_in_pool_preference() {
        let state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .active_player(PlayerId(1))
            .object(
                ObjectSpec::land(PlayerId(1), "Test Filter Land")
                    .in_zone(ZoneId::Battlefield)
                    .with_mana_ability(tap_for_colorless_costing_hybrid(
                        mtg_engine::ManaColor::Black,
                        mtg_engine::ManaColor::Red,
                    )),
            )
            .object(
                ObjectSpec::land(PlayerId(1), "Test Mountain")
                    .in_zone(ZoneId::Battlefield)
                    .with_mana_ability(ManaAbility::tap_for(mtg_engine::ManaColor::Red)),
            )
            .build()
            .expect("state builds");

        let id = id_of(&state, "Test Filter Land");
        let action = StubProvider
            .legal_actions(&state, PlayerId(1))
            .into_iter()
            .find(|a| matches!(a, LegalAction::TapForMana { source, .. } if *source == id));
        assert!(
            action.is_some(),
            "a {{B/R}} filter land must be offered when an untapped Mountain can pay the \
             Red half, even though the pool-preference heuristic defaults to Black \
             (finding #5): {action:?}"
        );
        let LegalAction::TapForMana { hybrid_choices, .. } = action.expect("checked Some") else {
            unreachable!("matched by discriminant above");
        };
        assert_eq!(
            hybrid_choices,
            vec![HybridManaPayment::Color(mtg_engine::ManaColor::Red)],
            "the fallback plan must choose the payable half (Red), not the pool-preferred \
             but unpayable half (Black)"
        );
    }

    // ── PB-DP4 / DP-11: payment LegalActions ────────────────────────────────────────

    /// CR 702.30a (PB-DP4 / DP-11): both `pay: true` and `pay: false` are offered for an
    /// affordable outstanding echo payment.
    #[test]
    fn provider_offers_both_echo_branches_when_the_cost_is_affordable() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .object(ObjectSpec::creature(p1, "Echo Permanent", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds");
        let perm = id_of(&state, "Echo Permanent");
        state.pending_echo_payments_mut().push_back((
            p1,
            perm,
            ManaCost {
                generic: 2,
                ..Default::default()
            },
        ));
        state.players_mut().get_mut(&p1).unwrap().mana_pool = mtg_engine::ManaPool {
            colorless: 2,
            ..Default::default()
        };

        let actions = StubProvider.legal_actions(&state, p1);
        assert!(
            actions.iter().any(
                |a| matches!(a, LegalAction::PayEcho { permanent, pay: true } if *permanent == perm)
            ),
            "an affordable echo payment must offer pay: true"
        );
        assert!(
            actions.iter().any(
                |a| matches!(a, LegalAction::PayEcho { permanent, pay: false } if *permanent == perm)
            ),
            "declining is always legal (CR 118.12a)"
        );
    }

    /// SR-38: the provider must never offer `PayEcho { pay: true }` when the engine would
    /// reject it for insufficient mana.
    #[test]
    fn provider_omits_echo_pay_when_the_cost_is_unaffordable() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .object(ObjectSpec::creature(p1, "Echo Permanent", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds");
        let perm = id_of(&state, "Echo Permanent");
        state.pending_echo_payments_mut().push_back((
            p1,
            perm,
            ManaCost {
                generic: 2,
                ..Default::default()
            },
        ));
        // Pool is empty by default.

        let actions = StubProvider.legal_actions(&state, p1);
        assert!(
            !actions.iter().any(
                |a| matches!(a, LegalAction::PayEcho { permanent, pay: true } if *permanent == perm)
            ),
            "SR-38: must not offer a payment the engine would reject"
        );
        assert!(
            actions.iter().any(
                |a| matches!(a, LegalAction::PayEcho { permanent, pay: false } if *permanent == perm)
            ),
            "decline must still be offered"
        );
    }

    /// CR 702.24b (PB-DP4 / DP-11): the cumulative upkeep mana gate multiplies
    /// `per_counter_cost` by the permanent's TOTAL age counter count, not a fixed amount.
    #[test]
    fn provider_gates_cumulative_upkeep_mana_on_age_counter_multiple() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let build = |pool_colorless: u32| {
            let mut state = GameStateBuilder::new()
                .add_player(p1)
                .add_player(p2)
                .active_player(p1)
                .object(
                    ObjectSpec::creature(p1, "CU Permanent", 2, 2)
                        .in_zone(ZoneId::Battlefield)
                        .with_counter(CounterType::Age, 3),
                )
                .build()
                .expect("state builds");
            let perm = id_of(&state, "CU Permanent");
            state.pending_cumulative_upkeep_payments_mut().push_back((
                p1,
                perm,
                mtg_engine::CumulativeUpkeepCost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
            ));
            state.players_mut().get_mut(&p1).unwrap().mana_pool = mtg_engine::ManaPool {
                colorless: pool_colorless,
                ..Default::default()
            };
            (state, perm)
        };

        let (state_underfunded, perm) = build(2);
        let actions = StubProvider.legal_actions(&state_underfunded, p1);
        assert!(
            !actions.iter().any(
                |a| matches!(a, LegalAction::PayCumulativeUpkeep { permanent, pay: true } if *permanent == perm)
            ),
            "3 age counters x {{1}} = {{3}}; a pool of 2 must not offer pay: true"
        );

        let (state_funded, perm2) = build(3);
        let actions2 = StubProvider.legal_actions(&state_funded, p1);
        assert!(
            actions2.iter().any(
                |a| matches!(a, LegalAction::PayCumulativeUpkeep { permanent, pay: true } if *permanent == perm2)
            ),
            "a pool of exactly 3 must offer pay: true"
        );
    }

    /// CR 119.4 (PB-DP4 / DP-11): the cumulative upkeep life gate mirrors
    /// `engine.rs`'s Change 2e affordability check.
    #[test]
    fn provider_gates_cumulative_upkeep_life_on_life_total() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let build = |life_total: i32| {
            let mut state = GameStateBuilder::new()
                .add_player(p1)
                .add_player(p2)
                .active_player(p1)
                .player_life(p1, life_total)
                .object(
                    ObjectSpec::creature(p1, "CU Life Permanent", 2, 2)
                        .in_zone(ZoneId::Battlefield)
                        .with_counter(CounterType::Age, 2),
                )
                .build()
                .expect("state builds");
            let perm = id_of(&state, "CU Life Permanent");
            state.pending_cumulative_upkeep_payments_mut().push_back((
                p1,
                perm,
                mtg_engine::CumulativeUpkeepCost::Life(3),
            ));
            (state, perm)
        };

        // 2 age counters x 3 life = 6 owed.
        let (state_poor, perm) = build(5);
        let actions = StubProvider.legal_actions(&state_poor, p1);
        assert!(
            !actions.iter().any(
                |a| matches!(a, LegalAction::PayCumulativeUpkeep { permanent, pay: true } if *permanent == perm)
            ),
            "CR 119.4: 5 life cannot pay a 6-life cost"
        );

        let (state_rich, perm2) = build(6);
        let actions2 = StubProvider.legal_actions(&state_rich, p1);
        assert!(
            actions2.iter().any(
                |a| matches!(a, LegalAction::PayCumulativeUpkeep { permanent, pay: true } if *permanent == perm2)
            ),
            "6 life exactly covers a 6-life cost"
        );
    }

    /// CR 119.4b (fix cycle, T7): a `Life(0)` cost is offered as `pay: true` even at a
    /// negative life total, matching engine.rs's `if total_life > 0 { check }` guard
    /// (which never runs the affordability check at all for a zero-cost payment).
    /// Pre-fix: `life_total >= (amount * age_count) as i32` compared -1 >= 0, rejecting
    /// a payment the engine always accepts -- a real divergence, not a conservative one.
    #[test]
    fn provider_offers_cumulative_upkeep_zero_life_cost_even_at_negative_life_total() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .player_life(p1, -3)
            .object(
                ObjectSpec::creature(p1, "CU Zero Life Permanent", 2, 2)
                    .in_zone(ZoneId::Battlefield)
                    .with_counter(CounterType::Age, 5),
            )
            .build()
            .expect("state builds");
        let perm = id_of(&state, "CU Zero Life Permanent");
        state.pending_cumulative_upkeep_payments_mut().push_back((
            p1,
            perm,
            mtg_engine::CumulativeUpkeepCost::Life(0),
        ));

        let actions = StubProvider.legal_actions(&state, p1);
        assert!(
            actions.iter().any(
                |a| matches!(a, LegalAction::PayCumulativeUpkeep { permanent, pay: true } if *permanent == perm)
            ),
            "CR 119.4b: a life cost of 0 is always payable, even at a negative life total"
        );
    }

    /// CR 702.59a (PB-DP4 / DP-11): decline is always offered for a recover payment,
    /// regardless of affordability.
    #[test]
    fn provider_offers_recover_decline_always() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .object(ObjectSpec::card(p1, "Recover Card").in_zone(ZoneId::Graveyard(p1)))
            .build()
            .expect("state builds");
        let card = id_of(&state, "Recover Card");
        state.pending_recover_payments_mut().push_back((
            p1,
            card,
            ManaCost {
                generic: 2,
                ..Default::default()
            },
        ));
        // Pool is empty by default -- pay: true must be absent.

        let actions = StubProvider.legal_actions(&state, p1);
        assert!(
            actions.iter().any(
                |a| matches!(a, LegalAction::PayRecover { recover_card, pay: false } if *recover_card == card)
            ),
            "decline must always be offered"
        );
        assert!(
            !actions.iter().any(
                |a| matches!(a, LegalAction::PayRecover { recover_card, pay: true } if *recover_card == card)
            ),
            "SR-38: pay: true must not be offered when unaffordable"
        );
    }

    /// CR 608.2g (PB-DP4 / DP-11): a pending payment does not exclude ordinary
    /// priority-window actions like TapForMana -- the player may fund the payment first.
    #[test]
    fn provider_still_offers_tap_for_mana_alongside_a_pending_payment() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .object(ObjectSpec::creature(p1, "Echo Permanent", 2, 2).in_zone(ZoneId::Battlefield))
            .object(
                ObjectSpec::land(p1, "Untapped Land")
                    .in_zone(ZoneId::Battlefield)
                    .with_mana_ability(ManaAbility::tap_for(mtg_engine::ManaColor::Colorless)),
            )
            .build()
            .expect("state builds");
        let perm = id_of(&state, "Echo Permanent");
        state.pending_echo_payments_mut().push_back((
            p1,
            perm,
            ManaCost {
                generic: 1,
                ..Default::default()
            },
        ));

        let actions = StubProvider.legal_actions(&state, p1);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, LegalAction::PayEcho { permanent, .. } if *permanent == perm)),
            "the payment must be offered"
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, LegalAction::TapForMana { .. })),
            "CR 608.2g: TapForMana must remain available alongside a pending payment"
        );
    }

    /// A player's pending payment must not leak into another player's legal-action list.
    #[test]
    fn provider_offers_no_payment_action_to_a_player_who_owes_nothing() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p2)
            .object(ObjectSpec::creature(p1, "Echo Permanent", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds");
        let perm = id_of(&state, "Echo Permanent");
        state.pending_echo_payments_mut().push_back((
            p1,
            perm,
            ManaCost {
                generic: 2,
                ..Default::default()
            },
        ));
        state.turn_mut().priority_holder = Some(p2);

        let actions = StubProvider.legal_actions(&state, p2);
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, LegalAction::PayEcho { .. })),
            "p1's pending echo payment must not appear in p2's legal-action list"
        );
    }

    /// `action_to_command` must round-trip all three payment `LegalAction`s to the
    /// matching `Command`, preserving the `pay` flag.
    #[test]
    fn action_to_command_round_trips_the_three_payment_actions() {
        use rand::SeedableRng;
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .build()
            .expect("state builds");
        let permanent = ObjectId(1);
        let recover_card = ObjectId(2);
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);

        let echo_cmd = crate::random_bot::action_to_command(
            &mut rng,
            &state,
            p1,
            &LegalAction::PayEcho {
                permanent,
                pay: true,
            },
        );
        assert!(matches!(
            echo_cmd,
            mtg_engine::Command::PayEcho { player, permanent: perm, pay: true } if player == p1 && perm == permanent
        ));

        let cu_cmd = crate::random_bot::action_to_command(
            &mut rng,
            &state,
            p1,
            &LegalAction::PayCumulativeUpkeep {
                permanent,
                pay: false,
            },
        );
        assert!(matches!(
            cu_cmd,
            mtg_engine::Command::PayCumulativeUpkeep { player, permanent: perm, pay: false } if player == p1 && perm == permanent
        ));

        let recover_cmd = crate::random_bot::action_to_command(
            &mut rng,
            &state,
            p1,
            &LegalAction::PayRecover {
                recover_card,
                pay: true,
            },
        );
        assert!(matches!(
            recover_cmd,
            mtg_engine::Command::PayRecover { player, recover_card: rc, pay: true } if player == p1 && rc == recover_card
        ));
    }

    // ── PB-DP7 / DP-3 (T16): StubProvider offers only the discard while blocked ──

    /// CR 514.1 (PB-DP7 / DP-3): while a cleanup discard is pending, the
    /// blocked player is offered EXACTLY one action (`DiscardToHandSize`,
    /// `count` cards drawn from `hand`), every other player is offered
    /// nothing, and the offered `cards` is accepted by `process_command`
    /// verbatim (SR-38: never offer an action the engine rejects).
    ///
    /// Fix-cycle Finding 17 (LOW): rebuilt to reach the pause LEGITIMATELY by
    /// driving real `PassPriority` commands through `Step::End` into
    /// `Step::Cleanup`. The old version called `cleanup_actions` directly at
    /// `Step::End`, which recorded an entry in a state the engine can never
    /// actually produce (`turn.step == End` with `pending_cleanup_discard`
    /// set) -- and then submitted `DiscardToHandSize`, which was ACCEPTED and
    /// re-entered `enter_step` at `Step::End`, silently exercising exactly the
    /// hazard fix-cycle Finding 2 closed (a command that re-runs whatever
    /// turn-based actions belong to the CURRENT step, not the step the entry
    /// was recorded in).
    #[test]
    fn provider_offers_only_the_discard_while_blocked() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut builder = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .at_step(mtg_engine::Step::End);
        for i in 0..9u32 {
            builder = builder
                .object(ObjectSpec::card(p1, &format!("Card {i}")).in_zone(ZoneId::Hand(p1)));
        }
        let state = builder.build().expect("state builds");

        let (state, _) =
            mtg_engine::process_command(state, mtg_engine::Command::PassPriority { player: p1 })
                .expect("p1 passes at End");
        let (state, _) =
            mtg_engine::process_command(state, mtg_engine::Command::PassPriority { player: p2 })
                .expect("p2 passes at End");

        assert_eq!(state.turn().step, mtg_engine::Step::Cleanup);
        let entry = state
            .pending_cleanup_discard()
            .expect("cleanup discard should be pending");
        assert_eq!(entry.player, p1);
        assert_eq!(entry.count, 2);

        // The blocked player gets exactly one action.
        let p1_actions = StubProvider.legal_actions(&state, p1);
        assert_eq!(p1_actions.len(), 1);
        let (count, hand, cards) = match &p1_actions[0] {
            LegalAction::DiscardToHandSize { count, hand, cards } => {
                (*count, hand.clone(), cards.clone())
            }
            other => panic!("expected DiscardToHandSize, got {other:?}"),
        };
        assert_eq!(count, 2);
        assert_eq!(cards.len(), 2);
        for id in &cards {
            assert!(hand.contains(id), "every offered card must be in `hand`");
        }

        // Every other player gets nothing (CR 514.3: no priority in cleanup).
        assert!(StubProvider.legal_actions(&state, p2).is_empty());

        // The offered subset is accepted by process_command verbatim.
        let result = mtg_engine::process_command(
            state,
            mtg_engine::Command::DiscardToHandSize { player: p1, cards },
        );
        assert!(
            result.is_ok(),
            "the provider's offered action must be accepted: {:?}",
            result.err()
        );
    }

    // ── UI-2 (CR 118.8 / CR 702.157): additional-cost surfacing ─────────────────

    fn ui2_registry() -> std::sync::Arc<mtg_engine::CardRegistry> {
        mtg_engine::CardRegistry::new(mtg_engine::all_cards())
    }

    /// T1: `eligible_spell_sacrifice_targets` mirrors each of the five
    /// `SpellAdditionalCost` filters, gate for gate with `casting.rs:3300-3369`.
    /// `SacrificeSubtype`/`SacrificeColorPermanent` are each checked against a
    /// permanent whose subtype/color is granted ONLY by a layer-resolved continuous
    /// effect -- a raw `obj.characteristics` read would miss it, which is exactly
    /// the mistake §1.2 warns against.
    #[test]
    fn eligible_spell_sacrifice_targets_mirrors_each_of_the_five_filters() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .object(ObjectSpec::creature(p1, "P1 Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .object(ObjectSpec::land(p1, "P1 Land").in_zone(ZoneId::Battlefield))
            .object(ObjectSpec::artifact(p1, "P1 Sword").in_zone(ZoneId::Battlefield))
            .object(
                ObjectSpec::creature(p1, "P1 Printed Goblin", 1, 1)
                    .in_zone(ZoneId::Battlefield)
                    .with_subtypes(vec![SubType("Goblin".to_string())]),
            )
            .object(ObjectSpec::artifact(p1, "P1 Granted Goblin").in_zone(ZoneId::Battlefield))
            .object(
                ObjectSpec::artifact(p1, "P1 Printed Blue")
                    .in_zone(ZoneId::Battlefield)
                    .with_colors(vec![Color::Blue]),
            )
            .object(ObjectSpec::artifact(p1, "P1 Granted Blue").in_zone(ZoneId::Battlefield))
            .object(ObjectSpec::creature(p2, "P2 Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds");

        // Grant the subtype/color via LAYER-RESOLVED continuous effects only --
        // neither object's printed `characteristics` carries it.
        let granted_goblin = id_of(&state, "P1 Granted Goblin");
        let granted_blue = id_of(&state, "P1 Granted Blue");
        state.continuous_effects_mut().push_back(ContinuousEffect {
            id: EffectId(9001),
            source: None,
            timestamp: 0,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::SingleObject(granted_goblin),
            modification: LayerModification::AddSubtypes(
                [SubType("Goblin".to_string())].into_iter().collect(),
            ),
            is_cda: false,
            condition: None,
            affected_set: None,
        });
        state.continuous_effects_mut().push_back(ContinuousEffect {
            id: EffectId(9002),
            source: None,
            timestamp: 1,
            layer: EffectLayer::ColorChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::SingleObject(granted_blue),
            modification: LayerModification::AddColors([Color::Blue].into_iter().collect()),
            is_cda: false,
            condition: None,
            affected_set: None,
        });

        let bear = id_of(&state, "P1 Bear");
        let land = id_of(&state, "P1 Land");
        let sword = id_of(&state, "P1 Sword");
        let printed_goblin = id_of(&state, "P1 Printed Goblin");
        let printed_blue = id_of(&state, "P1 Printed Blue");
        let p2_bear = id_of(&state, "P2 Bear");

        let creature =
            eligible_spell_sacrifice_targets(&state, p1, &SpellAdditionalCost::SacrificeCreature);
        assert!(creature.contains(&bear) && creature.contains(&printed_goblin));
        assert!(
            !creature.contains(&land) && !creature.contains(&sword) && !creature.contains(&p2_bear),
            "SacrificeCreature must exclude non-creatures and P2's creature: {creature:?}"
        );

        let land_set =
            eligible_spell_sacrifice_targets(&state, p1, &SpellAdditionalCost::SacrificeLand);
        assert_eq!(land_set, vec![land]);

        let artifact_or_creature = eligible_spell_sacrifice_targets(
            &state,
            p1,
            &SpellAdditionalCost::SacrificeArtifactOrCreature,
        );
        for id in [
            bear,
            printed_goblin,
            sword,
            granted_goblin,
            printed_blue,
            granted_blue,
        ] {
            assert!(
                artifact_or_creature.contains(&id),
                "{id:?} is an artifact or a creature and must be eligible"
            );
        }
        assert!(!artifact_or_creature.contains(&land));

        let goblin = eligible_spell_sacrifice_targets(
            &state,
            p1,
            &SpellAdditionalCost::SacrificeSubtype(SubType("Goblin".to_string())),
        );
        assert!(
            goblin.contains(&printed_goblin) && goblin.contains(&granted_goblin),
            "both the printed AND the layer-GRANTED Goblin must be eligible: {goblin:?}"
        );
        assert!(!goblin.contains(&bear));

        let blue = eligible_spell_sacrifice_targets(
            &state,
            p1,
            &SpellAdditionalCost::SacrificeColorPermanent(Color::Blue),
        );
        assert!(
            blue.contains(&printed_blue) && blue.contains(&granted_blue),
            "both the printed AND the layer-GRANTED blue permanent must be eligible: {blue:?}"
        );
        assert!(!blue.contains(&sword));
    }

    /// T2: a permanent under `GameRestriction::CantBeSacrificed` is excluded from
    /// `eligible_spell_sacrifice_targets`, mirroring PB-AC8 / CR 701.21a.
    #[test]
    fn cant_be_sacrificed_permanent_is_excluded() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .object(ObjectSpec::creature(p1, "Protected Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .object(ObjectSpec::creature(p1, "Plain Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds");

        let protected = id_of(&state, "Protected Bear");
        let plain = id_of(&state, "Plain Bear");
        state
            .restrictions_mut()
            .push_back(mtg_engine::state::ActiveRestriction {
                source: protected,
                controller: p1,
                restriction: GameRestriction::CantBeSacrificed,
            });

        let eligible =
            eligible_spell_sacrifice_targets(&state, p1, &SpellAdditionalCost::SacrificeCreature);
        assert_eq!(
            eligible,
            vec![plain],
            "the CantBeSacrificed creature must be excluded: {eligible:?}"
        );
    }

    /// T3 (SR-38 criterion 5999): with the only creature ineligible (an opponent's,
    /// so it fails the CONTROLLER check), the `CastSpell` offer for a mandatory-
    /// sacrifice spell is SUPPRESSED ENTIRELY -- not merely offered with an empty
    /// eligible set. Two-sided: once an eligible creature exists, the SAME action IS
    /// offered, proving the suppression is conditional and not a blanket omission of
    /// Life's Legacy from the hand loop.
    #[test]
    fn offer_is_suppressed_with_no_eligible_sacrifice_then_offered_once_one_exists() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let make_lifes_legacy = || {
            mtg_engine::enrich_spec_from_def(
                ObjectSpec::card(p1, "Life's Legacy")
                    .with_card_id(mtg_engine::CardId("lifes-legacy".to_string()))
                    .in_zone(ZoneId::Hand(p1)),
                &defs,
            )
        };
        // {1}{G}: green >= 1 and total >= 2.
        let lifes_legacy_pool = || ManaPool {
            green: 1,
            white: 1,
            ..Default::default()
        };
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(ui2_registry())
            .active_player(p1)
            .player_mana(p1, lifes_legacy_pool())
            .object(make_lifes_legacy())
            // Only P2's creature exists -- fails the CONTROLLER check, so
            // `eligible` is empty and the whole offer must be suppressed.
            .object(ObjectSpec::creature(p2, "Opponent's Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds");

        let card_id = id_of(&state, "Life's Legacy");
        let actions = StubProvider.legal_actions(&state, p1);
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == card_id)),
            "with no eligible sacrifice the CastSpell offer must be absent entirely: {actions:?}"
        );

        // Now add an eligible creature P1 controls and rebuild.
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(ui2_registry())
            .active_player(p1)
            .player_mana(p1, lifes_legacy_pool())
            .object(make_lifes_legacy())
            .object(ObjectSpec::creature(p2, "Opponent's Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .object(ObjectSpec::creature(p1, "My Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds");
        let card_id = id_of(&state, "Life's Legacy");
        let my_bear = id_of(&state, "My Bear");
        let actions = StubProvider.legal_actions(&state, p1);
        let action = actions
            .iter()
            .find(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == card_id))
            .expect("with an eligible creature the CastSpell offer must be present");
        let LegalAction::CastSpell {
            additional_costs, ..
        } = action
        else {
            unreachable!("matched by discriminant above");
        };
        let sac = additional_costs
            .sacrifice
            .as_ref()
            .expect("Life's Legacy declares a required sacrifice");
        assert_eq!(sac.eligible, vec![my_bear]);
        assert_eq!(sac.default, my_bear);
    }

    /// T4: a spell with no `spell_additional_costs` and no Squad ability gets
    /// `AdditionalCostPlan::default()` (both fields `None`) and is otherwise
    /// unaffected -- the no-op case that is nearly every spell in the corpus.
    #[test]
    fn build_additional_cost_plan_is_default_for_a_plain_spell() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let bolt = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Lightning Bolt")
                .with_card_id(mtg_engine::CardId("lightning-bolt".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        );
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(ui2_registry())
            .active_player(p1)
            .object(bolt)
            .build()
            .expect("state builds");
        let card_id = id_of(&state, "Lightning Bolt");
        let obj = state.object(card_id).expect("object exists");
        let plan = build_additional_cost_plan(&state, p1, obj);
        assert!(plan.sacrifice.is_none());
        assert!(plan.squad.is_none());
    }

    /// **Fix cycle (review Issue 1), and the answer is not the one the review
    /// expected.** The reviewer's premise -- that `squad_max_count` is pool-blind --
    /// is wrong: `can_afford` checks the pool BEFORE reaching the solver, and
    /// `LocalGame::auto_tap_commands_for` takes the same two branches, so the offer
    /// and the payment path cannot disagree.
    ///
    /// What IS real is one layer down, and this test is its historical record:
    /// `mana_solver`'s Phase 3 pays **one generic pip per SOURCE**
    /// (`remaining.generic -= 1` after tapping a source regardless of what it
    /// produces), so a Sol Ring counts as one mana. That is playtest triage **F4**,
    /// pre-existing and open. `squad_max_count`'s own upper bound now SUMS each
    /// source's real output rather than counting sources -- which is sound, and is
    /// the right direction, but is **not observable today** because the solver's cap
    /// binds first.
    ///
    /// So this asserts **0**, not 1, and says why. It is a pin on a defect, not on a
    /// design: when F4 closes, this test must go red and be changed to 1. Writing it
    /// the other way round -- asserting 1 and marking it `#[ignore]` -- would have
    /// hidden which layer is actually wrong. Filed as `OOS-UI2-3`.
    #[test]
    fn squad_max_count_counts_true_production_now_that_f4_is_closed() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let honour_guard = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Ultramarines Honour Guard")
                .with_card_id(mtg_engine::CardId("ultramarines-honour-guard".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        );
        // Base {3}{W} = 4, Squad {2} = 2 per copy. One Plains ({W}) plus three
        // two-mana rocks is SEVEN mana by the rules, which covers the base plus one
        // extra copy ({W} + 5 generic). The solver sees FOUR.
        let two_mana = || ManaAbility {
            produces: [(ManaColor::Colorless, 2u32)].into_iter().collect(),
            requires_tap: true,
            ..Default::default()
        };
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(ui2_registry())
            .active_player(p1)
            .object(honour_guard)
            .object(
                ObjectSpec::land(p1, "White Source").with_mana_ability(ManaAbility {
                    produces: [(ManaColor::White, 1u32)].into_iter().collect(),
                    requires_tap: true,
                    ..Default::default()
                }),
            )
            .object(ObjectSpec::artifact(p1, "Rock A").with_mana_ability(two_mana()))
            .object(ObjectSpec::artifact(p1, "Rock B").with_mana_ability(two_mana()))
            .object(ObjectSpec::artifact(p1, "Rock C").with_mana_ability(two_mana()))
            .build()
            .expect("state builds");
        let card_id = id_of(&state, "Ultramarines Honour Guard");
        let obj = state.object(card_id).expect("object exists");
        let plan = build_additional_cost_plan(&state, p1, obj);
        let squad = plan.squad.as_ref().expect("Squad must be detected");
        assert_eq!(
            squad.max_count, 1,
            "playtest triage F4 is CLOSED (SIM-2, `scutemob-176`): the solver counts \
             true production, so one Plains plus three two-mana rocks is seven mana, \
             covering base {{3}}{{W}} plus one {{2}} Squad copy. This test was authored \
             pinning the pre-fix 0 with the instruction to flip it to 1 when F4 \
             closed; the flip happened at the UI-2/SIM-2 merge (`scutemob-178` \
             collect). Do not delete the test."
        );
        // The offer itself is NOT suppressed: Squad is optional, so an unaffordable
        // extra copy never stops the spell being cast (CR 702.157a).
        assert!(
            StubProvider
                .legal_actions(&state, p1)
                .iter()
                .any(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == card_id)),
            "Squad's max_count == 0 must not suppress the cast"
        );
    }

    /// **Fix cycle (review Issue 2): `effective_cast_cost_with_additional` must take
    /// the LAST `Squad` entry, not the sum** -- `casting.rs`'s own destructuring
    /// loop is a plain assignment (`squad_count = *count`), so a command carrying
    /// two entries is charged for the last one only.
    ///
    /// Summing made the auto-tap reach for strictly more mana than the engine
    /// charges: the solver then found no plan, no taps were issued, and the engine
    /// refused the cast for want of mana -- a 422 after a clean offer, which is the
    /// SR-38 shape this whole batch exists to delete.
    #[test]
    fn squad_cost_takes_the_last_announced_entry_exactly_as_the_engine_does() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let honour_guard = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Ultramarines Honour Guard")
                .with_card_id(mtg_engine::CardId("ultramarines-honour-guard".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        );
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(ui2_registry())
            .active_player(p1)
            .object(honour_guard)
            .build()
            .expect("state builds");
        let card_id = id_of(&state, "Ultramarines Honour Guard");

        // Base {3}{W} has mana value 4; Squad {2} adds 2 per payment.
        let one = effective_cast_cost_with_additional(
            &state,
            p1,
            card_id,
            &[AdditionalCost::Squad { count: 1 }],
        )
        .expect("has a mana cost");
        assert_eq!(one.mana_value(), 6);

        // Two entries: 2 then 1. LAST wins -> +2, total 6. Summing would give +6.
        let last_wins = effective_cast_cost_with_additional(
            &state,
            p1,
            card_id,
            &[
                AdditionalCost::Squad { count: 2 },
                AdditionalCost::Squad { count: 1 },
            ],
        )
        .expect("has a mana cost");
        assert_eq!(
            last_wins.mana_value(),
            6,
            "the LAST entry (count 1) is what casting.rs charges; summing would give 10"
        );

        // And the other order, so this cannot pass by coincidence of which is smaller.
        let last_wins_other = effective_cast_cost_with_additional(
            &state,
            p1,
            card_id,
            &[
                AdditionalCost::Squad { count: 1 },
                AdditionalCost::Squad { count: 2 },
            ],
        )
        .expect("has a mana cost");
        assert_eq!(last_wins_other.mana_value(), 8);
    }

    /// **Fix cycle (review Issue 4): the COMMAND-ZONE loop suppresses too.**
    ///
    /// The two cast loops now share `offerable_cast_plan`, and this is the test the
    /// command-zone copy never had. A commander with a CR 118.8 additional cost is a
    /// shape no card in the corpus has, so the fixture builds one directly: the
    /// object is a real registry card (Life's Legacy, which declares
    /// `SacrificeCreature`) placed in `ZoneId::Command(p1)` and registered in
    /// `commander_ids`, which is exactly what `casting.rs` keys CR 903.8 on.
    #[test]
    fn command_zone_cast_is_suppressed_with_no_eligible_sacrifice() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let build = |with_creature: bool| {
            let mut b = GameStateBuilder::new()
                .add_player(p1)
                .add_player(p2)
                .with_registry(ui2_registry())
                .active_player(p1)
                .player_commander(p1, mtg_engine::CardId("lifes-legacy".to_string()))
                .player_mana(
                    p1,
                    ManaPool {
                        green: 1,
                        white: 1,
                        ..Default::default()
                    },
                )
                .object(mtg_engine::enrich_spec_from_def(
                    ObjectSpec::card(p1, "Life's Legacy")
                        .with_card_id(mtg_engine::CardId("lifes-legacy".to_string()))
                        .in_zone(ZoneId::Command(p1)),
                    &defs,
                ));
            if with_creature {
                b = b
                    .object(ObjectSpec::creature(p1, "My Bear", 2, 2).in_zone(ZoneId::Battlefield));
            }
            b.build().expect("state builds")
        };

        let without = build(false);
        let card_id = id_of(&without, "Life's Legacy");
        assert!(
            !StubProvider
                .legal_actions(&without, p1)
                .iter()
                .any(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == card_id)),
            "SR-38: a command-zone cast whose CR 118.8 sacrifice has no eligible \
             permanent must not be offered either"
        );

        let with = build(true);
        let card_id = id_of(&with, "Life's Legacy");
        let action = StubProvider
            .legal_actions(&with, p1)
            .into_iter()
            .find(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == card_id))
            .expect("two-sided: with an eligible creature the command-zone cast IS offered");
        let LegalAction::CastSpell {
            from_zone,
            additional_costs,
            ..
        } = &action
        else {
            unreachable!("matched by discriminant above");
        };
        assert_eq!(*from_zone, ZoneId::Command(p1));
        assert!(additional_costs.sacrifice.is_some());
    }

    /// T5: Squad's `max_count` is a REAL bound derived from what the player can
    /// actually afford, not an arbitrary cap. Ultramarines Honour Guard is `{3}{W}`
    /// with Squad `{2}`; its all-generic squad cost keeps the arithmetic below
    /// readable, which is the only reason it is preferred here over the corpus's
    /// other Squad card (`galadhrim-brigade`, `{2}{G}` + Squad `{1}{G}`, repaired by
    /// UI-2 and exercised end to end by the play-server probes instead).
    #[test]
    fn squad_max_count_is_a_real_bound_not_an_arbitrary_cap() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let defs = mtg_engine::all_cards()
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let make_state = |pool: ManaPool| {
            let honour_guard = mtg_engine::enrich_spec_from_def(
                ObjectSpec::card(p1, "Ultramarines Honour Guard")
                    .with_card_id(mtg_engine::CardId("ultramarines-honour-guard".to_string()))
                    .in_zone(ZoneId::Hand(p1)),
                &defs,
            );
            GameStateBuilder::new()
                .add_player(p1)
                .add_player(p2)
                .with_registry(ui2_registry())
                .active_player(p1)
                .player_mana(p1, pool)
                .object(honour_guard)
                .build()
                .expect("state builds")
        };

        // Base cost {3}{W} = 4. Squad {2} = 2 per copy. A pool of 8 total mana
        // (4 base + 2*2 for exactly 2 copies) must yield max_count == 2, not more
        // (a 3rd copy would need 6, total 10, which this pool cannot pay).
        let state = make_state(ManaPool {
            white: 1,
            colorless: 7,
            ..Default::default()
        });
        let card_id = id_of(&state, "Ultramarines Honour Guard");
        let obj = state.object(card_id).expect("object exists");
        let plan = build_additional_cost_plan(&state, p1, obj);
        let squad = plan.squad.as_ref().expect("Squad must be detected");
        assert_eq!(
            squad.max_count, 2,
            "exactly 2 copies are affordable with this pool"
        );

        // No spare mana beyond the base cost: max_count == 0, but the cast itself
        // is still offered (Squad never gates -- §1.3).
        let state = make_state(ManaPool {
            white: 1,
            colorless: 3,
            ..Default::default()
        });
        let card_id = id_of(&state, "Ultramarines Honour Guard");
        let obj = state.object(card_id).expect("object exists");
        let plan = build_additional_cost_plan(&state, p1, obj);
        let squad = plan.squad.as_ref().expect("Squad must be detected");
        assert_eq!(squad.max_count, 0, "no spare mana means 0 extra copies");
        let actions = StubProvider.legal_actions(&state, p1);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == card_id)),
            "Squad's own max_count == 0 must not suppress the cast itself"
        );
    }
}
