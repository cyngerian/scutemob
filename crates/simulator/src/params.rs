//! Action parameterization: the single `LegalAction` -> `Command` mapping table
//! (M11-local Session 3, item 5/6/7).
//!
//! Everything CR 601.2b-601.2h lets a caster announce, as data — `ActionParams` is
//! NOT a `Command` variant and never crosses the wire; it is assembled into an
//! existing `Command` by `action_to_command_with_params` below, entirely inside
//! `crates/simulator`. `PROTOCOL_VERSION` / `PROTOCOL_SCHEMA_FINGERPRINT` /
//! `HASH_SCHEMA_VERSION` are unaffected.
//!
//! `action_to_command_with_params` replaces the two independent `LegalAction` ->
//! `Command` mappings that used to exist (`random_bot::action_to_command`'s direct
//! build, and Session 1's `HumanChoice::Command` bypass) with exactly one. Both
//! `random_bot::action_to_command` (item 6) and `LocalGame::submit` (item 7) now
//! delegate to it.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    AdditionalCost, AltCostKind, AttackTarget, Command, EffectChoiceAnswer, GameState, ManaColor,
    ObjectId, PlayerId, Target,
};

use crate::legal_actions::{self, LegalAction};

/// Everything CR 601.2b-601.2h lets a caster announce, as data. NOT a `Command`
/// variant — this is assembled into an existing `Command` by
/// `action_to_command_with_params`, so nothing here ever crosses the wire.
#[derive(Clone, Debug, Default)]
pub struct ActionParams {
    /// CR 601.2c: targets announced at cast/activation time.
    pub targets: Vec<Target>,
    /// CR 601.2b / 107.3m: the value chosen for X. 0 for non-X spells/abilities.
    pub x_value: u32,
    /// CR 700.2: mode indices chosen for a modal spell or activated ability.
    pub modes_chosen: Vec<usize>,
    /// CR 508.1: (attacker, attack target) pairs for `DeclareAttackers`.
    pub attackers: Vec<(ObjectId, AttackTarget)>,
    /// CR 509.1: (blocker, attacker) pairs for `DeclareBlockers`.
    pub blockers: Vec<(ObjectId, ObjectId)>,
    /// CR 509.2: the chosen damage-assignment order for a `LegalAction::OrderBlockers`,
    /// front (assigned damage first) to back. Empty means "keep the engine's default
    /// order", in which case the action's own `blockers` candidate list is submitted
    /// verbatim — see the `OrderBlockers` arm of [`action_to_command_with_params`].
    pub blocker_order: Vec<ObjectId>,
    /// CR 103.5: cards nominated for the bottom of the library on `KeepHand`.
    pub cards_to_bottom: Vec<ObjectId>,
    /// Consolidated additional costs (sacrifice, discard, exile-from-zone, etc.)
    /// for a `CastSpell`.
    pub additional_costs: Vec<AdditionalCost>,
    /// CR 514.1 / CR 701.9b (UI-1): the cards this player chose to discard for a
    /// [`LegalAction::DiscardToHandSize`]. Empty means "accept the engine's own
    /// default", in which case the action's `cards` field is submitted verbatim —
    /// the same "empty means default" contract [`Self::blocker_order`] uses, and
    /// unambiguous for the same reason: a real answer is never empty, because the
    /// decision only exists when `count >= 1` (`BlockingDecision::CleanupDiscard`
    /// is raised only for a hand OVER the maximum).
    ///
    /// Membership, cardinality and duplication are the ENGINE's judgment
    /// (`rules::turn_actions::handle_discard_to_hand_size`), never re-derived here.
    pub discard_cards: Vec<ObjectId>,
    /// CR 608.2d (UI-1): this player's answer to a
    /// [`LegalAction::AnswerEffectChoice`] — a library search, a scry or a
    /// surveil. `None` means "accept the engine's own default", in which case the
    /// action's `answer` field is submitted verbatim.
    ///
    /// An `Option` rather than the "empty means default" convention above because
    /// the answer is an enum, not a collection, and several of its variants have a
    /// legitimately empty payload: `Scry { bottom: [], top: [] }` on a scry whose
    /// `looked_at` is empty, and `SearchLibrary { found: None }` — CR 701.23b's
    /// deliberate fail-to-find, which is a REAL answer and must not be readable as
    /// "no answer given".
    ///
    /// Legality is checked against the engine's own recorded question
    /// (`effects::handle_answer_effect_choice`), never re-derived here.
    pub effect_choice_answer: Option<EffectChoiceAnswer>,
    /// CR 603.3d / CR 601.2c (UI-1, OOS-DP8-2): this player's per-slot targets for
    /// a [`LegalAction::ChooseTriggerTargets`], outer index = slot in the action's
    /// own `slots` order. Empty means "accept the engine's own default", in which
    /// case the action's `targets` field is submitted verbatim.
    ///
    /// Unambiguous for the same reason `discard_cards` is: a `ChooseTriggerTargets`
    /// action is only ever offered when the trigger has at least one slot AND the
    /// answer is not forced (`abilities::forced_trigger_target_answer` short-circuits
    /// the question otherwise), and `default_trigger_targets` emits one entry per
    /// slot — so a real answer's OUTER vector is always non-empty even when every
    /// inner one is (an all-optional trigger answered "choose none" is
    /// `vec![vec![], vec![]]`, not `vec![]`).
    pub trigger_targets: Vec<Vec<Target>>,
    /// If true, `LocalGame::submit` (item 7) will tap mana sources on the human
    /// seat's behalf before applying a `CastSpell` command, but only when the
    /// player's EXISTING mana pool cannot already cover the cost.
    pub auto_tap: bool,
}

/// A human choice handed to `LocalGame::submit`: `action_index` resolves against
/// the currently pending decision's `actions`, and `params` supplies whatever that
/// `LegalAction` needs beyond what the engine already fixed (targets, X, modes,
/// combat declarations, ...). Moved here from `local_game.rs` in Session 3 (item
/// 7) — Session 1 accepted a pre-built `Command` directly (`HumanChoice::Command`).
/// Because `submit` now builds the `Command` itself for `pending.player`, a
/// command naming a different seat is structurally unrepresentable.
#[derive(Clone, Debug)]
pub struct HumanChoice {
    pub action_index: usize,
    pub params: ActionParams,
}

/// Errors `action_to_command_with_params` can return. Kept small and specific
/// (not a stringly-typed catch-all) — every variant names exactly what went wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamError {
    /// CR 605.3b / 106.1b / 111.10a: the mana ability is `any_color: true` (a
    /// layer-resolved read, not the `LegalAction`'s cached copy) and no
    /// `chosen_color` was supplied.
    MissingChosenColor,
    /// CR 106.1b: `Colorless` is never a legal choice for an `any_color` ability.
    InvalidChosenColor,
    /// A param was supplied that this `LegalAction` variant has no channel for —
    /// carries the name of the first offending `ActionParams` field. Refusing beats
    /// silently discarding a human's announced targets: the `action_index` already
    /// fixed which action this is, so a param the action cannot carry means the
    /// client and the pending decision disagree about what is being answered.
    UnsupportedParam(&'static str),
}

impl ActionParams {
    /// The first field naming something this `ActionParams` announces, or `None` if
    /// it announces nothing. `auto_tap` is deliberately excluded — it is a
    /// `LocalGame::submit` flag about *how* to pay, not an announcement, and is
    /// harmless (a no-op) on an action that is not a `CastSpell`.
    fn first_announced_field(&self) -> Option<&'static str> {
        if !self.targets.is_empty() {
            return Some("targets");
        }
        if self.x_value != 0 {
            return Some("x_value");
        }
        if !self.modes_chosen.is_empty() {
            return Some("modes_chosen");
        }
        if !self.attackers.is_empty() {
            return Some("attackers");
        }
        if !self.blockers.is_empty() {
            return Some("blockers");
        }
        if !self.blocker_order.is_empty() {
            return Some("blocker_order");
        }
        if !self.cards_to_bottom.is_empty() {
            return Some("cards_to_bottom");
        }
        if !self.additional_costs.is_empty() {
            return Some("additional_costs");
        }
        // UI-1: the three blocking-decision answers. Listed here for the same
        // reason every field above is — announcing a discard subset on a
        // `PassPriority` means the client and the pending decision disagree about
        // what is being answered, and refusing beats discarding it in silence.
        if !self.discard_cards.is_empty() {
            return Some("discard_cards");
        }
        if self.effect_choice_answer.is_some() {
            return Some("effect_choice_answer");
        }
        if !self.trigger_targets.is_empty() {
            return Some("trigger_targets");
        }
        None
    }
}

impl std::fmt::Display for ParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamError::MissingChosenColor => write!(
                f,
                "an any_color mana ability requires a chosen_color (CR 605.3b), but none was supplied"
            ),
            ParamError::InvalidChosenColor => write!(
                f,
                "Colorless is not a legal chosen_color for an any_color mana ability (CR 106.1b)"
            ),
            ParamError::UnsupportedParam(name) => {
                write!(f, "param {name:?} is not usable by this action")
            }
        }
    }
}

impl std::error::Error for ParamError {}

/// The single `LegalAction` -> `Command` mapping table in `crates/simulator` (item
/// 5/6). `random_bot::action_to_command` (item 6) and `LocalGame::submit` (item 7)
/// both delegate here, so within this crate there is exactly one place a
/// `LegalAction` becomes a `Command`.
///
/// **Not yet the only one in the workspace**: `tools/tui/src/play/input.rs` still
/// reads an `ability_index` out of a `LegalAction` and hand-builds
/// `Command::CastSpell` / `Command::ActivateAbility` with `targets: Vec::new()` —
/// which is exactly why the TUI still cannot cast a targeted spell (plan §8 R1).
/// Migrating that call site onto this function is the remaining half; the session
/// plan permits it opportunistically and it is not in this session's scope.
///
/// Every arm ports `random_bot::action_to_command`'s pre-Session-3 behavior
/// verbatim except where a `LegalAction` variant honours `params` (`CastSpell`,
/// `TapForMana`'s validation, `ActivateAbility`, `DeclareAttackers`,
/// `DeclareBlockers`, `KeepHand`, and — added by UI-1 — `DiscardToHandSize`,
/// `ChooseTriggerTargets`, `AnswerEffectChoice`). Every other arm is an identical
/// port, and announcing any param on one of them is rejected with
/// `ParamError::UnsupportedParam` rather than silently discarded.
///
/// Residual, deliberately not guarded: a param announced on a *consuming* arm that
/// that arm does not read (e.g. `attackers` alongside a `CastSpell`) is still
/// ignored. The nine consuming arms would each need their own field allowlist to
/// catch that, and the failure mode is far less confusing than a wholly unread
/// `targets` — the action being answered is still the one the client picked.
/// `tools/play-server`'s `api.rs` closes the half of this that a browser client can
/// actually hit, by checking a submitted answer against the candidate lists the
/// same response carried before `submit` is ever called.
pub fn action_to_command_with_params(
    state: &GameState,
    player: PlayerId,
    action: &LegalAction,
    params: &ActionParams,
) -> Result<Command, ParamError> {
    // Exactly nine `LegalAction` variants have a parameterization channel — the six
    // CR 601.2b-601.2h announcement arms, plus (UI-1) the three blocking-decision
    // arms whose answer used to be baked into the `LegalAction` itself. For every
    // other action, an announced param means the client and the pending decision
    // disagree about what is being answered — refuse rather than silently discard it.
    // `auto_tap` is excluded (see `first_announced_field`).
    if !matches!(
        action,
        LegalAction::CastSpell { .. }
            | LegalAction::ActivateAbility { .. }
            | LegalAction::DeclareAttackers { .. }
            | LegalAction::DeclareBlockers { .. }
            | LegalAction::OrderBlockers { .. }
            | LegalAction::KeepHand
            | LegalAction::DiscardToHandSize { .. }
            | LegalAction::ChooseTriggerTargets { .. }
            | LegalAction::AnswerEffectChoice { .. }
    ) {
        if let Some(field) = params.first_announced_field() {
            return Err(ParamError::UnsupportedParam(field));
        }
    }
    match action {
        LegalAction::PassPriority => Ok(Command::PassPriority { player }),
        LegalAction::Concede => Ok(Command::Concede { player }),
        LegalAction::PlayLand { card } => Ok(Command::PlayLand {
            player,
            card: *card,
        }),
        LegalAction::CastSpell { card, .. } => Ok(Command::CastSpell(Box::new(CastSpellData {
            player,
            card: *card,
            targets: params.targets.clone(),
            convoke_creatures: Vec::new(),
            improvise_artifacts: Vec::new(),
            delve_cards: Vec::new(),
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            // CR 601.2b/700.2a (PB-DP3): announce the caller's modes if given, else
            // fall back to the deterministic first-`min_modes` default (a no-op for
            // non-modal cards) so a cast is never silently rejected for lack of an
            // announcement.
            modes_chosen: if params.modes_chosen.is_empty() {
                legal_actions::spell_default_modes(state, *card)
            } else {
                params.modes_chosen.clone()
            },
            x_value: params.x_value,
            face_down_kind: None,
            additional_costs: params.additional_costs.clone(),
            // KNOWN GAP: unlike `TapForMana`/`ActivateAbility`, `LegalAction::CastSpell`
            // carries no provider-resolved hybrid/Phyrexian payment plan, so there is
            // nothing to forward here. Empty is the documented default: each hybrid
            // pip pays with its first colour option, each Phyrexian pip pays with mana
            // (see `CastSpellData::hybrid_choices`/`phyrexian_life_payments`'s field
            // docs). A human casting a hybrid-cost spell cannot yet choose otherwise —
            // out of scope for this session.
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        }))),
        LegalAction::TapForMana {
            source,
            ability_index,
            chosen_color,
            hybrid_choices,
            phyrexian_life_payments,
        } => {
            // PB-EF12 (CR 106.1a/106.1b/605.3b): validate against the LAYER-RESOLVED
            // ability (never the LegalAction's cached copy, which could be stale by
            // the time this runs) — mirrors the checks `legal_actions.rs` and
            // `mana_solver.rs` already make when they build this same offer.
            let any_color = mtg_engine::rules::layers::calculate_characteristics(state, *source)
                .and_then(|chars| {
                    chars
                        .mana_abilities
                        .get(*ability_index)
                        .map(|a| a.any_color)
                })
                .unwrap_or(false);
            if any_color {
                match chosen_color {
                    None => return Err(ParamError::MissingChosenColor),
                    Some(ManaColor::Colorless) => return Err(ParamError::InvalidChosenColor),
                    Some(_) => {}
                }
            }
            Ok(Command::TapForMana {
                player,
                source: *source,
                ability_index: *ability_index,
                chosen_color: *chosen_color,
                // PB-RS2: pass through the fully-payable, non-suicidal plan the
                // provider already resolved (`resolve_hybrid_phyrexian_plan`) — never
                // re-derive it here, or the two could drift (OOS-RS-2).
                hybrid_choices: hybrid_choices.clone(),
                phyrexian_life_payments: phyrexian_life_payments.clone(),
            })
        }
        LegalAction::ActivateAbility {
            source,
            ability_index,
            hybrid_choices,
            phyrexian_life_payments,
        } => Ok(Command::ActivateAbility {
            player,
            source: *source,
            ability_index: *ability_index,
            targets: params.targets.clone(),
            discard_card: None,
            sacrifice_target: None,
            x_value: if params.x_value != 0 {
                Some(params.x_value)
            } else {
                None
            },
            // CR 602.2b/700.2a (PB-DP3): see the CastSpell arm above.
            modes_chosen: if params.modes_chosen.is_empty() {
                legal_actions::ability_default_modes(state, *source, *ability_index)
            } else {
                params.modes_chosen.clone()
            },
            // PB-RS2: see the TapForMana arm above.
            hybrid_choices: hybrid_choices.clone(),
            phyrexian_life_payments: phyrexian_life_payments.clone(),
        }),
        LegalAction::DeclareAttackers { .. } => {
            // PB-DX6 §9.2 (CR 508.1h/107.4e/107.4f): unlike `TapForMana`/
            // `ActivateAbility`, the tax total for a `DeclareAttackers`
            // announcement is NOT knowable at `LegalAction` enumeration time —
            // `LegalAction::DeclareAttackers { eligible, targets }` carries no
            // attacker SET, and the CR 508.1h total is a function of exactly
            // which creatures attack which defenders. So the plan is built
            // here, at command-construction time, once `params.attackers` is
            // known — never on the `LegalAction` itself (that would require a
            // field that lies about being determined before the attacker
            // subset is chosen).
            //
            // `mtg_engine::rules::queries::attack_tax_total` is the ONE
            // supported way to obtain the unflattened CR 508.1h total in the
            // canonical (copy-major) pip order `hybrid_choices`/
            // `phyrexian_life_payments` index against — re-deriving that
            // accumulation here would be the exact OOS-RS-2 drift class this
            // suite keeps closing. If no payable-and-non-suicidal plan exists
            // (`None`, either because there is no tax or because the tax is
            // genuinely unpayable), fall back to empty vectors and let the
            // engine reject the declaration — mutating the attacker set to
            // dodge the tax is out of scope and would hide a legality
            // problem from the caller.
            //
            // KNOWN SR-38 RESIDUE (OOS-DX6-1, PB-DX6 fix-cycle Finding 7):
            // `attack_tax_total` also returns `None` for a defender whose ONLY
            // restriction is an X-carrying tax (X has no announcement channel
            // on `Command::DeclareAttackers` and is excluded from the total
            // entirely — see that function's own doc). This arm cannot tell
            // "no tax" apart from "an X tax this query cannot express" from
            // the `None` alone, so it falls back to empty vectors in BOTH
            // cases and the engine hard-rejects the latter — an SR-38
            // violation in the strict sense (offering an action the engine
            // will refuse), latent only because the corpus carries no X or
            // mixed X/pip attack tax today (PB-DX6's roster gate, R4, pinned
            // empty). Fixing this needs the X-announcement channel
            // OOS-DX6-1 itself is filed for, not a change to this arm.
            let (hybrid_choices, phyrexian_life_payments) =
                mtg_engine::rules::queries::attack_tax_total(state, player, &params.attackers)
                    .and_then(|total| {
                        legal_actions::resolve_hybrid_phyrexian_plan(state, player, &total, 0)
                    })
                    .unwrap_or_default();
            Ok(Command::DeclareAttackers {
                player,
                attackers: params.attackers.clone(),
                enlist_choices: Vec::new(),
                exert_choices: Vec::new(),
                hybrid_choices,
                phyrexian_life_payments,
            })
        }
        LegalAction::DeclareBlockers { .. } => Ok(Command::DeclareBlockers {
            player,
            blockers: params.blockers.clone(),
        }),
        // CR 509.2 (M11-local S8, item 2). Empty `blocker_order` means "keep the
        // engine's default", and the default IS `blockers` — `apply_combat_damage`
        // falls back to `combat.blockers`' `OrdMap` order, which is the order this
        // candidate list was built in (`LocalGame::human_only_actions`). Submitting
        // it verbatim is therefore an exact no-op rather than an arbitrary pick, and
        // it satisfies `handle_order_blockers`' CR 509.2 completeness check
        // (`GameStateError::IncompleteBlockerOrder` when a blocker is omitted), which
        // an empty vector would fail.
        LegalAction::OrderBlockers { attacker, blockers } => Ok(Command::OrderBlockers {
            player,
            attacker: *attacker,
            order: if params.blocker_order.is_empty() {
                blockers.clone()
            } else {
                params.blocker_order.clone()
            },
        }),
        LegalAction::TakeMulligan => Ok(Command::TakeMulligan { player }),
        LegalAction::KeepHand => Ok(Command::KeepHand {
            player,
            // CR 103.5: the half of the mulligan `setup::redeal` (Session 2) does
            // not do — bottoming `mulligan_count - 1` cards on a KEPT hand.
            cards_to_bottom: params.cards_to_bottom.clone(),
        }),
        LegalAction::ReturnCommanderToCommandZone { object_id } => {
            Ok(Command::ReturnCommanderToCommandZone {
                player,
                object_id: *object_id,
            })
        }
        LegalAction::LeaveCommanderInZone { object_id } => Ok(Command::LeaveCommanderInZone {
            player,
            object_id: *object_id,
        }),
        // ── Every other arm: identical to `random_bot::action_to_command`'s
        // pre-Session-3 behavior. `params` is deliberately unread here — none of
        // these `LegalAction` variants gained a parameterization channel this
        // session. ──────────────────────────────────────────────────────────────
        LegalAction::ActivateBloodrush { card, target } => Ok(Command::ActivateBloodrush {
            player,
            card: *card,
            target: *target,
        }),
        LegalAction::SaddleMount {
            mount,
            saddle_creatures,
        } => Ok(Command::SaddleMount {
            player,
            mount: *mount,
            saddle_creatures: saddle_creatures.clone(),
        }),
        LegalAction::CastWithMutate {
            card,
            mutate_target,
        } => Ok(Command::CastSpell(Box::new(CastSpellData {
            player,
            card: *card,
            targets: Vec::new(),
            convoke_creatures: Vec::new(),
            improvise_artifacts: Vec::new(),
            delve_cards: Vec::new(),
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: legal_actions::spell_default_modes(state, *card),
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![AdditionalCost::Mutate {
                target: *mutate_target,
                on_top: true,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        }))),
        LegalAction::TurnFaceUp {
            permanent,
            method,
            hybrid_choices,
            phyrexian_life_payments,
        } => Ok(Command::TurnFaceUp {
            player,
            permanent: *permanent,
            method: method.clone(),
            // PB-DX6 (CR 107.4e/107.4f, PB-RS2 pattern): forward the
            // `LegalActionProvider`'s already-resolved, already-payable plan
            // VERBATIM -- `legal_actions.rs::turn_face_up_payment_plan` is the one
            // place that decides which half of a pip to pay, so this arm must not
            // re-derive or default it.
            hybrid_choices: hybrid_choices.clone(),
            phyrexian_life_payments: phyrexian_life_payments.clone(),
        }),
        LegalAction::CastMorphFaceDown { card, .. } => {
            Ok(Command::CastSpell(Box::new(CastSpellData {
                player,
                card: *card,
                targets: Vec::new(),
                convoke_creatures: Vec::new(),
                improvise_artifacts: Vec::new(),
                delve_cards: Vec::new(),
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Morph),
                prototype: false,
                modes_chosen: legal_actions::spell_default_modes(state, *card),
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })))
        }
        // KNOWN GAP, filed as **OOS-M11-10** (`docs/audits/decision-point-audit.md`
        // §8.1) by the M11-local close-out: a loyalty ability's targets cannot be
        // announced. This comment read "to be filed for S6/S7" from S5 until S8's
        // close, and S6, S7 and S8 all shipped without filing it — review MR-M11-06.
        // A comment asserting a seed exists is not a seed; the seed is the row.
        // `ActivateLoyaltyAbility` is outside the nine-arm
        // allowlist above, so `ActionParams { targets, .. }` on a planeswalker
        // ability is REJECTED with `UnsupportedParam("targets")` rather than
        // forwarded — loud, not silently wrong, but it means a human still cannot
        // use a targeted loyalty ability. Planeswalkers are common in Commander
        // (Architecture Invariant 6), so this will surface the moment the browser
        // client offers a loyalty picker. Same shape applies to
        // `ActivateBloodrush` and the Mutate/Morph casts below, which also
        // hard-code `targets: Vec::new()`.
        LegalAction::ActivateLoyaltyAbility {
            source,
            ability_index,
        } => Ok(Command::ActivateLoyaltyAbility {
            player,
            source: *source,
            ability_index: *ability_index,
            targets: Vec::new(),
            x_value: None,
        }),
        LegalAction::PayEcho { permanent, pay } => Ok(Command::PayEcho {
            player,
            permanent: *permanent,
            pay: *pay,
        }),
        LegalAction::PayCumulativeUpkeep { permanent, pay } => Ok(Command::PayCumulativeUpkeep {
            player,
            permanent: *permanent,
            pay: *pay,
        }),
        LegalAction::PayRecover { recover_card, pay } => Ok(Command::PayRecover {
            player,
            recover_card: *recover_card,
            pay: *pay,
        }),
        // ── The three blocking-decision arms (UI-1) ──────────────────────────────
        //
        // Each of these used to submit the `LegalAction`'s own default
        // UNCONDITIONALLY, and `params` was unread. That is the mechanism behind
        // the first human playtest's "the game discards for me" and "it never asks
        // me to scry" (`memory/playtest-triage-2026-08-02.md` F8): `StubProvider`
        // bakes the engine-accepted default into the action, so a client holding
        // only an action index could echo the default and nothing else.
        //
        // The default is still the fallback, and that is load-bearing rather than
        // conservative: `random_bot::action_to_command` reaches these arms with a
        // `ActionParams::default()`, so an unparameterized submission produces a
        // BYTE-IDENTICAL `Command` to the pre-UI-1 one and no recorded fuzz seed's
        // outcome moves (OOS-DP8-1, the same constraint PB-DP8/DP9 wrote these arms
        // under). What changes is only that a caller who *does* announce an answer
        // now has it forwarded instead of dropped.
        //
        // Nothing here re-derives legality. `handle_discard_to_hand_size`,
        // `handle_choose_trigger_targets` and `handle_answer_effect_choice` each
        // validate an arbitrary answer against the engine's OWN recorded entry
        // (count/membership/duplication; slot count/cardinality/membership/
        // distinctness; question-variant agreement/partition), and re-deriving any
        // of that here would be the OOS-RS-2 drift class.
        //
        // CR 514.1 / CR 701.9b (PB-DP7 / DP-3).
        LegalAction::DiscardToHandSize { cards, .. } => Ok(Command::DiscardToHandSize {
            player,
            cards: if params.discard_cards.is_empty() {
                cards.clone()
            } else {
                params.discard_cards.clone()
            },
        }),
        // CR 603.3d / CR 601.2c (PB-DP8 / DP-6). This arm is OOS-DP8-2's half of
        // UI-1: the channel exists and is symmetrical with the two beside it.
        LegalAction::ChooseTriggerTargets {
            choice_id, targets, ..
        } => Ok(Command::ChooseTriggerTargets {
            player,
            choice_id: *choice_id,
            targets: if params.trigger_targets.is_empty() {
                targets.clone()
            } else {
                params.trigger_targets.clone()
            },
        }),
        // CR 608.2d (PB-DP9 / DP-7/8/9).
        LegalAction::AnswerEffectChoice {
            choice_id, answer, ..
        } => Ok(Command::AnswerEffectChoice {
            player,
            choice_id: *choice_id,
            answer: params
                .effect_choice_answer
                .clone()
                .unwrap_or_else(|| answer.clone()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtg_engine::state::ActiveRestriction;
    use mtg_engine::{
        process_command, GameRestriction, GameStateBuilder, HybridMana, ManaCost, ObjectSpec, Step,
        ZoneId,
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

    /// Fixture: P2 controls a permanent bearing `CantAttackYouUnlessPay { cost_per_creature:
    /// pip_cost }`, P1 controls one creature able to attack P2. Mirrors
    /// `crates/engine/tests/primitives/pb_dx6_unflattened_payment_sites.rs`'s
    /// `attack_tax_state` fixture (that helper is private to the engine's own test
    /// crate, so this is a deliberate, minimal re-derivation from the same public
    /// `GameStateBuilder` API rather than a shared dependency).
    fn attack_tax_state(pip_cost: ManaCost) -> (GameState, PlayerId, PlayerId, ObjectId) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .object(ObjectSpec::creature(p2, "Tax Source", 0, 4).in_zone(ZoneId::Battlefield))
            .object(ObjectSpec::creature(p1, "Attacking Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .unwrap();
        let tax_source = id_of(&state, "Tax Source");
        state.restrictions_mut().push_back(ActiveRestriction {
            source: tax_source,
            controller: p2,
            restriction: GameRestriction::CantAttackYouUnlessPay {
                cost_per_creature: pip_cost,
            },
        });
        state.turn_mut().priority_holder = Some(p1);
        let bear = id_of(&state, "Attacking Bear");
        (state, p1, p2, bear)
    }

    /// PB-DX6 §9.2: `action_to_command_with_params`'s `DeclareAttackers` arm must
    /// build a REAL, non-empty payment plan for a pipped CR 508.1h attack tax once
    /// `params.attackers` is known — the plan cannot live on the `LegalAction`
    /// itself (§9.2's structural difference from PB-RS2/`TapForMana`). Verified by
    /// EXECUTION: the resulting `Command` is submitted through `process_command`
    /// and the attacker is confirmed actually declared, not merely constructed.
    #[test]
    fn declare_attackers_arm_builds_a_real_hybrid_tax_plan_and_the_engine_accepts_it() {
        let (mut state, p1, p2, bear) = attack_tax_state(ManaCost {
            hybrid: vec![HybridMana::ColorColor(
                mtg_engine::ManaColor::Green,
                mtg_engine::ManaColor::White,
            )],
            ..Default::default()
        });
        // Pool covers ONLY the Green half -- proves the plan actually chose the
        // payable half rather than defaulting blindly to the flattener's own
        // first-colour default (which here would also be Green, so this alone
        // would not discriminate; case 2 below closes that gap).
        state.players_mut().get_mut(&p1).unwrap().mana_pool.green = 1;

        let params = ActionParams {
            attackers: vec![(bear, AttackTarget::Player(p2))],
            ..ActionParams::default()
        };
        let action = LegalAction::DeclareAttackers {
            eligible: vec![bear],
            targets: vec![AttackTarget::Player(p2)],
        };
        let cmd = action_to_command_with_params(&state, p1, &action, &params)
            .expect("DeclareAttackers is an unconditional Ok arm");
        let Command::DeclareAttackers {
            hybrid_choices,
            phyrexian_life_payments,
            ..
        } = &cmd
        else {
            panic!("expected Command::DeclareAttackers, got {cmd:?}");
        };
        assert!(
            !hybrid_choices.is_empty(),
            "a pipped attack tax must produce a NON-EMPTY plan, not the pre-PB-DX6 \
             empty-vectors placeholder: {cmd:?}"
        );
        assert!(
            phyrexian_life_payments.is_empty(),
            "no Phyrexian pip in this cost"
        );

        let (state, _events) =
            process_command(state, cmd).expect("the built plan must actually pay the tax");
        assert!(
            state
                .combat()
                .as_ref()
                .map(|c| c.attackers.contains_key(&bear))
                .unwrap_or(false),
            "the attacker must be genuinely declared, not merely affordable on paper"
        );
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(
            pool.total(),
            0,
            "the Green pip must have been spent: {pool:?}"
        );
    }

    /// Case 2 of the same arm: pool covers ONLY the White half. If the plan builder
    /// ever regressed to hard-coding `Color(Green)` (the flattener's own default,
    /// which this case's pool cannot pay), this would fail with an
    /// insufficient-mana rejection instead of succeeding.
    #[test]
    fn declare_attackers_arm_plan_pays_with_the_actually_available_half() {
        let (mut state, p1, p2, bear) = attack_tax_state(ManaCost {
            hybrid: vec![HybridMana::ColorColor(
                mtg_engine::ManaColor::Green,
                mtg_engine::ManaColor::White,
            )],
            ..Default::default()
        });
        state.players_mut().get_mut(&p1).unwrap().mana_pool.white = 1;

        let params = ActionParams {
            attackers: vec![(bear, AttackTarget::Player(p2))],
            ..ActionParams::default()
        };
        let action = LegalAction::DeclareAttackers {
            eligible: vec![bear],
            targets: vec![AttackTarget::Player(p2)],
        };
        let cmd = action_to_command_with_params(&state, p1, &action, &params).unwrap();
        let (state, _events) =
            process_command(state, cmd).expect("the plan must have chosen the payable half");
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(
            pool.total(),
            0,
            "the White pip must have been spent: {pool:?}"
        );
    }

    /// No tax at all (the common case, and the vast majority of declarations):
    /// `attack_tax_total` returns `None`, and the arm must fall back to the
    /// documented empty-vectors shape rather than panic or fabricate a plan.
    #[test]
    fn declare_attackers_arm_is_untaxed_by_default() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .object(ObjectSpec::creature(p1, "Plain Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .unwrap();
        let bear = id_of(&state, "Plain Bear");
        let params = ActionParams {
            attackers: vec![(bear, AttackTarget::Player(p2))],
            ..ActionParams::default()
        };
        let action = LegalAction::DeclareAttackers {
            eligible: vec![bear],
            targets: vec![AttackTarget::Player(p2)],
        };
        let cmd = action_to_command_with_params(&state, p1, &action, &params).unwrap();
        let Command::DeclareAttackers {
            hybrid_choices,
            phyrexian_life_payments,
            ..
        } = &cmd
        else {
            panic!("expected Command::DeclareAttackers, got {cmd:?}");
        };
        assert!(hybrid_choices.is_empty() && phyrexian_life_payments.is_empty());
    }

    // ── UI-1: the three blocking-decision arms ────────────────────────────────
    //
    // These check the MAPPING only — that an announced answer reaches the
    // `Command` and that an unannounced one still produces the pre-UI-1 default.
    // Whether the engine ACCEPTS a given answer is the engine's own business and
    // is tested where that validation lives (`handle_discard_to_hand_size`,
    // `handle_answer_effect_choice`, `handle_choose_trigger_targets`) and,
    // end-to-end over HTTP, in `tools/play-server`.

    /// A bare two-player state. These three arms read nothing off `GameState` —
    /// only the allowlist match and the params — so the fixture is deliberately
    /// empty rather than elaborate.
    fn bare_state() -> (GameState, PlayerId) {
        let p1 = PlayerId(1);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(PlayerId(2))
            .active_player(p1)
            .build()
            .unwrap();
        (state, p1)
    }

    fn discard_action() -> LegalAction {
        LegalAction::DiscardToHandSize {
            count: 2,
            hand: vec![ObjectId(10), ObjectId(11), ObjectId(12), ObjectId(13)],
            // `default_cleanup_discard`'s shape: the `count` HIGHEST ids.
            cards: vec![ObjectId(12), ObjectId(13)],
        }
    }

    /// CR 514.1: an unparameterized submission is byte-identical to the pre-UI-1
    /// one. This is the property that keeps every recorded fuzz seed reproducing
    /// (OOS-DP8-1) — `random_bot` reaches this arm with `ActionParams::default()`.
    #[test]
    fn discard_arm_without_params_submits_the_engines_own_default() {
        let (state, p1) = bare_state();
        let cmd =
            action_to_command_with_params(&state, p1, &discard_action(), &ActionParams::default())
                .unwrap();
        let Command::DiscardToHandSize { cards, .. } = &cmd else {
            panic!("expected Command::DiscardToHandSize, got {cmd:?}");
        };
        assert_eq!(
            cards,
            &vec![ObjectId(12), ObjectId(13)],
            "the default is the action's own `cards`"
        );
    }

    /// CR 514.1 (playtest triage F8): the human's subset reaches the `Command`.
    /// The chosen pair is disjoint from the default, so this cannot pass by
    /// accidentally still submitting the default.
    #[test]
    fn discard_arm_forwards_a_human_chosen_subset() {
        let (state, p1) = bare_state();
        let params = ActionParams {
            discard_cards: vec![ObjectId(10), ObjectId(11)],
            ..ActionParams::default()
        };
        let cmd = action_to_command_with_params(&state, p1, &discard_action(), &params).unwrap();
        let Command::DiscardToHandSize { cards, .. } = &cmd else {
            panic!("expected Command::DiscardToHandSize, got {cmd:?}");
        };
        assert_eq!(cards, &vec![ObjectId(10), ObjectId(11)]);
    }

    fn scry_action() -> LegalAction {
        let question = mtg_engine::EffectChoiceQuestion::Scry {
            looked_at: vec![ObjectId(20), ObjectId(21)],
        };
        LegalAction::AnswerEffectChoice {
            choice_id: 7,
            source: ObjectId(5),
            // Built through the engine's own helper, so the "default" this test
            // pins cannot drift from the one `StubProvider` actually offers.
            answer: mtg_engine::effects::default_effect_choice_answer(&question),
            question,
        }
    }

    /// CR 608.2d: the scry default is the IDENTITY (everything stays on top), and
    /// an unparameterized submission still produces exactly it.
    #[test]
    fn effect_choice_arm_without_params_submits_the_engines_own_default() {
        let (state, p1) = bare_state();
        let cmd =
            action_to_command_with_params(&state, p1, &scry_action(), &ActionParams::default())
                .unwrap();
        let Command::AnswerEffectChoice { answer, .. } = &cmd else {
            panic!("expected Command::AnswerEffectChoice, got {cmd:?}");
        };
        assert_eq!(
            answer,
            &EffectChoiceAnswer::Scry {
                bottom: vec![],
                top: vec![ObjectId(20), ObjectId(21)],
            },
            "the identity partition is the pre-UI-1 behaviour and must be preserved"
        );
    }

    /// CR 701.22a (playtest triage F8): a real scry — one card bottomed, the other
    /// kept — reaches the `Command`. `bottom` being non-empty is exactly what the
    /// default can never produce, so this discriminates.
    #[test]
    fn effect_choice_arm_forwards_a_human_answer() {
        let (state, p1) = bare_state();
        let chosen = EffectChoiceAnswer::Scry {
            bottom: vec![ObjectId(20)],
            top: vec![ObjectId(21)],
        };
        let params = ActionParams {
            effect_choice_answer: Some(chosen.clone()),
            ..ActionParams::default()
        };
        let cmd = action_to_command_with_params(&state, p1, &scry_action(), &params).unwrap();
        let Command::AnswerEffectChoice { answer, .. } = &cmd else {
            panic!("expected Command::AnswerEffectChoice, got {cmd:?}");
        };
        assert_eq!(answer, &chosen);
    }

    /// CR 701.23b: `SearchLibrary { found: None }` is a genuine answer (fail to
    /// find), not an absent one. This is why `effect_choice_answer` is an
    /// `Option<..>` and not an "empty means default" collection — with the latter
    /// encoding this submission would silently become "find the first candidate",
    /// which is the opposite of what the human asked for.
    #[test]
    fn a_deliberate_fail_to_find_is_not_read_as_no_answer() {
        let (state, p1) = bare_state();
        let question = mtg_engine::EffectChoiceQuestion::SearchLibrary {
            candidates: vec![ObjectId(30), ObjectId(31)],
            may_fail_to_find: true,
        };
        let action = LegalAction::AnswerEffectChoice {
            choice_id: 1,
            source: ObjectId(5),
            answer: mtg_engine::effects::default_effect_choice_answer(&question),
            question,
        };
        let params = ActionParams {
            effect_choice_answer: Some(EffectChoiceAnswer::SearchLibrary { found: None }),
            ..ActionParams::default()
        };
        let cmd = action_to_command_with_params(&state, p1, &action, &params).unwrap();
        let Command::AnswerEffectChoice { answer, .. } = &cmd else {
            panic!("expected Command::AnswerEffectChoice, got {cmd:?}");
        };
        assert_eq!(
            answer,
            &EffectChoiceAnswer::SearchLibrary { found: None },
            "a fail-to-find must not be overwritten by the default's \
             `candidates.first()`"
        );
    }

    fn trigger_targets_action() -> LegalAction {
        LegalAction::ChooseTriggerTargets {
            choice_id: 3,
            source: ObjectId(5),
            slots: vec![],
            targets: vec![vec![Target::Player(PlayerId(2))]],
        }
    }

    /// CR 603.3d (OOS-DP8-2): the same two properties for the trigger-target arm —
    /// the channel this batch adds so the identical gap is closed by construction
    /// rather than by a follow-up.
    #[test]
    fn trigger_targets_arm_defaults_then_forwards() {
        let (state, p1) = bare_state();
        let cmd = action_to_command_with_params(
            &state,
            p1,
            &trigger_targets_action(),
            &ActionParams::default(),
        )
        .unwrap();
        let Command::ChooseTriggerTargets { targets, .. } = &cmd else {
            panic!("expected Command::ChooseTriggerTargets, got {cmd:?}");
        };
        assert_eq!(targets, &vec![vec![Target::Player(PlayerId(2))]]);

        let chosen = vec![vec![Target::Player(PlayerId(1))]];
        let params = ActionParams {
            trigger_targets: chosen.clone(),
            ..ActionParams::default()
        };
        let cmd =
            action_to_command_with_params(&state, p1, &trigger_targets_action(), &params).unwrap();
        let Command::ChooseTriggerTargets { targets, .. } = &cmd else {
            panic!("expected Command::ChooseTriggerTargets, got {cmd:?}");
        };
        assert_eq!(targets, &chosen);
    }

    /// The allowlist half: an answer announced on an action that has no channel
    /// for it is REFUSED, not discarded. Without this, a client whose picker state
    /// desynchronised from the decision would silently pass priority while
    /// believing it had discarded.
    #[test]
    fn an_answer_announced_on_the_wrong_action_is_refused() {
        let (state, p1) = bare_state();
        for (params, expected) in [
            (
                ActionParams {
                    discard_cards: vec![ObjectId(10)],
                    ..ActionParams::default()
                },
                "discard_cards",
            ),
            (
                ActionParams {
                    effect_choice_answer: Some(EffectChoiceAnswer::SearchLibrary { found: None }),
                    ..ActionParams::default()
                },
                "effect_choice_answer",
            ),
            (
                ActionParams {
                    trigger_targets: vec![vec![]],
                    ..ActionParams::default()
                },
                "trigger_targets",
            ),
        ] {
            let err =
                action_to_command_with_params(&state, p1, &LegalAction::PassPriority, &params)
                    .expect_err("PassPriority has no answer channel");
            assert_eq!(err, ParamError::UnsupportedParam(expected));
        }
    }

    /// The converse, and the reason the allowlist edit is not merely cosmetic: the
    /// three arms are now IN it, so the same params reach their own action instead
    /// of being refused. Before UI-1 this returned `UnsupportedParam`.
    #[test]
    fn the_three_arms_are_inside_the_allowlist() {
        let (state, p1) = bare_state();
        let params = ActionParams {
            discard_cards: vec![ObjectId(10), ObjectId(11)],
            ..ActionParams::default()
        };
        assert!(action_to_command_with_params(&state, p1, &discard_action(), &params).is_ok());
    }
}
