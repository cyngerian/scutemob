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
    AdditionalCost, AltCostKind, AttackTarget, Command, GameState, ManaColor, ObjectId, PlayerId,
    Target,
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
    /// CR 103.5: cards nominated for the bottom of the library on `KeepHand`.
    pub cards_to_bottom: Vec<ObjectId>,
    /// Consolidated additional costs (sacrifice, discard, exile-from-zone, etc.)
    /// for a `CastSpell`.
    pub additional_costs: Vec<AdditionalCost>,
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
        if !self.cards_to_bottom.is_empty() {
            return Some("cards_to_bottom");
        }
        if !self.additional_costs.is_empty() {
            return Some("additional_costs");
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
/// verbatim except where a `LegalAction` variant now honours `params` (`CastSpell`,
/// `TapForMana`'s validation, `ActivateAbility`, `DeclareAttackers`,
/// `DeclareBlockers`, `KeepHand`). Every other arm is an identical port, and
/// announcing any param on one of them is rejected with
/// `ParamError::UnsupportedParam` rather than silently discarded.
///
/// Residual, deliberately not guarded: a param announced on a *consuming* arm that
/// that arm does not read (e.g. `attackers` alongside a `CastSpell`) is still
/// ignored. The five consuming arms would each need their own field allowlist to
/// catch that, and the failure mode is far less confusing than a wholly unread
/// `targets` — the action being answered is still the one the client picked.
pub fn action_to_command_with_params(
    state: &GameState,
    player: PlayerId,
    action: &LegalAction,
    params: &ActionParams,
) -> Result<Command, ParamError> {
    // Exactly five `LegalAction` variants have a parameterization channel. For every
    // other action, an announced param means the client and the pending decision
    // disagree about what is being answered — refuse rather than silently discard it.
    // `auto_tap` is excluded (see `first_announced_field`).
    if !matches!(
        action,
        LegalAction::CastSpell { .. }
            | LegalAction::ActivateAbility { .. }
            | LegalAction::DeclareAttackers { .. }
            | LegalAction::DeclareBlockers { .. }
            | LegalAction::KeepHand
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
        LegalAction::DeclareAttackers { .. } => Ok(Command::DeclareAttackers {
            player,
            attackers: params.attackers.clone(),
            enlist_choices: Vec::new(),
            exert_choices: Vec::new(),
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        }),
        LegalAction::DeclareBlockers { .. } => Ok(Command::DeclareBlockers {
            player,
            blockers: params.blockers.clone(),
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
        LegalAction::TurnFaceUp { permanent, method } => Ok(Command::TurnFaceUp {
            player,
            permanent: *permanent,
            method: method.clone(),
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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
        // KNOWN GAP (M11-local, to be filed for S6/S7): a loyalty ability's targets
        // cannot be announced. `ActivateLoyaltyAbility` is outside the five-arm
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
        // CR 603.3d / CR 601.2c (PB-DP8 / DP-6): submit the engine's OWN default
        // verbatim, do not randomise/parameterize. Randomising would change every
        // fuzzer seed's outcome (OOS-DP8-1); a human client wanting a different
        // answer is out of scope for this session.
        LegalAction::DiscardToHandSize { cards, .. } => Ok(Command::DiscardToHandSize {
            player,
            cards: cards.clone(),
        }),
        LegalAction::ChooseTriggerTargets {
            choice_id, targets, ..
        } => Ok(Command::ChooseTriggerTargets {
            player,
            choice_id: *choice_id,
            targets: targets.clone(),
        }),
        // CR 608.2d (PB-DP9 / DP-7/8/9): submit the engine's OWN default verbatim,
        // for the same reason as ChooseTriggerTargets above.
        LegalAction::AnswerEffectChoice {
            choice_id, answer, ..
        } => Ok(Command::AnswerEffectChoice {
            player,
            choice_id: *choice_id,
            answer: answer.clone(),
        }),
    }
}
