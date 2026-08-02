//! Engine integration: command processing and game loop (CR 500-514).
//!
//! `process_command` is the single public entry point. It takes an immutable
//! GameState and a Command, produces a new GameState and a list of events.
//! State module = data, rules module = behavior.
use super::abilities;
use super::casting;
use super::combat;
use super::command::{CastSpellData, Command};
use super::commander;
use super::events::GameEvent;
use super::foretell;
use super::lands;
use super::loop_detection;
use super::mana;
use super::miracle;
use super::plot;
use super::priority::{self, PriorityResult};
use super::replacement;
use super::resolution;
use super::sba;
use super::suspend;
use super::turn_actions;
use super::turn_structure;
use crate::state::diagnostics::debug_assert_object_live;
use crate::state::error::GameStateError;
use crate::state::game_object::{Designations, ObjectId};
use crate::state::player::PlayerId;
use crate::state::stubs::FlushResumeSite;
use crate::state::GameState;
/// CR 603.3: Check for triggered abilities arising from events and flush
/// pending triggers to the stack. Extracted from per-command-arm boilerplate.
fn check_and_flush_triggers(state: &mut GameState, events: &mut Vec<GameEvent>) {
    let new_triggers = abilities::check_triggers(state, events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    // CR 610.3 cleanup: Remove WhenSourceLeavesBattlefield delayed triggers whose
    // source is no longer on the battlefield. This prevents re-firing on subsequent
    // event batches. Also remove triggers that have already fired.
    {
        use crate::state::stubs::DelayedTriggerTiming;
        use crate::state::zone::ZoneId;
        // Collect IDs of sources that are still on the battlefield.
        let sources_on_bf: std::collections::HashSet<crate::state::game_object::ObjectId> = state
            .objects
            .values()
            .filter(|o| o.zone == ZoneId::Battlefield)
            .map(|o| o.id)
            .collect();
        state.delayed_triggers.retain(|dt| {
            if dt.fired {
                return false;
            }
            if dt.timing == DelayedTriggerTiming::WhenSourceLeavesBattlefield {
                return sources_on_bf.contains(&dt.source);
            }
            true
        });
    }
    let trigger_events = abilities::flush_pending_triggers(state);
    events.extend(trigger_events);
}
/// The one decision, if any, that is currently gating the game (PB-DP7 / DP-3,
/// PB-DP8 / DP-6).
///
/// Two kinds today: CR 514.1's cleanup discard and CR 603.3d's triggered-ability
/// target announcement. PB-DP9 (CR 701.22a/701.23/701.25a) is expected to append
/// more.
///
/// **What genuinely generalises, and what does not (fix-cycle Findings 3+4):**
/// the PROGRESS gate (`enter_step`'s consult at the bottom of this file) is
/// written against this enum, not the field it wraps, so a new variant needs
/// no new progress-gate site -- that part of the original claim holds. It is
/// **not** true of every consult site: the ADMISSION gate below
/// (`process_command`) hard-codes `Command::DiscardToHandSize` in its
/// allow-list, and a new variant's answering command must be added there
/// explicitly. Consumers outside this crate (the simulator, the TUI) must go
/// through `GameState::blocking_decision()` -- which applies the same
/// liveness filter this type's constructor does -- rather than reading
/// `pending_cleanup_discard()` or any future per-kind field directly; reading
/// the raw field bypasses the filter and can disagree with the engine's own
/// gate (see `handle_concede`'s dead-player note). See the plan's §1 for the
/// full design ("The blocking pending-decision mechanism") and its §1.5,
/// which this comment amends.
///
/// **What a SECOND variant actually cost (PB-DP8 measured it; this replaces
/// PB-DP7's aspirational "no new consult site" claim).** **Six** per-variant
/// obligations, none of them compile-forced by this enum alone (the last two
/// were found by PB-DP8's own review, after four shipped):
/// 1. The ADMISSION gate's allow-list in `process_command` below -- the
///    answering `Command` must be named there explicitly or the engine rejects
///    its own answer.
/// 2. `handle_concede` must clear the new per-kind field, or a stale entry
///    outlives the player who owned it (see the dead-player note above). For
///    PB-DP8 that clear is not a mere `= None`: CR 800.4d drops the conceding
///    player's un-placed trigger and CR 603.3b/800.4j require the REST of the
///    batch to still be placed, so `handle_concede` resumes the flush.
/// 3. The field is hashed BY NAME in two places -- `rules/loop_detection.rs`'s
///    mandatory-state fingerprint and `state/hash.rs`'s `public_state_hash` --
///    and the `HashInto` impl for its payload struct must use the BARE type
///    name, or SR-19's `every_hashed_struct_field_is_hashed_or_allowlisted`
///    gate silently skips it (OOS-DP7-11).
/// 4. `crates/simulator/src/local_game.rs`'s `advance()` maps the decision to a
///    `DecisionKind` for the client. Before PB-DP8 it hard-coded
///    `DecisionKind::CleanupDiscard` without matching the variant; it is now an
///    exhaustive `match`, which makes THIS obligation compile-forced for every
///    future variant (this enum is deliberately not `#[non_exhaustive]`).
/// 5. `handle_concede` must ALSO refuse to advance priority or the turn while
///    ANOTHER player's decision is outstanding (fix-cycle Finding 5 /
///    seed OOS-DP8-9). Item 2 covers the conceder's own entry; a foreign concede
///    otherwise walks straight into `handle_all_passed` -> resolution and, for
///    PB-DP8, into `flush_pending_triggers`' re-entrancy `debug_assert!`.
/// 6. Every site that RESUMES the blocked engine must reproduce what the guard
///    it replaced was about to do -- not just "grant priority". PB-DP8 shipped a
///    `bool` for that and it was too narrow: `enter_step`'s two guards also owed
///    a CR 726 mandatory-loop check, and its Cleanup guard a `cleanup_sba_rounds`
///    ratchet (fix-cycle Finding 4 / seed OOS-DP8-10). See
///    [`crate::state::stubs::FlushResumeSite`].
///
/// **What the THIRD variant cost (PB-DP9 measured it).** The list generalises --
/// this is the first evidence of that -- and PB-DP9 discharged it as follows:
/// (1) admission gate -- **yes**, one line;
/// (2) `handle_concede` clear -- **yes**, and not a bare `= None`: see
///     `discharge_departed_effect_choice`, which must also DRIVE the rolled-back
///     resolution or the game deadlocks with `priority_holder == None` and every
///     player already passed;
/// (3) the two by-name hash sites -- **`public_state_hash` yes,
///     `loop_detection` deliberately NO**; see obligation (7);
/// (4) `LocalGame`'s exhaustive `match` -- compile-forced, as designed;
/// (5) `handle_concede`'s foreign-decision gate -- **inherited free**, because it
///     reads `blocking_decision(..)` rather than any one field;
/// (6) resume-site debt -- **does not apply**. PB-DP9's suspension is a total
///     state RESTORE, so nothing was skipped and there is no debt to reproduce.
///     Its one residual, `handle_all_passed`'s two post-resolution statements,
///     is factored into `finish_stack_resolution` so the two sites cannot drift.
///
/// **Obligation (7), added by PB-DP9**: a new blocking kind must STATE whether
/// its pending state belongs in `rules/loop_detection.rs`'s mandatory-state
/// fingerprint, and argue it. PB-DP7 and PB-DP8 both folded theirs in; PB-DP9
/// deliberately does not, because its entry and its answer bank GROW between
/// successive replays of the SAME resolution, so including them would make two
/// structurally identical CR 726 positions fingerprint differently and could
/// silently mask a mandatory loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockingDecision {
    /// CR 514.1: `player` must discard `count` cards to reach their maximum
    /// hand size before the cleanup step can complete.
    CleanupDiscard { player: PlayerId, count: u32 },
    /// CR 603.3d (PB-DP8 / DP-6): `player` must announce the targets of the
    /// triggered ability from `source` before the CR 603.3b batch can continue.
    TriggerTargets {
        player: PlayerId,
        choice_id: u64,
        source: ObjectId,
    },
    /// CR 608.2d (PB-DP9 / DP-7/8/9): `player` must announce a resolution-time
    /// choice offered by the spell or ability resolving from `source` before
    /// that resolution can be retried. The resolution has been rolled back --
    /// see `rules::resolution::resolve_top_of_stack`.
    EffectChoice {
        player: PlayerId,
        choice_id: u64,
        source: ObjectId,
    },
}
impl BlockingDecision {
    /// The player whose answer would clear this decision.
    pub fn player(&self) -> PlayerId {
        match self {
            BlockingDecision::CleanupDiscard { player, .. } => *player,
            BlockingDecision::TriggerTargets { player, .. } => *player,
            BlockingDecision::EffectChoice { player, .. } => *player,
        }
    }
}
impl std::fmt::Display for BlockingDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockingDecision::CleanupDiscard { player, count } => {
                write!(
                    f,
                    "CR 514.1 cleanup discard: player {:?} must discard {} card(s)",
                    player, count
                )
            }
            BlockingDecision::TriggerTargets {
                player,
                choice_id,
                source,
            } => {
                write!(
                    f,
                    "CR 603.3d trigger targets: player {:?} must announce targets \
                     for the triggered ability from {:?} (choice {})",
                    player, source, choice_id
                )
            }
            BlockingDecision::EffectChoice {
                player,
                choice_id,
                source,
            } => {
                write!(
                    f,
                    "CR 608.2d resolution-time choice: player {:?} must answer the \
                     choice offered by {:?} (choice {})",
                    player, source, choice_id
                )
            }
        }
    }
}
/// Read the one pending decision (if any) currently gating the game.
///
/// Returns `None` if the entry's player is no longer alive (`has_lost ||
/// has_conceded`) -- a dead player's entry cannot block anyone (see
/// `handle_concede`, which also clears the field itself so it does not
/// pollute the state hash).
pub(crate) fn blocking_decision(state: &GameState) -> Option<BlockingDecision> {
    fn alive(state: &GameState, p: PlayerId) -> bool {
        state
            .players()
            .get(&p)
            .map(|pl| !pl.has_lost && !pl.has_conceded)
            .unwrap_or(false)
    }
    // CR 603.3d (PB-DP8): a suspended trigger flush outranks a cleanup discard --
    // the two cannot legally coexist (the cleanup discard is recorded by
    // `cleanup_actions`, which the progress gate stops before it can run while a
    // flush is suspended), but if they ever did, the flush is the one the engine
    // is mid-way through and must finish first (CR 603.3b).
    if let Some(entry) = state.pending_trigger_targets() {
        if alive(state, entry.player) {
            return Some(BlockingDecision::TriggerTargets {
                player: entry.player,
                choice_id: entry.choice_id,
                source: entry.source,
            });
        }
        return None;
    }
    // CR 608.2d (PB-DP9): a suspended resolution outranks a cleanup discard for
    // the same reason -- it is the thing the engine is mid-way through. It and a
    // suspended trigger flush are mutually exclusive AT A COMMAND BOUNDARY, but
    // not by construction: the aborted inner pass CAN set both (its tail calls
    // `flush_pending_triggers`), and it is `resolve_top_of_stack`'s whole-state
    // restore that un-sets the trigger one. That is why the trigger lookup is
    // still first -- if a state ever carried both, the flush is the older
    // obligation.
    if let Some(entry) = state.pending_effect_choice() {
        if alive(state, entry.player) {
            return Some(BlockingDecision::EffectChoice {
                player: entry.player,
                choice_id: entry.choice_id,
                source: entry.source,
            });
        }
        return None;
    }
    let entry = state.pending_cleanup_discard()?;
    if !alive(state, entry.player) {
        return None;
    }
    Some(BlockingDecision::CleanupDiscard {
        player: entry.player,
        count: entry.count,
    })
}
/// Process a player command against the current game state.
///
/// Returns the new game state and a list of events describing what happened.
/// The old state is not modified (immutable state model).
pub fn process_command(
    state: GameState,
    command: Command,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    let mut all_events = Vec::new();
    // Validate: game not over
    if is_game_over(&state) {
        return Err(GameStateError::GameAlreadyOver);
    }
    // PB-DP7 / DP-3 admission gate (CR 514.3: no player has priority in
    // cleanup, so no spell can be cast and no ability activated while a
    // BlockingDecision is outstanding). Only the answering command from the
    // named player, and `Concede` from any player, are legal -- see the
    // plan's §1.4. Mana abilities are NOT exempted, unlike PB-DP4's payment
    // deadlines: CR 605.3a requires priority to activate one, and nobody has
    // priority here.
    if let Some(decision) = blocking_decision(&state) {
        let allowed = matches!(&command, Command::Concede { .. })
            || matches!(&command, Command::DiscardToHandSize { player, .. } if *player == decision.player())
            // CR 603.3d (PB-DP8 / DP-6): the trigger's controller announcing its
            // targets. CR 603.3 gives priority only AFTER the whole batch is on
            // the stack, so nothing else is legal mid-flush either.
            || matches!(&command, Command::ChooseTriggerTargets { player, .. } if *player == decision.player())
            // CR 608.2d (PB-DP9 / DP-7/8/9): the named player answering the
            // resolution-time choice. The resolution is rolled back and the
            // spell is still on the stack, so nothing else is legal either.
            || matches!(&command, Command::AnswerEffectChoice { player, .. } if *player == decision.player());
        if !allowed {
            return Err(GameStateError::BlockedByPendingDecision {
                player: decision.player(),
                decision: decision.to_string(),
            });
        }
    }
    match command {
        Command::PassPriority { player } => {
            validate_player_active(&state, player)?;
            let events = handle_pass_priority(&mut state, player)?;
            all_events.extend(events);
        }
        Command::Concede { player } => {
            validate_player_exists(&state, player)?;
            let events = handle_concede(&mut state, player)?;
            all_events.extend(events);
        }
        Command::TapForMana {
            player,
            source,
            ability_index,
            chosen_color,
            hybrid_choices,
            phyrexian_life_payments,
        } => {
            validate_player_active(&state, player)?;
            let events = mana::handle_tap_for_mana(
                &mut state,
                player,
                source,
                ability_index,
                chosen_color,
                hybrid_choices,
                phyrexian_life_payments,
            )?;
            all_events.extend(events);
        }
        Command::PlayLand { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: playing a land is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = lands::handle_play_land(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        Command::CastSpell(cast) => {
            let CastSpellData {
                player,
                card,
                targets,
                convoke_creatures,
                improvise_artifacts,
                delve_cards,
                kicker_times,
                alt_cost,
                prototype,
                modes_chosen,
                x_value,
                hybrid_choices,
                phyrexian_life_payments,
                face_down_kind,
                additional_costs,
            } = *cast;
            validate_player_active(&state, player)?;
            // CR 104.4b: casting a spell is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = casting::handle_cast_spell(
                &mut state,
                player,
                card,
                targets,
                convoke_creatures,
                improvise_artifacts,
                delve_cards,
                kicker_times,
                alt_cost,
                prototype,
                modes_chosen,
                x_value,
                face_down_kind,
                additional_costs,
                hybrid_choices,
                phyrexian_life_payments,
            )?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        Command::ActivateAbility {
            player,
            source,
            ability_index,
            targets,
            discard_card,
            sacrifice_target,
            x_value,
            modes_chosen,
            hybrid_choices,
            phyrexian_life_payments,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: activating an ability is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_activate_ability(
                &mut state,
                player,
                source,
                ability_index,
                targets,
                discard_card,
                sacrifice_target,
                x_value,
                modes_chosen,
                hybrid_choices,
                phyrexian_life_payments,
            )?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        Command::DeclareAttackers {
            player,
            attackers,
            enlist_choices,
            exert_choices,
            hybrid_choices,
            phyrexian_life_payments,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: declaring attackers is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let events = combat::handle_declare_attackers(
                &mut state,
                player,
                attackers,
                enlist_choices,
                exert_choices,
                hybrid_choices,
                phyrexian_life_payments,
            )?;
            all_events.extend(events);
        }
        Command::DeclareBlockers { player, blockers } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: declaring blockers is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let events = combat::handle_declare_blockers(&mut state, player, blockers)?;
            all_events.extend(events);
        }
        Command::OrderBlockers {
            player,
            attacker,
            order,
        } => {
            validate_player_active(&state, player)?;
            let events = combat::handle_order_blockers(&mut state, player, attacker, order)?;
            all_events.extend(events);
        }
        Command::OrderReplacements { player, ids } => {
            validate_player_active(&state, player)?;
            let events = replacement::handle_order_replacements(&mut state, player, ids)?;
            all_events.extend(events);
        }
        Command::ReturnCommanderToCommandZone { player, object_id } => {
            // CR 903.9a / CR 704.6d: owner chooses to return their commander
            // from graveyard or exile to the command zone. Clears the pending
            // commander zone-return choice recorded by the SBA.
            validate_player_exists(&state, player)?;
            let events =
                commander::handle_return_commander_to_command_zone(&mut state, player, object_id)?;
            all_events.extend(events);
        }
        Command::LeaveCommanderInZone { player, object_id } => {
            // CR 903.9a: owner chooses to leave their commander in graveyard or
            // exile rather than returning it to the command zone.
            validate_player_exists(&state, player)?;
            let events = commander::handle_leave_commander_in_zone(&mut state, player, object_id)?;
            all_events.extend(events);
        }
        // ── M9: Mulligan commands (CR 103.5 / CR 103.5c) ─────────────────
        Command::TakeMulligan { player } => {
            validate_player_exists(&state, player)?;
            let events = commander::handle_take_mulligan(&mut state, player)?;
            all_events.extend(events);
        }
        Command::KeepHand {
            player,
            cards_to_bottom,
        } => {
            validate_player_exists(&state, player)?;
            let events = commander::handle_keep_hand(&mut state, player, cards_to_bottom)?;
            all_events.extend(events);
        }
        // ── M9: Companion command (CR 702.139a) ───────────────────────────
        Command::BringCompanion { player } => {
            validate_player_active(&state, player)?;
            let events = commander::handle_bring_companion(&mut state, player)?;
            all_events.extend(events);
        }
        // ── Forecast (CR 702.57) ──────────────────────────────────────────
        Command::ActivateForecast {
            player,
            card,
            targets,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: forecast activation is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events =
                abilities::handle_activate_forecast(&mut state, player, card, targets)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Bloodrush (CR 207.2c) ─────────────────────────────────────────
        Command::ActivateBloodrush {
            player,
            card,
            target,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: bloodrush activation is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events =
                abilities::handle_activate_bloodrush(&mut state, player, card, target)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Cycling (CR 702.29) ───────────────────────────────────────────
        Command::CycleCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: cycling is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_cycle_card(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Dredge (CR 702.52) ───────────────────────────────────────────
        Command::ChooseDredge { player, card } => {
            // CR 702.52: Handle the player's dredge choice.
            // No validate_player_active needed — dredge can replace draws during any
            // draw effect, not just the active player's draw step.
            validate_player_exists(&state, player)?;
            // CR 104.4b: dredge is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = replacement::handle_choose_dredge(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Miracle (CR 702.94) ──────────────────────────────────────────
        Command::ChooseMiracle {
            player,
            card,
            reveal,
        } => {
            // CR 702.94a: Handle the player's miracle reveal choice.
            // No validate_player_active needed — miracle can trigger on any player's draw,
            // not just the active player's draw step.
            validate_player_exists(&state, player)?;
            // CR 104.4b: choosing to reveal a miracle card is a meaningful player choice.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = miracle::handle_choose_miracle(&mut state, player, card, reveal)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Cleanup discard (CR 514.1, PB-DP7 / DP-3) ────────────────────
        Command::DiscardToHandSize { player, cards } => {
            // validate_player_exists, NOT validate_player_active: the entry's
            // player is by construction the active player, who is by
            // construction alive (a dead player's entry is dropped in
            // blocking_decision). Precedent: ChooseDredge above.
            validate_player_exists(&state, player)?;
            // CR 104.4b: a cleanup discard is a meaningful player choice.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = turn_actions::handle_discard_to_hand_size(&mut state, player, cards)?;
            // Fix-cycle Finding 7 (LOW): the events above (including
            // `GameEvent::DiscardedToHandSize`) are never passed through
            // `abilities::check_triggers` / `check_and_flush_triggers` -- this
            // is harmless ONLY because `check_triggers` has no
            // `DiscardedToHandSize` arm today, so nothing is watching for it.
            // If a "whenever you discard a card" trigger arm is ever added
            // there (seed OOS-DP7-10 -- Waste Not, Bone Miser, Containment
            // Construct), this call site will need one too, or those
            // abilities will silently never fire off a CR 514.1 discard.
            // Resume the cleanup step: `cleanup_actions` is idempotent once the
            // hand is at max size (the pause is taken before any discard), so
            // this second pass completes CR 514.2 / CR 500.5 / CleanupPerformed
            // normally -- the same shape the CR 514.3a extra-round machinery
            // already relies on. `enter_step` also runs the SBA/trigger/priority
            // logic that a plain `check_and_flush_triggers` would not.
            //
            // Fix-cycle Finding 2 (HIGH): `handle_discard_to_hand_size` above
            // now rejects the command outright unless `state.turn.step ==
            // Step::Cleanup`, so mutation can never happen out of step and
            // this resume is already unreachable when it should not run.
            // Guarded explicitly here too, defense-in-depth, so a future
            // change to the handler's validation order cannot silently
            // resurrect the hazard the handler's own check exists to close.
            debug_assert_eq!(
                state.turn.step,
                crate::state::turn::Step::Cleanup,
                "handle_discard_to_hand_size must have rejected this command otherwise"
            );
            if state.turn.step == crate::state::turn::Step::Cleanup {
                let enter_events = enter_step(&mut state)?;
                events.extend(enter_events);
            }
            all_events.extend(events);
        }
        // ── Triggered-ability targets (CR 603.3d) ─────────────────────────
        Command::ChooseTriggerTargets {
            player,
            choice_id,
            targets,
        } => {
            // validate_player_exists, NOT validate_player_active: the entry's
            // player is by construction alive (a dead player's entry is dropped
            // in blocking_decision, and the flush never asks a dead controller).
            // Precedent: ChooseDredge / DiscardToHandSize above.
            validate_player_exists(&state, player)?;
            // CR 104.4b: announcing a target is a meaningful player choice.
            loop_detection::reset_loop_detection(&mut state);
            let mut events =
                abilities::handle_choose_trigger_targets(&mut state, player, choice_id, targets)?;
            // CR 603.3: the batch may have completed, in which case the abilities
            // just placed can themselves have triggered something (e.g. a
            // Panharmonicon-style watcher on `AbilityTriggered`). Run the normal
            // post-command trigger sweep -- but ONLY if the flush did not suspend
            // again on the next trigger of the same batch, because
            // `check_and_flush_triggers` would then re-enter a suspended flush.
            if state.pending_trigger_targets.is_none() {
                check_and_flush_triggers(&mut state, &mut events);
            }
            all_events.extend(events);
        }
        // ── Resolution-time choices (CR 608.2d) ──────────────────────────
        Command::AnswerEffectChoice {
            player,
            choice_id,
            answer,
        } => {
            // validate_player_exists, NOT validate_player_active: the entry's
            // player is alive by construction (`blocking_decision` filters a dead
            // one out and `ask_or_consume_effect_choice` never asks a departed
            // player). Precedent: ChooseTriggerTargets / DiscardToHandSize.
            validate_player_exists(&state, player)?;
            // CR 104.4b: a resolution-time announcement is a meaningful player
            // choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events =
                crate::effects::handle_answer_effect_choice(&mut state, player, choice_id, answer)?;
            // CR 608.1: the resolution either completed or suspended again on a
            // LATER choice of the same resolution. Either way this site inherits
            // `handle_all_passed`'s two post-resolution statements (§1.5 exit 1)
            // -- and on the suspended path both are provable no-ops, for the
            // reason spelled out at that call site.
            if finish_stack_resolution(&mut state, &mut events) {
                return Ok((state, events));
            }
            // CR 603.3: a completed resolution can have produced triggers. Skip
            // the sweep when the engine is blocked again -- on a later CR 608.2d
            // choice (the stack object is back and its triggers have not
            // happened) or on a CR 603.3d flush that `resolve_top_of_stack`'s
            // tail suspended, where `check_and_flush_triggers` would re-enter a
            // suspended flush. Mirrors `Command::ChooseTriggerTargets`'s guard.
            if blocking_decision(&state).is_none() {
                check_and_flush_triggers(&mut state, &mut events);
            }
            all_events.extend(events);
        }
        // ── Crew (CR 702.122) ────────────────────────────────────────────
        Command::CrewVehicle {
            player,
            vehicle,
            crew_creatures,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: crewing a vehicle is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events =
                abilities::handle_crew_vehicle(&mut state, player, vehicle, crew_creatures)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Saddle (CR 702.171) ──────────────────────────────────────────────
        Command::SaddleMount {
            player,
            mount,
            saddle_creatures,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: saddling is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events =
                abilities::handle_saddle_mount(&mut state, player, mount, saddle_creatures)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Foretell (CR 702.143) ─────────────────────────────────────────
        Command::ForetellCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: foretelling is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = foretell::handle_foretell_card(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Plot (CR 702.170) ─────────────────────────────────────────────
        Command::PlotCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: plotting is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = plot::handle_plot_card(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
            // CR 116.3: Special action => player receives priority afterward.
            // Priority is already set to the player since they have priority.
        }
        // ── Suspend (CR 702.62) ───────────────────────────────────────────
        Command::SuspendCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: suspending is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = suspend::handle_suspend_card(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        Command::UnearthCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: unearth is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_unearth_card(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Embalm (CR 702.128) ──────────────────────────────────────────────
        Command::EmbalmCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: embalm is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_embalm_card(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Eternalize (CR 702.129) ──────────────────────────────────────────
        Command::EternalizeCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: eternalize is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_eternalize_card(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Encore (CR 702.141) ─────────────────────────────────────────────
        Command::EncoreCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: encore is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_encore_card(&mut state, player, card)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Scavenge (CR 702.97) ─────────────────────────────────────────────
        Command::ScavengeCard {
            player,
            card,
            target_creature,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: scavenge is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events =
                abilities::handle_scavenge_card(&mut state, player, card, target_creature)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        Command::ActivateNinjutsu {
            player,
            ninja_card,
            attacker_to_return,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: ninjutsu is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events =
                abilities::handle_ninjutsu(&mut state, player, ninja_card, attacker_to_return)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Echo (CR 702.30) ─────────────────────────────────────────────
        Command::PayEcho {
            player,
            permanent,
            pay,
        } => {
            // CR 702.30a: Handle the player's echo payment choice.
            // No validate_player_active needed -- echo can resolve during any upkeep,
            // but the player must be the permanent's controller.
            validate_player_exists(&state, player)?;
            // CR 104.4b: paying echo is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = handle_pay_echo(&mut state, player, permanent, pay)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Recover (CR 702.59) ──────────────────────────────────────────
        Command::PayRecover {
            player,
            recover_card,
            pay,
        } => {
            // CR 702.59a: Handle the player's recover payment choice.
            validate_player_exists(&state, player)?;
            // CR 104.4b: paying recover is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = handle_pay_recover(&mut state, player, recover_card, pay)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Cumulative Upkeep (CR 702.24) ────────────────────────────────
        Command::PayCumulativeUpkeep {
            player,
            permanent,
            pay,
        } => {
            // CR 702.24a: Handle the player's cumulative upkeep payment choice.
            validate_player_exists(&state, player)?;
            // CR 104.4b: paying cumulative upkeep is a meaningful player choice.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = handle_pay_cumulative_upkeep(&mut state, player, permanent, pay)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Transform (CR 701.27 / CR 712) ───────────────────────────────
        Command::Transform { player, permanent } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: transforming is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = handle_transform(&mut state, player, permanent)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Craft (CR 702.167) ────────────────────────────────────────────
        Command::ActivateCraft {
            player,
            source,
            material_ids,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: activating craft is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = handle_activate_craft(&mut state, player, source, material_ids)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── The Ring Tempts You (CR 701.54) ──────────────────────────────────
        Command::TheRingTemptsYou { player } => {
            let mut events = handle_ring_tempts_you(&mut state, player)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        // ── Dungeon / Venture (CR 701.49) ────────────────────────────────────
        Command::VentureIntoDungeon { player } => {
            let mut events = handle_venture_into_dungeon(&mut state, player, false)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        Command::ChooseDungeonRoom { player: _, room: _ } => {
            // CR 309.5a: Deterministic fallback — the engine already picked the first exit.
            // This command is accepted but does nothing in the current implementation.
            // Full interactive branching is deferred to M10+.
        }
        // ── Morph / Manifest / Cloak: Turn Face Up (CR 702.37e, 701.40b, 701.58b) ─
        Command::TurnFaceUp {
            player,
            permanent,
            method,
            // PB-DX6 stage B: threaded into `handle_turn_face_up`, which flattens them
            // against whichever cost `method` resolves to (CR 107.4e/107.4f via CR
            // 701.40b/702.37e/702.168d).
            hybrid_choices,
            phyrexian_life_payments,
        } => {
            validate_player_active(&state, player)?;
            // CR 116.2b: Turn face up is a special action; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = handle_turn_face_up(
                &mut state,
                player,
                permanent,
                method,
                hybrid_choices,
                phyrexian_life_payments,
            )?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        Command::ActivateLoyaltyAbility {
            player,
            source,
            ability_index,
            targets,
            x_value,
        } => {
            validate_player_active(&state, player)?;
            loop_detection::reset_loop_detection(&mut state);
            let mut events = handle_activate_loyalty_ability(
                &mut state,
                player,
                source,
                ability_index,
                targets,
                x_value,
            )?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
        Command::LevelUpClass {
            player,
            source,
            target_level,
        } => {
            validate_player_active(&state, player)?;
            loop_detection::reset_loop_detection(&mut state);
            let mut events = handle_level_up_class(&mut state, player, source, target_level)?;
            check_and_flush_triggers(&mut state, &mut events);
            all_events.extend(events);
        }
    }
    // Record events in history
    for event in &all_events {
        state.history.push_back(event.clone());
    }
    Ok((state, all_events))
}
/// CR 702.30a: Handle the player's echo payment choice.
///
/// If `pay` is true, deducts the echo cost from the player's mana pool and
/// clears `echo_pending` on the permanent. If `pay` is false (or the player
/// cannot afford it), the permanent is sacrificed (bypassing indestructible,
/// CR 701.21a) and `echo_pending` is cleared.
///
/// In both cases, the pending echo payment entry is removed.
fn handle_pay_echo(
    state: &mut GameState,
    player: PlayerId,
    permanent: crate::state::game_object::ObjectId,
    pay: bool,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::zone::ZoneId;
    let mut events = Vec::new();
    // Find and remove the matching pending echo payment.
    let payment_pos = state
        .pending_echo_payments
        .iter()
        .position(|(p, obj, _)| *p == player && *obj == permanent);
    let echo_cost = if let Some(pos) = payment_pos {
        let (_, _, cost) = state.pending_echo_payments.remove(pos);
        cost
    } else {
        // No pending payment for this permanent -- stale or invalid command.
        return Err(GameStateError::InvalidCommand(format!(
            "No pending echo payment for player {:?} permanent {:?}",
            player, permanent
        )));
    };
    // Validate: permanent must still be on the battlefield.
    let source_info = state.objects.get(&permanent).and_then(|obj| {
        if obj.zone == ZoneId::Battlefield {
            // CR 603.10a / CR 613.1d: capture full layer-resolved characteristics BEFORE any zone move.
            // `permanent` is present (this closure runs inside `state.objects.get(&permanent)`),
            // so calculate_characteristics cannot return None (its only failure is an absent id).
            let pre_chars = Some(crate::rules::layers::expect_characteristics(
                state, permanent,
            ));
            let lki_power = pre_chars
                .as_ref()
                .and_then(|c| c.power)
                .or(obj.characteristics.power);
            Some((
                obj.owner,
                obj.controller,
                obj.counters.clone(),
                lki_power,
                pre_chars,
            ))
        } else {
            None
        }
    });
    let Some((owner, controller, pre_death_counters, echo_lki_power, echo_pre_chars)) = source_info
    else {
        // Permanent left the battlefield since the trigger resolved; nothing to do.
        return Ok(events);
    };
    // CR 702.30a: Clear the echo_pending flag regardless of pay/sacrifice.
    // `permanent` was just proven live via the `source_info` guard above.
    if let Some(obj) = state.expect_object_mut(permanent) {
        obj.designations.remove(Designations::ECHO_PENDING);
    }
    if pay {
        // CR 702.30a: Player pays the echo cost.
        // Validate: player has sufficient mana.
        let pool = &state
            .players
            .get(&player)
            .ok_or(GameStateError::PlayerNotFound(player))?
            .mana_pool;
        let can_afford = casting::can_pay_cost(pool, &echo_cost);
        if !can_afford {
            return Err(GameStateError::InvalidCommand(format!(
                "Player {:?} cannot afford echo cost",
                player
            )));
        }
        // Deduct the mana.
        if let Some(p) = state.expect_player_mut(player) {
            casting::pay_cost(&mut p.mana_pool, &echo_cost);
        }
        events.push(GameEvent::EchoPaid { player, permanent });
    } else {
        // CR 702.30a: Player declines -- sacrifice the permanent (CR 701.21a: bypasses indestructible).
        let action = crate::rules::replacement::check_zone_change_replacement(
            state,
            permanent,
            crate::state::zone::ZoneType::Battlefield,
            crate::state::zone::ZoneType::Graveyard,
            owner,
            &std::collections::HashSet::new(),
        );
        match action {
            crate::rules::replacement::ZoneChangeAction::Redirect {
                to: dest,
                events: repl_events,
                ..
            } => {
                events.extend(repl_events);
                if let Ok((new_id, _old)) = state.move_object_to_zone(permanent, dest) {
                    match dest {
                        ZoneId::Exile => {
                            events.push(GameEvent::ObjectExiled {
                                player: owner,
                                object_id: permanent,
                                new_exile_id: new_id,
                                pre_lba_counters: pre_death_counters.clone(),
                                // CR 603.10a: LKI power snapshot for SourcePowerAtLastKnownInformation.
                                pre_lba_power: echo_lki_power,
                            });
                        }
                        ZoneId::Command(_) => {
                            // Commander redirected to command zone; no sacrifice event.
                        }
                        _ => {
                            events.push(GameEvent::CreatureDied {
                                object_id: permanent,
                                new_grave_id: new_id,
                                controller,
                                pre_death_counters,
                                // CR 603.10a: LKI power snapshot for SourcePowerAtLastKnownInformation.
                                pre_death_power: echo_lki_power,
                                pre_death_characteristics: echo_pre_chars.clone(),
                            });
                        }
                    }
                }
            }
            crate::rules::replacement::ZoneChangeAction::Proceed => {
                if let Ok((new_grave_id, _old)) =
                    state.move_object_to_zone(permanent, ZoneId::Graveyard(owner))
                {
                    events.push(GameEvent::CreatureDied {
                        object_id: permanent,
                        new_grave_id,
                        controller,
                        pre_death_counters,
                        // CR 603.10a: LKI power snapshot for SourcePowerAtLastKnownInformation.
                        pre_death_power: echo_lki_power,
                        pre_death_characteristics: echo_pre_chars.clone(),
                    });
                }
            }
            crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                player: choice_player,
                choices,
                event_description,
            } => {
                // CR 616.1: Multiple replacement effects -- defer to player choice.
                state.pending_zone_changes.push_back(
                    crate::state::replacement_effect::PendingZoneChange {
                        object_id: permanent,
                        original_from: crate::state::zone::ZoneType::Battlefield,
                        original_destination: crate::state::zone::ZoneType::Graveyard,
                        affected_player: choice_player,
                        already_applied: Vec::new(),
                    },
                );
                events.push(GameEvent::ReplacementChoiceRequired {
                    player: choice_player,
                    event_description,
                    choices,
                });
            }
        }
    }
    // CR 704.3: Check SBAs after echo resolution.
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);
    // CR 702.30a / CR 608.2d / CR 117.3c -- PB-DP4, closes OOS-DP1-1.
    //
    // Paying or declining is a resolution-time cost choice (CR 118.12 / 608.2d), not an
    // action that grants priority, so there is no actor for CR 117.3c to hand priority to.
    // This site used to write `priority_holder = Some(active_player)` and clear
    // `players_passed` as a bodge standing in for the payment pause DP-11 said was never
    // implemented (PB-DP1 correctly left it alone; it is exactly the OOS-DP1-1 seed). The
    // pause now exists as a DEADLINE (`force_resolve_overdue_payments`), so the bodge is
    // gone: the controller of an echo trigger IS the active player (the trigger reads "at
    // the beginning of YOUR upkeep"), and `resolve_top_of_stack`
    // (rules/resolution.rs:~2799-2841) already cleared `players_passed` and granted
    // priority to the active player before this command could arrive. Removing the write
    // is an identity write in the common case where no priority-granting action has
    // intervened since resolution (fix cycle, E4: the pre-fix comment overstated this as
    // an unconditional no-op). Post-PB-DP1, priority follows the actor (CR 117.3c) --
    // if a RESPONDER cast a spell or activated an ability during the window between
    // resolution and this command, priority now sits with that responder, and the
    // deleted write would have yanked it to the active player instead. So the deletion
    // is a fix in that case too, exactly like recover below, just less commonly
    // triggered (echo/CU only fire on the controller's own upkeep, before most
    // responses have had a chance to happen).
    //
    // CR 117.4 is not engaged: answering an out-of-band resolution-time payment is not
    // "taking an action" between passes, so the pass set is left exactly as it is. Leaving
    // it alone is also what makes the deadline work -- a player must send `Pay*` BEFORE
    // passing, and a spurious pass-set reset would silently buy them an extra round.
    Ok(events)
}
/// CR 702.24a: Handle the player's cumulative upkeep payment choice.
///
/// If `pay` is true, deducts the total cost (per_counter_cost x age_count) from
/// the player's mana pool (mana variant) or life total (life variant) and the
/// permanent stays. If `pay` is false, the permanent is sacrificed (bypassing
/// indestructible, CR 701.21a).
///
/// In both cases, the pending payment entry is removed.
fn handle_pay_cumulative_upkeep(
    state: &mut GameState,
    player: PlayerId,
    permanent: crate::state::game_object::ObjectId,
    pay: bool,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::types::CumulativeUpkeepCost;
    use crate::state::zone::ZoneId;
    let mut events = Vec::new();
    // Find and remove the matching pending cumulative upkeep payment.
    let payment_pos = state
        .pending_cumulative_upkeep_payments
        .iter()
        .position(|(p, obj, _)| *p == player && *obj == permanent);
    let per_counter_cost = if let Some(pos) = payment_pos {
        let (_, _, cost) = state.pending_cumulative_upkeep_payments.remove(pos);
        cost
    } else {
        return Err(GameStateError::InvalidCommand(format!(
            "No pending cumulative upkeep payment for player {:?} permanent {:?}",
            player, permanent
        )));
    };
    // Validate: permanent must still be on the battlefield.
    let source_info = state.objects.get(&permanent).and_then(|obj| {
        if obj.zone == ZoneId::Battlefield {
            // CR 603.10a / CR 613.1d: capture full layer-resolved characteristics BEFORE any zone move.
            // `permanent` is present (this closure runs inside `state.objects.get(&permanent)`),
            // so calculate_characteristics cannot return None (its only failure is an absent id).
            let pre_chars = Some(crate::rules::layers::expect_characteristics(
                state, permanent,
            ));
            let lki_power = pre_chars
                .as_ref()
                .and_then(|c| c.power)
                .or(obj.characteristics.power);
            Some((
                obj.owner,
                obj.controller,
                obj.counters.clone(),
                lki_power,
                pre_chars,
            ))
        } else {
            None
        }
    });
    let Some((owner, controller, pre_death_counters, cu_lki_power, cu_pre_chars)) = source_info
    else {
        // Permanent left the battlefield since the trigger resolved; nothing to do.
        return Ok(events);
    };
    // Count age counters (already incremented during trigger resolution).
    // `permanent` was just proven live via `source_info` above; absent-counter still yields 0.
    let age_count = state
        .expect_object(permanent)
        .and_then(|obj| {
            obj.counters
                .get(&crate::state::types::CounterType::Age)
                .copied()
        })
        .unwrap_or(0);
    if pay {
        match &per_counter_cost {
            CumulativeUpkeepCost::Mana(mc) => {
                // CR 702.24a: Pay per_counter_cost x age_count mana.
                let total_cost = multiply_mana_cost(mc, age_count);
                let pool = &state
                    .players
                    .get(&player)
                    .ok_or(GameStateError::PlayerNotFound(player))?
                    .mana_pool;
                let can_afford = casting::can_pay_cost(pool, &total_cost);
                if !can_afford {
                    return Err(GameStateError::InvalidCommand(format!(
                        "Player {:?} cannot afford cumulative upkeep cost",
                        player
                    )));
                }
                if let Some(p) = state.expect_player_mut(player) {
                    casting::pay_cost(&mut p.mana_pool, &total_cost);
                }
            }
            CumulativeUpkeepCost::Life(amount) => {
                // CR 702.24a: Pay amount * age_count life.
                let total_life = amount * age_count;
                // CR 119.4: a life payment greater than 0 is legal only if
                // life_total >= the amount (CR 119.4b: 0 is always payable). PB-DP4: the
                // mana arm above already checked affordability; this one did not, so a
                // declined-by-inability upkeep silently drove the controller below 0
                // instead of sacrificing the permanent (CR 702.24a's "if you don't").
                if total_life > 0 {
                    // Fix cycle (E2): the primitive accessor `state.player(..)?` is not a
                    // bare `.players.get(` lookup (SR-25 ratchet) -- it is the vocabulary
                    // the ratchet steers new code toward, and it is byte-for-byte what the
                    // sibling `Mana` arm above uses for `.mana_pool`.
                    let life_total = state.player(player)?.life_total;
                    if (life_total as i64) < (total_life as i64) {
                        return Err(GameStateError::InsufficientLife {
                            player,
                            required: total_life,
                            actual: life_total,
                        });
                    }
                }
                if let Some(p) = state.expect_player_mut(player) {
                    p.life_lost_this_turn += total_life;
                    p.life_total -= total_life as i32;
                }
                events.push(GameEvent::LifeLost {
                    player,
                    amount: total_life,
                });
            }
        }
        events.push(GameEvent::CumulativeUpkeepPaid {
            player,
            permanent,
            age_counter_count: age_count,
        });
    } else {
        // CR 702.24a: Player declines -- sacrifice the permanent (CR 701.21a: bypasses indestructible).
        let action = crate::rules::replacement::check_zone_change_replacement(
            state,
            permanent,
            crate::state::zone::ZoneType::Battlefield,
            crate::state::zone::ZoneType::Graveyard,
            owner,
            &std::collections::HashSet::new(),
        );
        match action {
            crate::rules::replacement::ZoneChangeAction::Redirect {
                to: dest,
                events: repl_events,
                ..
            } => {
                events.extend(repl_events);
                if let Ok((new_id, _old)) = state.move_object_to_zone(permanent, dest) {
                    match dest {
                        ZoneId::Exile => {
                            events.push(GameEvent::ObjectExiled {
                                player: owner,
                                object_id: permanent,
                                new_exile_id: new_id,
                                pre_lba_counters: pre_death_counters.clone(),
                                // CR 603.10a: LKI power snapshot for SourcePowerAtLastKnownInformation.
                                pre_lba_power: cu_lki_power,
                            });
                        }
                        ZoneId::Command(_) => {
                            // Commander redirected to command zone; no sacrifice event.
                        }
                        _ => {
                            events.push(GameEvent::CreatureDied {
                                object_id: permanent,
                                new_grave_id: new_id,
                                controller,
                                pre_death_counters,
                                // CR 603.10a: LKI power snapshot for SourcePowerAtLastKnownInformation.
                                pre_death_power: cu_lki_power,
                                pre_death_characteristics: cu_pre_chars.clone(),
                            });
                        }
                    }
                }
            }
            crate::rules::replacement::ZoneChangeAction::Proceed => {
                if let Ok((new_grave_id, _old)) =
                    state.move_object_to_zone(permanent, ZoneId::Graveyard(owner))
                {
                    events.push(GameEvent::CreatureDied {
                        object_id: permanent,
                        new_grave_id,
                        controller,
                        pre_death_counters,
                        // CR 603.10a: LKI power snapshot for SourcePowerAtLastKnownInformation.
                        pre_death_power: cu_lki_power,
                        pre_death_characteristics: cu_pre_chars.clone(),
                    });
                }
            }
            crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                player: choice_player,
                choices,
                event_description,
            } => {
                state.pending_zone_changes.push_back(
                    crate::state::replacement_effect::PendingZoneChange {
                        object_id: permanent,
                        original_from: crate::state::zone::ZoneType::Battlefield,
                        original_destination: crate::state::zone::ZoneType::Graveyard,
                        affected_player: choice_player,
                        already_applied: Vec::new(),
                    },
                );
                events.push(GameEvent::ReplacementChoiceRequired {
                    player: choice_player,
                    event_description,
                    choices,
                });
            }
        }
    }
    // CR 704.3: Check SBAs after cumulative upkeep resolution.
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);
    // CR 702.24a / CR 608.2d / CR 117.3c -- PB-DP4, closes OOS-DP1-1.
    //
    // Paying or declining is a resolution-time cost choice (CR 118.12 / 608.2d), not an
    // action that grants priority, so there is no actor for CR 117.3c to hand priority to.
    // This site used to write `priority_holder = Some(active_player)` and clear
    // `players_passed` as a bodge standing in for the payment pause DP-11 said was never
    // implemented (PB-DP1 correctly left it alone; it is exactly the OOS-DP1-1 seed). The
    // pause now exists as a DEADLINE (`force_resolve_overdue_payments`), so the bodge is
    // gone: the controller of a cumulative-upkeep trigger IS the active player (the
    // trigger reads "at the beginning of YOUR upkeep"), and `resolve_top_of_stack`
    // (rules/resolution.rs:~2843-2902) already cleared `players_passed` and granted
    // priority to the active player before this command could arrive. Removing the write
    // is an identity write in the common case where no priority-granting action has
    // intervened since resolution (fix cycle, E4: the pre-fix comment overstated this as
    // an unconditional no-op). Post-PB-DP1, priority follows the actor (CR 117.3c) -- if
    // a RESPONDER cast a spell or activated an ability during the window between
    // resolution and this command, priority now sits with that responder, and the
    // deleted write would have yanked it to the active player instead. So the deletion
    // is a fix in that case too, exactly like recover below, just less commonly
    // triggered (echo/CU only fire on the controller's own upkeep, before most
    // responses have had a chance to happen).
    //
    // CR 117.4 is not engaged: answering an out-of-band resolution-time payment is not
    // "taking an action" between passes, so the pass set is left exactly as it is. Leaving
    // it alone is also what makes the deadline work -- a player must send `Pay*` BEFORE
    // passing, and a spurious pass-set reset would silently buy them an extra round.
    Ok(events)
}
/// Multiply a mana cost by a scalar, used for cumulative upkeep cost calculation.
///
/// **PIP-MAJOR, deliberately** (`flat_map(repeat_n)`: each distinct pip is repeated
/// `multiplier` times *before* moving to the next pip -- for `hybrid = [r1, r2]` and
/// `multiplier: 3` the result is `[r1, r1, r1, r2, r2, r2]`). Correct for cumulative
/// upkeep, which has exactly one pip source and one payer choosing once per pip
/// occurrence with no ordering contract to preserve.
///
/// **Do NOT reuse this for `rules/combat.rs`'s CR 508.1h attack-tax accumulation**
/// (`combat::add_mana_cost`, `combat::accumulate_attack_tax_total`) even though the
/// two now look similar (OOS-DP4-7). The attack tax's canonical pip order is
/// **copy-major** (`add_mana_cost`'s own doc has the full contract; a defender's
/// per-creature pips `[r1, r2]` with 3 attackers must be `[r1, r2, r1, r2, r1, r2]`,
/// not this function's `[r1, r1, r1, r2, r2, r2]`), because the Norn's Annex rulings
/// (`pb-plan-DX6.md` §1) require each COPY of a cost to be payable individually. A
/// "harmless" dedup onto this function would silently re-order the attack tax's pips
/// and therefore silently re-interpret every `hybrid_choices` vector a client had
/// already built, with no compile error -- and, absent
/// `pb_dx6_unflattened_payment_sites.rs::one_defender_two_distinct_restrictions_two_attackers_discriminates_copy_vs_pip_major`,
/// no test failure either: the batch's original order-pin test paired its only
/// two-restriction defender with a single attacker, so the two orders coincided by
/// construction there (PB-DX6 fix-cycle Finding 1). The discriminating test named
/// above -- one defender, two distinct restrictions, two attackers -- was verified by
/// revert-and-restore to redden under exactly this dedup. PB-DX6 §5.2.5.
fn multiply_mana_cost(
    cost: &crate::state::game_object::ManaCost,
    multiplier: u32,
) -> crate::state::game_object::ManaCost {
    crate::state::game_object::ManaCost {
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
/// CR 702.59a: Handle the player's recover payment choice.
///
/// If `pay` is true, deducts the recover cost from the player's mana pool and
/// moves the card from the graveyard to the player's hand (CR 702.59a: "return
/// this card from your graveyard to your hand").
///
/// If `pay` is false, moves the card from the graveyard to exile
/// (CR 702.59a: "Otherwise, exile this card.").
///
/// In both cases, the pending recover payment entry is removed.
fn handle_pay_recover(
    state: &mut GameState,
    player: PlayerId,
    recover_card: crate::state::game_object::ObjectId,
    pay: bool,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::zone::ZoneId;
    let mut events = Vec::new();
    // Find and remove the matching pending recover payment.
    let payment_pos = state
        .pending_recover_payments
        .iter()
        .position(|(p, obj, _)| *p == player && *obj == recover_card);
    let recover_cost = if let Some(pos) = payment_pos {
        let (_, _, cost) = state.pending_recover_payments.remove(pos);
        cost
    } else {
        // No pending payment for this card -- stale or invalid command.
        return Err(GameStateError::InvalidCommand(format!(
            "No pending recover payment for player {:?} card {:?}",
            player, recover_card
        )));
    };
    // Verify the card is still in a graveyard (CR 400.7).
    let card_info = state.objects.get(&recover_card).and_then(|obj| {
        if matches!(obj.zone, ZoneId::Graveyard(_)) {
            Some(obj.owner)
        } else {
            None
        }
    });
    let Some(owner) = card_info else {
        // Card left the graveyard since the trigger resolved; nothing to do.
        return Ok(events);
    };
    if pay {
        // CR 702.59a: Player pays the recover cost.
        let pool = &state
            .players
            .get(&player)
            .ok_or(GameStateError::PlayerNotFound(player))?
            .mana_pool;
        let can_afford = casting::can_pay_cost(pool, &recover_cost);
        if !can_afford {
            return Err(GameStateError::InvalidCommand(format!(
                "Player {:?} cannot afford recover cost",
                player
            )));
        }
        // Deduct the mana.
        if let Some(p) = state.expect_player_mut(player) {
            casting::pay_cost(&mut p.mana_pool, &recover_cost);
        }
        // Return card from graveyard to owner's hand (CR 702.59a).
        let (new_hand_id, _old) = state.move_object_to_zone(recover_card, ZoneId::Hand(owner))?;
        events.push(GameEvent::RecoverPaid {
            player,
            recover_card,
            new_hand_id,
        });
    } else {
        // CR 702.59a: the player declined -- exile the card from the graveyard.
        // SR-4 (engine-bug side): the card was proven to be in a graveyard above (the
        // `card_info` guard, CR 400.7), and zones are never removed, so every error
        // variant here is corrupted state, not a CR 400.7 fizzle. `expect_move_object_to_zone`
        // debug-asserts and returns None in release. Making this branch infallible is
        // load-bearing for PB-DP4's forced sweep (`force_resolve_overdue_payments`): the
        // pending entry is already removed at this point, so a `None` here (release
        // build) would silently abandon the exile -- the card stays in the graveyard
        // un-exiled with no error and no pending entry to retry it, a silent rules
        // failure, not a deadlock. (Fix cycle, E3: `force_resolve_overdue_payments`
        // swallows every handler `Err` into a `debug_assert!` and never propagates it,
        // so an `Err` here cannot reach `handle_pass_priority` / `process_command` and
        // cannot produce the deadlock the pre-fix comment described.)
        if let Some((new_exile_id, _old)) =
            state.expect_move_object_to_zone(recover_card, ZoneId::Exile)
        {
            events.push(GameEvent::RecoverDeclined {
                player,
                recover_card,
                new_exile_id,
            });
        }
    }
    // CR 704.3: Check SBAs after recover resolution.
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);
    // CR 702.59a / CR 608.2d / CR 117.3c -- PB-DP4, closes OOS-DP1-1.
    //
    // Paying or declining is a resolution-time cost choice (CR 118.12 / 608.2d), not an
    // action that grants priority, so there is no actor for CR 117.3c to hand priority to.
    // This site used to write `priority_holder = Some(active_player)` and clear
    // `players_passed` as a bodge standing in for the payment pause DP-11 said was never
    // implemented (PB-DP1 correctly left it alone; it is exactly the OOS-DP1-1 seed). The
    // pause now exists as a DEADLINE (`force_resolve_overdue_payments`), so the bodge is
    // gone. UNLIKE echo/cumulative upkeep, recover's controller can be a NON-active player
    // ("when a creature is put into YOUR graveyard" fires on any player's turn), so the old
    // write yanked priority away from whoever legitimately held it and restarted the pass
    // round -- removing it here is a FIX, not a no-op.
    //
    // CR 117.4 is not engaged: answering an out-of-band resolution-time payment is not
    // "taking an action" between passes, so the pass set is left exactly as it is. Leaving
    // it alone is also what makes the deadline work -- a player must send `Pay*` BEFORE
    // passing, and a spurious pass-set reset would silently buy them an extra round.
    Ok(events)
}
/// CR 702.30a / 702.24a / 702.59a + CR 118.12a: close out any pay-or-lose-it payment that
/// was not answered before the game left this priority round.
///
/// PB-DP4 / DP-11. Before this, the three `pending_*` vectors were inert queues: nothing in
/// `rules/priority.rs`, `handle_all_passed`, `rules/turn_structure.rs` or `rules/sba.rs`
/// consulted them, so passing priority left an echo permanent neither paid for nor
/// sacrificed, a cumulative-upkeep permanent accruing age counters forever, and a recover
/// card sitting un-exiled in its graveyard. `rules/resolution.rs`'s claim that "the game
/// pauses until a Command::PayEcho is received" described a pause that did not exist.
///
/// **Why decline and not auto-pay.** CR 118.12a: "[Do something] unless [a player does
/// something else]" means "[a player may do something else]. If [that player doesn't], [do
/// something]." Not answering is "doesn't". Auto-paying an affordable cost would spend mana
/// or life the player never elected to spend -- the DP-19 (`MayPayThenEffect`) bug class.
///
/// **Deviation from CR 608.2d, deliberate.** The CR makes this choice during the ability's
/// resolution. This engine defers it to the end of the FIRST SUBSEQUENT priority round that
/// terminates with an empty stack (fix cycle, E9: not "the following round" unconditionally --
/// see the precise boundary and its consequence below), which is the earliest boundary
/// reachable without a new `Command` (SR-8) and without a design that can hang a fuzzer, the
/// `GameDriver`, a golden script or an M11-local seat that never sends the command. The
/// permanent therefore survives, observably, until that boundary. The outcome at the
/// boundary is CR-correct. See `memory/primitives/pb-plan-DP4.md` §3 2.0 for the rejected
/// alternatives.
///
/// **The boundary can be postponed (seed OOS-DP4-12).** "The first subsequent round that
/// ends with an empty stack" is not the same as "the very next round": if any player casts a
/// spell or activates a non-mana ability before everyone passes, the stack is non-empty when
/// priority would otherwise leave the round, `handle_all_passed` takes the stack-non-empty
/// branch instead of this one, and the deadline does not fire. A player (any player, not just
/// the one who owes the payment) can therefore keep the permanent/card in its pre-consequence
/// state indefinitely, bounded only by their ability to keep putting something on the stack.
/// The eventual outcome is still CR-correct once the stack does empty; only the timing is
/// deferred further than "one extra round" might suggest.
///
/// **Ordering.** Players are visited in APNAP order (CR 101.4, `abilities::apnap_order`);
/// within a player, echo then cumulative upkeep then recover, each in insertion order
/// (which is the order the triggers resolved). Deterministic, as SR-9b requires.
///
/// **Termination.** Every call drains every entry. A new entry needs a new trigger to
/// resolve, which needs a permanent to leave the battlefield or a creature to reach a
/// graveyard, so the extra-round chain is bounded by the object count.
///
/// SR-4 classification: every `Err` from the three handlers is an **engine bug**, not an LKI
/// fizzle -- the entry was read out of the vector one statement earlier, and each handler
/// removes it before any fallible step, so a failure cannot loop and cannot be a legal
/// CR 400.7 fizzle (the handlers already return `Ok(vec![])` for that case). Mechanism is a
/// `debug_assert!`, mirroring `state::diagnostics`' `expect_*` family.
///
/// Do not name `KeywordAbility::Echo` / `::CumulativeUpkeep` / `::Recover` here (or in
/// `crates/simulator/src/legal_actions.rs`): `crates/engine/tests/core/keyword_registry.rs`
/// scans `crates/simulator/src` too, and neither variant declares this file (or the
/// simulator) as a site (§4.5 of the plan). Read the payment kind and cost from the pending
/// vectors, which already carry everything needed.
fn force_resolve_overdue_payments(state: &mut GameState) -> Vec<GameEvent> {
    let mut events = Vec::new();
    if state.pending_echo_payments.is_empty()
        && state.pending_cumulative_upkeep_payments.is_empty()
        && state.pending_recover_payments.is_empty()
    {
        return events;
    }
    for owing in abilities::apnap_order(state) {
        // Snapshot before mutating: each handler removes its own entry from the vector.
        let echoes: Vec<crate::state::game_object::ObjectId> = state
            .pending_echo_payments
            .iter()
            .filter(|(p, _, _)| *p == owing)
            .map(|(_, obj, _)| *obj)
            .collect();
        for permanent in echoes {
            match handle_pay_echo(state, owing, permanent, false) {
                Ok(evs) => events.extend(evs),
                Err(e) => debug_assert!(
                    false,
                    "engine invariant: forced echo decline for {owing:?}/{permanent:?} failed \
                     ({e}); the entry was read from pending_echo_payments one statement earlier"
                ),
            }
        }
        let cumulative_upkeeps: Vec<crate::state::game_object::ObjectId> = state
            .pending_cumulative_upkeep_payments
            .iter()
            .filter(|(p, _, _)| *p == owing)
            .map(|(_, obj, _)| *obj)
            .collect();
        for permanent in cumulative_upkeeps {
            match handle_pay_cumulative_upkeep(state, owing, permanent, false) {
                Ok(evs) => events.extend(evs),
                Err(e) => debug_assert!(
                    false,
                    "engine invariant: forced cumulative upkeep decline for \
                     {owing:?}/{permanent:?} failed ({e}); the entry was read from \
                     pending_cumulative_upkeep_payments one statement earlier"
                ),
            }
        }
        let recovers: Vec<crate::state::game_object::ObjectId> = state
            .pending_recover_payments
            .iter()
            .filter(|(p, _, _)| *p == owing)
            .map(|(_, obj, _)| *obj)
            .collect();
        for recover_card in recovers {
            match handle_pay_recover(state, owing, recover_card, false) {
                Ok(evs) => events.extend(evs),
                Err(e) => debug_assert!(
                    false,
                    "engine invariant: forced recover decline for {owing:?}/{recover_card:?} \
                     failed ({e}); the entry was read from pending_recover_payments one \
                     statement earlier"
                ),
            }
        }
    }
    events
}
/// CR 701.27a: Transform a double-faced permanent to its other face.
///
/// No new object is created (CR 712.18). Counters, damage, attachments, and
/// continuous effects all persist through transformation.
///
/// This is the `Command::Transform` path: it validates the permanent exists,
/// is on the battlefield, is controlled by `player`, and is not daybound/
/// nightbound (those reject with `Err` — they can only transform via their
/// own keyword enforcement system) — then delegates the actual flip to
/// `transform_permanent_in_place`, which is shared with the
/// `Effect::TransformSelf` executor path (PB-EF5).
fn handle_transform(
    state: &mut GameState,
    player: PlayerId,
    permanent: crate::state::game_object::ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::zone::ZoneId;
    let mut events = Vec::new();
    // Validate permanent exists and is on the battlefield.
    let obj = state
        .objects
        .get(&permanent)
        .ok_or(GameStateError::ObjectNotFound(permanent))?;
    if obj.zone != ZoneId::Battlefield {
        return Err(GameStateError::InvalidCommand(
            "transform target must be on the battlefield".into(),
        ));
    }
    if obj.controller != player {
        return Err(GameStateError::InvalidCommand(
            "can only transform permanents you control".into(),
        ));
    }
    // CR 702.145b/e: Permanents with daybound/nightbound can only transform via their
    // keyword enforcement system. Direct transform commands are rejected.
    let has_daybound = obj
        .characteristics
        .keywords
        .contains(&crate::state::types::KeywordAbility::Daybound);
    let has_nightbound = obj
        .characteristics
        .keywords
        .contains(&crate::state::types::KeywordAbility::Nightbound);
    if has_daybound || has_nightbound {
        return Err(GameStateError::InvalidCommand(
            "permanents with daybound/nightbound can only transform via their keyword ability"
                .into(),
        ));
    }
    events.extend(transform_permanent_in_place(state, permanent)?);
    Ok(events)
}

/// CR 701.27a-g / 712.18: flip a DFC permanent to its other face in place.
/// No new object (CR 712.18). Counters/damage/Auras persist. Runs the CR 704.3
/// SBA check. Returns `PermanentTransformed` (+ SBA) events, or an empty vec if
/// nothing happens (non-DFC 701.27c, instant/sorcery back 701.27d, meld-pair
/// 712.4c, daybound/nightbound 702.145, or the object no longer exists — CR
/// 400.7, e.g. a source that has since left the battlefield). Does NOT
/// validate zone/controller (caller's job) and does NOT run the CR 701.27f
/// once-per-instruction guard (caller's job — see the `Effect::TransformSelf`
/// executor in `effects/mod.rs`).
pub(crate) fn transform_permanent_in_place(
    state: &mut GameState,
    permanent: crate::state::game_object::ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();
    // CR 400.7: a `None` here means the object has since left its zone (become a
    // new object) -- a rules-correct no-op, not an engine bug.
    let obj = match state.fizzle_object(permanent) {
        Some(obj) => obj,
        None => return Ok(events),
    };
    // CR 702.145b/e: Permanents with daybound/nightbound can only transform via
    // their own keyword enforcement system -- a card-invoked TransformSelf is a
    // silent no-op here (the Command path rejects with Err before ever reaching
    // this helper, so Command::Transform behavior is unaffected).
    let has_daybound = obj
        .characteristics
        .keywords
        .contains(&crate::state::types::KeywordAbility::Daybound);
    let has_nightbound = obj
        .characteristics
        .keywords
        .contains(&crate::state::types::KeywordAbility::Nightbound);
    if has_daybound || has_nightbound {
        return Ok(events);
    }
    // Capture the fields we need before any further `state.card_registry` reads
    // (which don't conflict, but keeping this as a single snapshot avoids a
    // second bare `.objects.get` for the CR 701.27d back-face check below).
    let card_id = obj.card_id.clone();
    let is_transformed = obj.is_transformed;
    // CR 712.4c: Meld cards cannot be transformed or converted.
    if let Some(ref cid) = card_id {
        if let Some(def) = state.card_registry.get(cid.clone()) {
            if def.meld_pair.is_some() {
                return Ok(events); // Silently ignore transform instruction
            }
        }
    }
    // CR 701.27c: Only DFCs can transform.
    let is_dfc = if let Some(ref cid) = card_id {
        state
            .card_registry
            .get(cid.clone())
            .map(|def| def.back_face.is_some())
            .unwrap_or(false)
    } else {
        false
    };
    if !is_dfc {
        // CR 701.27c: Nothing happens when trying to transform a non-DFC.
        return Ok(events);
    }
    // CR 701.27d: Back face can't be an instant or sorcery.
    let would_transform_to_back = !is_transformed;
    if would_transform_to_back {
        if let Some(ref cid) = card_id {
            if let Some(def) = state.card_registry.get(cid.clone()) {
                if let Some(ref back) = def.back_face {
                    if back
                        .types
                        .card_types
                        .contains(&crate::state::types::CardType::Instant)
                        || back
                            .types
                            .card_types
                            .contains(&crate::state::types::CardType::Sorcery)
                    {
                        // CR 701.27d / CR 712.10: Nothing happens.
                        return Ok(events);
                    }
                }
            }
        }
    }
    // CR 712.18: Transform flips the face. No new object — same ObjectId.
    // `None` here would mean the object left between the fizzle_object read
    // above and here (no intervening mutation exists in this function, so this
    // cannot actually happen) — treated as a fizzle for symmetry with the read.
    let Some(obj) = state.fizzle_object(permanent) else {
        return Ok(events);
    };
    let to_back_face = !obj.is_transformed;
    // PB-OS4b (CR 712.8d/e, 712.18): route the flip through `apply_face_change` so
    // it deregisters the front face's static continuous effects, rebuilds the
    // Channel-A ability vectors (mana/activated/triggered) from the back face, and
    // registers the back face's static continuous effects — replacing the previous
    // raw `is_transformed` flip, which left all three of those stale.
    crate::rules::face::apply_face_change(state, permanent, to_back_face);
    events.push(GameEvent::PermanentTransformed {
        object_id: permanent,
        to_back_face,
    });
    // CR 704.3: Check SBAs after transformation (e.g., Aura's enchanted object changed type).
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);
    Ok(events)
}
/// CR 702.167a: Activate a permanent's craft ability.
///
/// Cost: pay mana + exile source + exile materials.
/// When the ability resolves: the exiled source returns to the battlefield
/// transformed (back face up) under its owner's control.
fn handle_activate_craft(
    state: &mut GameState,
    player: PlayerId,
    source: crate::state::game_object::ObjectId,
    material_ids: Vec<crate::state::game_object::ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::cards::card_definition::AbilityDefinition;
    use crate::rules::events::GameEvent;
    use crate::state::zone::ZoneId;
    let mut events = Vec::new();
    // Validate source is on battlefield and controlled by player.
    {
        let obj = state
            .objects
            .get(&source)
            .ok_or(GameStateError::ObjectNotFound(source))?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::InvalidCommand(
                "craft source must be on the battlefield".into(),
            ));
        }
        if obj.controller != player {
            return Err(GameStateError::InvalidCommand(
                "can only craft with permanents you control".into(),
            ));
        }
        // CR 702.167a: "Activate only as a sorcery."
        let is_main_phase = matches!(
            state.turn.phase,
            crate::state::turn::Phase::PreCombatMain | crate::state::turn::Phase::PostCombatMain
        );
        let stack_empty = state.stack_objects.is_empty();
        let is_active = state.turn.active_player == player;
        if !is_main_phase || !stack_empty || !is_active {
            return Err(GameStateError::InvalidCommand(
                "craft can only be activated as a sorcery (main phase, empty stack, active player)"
                    .into(),
            ));
        }
        // Verify the source has a Craft ability definition and extract cost + materials.
        let craft_def = if let Some(ref cid) = obj.card_id {
            state.card_registry.get(cid.clone()).and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Craft { cost, materials } = a {
                        Some((cost.clone(), materials.clone()))
                    } else {
                        None
                    }
                })
            })
        } else {
            None
        };
        if craft_def.is_none() {
            return Err(GameStateError::InvalidCommand(
                "permanent does not have a craft ability".into(),
            ));
        }
    }
    // Extract craft cost and material requirements (re-borrow from registry after block ends).
    use crate::cards::card_definition::CraftMaterials;
    use crate::state::types::CardType;
    let (craft_cost, craft_materials) = {
        let cid = state
            .objects
            .get(&source)
            .and_then(|o| o.card_id.clone())
            .ok_or_else(|| GameStateError::InvalidCommand("craft source has no card_id".into()))?;
        state
            .card_registry
            .get(cid)
            .and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Craft { cost, materials } = a {
                        Some((cost.clone(), materials.clone()))
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                GameStateError::InvalidCommand("permanent does not have a craft ability".into())
            })?
    };
    // CR 702.167a: Validate and pay the mana cost before exiling.
    {
        let pool = &state
            .players
            .get(&player)
            .ok_or(GameStateError::PlayerNotFound(player))?
            .mana_pool;
        if !casting::can_pay_cost(pool, &craft_cost) {
            return Err(GameStateError::InsufficientMana);
        }
    }
    // CR 702.167b: Validate material count and types before exiling.
    {
        let required_count = match craft_materials {
            CraftMaterials::Artifacts(n)
            | CraftMaterials::Creatures(n)
            | CraftMaterials::Lands(n)
            | CraftMaterials::AnyCards(n) => n as usize,
        };
        if material_ids.len() != required_count {
            return Err(GameStateError::InvalidCommand(format!(
                "craft requires exactly {} material(s), got {}",
                required_count,
                material_ids.len()
            )));
        }
        for mat_id in &material_ids {
            let mat_obj = state.objects.get(mat_id).ok_or_else(|| {
                GameStateError::InvalidCommand(format!(
                    "craft material {:?} does not exist",
                    mat_id
                ))
            })?;
            let mat_zone = mat_obj.zone;
            match mat_zone {
                ZoneId::Battlefield | ZoneId::Graveyard(_) => {}
                _ => {
                    return Err(GameStateError::InvalidCommand(
                        "craft materials must be permanents on battlefield or cards in graveyard"
                            .into(),
                    ));
                }
            }
            // Check the material is the required card type (CR 702.167b).
            let required_type = match craft_materials {
                CraftMaterials::Artifacts(_) => Some(CardType::Artifact),
                CraftMaterials::Creatures(_) => Some(CardType::Creature),
                CraftMaterials::Lands(_) => Some(CardType::Land),
                CraftMaterials::AnyCards(_) => None,
            };
            if let Some(req_type) = required_type {
                // For battlefield permanents, use layer-resolved characteristics.
                // For graveyard cards, use base characteristics (CR 702.167b).
                let has_type = if mat_zone == ZoneId::Battlefield {
                    // `mat_id` was just proven live at the top of this loop iteration;
                    // calculate_characteristics is total for a live id (CR 613.1d).
                    crate::rules::layers::expect_characteristics(state, *mat_id)
                        .card_types
                        .contains(&req_type)
                } else {
                    mat_obj.characteristics.card_types.contains(&req_type)
                };
                if !has_type {
                    return Err(GameStateError::InvalidCommand(format!(
                        "craft material {:?} is not of required type {:?} (CR 702.167b)",
                        mat_id, req_type
                    )));
                }
            }
        }
    }
    // Pay the mana cost (CR 702.167a).
    if let Some(p) = state.expect_player_mut(player) {
        casting::pay_cost(&mut p.mana_pool, &craft_cost);
    }
    events.push(GameEvent::ManaCostPaid {
        player,
        cost: craft_cost,
    });
    // CR 702.167a cost: Exile the source permanent.
    let (exiled_source_id, _) = state.move_object_to_zone(source, ZoneId::Exile)?;
    // CR 702.167a cost: Exile each material.
    let mut exiled_material_ids = Vec::new();
    for mat_id in material_ids {
        let (new_id, _) = state.move_object_to_zone(mat_id, ZoneId::Exile)?;
        exiled_material_ids.push(new_id);
    }
    events.push(GameEvent::CraftActivated {
        player,
        exiled_source: exiled_source_id,
        exiled_materials: exiled_material_ids.clone(),
    });
    // CR 702.167a: Return the exiled card to the battlefield transformed.
    // The card that was exiled as cost (exiled_source_id) now enters transformed.
    // CR 702.167a: "If the card isn't a DFC, it stays in exile."
    // `exiled_source_id` was just returned by the move above; card_id None is a legit tokenless read.
    let source_card_id = state
        .expect_object(exiled_source_id)
        .and_then(|o| o.card_id.clone());
    let is_dfc = source_card_id
        .as_ref()
        .and_then(|cid| {
            state
                .card_registry
                .get(cid.clone())
                .map(|def| def.back_face.is_some())
        })
        .unwrap_or(false);
    if is_dfc {
        // Move the exiled source card to the battlefield.
        let (battlefield_id, _) =
            state.move_object_to_zone(exiled_source_id, ZoneId::Battlefield)?;
        // Track the exiled materials for CR 702.167c abilities.
        // `battlefield_id` was just returned by the move above; it is live here.
        debug_assert_object_live!(state, battlefield_id);
        if let Some(obj) = state.objects.get_mut(&battlefield_id) {
            obj.craft_exiled_cards = exiled_material_ids.into_iter().collect();
        }
        // PB-OS4b (CR 702.167a, 712.8d/e): the returned card enters transformed
        // (back face up). Route through `apply_face_change` so it rebuilds the
        // Channel-A ability vectors from the back face and registers the back
        // face's static continuous effects — this path previously never
        // registered ANY statics for a crafted-in permanent; apply_face_change
        // closes that pre-existing gap as a side effect of the flip.
        crate::rules::face::apply_face_change(state, battlefield_id, true);
        events.push(GameEvent::PermanentEnteredBattlefield {
            player,
            object_id: battlefield_id,
        });
    }
    // If not a DFC, the card stays in exile (no PermanentEnteredBattlefield emitted).
    // CR 704.3: Check SBAs after craft resolution.
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);
    // CR 702.167a: craft is an activated ability (CR 602.2b -> 601.2i), so the activating
    // player receives priority (CR 117.3c). Identity write today: :1272 already requires
    // player == active_player ("activate only as a sorcery"). CR 117.4: reset the
    // pass-round.
    state.turn.players_passed = imbl::OrdSet::new();
    state.turn.priority_holder = Some(player);
    Ok(events)
}
/// CR 702.37e / 702.168d / 701.40b / 701.58b: Turn a face-down permanent face up.
///
/// This is a special action (CR 116.2b) — does NOT use the stack. The cost is paid,
/// the permanent turns face up, ETB abilities do NOT fire (CR 708.8), and "when turned
/// face up" triggers are queued. For Megamorph + MorphCost, a +1/+1 counter is added.
fn handle_turn_face_up(
    state: &mut GameState,
    player: PlayerId,
    permanent: crate::state::game_object::ObjectId,
    method: crate::state::types::TurnFaceUpMethod,
    hybrid_choices: Vec<crate::state::game_object::HybridManaPayment>,
    phyrexian_life_payments: Vec<bool>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::cards::card_definition::AbilityDefinition;
    use crate::state::types::{FaceDownKind, TurnFaceUpMethod};
    use crate::state::zone::ZoneId;
    let mut events = Vec::new();
    // CR 116.2b: turning a face-down permanent face up is a special action; the player
    // must have priority to take it.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // Validate: permanent exists, on battlefield, face-down, controlled by player.
    let obj = state
        .objects
        .get(&permanent)
        .ok_or(GameStateError::ObjectNotFound(permanent))?;
    if obj.zone != ZoneId::Battlefield {
        return Err(GameStateError::InvalidCommand(
            "TurnFaceUp: permanent not on battlefield".into(),
        ));
    }
    if !obj.status.face_down {
        return Err(GameStateError::InvalidCommand(
            "TurnFaceUp: permanent is not face-down".into(),
        ));
    }
    if obj.face_down_as.is_none() {
        return Err(GameStateError::InvalidCommand(
            "TurnFaceUp: permanent has no face_down_as (not a morph/manifest/cloak)".into(),
        ));
    }
    if obj.controller != player {
        return Err(GameStateError::InvalidCommand(
            "TurnFaceUp: permanent not controlled by player".into(),
        ));
    }
    let face_down_as = obj.face_down_as.clone().unwrap();
    let card_id = obj.card_id.clone();
    // Determine turn-face-up cost and validate legality.
    let mana_cost: crate::state::ManaCost = {
        let registry = state.card_registry.clone();
        let def = card_id
            .as_ref()
            .and_then(|cid| registry.get(cid.clone()))
            .ok_or_else(|| {
                GameStateError::InvalidCommand("TurnFaceUp: no card definition found".into())
            })?;
        match method {
            TurnFaceUpMethod::MorphCost => {
                // Look for Morph or Megamorph AbilityDefinition
                let morph_ability = def.abilities.iter().find_map(|a| match a {
                    AbilityDefinition::Morph { cost } => Some(cost.clone()),
                    AbilityDefinition::Megamorph { cost } => Some(cost.clone()),
                    _ => None,
                });
                morph_ability.ok_or_else(|| {
                    GameStateError::InvalidCommand(
                        "TurnFaceUp: card has no Morph or Megamorph cost".into(),
                    )
                })?
            }
            TurnFaceUpMethod::DisguiseCost => {
                let disguise_ability = def.abilities.iter().find_map(|a| match a {
                    AbilityDefinition::Disguise { cost } => Some(cost.clone()),
                    _ => None,
                });
                disguise_ability.ok_or_else(|| {
                    GameStateError::InvalidCommand("TurnFaceUp: card has no Disguise cost".into())
                })?
            }
            TurnFaceUpMethod::ManaCost => {
                // CR 701.40b: Only creature cards with a mana cost can be turned face up this way.
                // CR 701.40g: Instants and sorceries manifested stay face down.
                let is_creature = def
                    .types
                    .card_types
                    .contains(&crate::state::CardType::Creature);
                let is_instant_sorcery = def
                    .types
                    .card_types
                    .contains(&crate::state::CardType::Instant)
                    || def
                        .types
                        .card_types
                        .contains(&crate::state::CardType::Sorcery);
                if !is_creature || is_instant_sorcery {
                    return Err(GameStateError::InvalidCommand(
                        "TurnFaceUp: manifested card is not a creature (cannot turn face up)"
                            .into(),
                    ));
                }
                if face_down_as != FaceDownKind::Manifest && face_down_as != FaceDownKind::Cloak {
                    // Also allow ManaCost for morph/disguise cards that are manifested/cloaked.
                    // But if the card has no morph/disguise AND was cast as Morph/Megamorph/Disguise,
                    // ManaCost is not valid. Only Manifest/Cloak allow paying the mana cost.
                    return Err(GameStateError::InvalidCommand(
                        "TurnFaceUp: ManaCost method only valid for manifested/cloaked permanents"
                            .into(),
                    ));
                }
                def.mana_cost.clone().ok_or_else(|| {
                    GameStateError::InvalidCommand(
                        "TurnFaceUp: manifested card has no mana cost".into(),
                    )
                })?
            }
        }
    };
    // Validate and pay the cost from the player's mana pool.
    //
    // PB-DX6 (OOS-RS2-1; CR 701.40b via CR 107.4e/107.4f): `mana_cost` above is the
    // RAW, unflattened cost of whichever `TurnFaceUpMethod` branch produced it (the
    // printed mana cost for Manifest/Cloak, or the Morph/Megamorph/Disguise ability's
    // own cost) -- all three arms feed this single payment block, so fixing it once
    // fixes all three (CR 701.40c/701.40d). This mirrors
    // `rules/abilities.rs::handle_activate_ability`'s activation-cost payment block,
    // the canonical, reviewed form of this fix -- adapted here rather than reinvented,
    // per plan §5.1.
    //
    // CR 107.4e/107.4f: flatten hybrid/Phyrexian pips FIRST, unconditionally when
    // either choice vector is non-empty. Calls the INHERENT `ManaCost::
    // flatten_hybrid_phyrexian` directly (not `casting::flatten_hybrid_phyrexian`) --
    // reaching into `casting` from this non-cast payment path is the layering smell
    // plan §4 flags; `abilities.rs` and `legal_actions.rs` already call the inherent
    // method the same way.
    let (flat_cost, phyrexian_life) =
        if !mana_cost.hybrid.is_empty() || !mana_cost.phyrexian.is_empty() {
            mana_cost
                .flatten_hybrid_phyrexian(&hybrid_choices, &phyrexian_life_payments)
                .map_err(GameStateError::InvalidCommand)?
        } else {
            (mana_cost.clone(), 0)
        };
    // CR 119.4 (via CR 107.4f): unlike `abilities.rs`'s ability-cost block, this site
    // has no OTHER life component to combine with the Phyrexian pip -- there is no
    // `life_cost` field on a turn-face-up cost -- but the check is still written
    // through a `combined_life_cost` local (rather than checking `phyrexian_life`
    // directly) so this site reads identically to `abilities.rs`'s, and so a future
    // life component on this cost cannot slip past a hard-coded single-addend check.
    // CR 119.4b: 0 life is always payable, so the guard short-circuits on > 0.
    let combined_life_cost = phyrexian_life; // structurally zero other addend at this site
    if combined_life_cost > 0 {
        let player_state = state.player(player)?;
        if player_state.life_total < combined_life_cost as i32 {
            return Err(GameStateError::InsufficientLife {
                player,
                required: combined_life_cost,
                actual: player_state.life_total,
            });
        }
    }
    // The gate is on the FLATTENED cost, and the flatten runs above it: a pure
    // `{G/P}` paid entirely with life flattens to `{0}` (mana_value() == 0), so this
    // correctly skips the mana check while the (sibling, not nested) life deduction
    // below still fires. Error variant deliberately kept as the existing
    // `InvalidCommand` string (existing tests assert it) rather than harmonised to
    // `InsufficientMana` -- plan §5.1 explicit deviation from `abilities.rs`.
    if flat_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !player_state.mana_pool.can_spend(&flat_cost, None) {
            return Err(GameStateError::InvalidCommand(
                "TurnFaceUp: player cannot pay the turn-face-up cost".into(),
            ));
        }
        player_state.mana_pool.spend(&flat_cost, None);
    }
    // CR 107.4f: pay life for a Phyrexian pip paid with life. A SIBLING of the mana
    // gate above, not nested inside it -- see the pure-Phyrexian-paid-with-life case
    // in the comment above. Legality was already validated above, before any
    // mutation. No rollback is needed or invented here: `process_command` takes
    // `GameState` by value and returns it only on `Ok`, so an `Err` anywhere above
    // discards the whole state (Architecture Invariants 2/3) -- same as
    // `abilities.rs` documents at its own identical site.
    if phyrexian_life > 0 {
        let player_state = state.player_mut(player)?;
        player_state.life_total -= phyrexian_life as i32;
        events.push(GameEvent::LifeLost {
            player,
            amount: phyrexian_life,
        });
    }
    if flat_cost.mana_value() > 0 || phyrexian_life > 0 {
        // Architecture Invariant 4 repair: `handle_turn_face_up` emitted NO
        // `ManaCostPaid` before this batch, even though a mana-pool debit is a
        // state change and must be evented. This is a new event on an EXISTING
        // `GameEvent` variant, so it is not a wire change -- but it does change the
        // event stream of any test/golden script that flips a face-down permanent
        // for a non-zero cost (plan §5.1: expect and repair that, do not suppress
        // the event to keep a test green). Emit the ORIGINAL (unflattened)
        // `mana_cost`, mirroring `casting.rs`/`abilities.rs`, which carry the
        // pipped shape so event consumers see what was printed.
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: mana_cost.clone(),
        });
    }
    // Carry-forward limitation (plan §4, unchanged from PB-RS2): a hybrid-Phyrexian
    // pip (`PhyrexianMana::Hybrid(a, b)`) paid with MANA always resolves to `a` --
    // `hybrid_choices` only reaches plain hybrid pips, not the color half of a
    // hybrid-Phyrexian one. No card on either PB-DX6 roster carries a
    // hybrid-Phyrexian pip in a turn-face-up cost, so this is safe today; a future
    // one would need a third choice field (documented, not silently widened).
    // Check if this is a Megamorph turned face up via MorphCost (gets +1/+1 counter).
    let is_megamorph_flip =
        face_down_as == FaceDownKind::Megamorph && method == TurnFaceUpMethod::MorphCost;
    // Turn the permanent face up: clear face_down and face_down_as.
    if let Some(obj) = state.expect_object_mut(permanent) {
        obj.status.face_down = false;
        obj.face_down_as = None;
    }
    // CR 702.37b: Megamorph gets +1/+1 counter when turned face up via megamorph cost.
    if is_megamorph_flip {
        if let Some(obj) = state.expect_object_mut(permanent) {
            let current = obj
                .counters
                .get(&crate::state::types::CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0);
            obj.counters = obj.counters.update(
                crate::state::types::CounterType::PlusOnePlusOne,
                current + 1,
            );
        }
        events.push(GameEvent::CounterAdded {
            object_id: permanent,
            counter: crate::state::types::CounterType::PlusOnePlusOne,
            count: 1,
        });
    }
    // Emit PermanentTurnedFaceUp event.
    events.push(GameEvent::PermanentTurnedFaceUp { player, permanent });
    // Queue "when turned face up" triggered abilities as TurnFaceUpTrigger stack objects.
    // (The actual dispatch happens in abilities::check_triggers when it sees PermanentTurnedFaceUp.)
    // CR 116.2b / CR 116.3: turning a face-down permanent face up is a special action;
    // the player who took it receives priority afterward. The entry guard above proves
    // `priority_holder == Some(player)` already, so this write is a true identity write
    // (same shape as the Group-A AP-gated sites) -- it is kept explicit so the site
    // stays correct if the guard is ever loosened.
    // CR 117.4: an action was taken between passes, so the pass-round restarts.
    state.turn.players_passed = imbl::OrdSet::new();
    // CR 704.3: Check SBAs after the special action, before the priority grant --
    // matches craft's ordering (engine.rs handle_activate_craft) and closes review LOW 6.
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);
    state.turn.priority_holder = Some(player);
    Ok(events)
}
/// Handle a PassPriority command.
fn handle_pass_priority(
    state: &mut GameState,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let (result, mut events) = priority::pass_priority(state, player)?;
    match result {
        PriorityResult::PlayerHasPriority { player: next } => {
            state.turn.players_passed.insert(player);
            state.turn.priority_holder = Some(next);
        }
        PriorityResult::AllPassed => {
            // All players passed with empty stack — advance the game
            state.turn.players_passed.insert(player);
            state.turn.priority_holder = None;
            let advance_events = handle_all_passed(state)?;
            events.extend(advance_events);
        }
    }
    Ok(events)
}
/// CR 608.1 / CR 104.1 (PB-DP9): the two statements every completed stack
/// resolution owes, factored so the two call sites cannot drift.
///
/// Returns `true` if the game ended (the caller must stop).
///
/// The sites are `handle_all_passed`'s stack-non-empty branch and
/// `handle_answer_effect_choice`'s tail, which re-drives a resolution the
/// engine had rolled back. Before PB-DP9 these two lines lived only in the
/// former; §1.5's exit 1 is the whole of the debt this batch's suspension
/// creates, and this is where it is discharged.
fn finish_stack_resolution(state: &mut GameState, events: &mut Vec<GameEvent>) -> bool {
    // SR-13: once the stack and pending-trigger queue are both empty, no ability
    // can still reference a departed damage source, so drop the LKI snapshots
    // captured in `move_object_to_zone` (a no-op unless both are drained).
    state.maybe_clear_lki_objects();
    // CR 104.1 / PB-AC8: a resolving effect (e.g. Effect::WinGame) may end the
    // game immediately, independent of any SBA (CR 704.5: winning-by-effect is
    // NOT a state-based action -- this is not an SBA check, it's the same
    // finalize-if-decided poll used elsewhere in this file after SBAs run).
    if is_game_over(state) {
        events.extend(check_game_over(state));
        return true;
    }
    false
}
/// Handle when all players have passed priority in succession.
///
/// CR 608.1: If the stack is non-empty, resolve the top of the stack.
/// CR 500.4: If the stack is empty, empty mana pools and advance step or turn.
fn handle_all_passed(state: &mut GameState) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();
    if !state.stack_objects.is_empty() {
        // CR 608.1: Stack is non-empty — resolve the top object.
        let resolve_events = resolution::resolve_top_of_stack(state)?;
        events.extend(resolve_events);
        // CR 608.2d (PB-DP9): NO GUARD is owed here, and the argument is a
        // reachability claim, not a convenience. If `resolve_top_of_stack` rolled
        // the resolution back on an unanswered CR 608.2d choice, both statements
        // inside `finish_stack_resolution` are provable no-ops on the restored
        // state: the stack is non-empty (the object was put back), so
        // `maybe_clear_lki_objects` short-circuits; and `is_game_over` was false
        // at `process_command`'s entry over a state this one is byte-identical to
        // apart from the three PB-DP9 bookkeeping fields, which it does not read.
        if finish_stack_resolution(state, &mut events) {
            return Ok(events);
        }
    } else {
        // Stack is empty — advance step or turn.
        // SR-13: with the stack empty at a priority boundary (and no pending trigger),
        // no departed source's LKI can be needed; drop any lingering snapshots.
        state.maybe_clear_lki_objects();
        // PB-DP4 / DP-11 (CR 702.30a / 702.24a / 702.59a, CR 118.12a): a pay-or-lose-it
        // payment must not survive this boundary -- the first subsequent priority round
        // that terminates with an EMPTY stack (fix cycle, E9: not "the round in which it
        // resolved" -- a player can postpone the boundary by keeping the stack non-empty,
        // seed OOS-DP4-12). Every player has had priority since the entry was created --
        // `resolve_top_of_stack` re-grants it with an empty pass set -- so an unanswered
        // payment is a decline.
        //
        // This runs only in the stack-EMPTY branch, so it can never fire in the same
        // `handle_all_passed` call that created the entry (that call takes the stack-non-
        // empty branch above and returns). That disjointness is what keeps
        // `mechanics_e_l/echo.rs`, `mechanics_a_d/cumulative_upkeep.rs`,
        // `mechanics_m_z/recover.rs` and golden script stack/153 green: all of them send
        // `Pay*` immediately after the resolving pass round.
        let mut payment_events = force_resolve_overdue_payments(state);
        if !payment_events.is_empty() {
            // The sacrifice/exile can produce dies-triggers. They belong on the stack in
            // THIS step, so re-grant priority here instead of advancing -- the same "run
            // another round, don't advance" shape `enter_step` uses for CR 514.3a.
            check_and_flush_triggers(state, &mut payment_events);
            // CR 603.3 / CR 603.3d (PB-DP8 fix cycle, Finding 3): the 31st
            // `check_and_flush_triggers` call site, and the only one outside
            // `process_command`'s `match`. A PB-DP4 forced echo / cumulative-upkeep /
            // recover sacrifice can produce a targeted dies-trigger, so this flush can
            // suspend -- and the priority grant below is unconditional. CR 603.3b gives
            // priority only AFTER every triggered ability of the batch is on the stack,
            // so stop here and record that this site owes the grant.
            if state.pending_trigger_targets.is_some() {
                abilities::mark_flush_resume_site(state, FlushResumeSite::GrantPriority);
                events.extend(payment_events);
                return Ok(events);
            }
            events.extend(payment_events);
            if is_game_over(state) {
                events.extend(check_game_over(state));
                return Ok(events);
            }
            // CR 117.3b: grant priority to the active player (if still alive) for the new
            // round. Same idiom as `enter_step` (above, the has_priority() branch).
            //
            // Fix cycle (E2): `expect_player` (a NONSWALLOW predicate read, per
            // `state::diagnostics`) replaces the bare `.players.get(` lookup -- a
            // departed player legitimately answers `is_alive == false` here, so this is
            // exactly the vocabulary the SR-25 ratchet steers new code toward.
            let active = state.turn.active_player;
            let is_alive = state
                .expect_player(active)
                .map(|p| !p.has_lost && !p.has_conceded)
                .unwrap_or(false);
            if is_alive {
                let (passed, priority_events) = priority::grant_initial_priority(state);
                state.turn.players_passed = passed;
                state.turn.priority_holder = Some(active);
                events.extend(priority_events);
            } else if let Some(next) = priority::next_priority_player(state, active) {
                state.turn.players_passed = imbl::OrdSet::new();
                state.turn.priority_holder = Some(next);
                events.push(GameEvent::PriorityGiven { player: next });
            } else {
                state.turn.priority_holder = None;
            }
            return Ok(events);
        }
        // Empty mana pools at step transition (CR 500.4)
        let mana_events = turn_actions::empty_all_mana_pools(state);
        events.extend(mana_events);
        // CR 514.3a: When all pass with empty stack in Cleanup, do NOT advance
        // to the next step — run another cleanup round instead.  `enter_step`
        // will execute cleanup actions, check SBAs, and either grant priority
        // again (if SBAs fired) or auto-advance to the next turn (if none).
        if state.turn.step != crate::state::turn::Step::Cleanup {
            // Advance to next step or next turn
            if let Some((new_turn, step_events)) = turn_structure::advance_step(state) {
                state.turn = new_turn;
                events.extend(step_events);
            } else {
                // Past cleanup — advance to next turn
                let (new_turn, turn_events) = turn_structure::advance_turn(state)?;
                state.turn = new_turn;
                events.extend(turn_events);
                // Reset per-turn state for new active player
                turn_actions::reset_turn_state(state, state.turn.active_player);
            }
        }
        // Enter the new step (execute turn-based actions, grant priority or auto-advance)
        let enter_events = enter_step(state)?;
        events.extend(enter_events);
    }
    Ok(events)
}
/// Enter a step: execute turn-based actions, then either grant priority or
/// auto-advance if the step has no priority (Untap, Cleanup).
///
/// Uses a loop (not recursion) to handle steps that auto-advance.
fn enter_step(state: &mut GameState) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();
    loop {
        // Execute turn-based actions for this step
        let action_events = turn_actions::execute_turn_based_actions(state)?;
        // CR 510.3a: Check triggers from turn-based actions (e.g., CombatDamageDealt)
        // BEFORE extending events (so the reference is still valid) and BEFORE SBA
        // checking. This ensures "whenever ~ deals combat damage to a player" triggers
        // are queued alongside SBA-generated triggers.
        let tba_triggers = abilities::check_triggers(state, &action_events);
        for t in tba_triggers {
            state.pending_triggers.push_back(t);
        }
        events.extend(action_events);
        // Check if game ended due to turn-based actions (e.g., draw from empty library)
        if is_game_over(state) {
            let game_over_events = check_game_over(state);
            events.extend(game_over_events);
            return Ok(events);
        }
        // PB-DP7 / DP-3 progress gate: if a BlockingDecision is now outstanding
        // (today, only CR 514.1's cleanup discard), STOP here. Do not run an
        // SBA/trigger round against a half-performed turn-based action, and do
        // not auto-advance past the unanswered question. `process_command`'s
        // admission gate keeps every other command out in the meantime; the
        // `Command::DiscardToHandSize` handler re-enters this function on the
        // resume pass, at which point this check is false and the loop
        // continues normally.
        if blocking_decision(state).is_some() {
            return Ok(events);
        }
        // CR 514.3a: After cleanup turn-based actions, check SBAs and triggers.
        // If any events are produced, grant priority to the active player.
        // The active player (and others) then pass; `handle_all_passed` will
        // call `enter_step` again for another cleanup round instead of advancing.
        // A safety counter (max 100) guards against pathological infinite loops.
        if state.turn.step == crate::state::turn::Step::Cleanup {
            const MAX_CLEANUP_SBA_ROUNDS: u32 = 100;
            // Trigger checking is done inside check_and_apply_sbas (per-pass).
            let sba_events = sba::check_and_apply_sbas(state);
            events.extend(sba_events.clone());
            let trigger_events = abilities::flush_pending_triggers(state);
            events.extend(trigger_events.clone());
            // CR 603.3 / CR 603.3d (PB-DP8): the batch suspended on a target choice.
            // CR 603.3b gives priority only AFTER every triggered ability of this
            // batch is on the stack, so stop here without granting it, and record
            // that this site owes the grant. `handle_choose_trigger_targets` resumes.
            if state.pending_trigger_targets.is_some() {
                abilities::mark_flush_resume_site(state, FlushResumeSite::EnterStepCleanup);
                return Ok(events);
            }
            let had_events = !sba_events.is_empty() || !trigger_events.is_empty();
            if had_events && state.turn.cleanup_sba_rounds < MAX_CLEANUP_SBA_ROUNDS {
                state.turn.cleanup_sba_rounds += 1;
                // CR 104.4b / CR 726: After each mandatory SBA + trigger batch,
                // check for a recurring board state indicating a mandatory infinite loop.
                if let Some(loop_event) = loop_detection::check_for_mandatory_loop(state) {
                    events.push(loop_event);
                    // All active players lose — game is a draw.
                    let active_players: Vec<_> = state.active_players();
                    for p in active_players {
                        if let Some(player) = state.expect_player_mut(p) {
                            player.has_lost = true;
                        }
                    }
                    events.extend(check_game_over(state));
                    return Ok(events);
                }
                // Grant priority — when all pass, handle_all_passed will re-enter cleanup.
                let active = state.turn.active_player;
                let (passed, priority_events) = priority::grant_initial_priority(state);
                state.turn.players_passed = passed;
                state.turn.priority_holder = Some(active);
                events.extend(priority_events);
                return Ok(events);
            }
            // No SBAs (or safety limit reached) — fall through to auto-advance.
        }
        if state.turn.step.has_priority() {
            // CR 704.3: Check and apply all SBAs before granting priority.
            // Trigger checking is done inside check_and_apply_sbas (per-pass) so
            // that token dies triggers fire before SBA 704.5d removes the token.
            let sba_events = sba::check_and_apply_sbas(state);
            events.extend(sba_events);
            // If all players lost due to SBAs, end the game.
            if is_game_over(state) {
                events.extend(check_game_over(state));
                return Ok(events);
            }
            // Flush any pending triggers before granting priority (CR 603.3).
            let trigger_events = abilities::flush_pending_triggers(state);
            events.extend(trigger_events.clone());
            // CR 603.3 / CR 603.3d (PB-DP8): the batch suspended on a target choice.
            // CR 603.3b gives priority only AFTER every triggered ability of this
            // batch is on the stack, so stop here without granting it, and record
            // that this site owes the grant. `handle_choose_trigger_targets` resumes.
            if state.pending_trigger_targets.is_some() {
                abilities::mark_flush_resume_site(state, FlushResumeSite::EnterStepPriority);
                return Ok(events);
            }
            // CR 104.4b / CR 726: After each mandatory SBA + trigger batch,
            // check for a recurring board state indicating a mandatory infinite loop.
            if !trigger_events.is_empty() {
                if let Some(loop_event) = loop_detection::check_for_mandatory_loop(state) {
                    events.push(loop_event);
                    // All active players lose — game is a draw.
                    let active_players: Vec<_> = state.active_players();
                    for p in active_players {
                        if let Some(player) = state.expect_player_mut(p) {
                            player.has_lost = true;
                        }
                    }
                    events.extend(check_game_over(state));
                    return Ok(events);
                }
            }
            // Grant priority to active player (if still alive)
            let active = state.turn.active_player;
            let is_alive = state
                .players
                .get(&active)
                .map(|p| !p.has_lost && !p.has_conceded)
                .unwrap_or(false);
            if is_alive {
                let (passed, priority_events) = priority::grant_initial_priority(state);
                state.turn.players_passed = passed;
                state.turn.priority_holder = Some(active);
                events.extend(priority_events);
            } else {
                // Active player lost (e.g., drew from empty library).
                // Find next player in APNAP order.
                if let Some(next) = priority::next_priority_player(state, active) {
                    state.turn.players_passed = imbl::OrdSet::new();
                    state.turn.priority_holder = Some(next);
                    events.push(GameEvent::PriorityGiven { player: next });
                } else {
                    state.turn.priority_holder = None;
                }
            }
            return Ok(events);
        }
        // No priority in this step — auto-advance
        // Empty mana pools at step transition
        let mana_events = turn_actions::empty_all_mana_pools(state);
        events.extend(mana_events);
        if let Some((new_turn, step_events)) = turn_structure::advance_step(state) {
            state.turn = new_turn;
            events.extend(step_events);
            // Loop to enter the next step
        } else {
            // Past cleanup — advance to next turn
            let (new_turn, turn_events) = turn_structure::advance_turn(state)?;
            state.turn = new_turn;
            events.extend(turn_events);
            turn_actions::reset_turn_state(state, state.turn.active_player);
            // Loop to enter the first step of the new turn
        }
    }
}
/// CR 608.2d / CR 104.3a / CR 800.4j (PB-DP9): a concede happened while a
/// resolution-time choice was outstanding. Abandon the suspended resolution's
/// answer bank and re-drive it against the board as it now is.
///
/// # Why ANY concede, not just the entry owner's (fix-cycle Finding 2, HIGH)
///
/// `resolve_top_of_stack`'s abort-and-replay is sound on exactly one premise:
/// the state the replay re-executes against is the state the banked questions
/// were asked against. Nothing normally threatens it, because the admission gate
/// (`process_command`, `blocking_decision`) rejects every command while the
/// block stands -- **except two**: the answer itself, which is the mechanism,
/// and `Concede`, which mutates the board (`has_conceded`, CR 611.2b
/// `UntilYourNextTurn` expiry, `temporary_protection_qualities`, CR 725.4
/// initiative, and via `check_game_over` the game's own liveness).
///
/// So the correct invalidation condition is **"has the board changed since the
/// questions were asked"**, and the only admitted command that changes it is a
/// concede -- by anybody. The first version of this function keyed on the
/// ENTRY'S OWNER instead, which left a foreign concede holding a bank bound to a
/// pre-concede board. That is not a theoretical gap: on
/// `SearchLibrary { player: EachPlayer }` a departure shifts the question
/// *positions*, so the next answer was compared against the previous player's
/// banked question and `ask_or_consume_effect_choice`'s determinism
/// `debug_assert!` fired -- a panic in every debug, test and fuzzer build,
/// reached by a legal command sequence.
///
/// Dropping the bank is always safe and never loses a legal announcement: a
/// still-live player is simply re-asked with a fresh `choice_id`, which is what
/// CR 608.2d requires anyway ("the player announces these **while applying the
/// effect**" -- i.e. against the state as it then is). It is also what keeps
/// `ask_or_consume_effect_choice`'s mismatch `debug_assert!` an honest SR-4
/// engine-bug classification: with this widened, a mismatch can no longer be
/// produced by a legal command.
///
/// # Why it must also DRIVE the resolution, not merely clear it
///
/// At the block `priority_holder` is `None` and `players_passed` is full, so
/// nobody can pass and nothing else drives `handle_all_passed`. Clearing the
/// entry without re-driving deadlocks the game -- PB-DP8's exact bug class,
/// which it shipped three times. `test_dp9_owner_concedes_mid_choice` is built
/// on a **three-player** fixture precisely so this path executes.
///
/// # §1.5 exit 4 (elimination by an SBA rather than a concede)
///
/// **Not covered here, and unreachable rather than unhandled.** This function's
/// only caller is [`handle_concede`]. An SBA elimination cannot happen while the
/// block stands because no SBA runs while it stands: `process_command`'s
/// admission gate rejects every command except the answer and `Concede`, and
/// neither reaches `sba::check_state_based_actions` without first clearing the
/// entry. `blocking_decision`'s liveness filter is defence in depth for the
/// same case -- it stops a dead owner's entry from blocking the game, but it
/// does NOT clear the field, so if a future admitted command ever did run an SBA
/// here the residue would be a trap state (seed **OOS-DP9-14**). If that
/// admission gate is ever widened, this function must gain a second caller.
/// # Why the re-drive cannot fail the concede (fix-cycle Finding 13, LOW)
///
/// The re-drive used to be `resolve_top_of_stack(state)?`, so a resolution error
/// propagated out of [`handle_concede`] and `process_command` discarded the whole
/// state -- leaving the player marked as *not* conceded and blocked, i.e.
/// **permanently unable to concede**. Conceding is the one action CR 104.3a
/// always allows, so it must not be gated on a resolution succeeding. The
/// resolution is therefore driven on a clone that is committed only on success;
/// on failure the concede stands, the entry and bank stay cleared, and the
/// unresolved stack object is picked up by the next ordinary priority round
/// (`blocking_decision` is `None` by then, so one can happen).
fn discharge_effect_choice_on_concede(state: &mut GameState, events: &mut Vec<GameEvent>) {
    if state.pending_effect_choice.is_none() {
        return;
    }
    state.pending_effect_choice = None;
    state.effect_choice_answers = imbl::Vector::new();
    // Nothing to resume into: `process_command` answers `GameAlreadyOver` from
    // here on, and the cleared entry keeps the terminal hash clean (§1.5 exit 6).
    if is_game_over(state) {
        return;
    }
    // §1.5 exit 5 as a property rather than an assertion: the suspended object
    // cannot have left the stack (the roll-back put it back, and no admitted
    // command removes stack objects), so this is always non-empty here. Checked
    // rather than `expect`ed so a future admitted command cannot turn a rules
    // bug into a rejected concede.
    if state.stack_objects.is_empty() {
        return;
    }
    let mut probe = state.clone();
    match resolution::resolve_top_of_stack(&mut probe) {
        Ok(resume_events) => {
            *state = probe;
            events.extend(resume_events);
            // §1.5 exit 1's debt, discharged at this site too. On the "suspended
            // again on ANOTHER player's choice" path both statements are the
            // same provable no-ops as at `handle_all_passed`.
            finish_stack_resolution(state, events);
        }
        Err(_e) => {
            // SR-4, engine-bug side: a stack resolution that errors is a rules
            // bug, not a player-visible outcome -- but it must not cost the
            // player their concede. Recorded loudly in debug, swallowed in
            // release with the state left recoverable (see the doc above).
            debug_assert!(
                false,
                "CR 608.2d (PB-DP9): re-driving the suspended resolution after a \
                 concede failed: {_e:?}"
            );
        }
    }
}
/// Handle a Concede command.
fn handle_concede(
    state: &mut GameState,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();
    // Mark player as conceded
    if let Some(p) = state.players.get_mut(&player) {
        if p.has_lost || p.has_conceded {
            return Err(GameStateError::PlayerEliminated(player));
        }
        p.has_conceded = true;
    } else {
        return Err(GameStateError::PlayerNotFound(player));
    }
    // PB-DP7 / DP-3: clear a stale cleanup-discard entry belonging to the
    // conceding player. `blocking_decision` already treats a dead player's
    // entry as absent (so this is not load-bearing for correctness), but the
    // field must still be cleared here or it pollutes the state hash forever.
    //
    // Fix-cycle Finding 5 (MEDIUM): record WHETHER an entry existed for this
    // player before clearing it. If one did, CR 514.1 had paused before any
    // of CR 514.2 (damage clear / "until end of turn" expiry) ran -- see
    // `cleanup_actions`'s pause point. Conceding must not abandon CR 514.2 for
    // the turn (CR 800.4j: the turn continues to completion even without an
    // active player), so the turn-advance branch below re-runs
    // `cleanup_actions` once to finish it before advancing.
    let had_pending_discard_for_conceding_player = state
        .pending_cleanup_discard
        .as_ref()
        .map(|e| e.player == player)
        .unwrap_or(false);
    if had_pending_discard_for_conceding_player {
        state.pending_cleanup_discard = None;
    }
    // PB-DP8 / DP-6 (CR 800.4d / 603.3b / 800.4j): if the conceding player was
    // mid-announcement of a triggered ability's targets, that ability is NOT put
    // on the stack ("If a triggered ability that would be controlled by a player
    // who has left the game would be put onto the stack, it isn't put on the
    // stack"), and neither is any other trigger of the suspended batch they
    // controlled. The REST of the CR 603.3b batch must still be placed -- CR
    // 800.4j: the turn continues to its completion -- so the flush resumes here,
    // and may legitimately suspend again on a different player's trigger.
    if let Some(resume_events) = abilities::drop_departed_trigger_flush(state, player) {
        events.extend(resume_events);
    }
    events.push(GameEvent::PlayerConceded { player });
    // CR 611.2b: Expire any UntilYourNextTurn continuous effects belonging to the
    // conceding player. If the player's turn never arrives, these effects would
    // otherwise persist forever.
    {
        use crate::state::continuous_effect::EffectDuration;
        let keep: imbl::Vector<_> = state
            .continuous_effects
            .iter()
            .filter(|e| e.duration != EffectDuration::UntilYourNextTurn(player))
            .cloned()
            .collect();
        state.continuous_effects = keep;
        if let Some(ps) = state.expect_player_mut(player) {
            ps.temporary_protection_qualities.clear();
        }
    }
    // CR 725.4: If the conceding player had the initiative, transfer it to the
    // next active player in turn order.
    let initiative_events = sba::transfer_initiative_on_player_leave(state, player);
    events.extend(initiative_events);
    // Check game over
    let game_over_events = check_game_over(state);
    events.extend(game_over_events);
    // CR 608.2d / CR 104.3a / CR 800.4j (PB-DP9 / DP-7/8/9): the plan's §1.5
    // exit 2. Placed HERE, and the placement is load-bearing in both directions:
    //
    //  * AFTER every board mutation this command performs (the `has_conceded`
    //    mark, `drop_departed_trigger_flush`, the CR 611.2b `UntilYourNextTurn`
    //    expiry + protection clear, the CR 725.4 initiative transfer,
    //    `check_game_over`) -- because the discharge RE-DRIVES the suspended
    //    resolution and records a fresh question. Running it earlier would
    //    record that question against a board this same command then goes on to
    //    change, which is the very defect (a banked question the state no longer
    //    matches) the widened invalidation exists to prevent, one step out.
    //  * BEFORE the priority/turn blocks below, which are gated on
    //    `blocking_decision(state).is_none()` -- an outstanding entry would
    //    otherwise skip them, and the conceding priority holder's repair with
    //    them.
    discharge_effect_choice_on_concede(state, &mut events);
    // PB-DP8 fix cycle, Finding 5 (CR 104.3a / 603.3b / 800.4j; seed OOS-DP8-9):
    // `drop_departed_trigger_flush` above has already dealt with an entry belonging
    // to the CONCEDING player. Anything still outstanding here therefore belongs to
    // a DIFFERENT player, and the game is mid-way through placing a CR 603.3b batch.
    // Neither of the blocks below may run under it:
    //
    //  * the priority-advance block can reach `handle_all_passed`, which resolves the
    //    top of the stack; the resolution tail calls `flush_pending_triggers`, which
    //    fires its "re-entered while a CR 603.3d target choice is outstanding"
    //    `debug_assert!` (a panic in every test and fuzzer build) and, in release,
    //    silently resolves a spell with the batch half-placed;
    //  * the turn-advance block runs `advance_turn` + `enter_step`, which executes the
    //    new step's turn-based actions before PB-DP7's progress gate stops it -- so the
    //    previous turn's un-placed triggers would land in the NEXT turn.
    //
    // Skipping both cannot hang: the outstanding entry's player is alive by
    // construction (`abilities::flush_sorted` never asks a departed controller), so
    // the block always has an answerer. CR 800.4j is satisfied by an ordinary
    // priority round on resume rather than by the shortcut here: the turn still
    // continues to its completion.
    //
    // CLOSING-REVIEW Finding 1 (HIGH) corrects what this comment used to claim.
    // "The resume grants priority itself (`abilities::finish_resumed_flush`)" is
    // FALSE for `FlushResumeSite::None` -- the resume site of all 30 in-match
    // `check_and_flush_triggers` calls, i.e. the common case -- which returns
    // without touching `priority_holder`. So a conceding PRIORITY HOLDER used to
    // leave the field naming a player who can never act again, and the game was
    // unrecoverable once the batch resumed. The debt is discharged by
    // `abilities::repair_departed_priority_holder` at the end of the resume, which
    // is the earliest moment CR 603.3b allows a grant.
    //
    // SECOND CLOSING-REVIEW Finding 1 (MEDIUM) corrects the *replacement* claim.
    // "Nothing is skipped here that is not picked up there" was ALSO false: the
    // resume is only reached when somebody ANSWERS, and a departure is the other
    // way out of a suspended batch. `drop_departed_trigger_flush` above completes
    // the batch without ever reaching `resume_trigger_flush`, and the block below
    // can only repair a holder that is the CONCEDER (`priority_holder ==
    // Some(player)`) -- never one stranded by an EARLIER departure. Two concedes
    // in a row under one suspended batch therefore reproduced the identical
    // deadlock one step further out. The backstop at the end of this function is
    // what now makes the claim true; see the note there. Nothing is skipped here
    // that is not picked up either there or at the resume.
    //
    // CLOSING-REVIEW HIGH-1 (PB-DP9): the gate reads `blocking_decision`, so it
    // also skips this block for an outstanding CR 608.2d *effect* choice, and
    // PB-DP8's transferable rule (i) demands that each skipped part be named
    // along with where it is picked up. There are exactly two:
    //
    //  * The priority advance. A NO-OP by construction, not a debt: it is
    //    guarded on `priority_holder == Some(player)`, and while a CR 608.2d
    //    entry stands `priority_holder` is `None` (the roll-back restores the
    //    state `resolve_top_of_stack` was entered with, and both of its callers
    //    set the field to `None` immediately beforehand). Asserted in
    //    `abilities::repair_departed_priority_holder`.
    //  * `advance_turn`, when the conceder is the ACTIVE player. Genuinely
    //    skipped, and it MUST be: `discharge_effect_choice_on_concede` has just
    //    put the suspended object back on the stack, and advancing a turn out
    //    from under a mid-resolution spell is exactly the corruption the gate
    //    exists to prevent. It is also not owed: CR 800.4j says the turn
    //    "continues to its completion without an active player" -- the immediate
    //    `advance_turn` below is a shortcut the CR does not require. What CR
    //    800.4j DOES require -- that the departed active player never receive
    //    priority -- is discharged at every grant site that could hand it to
    //    them: `priority::grant_priority_to_active_player` (four sites: both
    //    `resolve_top_of_stack_inner` grants, fixed by closing-review HIGH-1,
    //    plus `handle_declare_blockers` and `counter_stack_object`, fixed by
    //    the second closing review), `enter_step`'s ordinary step grant, and
    //    the forced-payment branch of `handle_all_passed`. NOT `enter_step`'s
    //    cleanup-SBA-round grant, which is still unconditional (OOS-DP9-19).
    //    Probe:
    //    `test_dp9_active_player_concedes_under_a_foreign_block`, which drives a
    //    step boundary past the concede to pin it.
    if !is_game_over(state) && blocking_decision(state).is_none() {
        // If the conceding player held priority, advance priority
        if state.turn.priority_holder == Some(player) {
            let next = priority::next_priority_player(state, player);
            match next {
                Some(next_player) => {
                    state.turn.priority_holder = Some(next_player);
                    events.push(GameEvent::PriorityGiven {
                        player: next_player,
                    });
                }
                None => {
                    // All remaining have passed. MR-M2-03: if the conceding
                    // player is also the active player, do NOT call
                    // handle_all_passed (which would advance the step); the
                    // turn-advance block below handles that path.
                    state.turn.priority_holder = None;
                    if state.turn.active_player != player {
                        let advance_events = handle_all_passed(state)?;
                        events.extend(advance_events);
                    }
                }
            }
        }
        // If it was the conceding player's turn, advance to next turn
        if state.turn.active_player == player {
            // Fix-cycle Finding 5 (MEDIUM), CR 800.4j / CR 514.2: the
            // conceding player's own abandoned cleanup step must still
            // complete CR 514.2 before the turn advances. This can only be
            // reachable when `state.turn.step == Step::Cleanup` (the entry is
            // recorded exclusively by `cleanup_actions`, which only ever runs
            // in that step), so the step check is a defensive mirror, not
            // load-bearing. `cleanup_actions` is safe to call again here even
            // though the entry is already cleared and the player is already
            // marked conceded: with Finding 1's `active_is_alive` guard in
            // place, it will not try to re-record anything -- it will simply
            // fall straight through to `clear_damage` / the saddle clear /
            // `expire_end_of_turn_effects` / `empty_all_mana_pools` /
            // `CleanupPerformed`, exactly the CR 514.2 completion this
            // abandoned turn is owed.
            if had_pending_discard_for_conceding_player
                && state.turn.step == crate::state::turn::Step::Cleanup
            {
                let cleanup_events = turn_actions::cleanup_actions(state);
                events.extend(cleanup_events);
            }
            // MR-M2-15: Clear stale combat state so the next player doesn't
            // inherit an in-progress combat from the conceded turn.
            state.combat = None;
            let mana_events = turn_actions::empty_all_mana_pools(state);
            events.extend(mana_events);
            let (new_turn, turn_events) = turn_structure::advance_turn(state)?;
            state.turn = new_turn;
            events.extend(turn_events);
            turn_actions::reset_turn_state(state, state.turn.active_player);
            let enter_events = enter_step(state)?;
            events.extend(enter_events);
        }
    }
    // CR 800.4 (second closing review, Finding 1 -- MEDIUM): the invariant is that
    // no reachable state leaves `priority_holder` naming a departed player with no
    // pending entry left to repair it. The block above cannot enforce it: its
    // advance is guarded on `priority_holder == Some(player)`, so it only ever
    // repairs holdership belonging to THIS conceder, and PB-DP8's
    // `blocking_decision` gate can skip it entirely. A holder stranded by an
    // earlier departure (a first concede under a suspended batch, whose advance the
    // gate skipped) is invisible to every branch above.
    //
    // So it is caught here, last, after everything that legitimately reassigns
    // priority has had its turn -- which is also why this cannot double-grant:
    // if any branch above set a live holder, `repair_departed_priority_holder`
    // sees one and no-ops. It also no-ops while a CR 603.3b batch is still
    // suspended (a grant there is exactly what the gate exists to prevent); that
    // case is picked up by the repair at the end of `resume_trigger_flush`, or by
    // the next concede's pass through here.
    //
    // Placed at the END rather than immediately after `drop_departed_trigger_flush`
    // (the review's suggested site) because there it would PREEMPT the block above
    // on the ordinary concede path -- a conceding priority holder whose opponents
    // have all passed reaches `next_priority_player() == None`, where the block
    // above owes `handle_all_passed` (MR-M2-03) and the repair would instead grant
    // the active player priority. Same invariant, no behaviour change to a path
    // that was already correct.
    if !is_game_over(state) {
        abilities::repair_departed_priority_holder(state, &mut events);
    }
    Ok(events)
}
/// Check if the game is over (one or fewer active players).
/// Returns GameOver event if applicable.
///
/// `pub(crate)` since PB-DP8's fix cycle: `abilities::finish_resumed_flush`
/// reproduces `enter_step`'s CR 726 mandatory-loop branch, which ends here.
pub(crate) fn check_game_over(state: &GameState) -> Vec<GameEvent> {
    let active = state.active_players();
    match active.len() {
        0 => vec![GameEvent::GameOver { winner: None }],
        1 => vec![GameEvent::GameOver {
            winner: Some(active[0]),
        }],
        _ => Vec::new(),
    }
}
/// Returns true if the game is over.
fn is_game_over(state: &GameState) -> bool {
    let active = state.active_players();
    active.len() <= 1
}
fn validate_player_active(state: &GameState, player: PlayerId) -> Result<(), GameStateError> {
    let p = state.player(player)?;
    if p.has_lost || p.has_conceded {
        return Err(GameStateError::PlayerEliminated(player));
    }
    Ok(())
}
fn validate_player_exists(state: &GameState, player: PlayerId) -> Result<(), GameStateError> {
    state.player(player)?;
    Ok(())
}
/// CR 113.6b: Move opening-hand permanents to the battlefield before the game starts.
///
/// Scans each player's hand for cards whose CardDefinition contains
/// `AbilityDefinition::OpeningHand`. If found, the card is moved from
/// hand to battlefield as a pre-game action (not cast; no spell or ETB triggers fire).
/// This implements the Leyline family rule: "If ~ is in your opening hand, you may
/// begin the game with it on the battlefield."
///
/// Deterministic M9.4 simplification: always place the card on the battlefield
/// (the "may" choice is always taken). Interactive player choice is deferred.
fn place_opening_hand_permanents(
    state: &mut GameState,
    events: &mut Vec<GameEvent>,
) -> Result<(), GameStateError> {
    use crate::cards::card_definition::AbilityDefinition;
    use crate::state::zone::ZoneId;
    // Collect player IDs first (can't borrow state and iterate players simultaneously).
    let player_ids: Vec<crate::state::player::PlayerId> = state.players.keys().copied().collect();
    for player_id in player_ids {
        // Collect (ObjectId, CardId) pairs in hand before moving.
        let hand_ids: Vec<crate::state::game_object::ObjectId> = state
            .expect_zone(&ZoneId::Hand(player_id))
            .map(|z| z.object_ids())
            .unwrap_or_default();
        let hand_entries: Vec<(
            crate::state::game_object::ObjectId,
            Option<crate::state::player::CardId>,
        )> = hand_ids
            .into_iter()
            .map(|obj_id| {
                // obj_id came from the live Hand zone just above; card_id None is a legit token read.
                let card_id = state.expect_object(obj_id).and_then(|o| o.card_id.clone());
                (obj_id, card_id)
            })
            .collect();
        for (obj_id, card_id_opt) in hand_entries {
            // Check if this card has the OpeningHand ability.
            let has_opening_hand: bool = card_id_opt
                .as_ref()
                .and_then(|cid| state.card_registry.get(cid.clone()))
                .map(|def| {
                    def.abilities
                        .iter()
                        .any(|a| matches!(a, AbilityDefinition::OpeningHand))
                })
                .unwrap_or(false);
            if has_opening_hand {
                // CR 113.6b: Move from hand to battlefield (pre-game, not cast).
                let (new_id, _old) = state.move_object_to_zone(obj_id, ZoneId::Battlefield)?;
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: player_id,
                    object_id: new_id,
                });
                // Register replacement abilities and static continuous effects from
                // this permanent's card definition so its effects are active from
                // the start of the game (e.g., Leyline exile replacement).
                let registry = std::sync::Arc::clone(&state.card_registry);
                replacement::register_permanent_replacement_abilities(
                    state,
                    new_id,
                    player_id,
                    card_id_opt.as_ref(),
                    &registry,
                );
                replacement::register_static_continuous_effects(
                    state,
                    new_id,
                    card_id_opt.as_ref(),
                    &registry,
                    false,
                );
            }
        }
    }
    Ok(())
}
/// Build a `StackObject` for a ring-bearer triggered ability (CR 701.54c).
///
/// Ring ability stack objects are triggered abilities pushed onto the stack when a
/// ring level condition is met (level 2 on attack, level 3 on block, level 4 on
/// combat damage). All alt-cost and mode fields are left at their zero/empty defaults.
pub fn ring_ability_stack_object(
    id: crate::state::ObjectId,
    source_object: crate::state::ObjectId,
    controller: crate::state::PlayerId,
    effect: crate::cards::card_definition::Effect,
) -> crate::state::stack::StackObject {
    use crate::state::stack::{StackObject, StackObjectKind};
    StackObject {
        id,
        controller,
        kind: StackObjectKind::RingAbility {
            source_object,
            effect: Box::new(effect),
            controller,
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_warped: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        was_cast_as_adventure: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_lki: vec![],
        lki_counters: imbl::OrdMap::new(),
        lki_power: None,
        defending_player: None,
    }
}
/// Build a `StackObject` for a dungeon room ability (CR 309.4c).
///
/// Room abilities are triggered abilities pushed onto the stack when the venture
/// marker advances to a new room. All alt-cost and mode fields are irrelevant for
/// room abilities and are left at their zero/empty defaults.
fn room_ability_stack_object(
    id: crate::state::ObjectId,
    player: crate::state::PlayerId,
    dungeon: crate::state::dungeon::DungeonId,
    room: usize,
) -> crate::state::stack::StackObject {
    use crate::state::stack::{StackObject, StackObjectKind};
    StackObject {
        id,
        controller: player,
        kind: StackObjectKind::RoomAbility {
            owner: player,
            dungeon,
            room,
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_warped: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        was_cast_as_adventure: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_lki: vec![],
        lki_counters: imbl::OrdMap::new(),
        lki_power: None,
        defending_player: None,
    }
}
/// CR 701.49: Handle a venture-into-the-dungeon action.
///
/// Implements all three CR 701.49 cases:
/// (a) Player has no dungeon in command zone → choose new dungeon, place marker on room 0.
/// (b) Player is not on bottommost room → advance marker to next room (first exit).
/// (c) Player is on bottommost room → complete dungeon, then start a new one (case a).
///
/// Deterministic fallback: enter LostMineOfPhandelver for regular venture,
/// TheUndercity for force_undercity == true.
///
/// After advancing the marker, a `StackObjectKind::RoomAbility` is pushed onto the
/// stack for the room just entered (CR 309.4c: room abilities are triggered abilities).
pub fn handle_venture_into_dungeon(
    state: &mut GameState,
    player: PlayerId,
    force_undercity: bool,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::dungeon::{get_dungeon, DungeonId, DungeonState};
    let mut events = Vec::new();
    // Determine the current dungeon state for this player.
    let dungeon_state_opt = state.dungeon_state.get(&player).cloned();
    match dungeon_state_opt {
        None => {
            // CR 701.49a: Player has no dungeon in command zone — choose a new dungeon.
            let chosen_dungeon = if force_undercity {
                DungeonId::TheUndercity
            } else {
                DungeonId::LostMineOfPhandelver
            };
            // Place marker on room 0 (topmost room, CR 309.4a).
            state.dungeon_state.insert(
                player,
                DungeonState {
                    dungeon: chosen_dungeon,
                    current_room: 0,
                },
            );
            events.push(GameEvent::VenturedIntoDungeon {
                player,
                dungeon: chosen_dungeon,
                room: 0,
            });
            // CR 309.4c: Push room ability for room 0 onto the stack.
            let room_ability_id = state.next_object_id();
            let room_so = room_ability_stack_object(room_ability_id, player, chosen_dungeon, 0);
            state.stack_objects.push_back(room_so);
        }
        Some(ds) => {
            let dungeon_def = get_dungeon(ds.dungeon);
            let bottommost = dungeon_def.bottommost_room;
            if ds.current_room == bottommost {
                // CR 701.49c: On the bottommost room — complete the dungeon, then start new.
                state.dungeon_state.remove(&player);
                if let Some(ps) = state.expect_player_mut(player) {
                    ps.dungeons_completed += 1;
                    ps.dungeons_completed_set.insert(ds.dungeon);
                }
                events.push(GameEvent::DungeonCompleted {
                    player,
                    dungeon: ds.dungeon,
                });
                // Start a new dungeon (same as case a).
                let new_events = handle_venture_into_dungeon(state, player, force_undercity)?;
                events.extend(new_events);
            } else {
                // CR 701.49b: Not on bottommost room — advance to next room (first exit).
                let current_room_def = &dungeon_def.rooms[ds.current_room];
                if let Some(&next_room) = current_room_def.exits.first() {
                    let dungeon_id = ds.dungeon;
                    state.dungeon_state.insert(
                        player,
                        DungeonState {
                            dungeon: dungeon_id,
                            current_room: next_room,
                        },
                    );
                    events.push(GameEvent::VenturedIntoDungeon {
                        player,
                        dungeon: dungeon_id,
                        room: next_room,
                    });
                    // CR 309.4c: Push room ability for the new room onto the stack.
                    let room_ability_id = state.next_object_id();
                    let room_so =
                        room_ability_stack_object(room_ability_id, player, dungeon_id, next_room);
                    state.stack_objects.push_back(room_so);
                }
            }
        }
    }
    Ok(events)
}
/// CR 701.54a-c: Process "the Ring tempts you" for a player.
///
/// Steps:
/// 1. Advance ring_level (cap at 4). Emit `RingTempted`.
/// 2. Find all creatures this player controls on the battlefield.
/// 3. If any: choose the one with the lowest ObjectId (deterministic fallback).
/// 4. Clear `RING_BEARER` from the previous ring-bearer (if different creature).
/// 5. Set `RING_BEARER` on the new ring-bearer. Update `player.ring_bearer_id`.
/// 6. Emit `RingBearerChosen`.
/// 7. If no creatures: ring_bearer_id is unchanged (if previously None, stays None).
///
/// Per CR 701.54d, the ring still tempts the player even if no creature is available
/// (the `RingTempted` event fires regardless).
pub fn handle_ring_tempts_you(
    state: &mut GameState,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::game_object::ObjectId;
    use crate::state::types::CardType;
    use crate::state::zone::ZoneId;
    let mut events = Vec::new();
    // Step 1: Advance ring level (cap at 4).
    let new_level = {
        let ps = state.players.get_mut(&player).ok_or_else(|| {
            GameStateError::InvalidCommand(format!("Unknown player {:?}", player))
        })?;
        if ps.ring_level < 4 {
            ps.ring_level += 1;
        }
        ps.ring_level
    };
    events.push(GameEvent::RingTempted { player, new_level });
    // Step 2: Find all creatures this player controls on the battlefield.
    // Collect as sorted Vec so deterministic (lowest ObjectId wins).
    let creature_ids: Vec<ObjectId> = {
        let mut ids: Vec<ObjectId> = state
            .objects
            .values()
            .filter(|obj| {
                obj.zone == ZoneId::Battlefield
                    && obj.is_phased_in()
                    && obj.controller == player
                    // CR 613.1d: Use layer-resolved types (animated permanents are creatures).
                    // obj.id comes from the live `state.objects` iteration, so characteristics
                    // are total; expect_characteristics keeps the layer-resolved types.
                    && crate::rules::layers::expect_characteristics(state, obj.id)
                        .card_types
                        .contains(&CardType::Creature)
            })
            .map(|obj| obj.id)
            .collect();
        ids.sort();
        ids
    };
    // Step 3: Choose ring-bearer — deterministic: lowest ObjectId creature.
    if let Some(&chosen_id) = creature_ids.first() {
        let previous_bearer_id = state.expect_player(player).and_then(|ps| ps.ring_bearer_id);
        // Step 4: Clear RING_BEARER from previous ring-bearer if it's a different creature.
        if let Some(prev_id) = previous_bearer_id {
            if prev_id != chosen_id {
                // CR 400.7: the stored ring_bearer_id may name a creature that has since
                // left the battlefield (cleared by SBA on zone change, per below); a stale
                // id is a legal fizzle, so nothing to clear.
                if let Some(prev_obj) = state.fizzle_object_mut(prev_id) {
                    prev_obj.designations.remove(Designations::RING_BEARER);
                }
            }
        }
        // Step 5: Set RING_BEARER on the chosen creature.
        // chosen_id was just collected from the live `state.objects` iteration above.
        if let Some(chosen_obj) = state.expect_object_mut(chosen_id) {
            chosen_obj.designations.insert(Designations::RING_BEARER);
        }
        // Update player's ring_bearer_id.
        if let Some(ps) = state.expect_player_mut(player) {
            ps.ring_bearer_id = Some(chosen_id);
        }
        // Step 6: Emit RingBearerChosen (fires even when re-choosing same creature).
        events.push(GameEvent::RingBearerChosen {
            player,
            creature: chosen_id,
        });
    }
    // If no creatures: ring_bearer_id stays as-is (cleared elsewhere by SBA on zone change).
    Ok(events)
}
/// Start the game: set up the first turn and enter the first step.
///
/// Call this after building the initial state to begin gameplay.
///
/// # Architecture Invariant 9 (SR-12, SR-21)
///
/// This is the structural companion to `validate_deck`. The completeness marker
/// gate (`DeckViolation::IncompleteCard`) only fires where a caller happens to
/// run `validate_deck`; `GameStateBuilder`, the simulator, and the fuzzer all
/// assemble games straight from `all_cards()` and never call it. So the *only*
/// choke point the turn-running paths share is `start_game`, and this is
/// where the marker is made unbypassable: a game whose objects reference an
/// inert / partial / knowingly-wrong `CardDefinition` is refused with
/// `GameStateError::IncompleteCardsInGame` before the first turn begins.
///
/// ## The complete set of game-running entry paths (SR-21)
///
/// `start_game` is not the *only* way a `GameState` is driven through
/// `process_command`. The script/replay regime builds a state with
/// [`crate::testing::replay_harness::build_initial_state`] and steps it with
/// `process_command` directly, never calling `start_game` — so SR-12's original
/// "the only choke point is `start_game`" was true for the simulator/fuzzer/TUI
/// but **false for the replay path** (found by the 2026-07-11 re-audit; the
/// replay-viewer ran whole games out of ungated inert/partial defs). Every
/// entry path and its opt-out are now:
///
/// | Entry (checked) | Opt-out (incomplete allowed) | Runs the same check |
/// |---|---|---|
/// | [`start_game`] | [`start_game_allowing_incomplete`] | `check_all_defs_complete` |
/// | [`crate::testing::replay_harness::build_initial_state_checked`] | [`crate::testing::replay_harness::build_initial_state`] | `check_all_defs_complete` |
///
/// A caller that genuinely wants an incomplete def in play (engine tests that
/// deliberately exercise a placeholder, harness fixtures, retired scripts that
/// name a not-yet-authored card) must say so explicitly via the opt-out symbol
/// on its path. Both opt-outs are distinct, greppable identifiers — there is no
/// silent bypass on either path.
pub fn start_game(state: GameState) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    check_all_defs_complete(&state)?;
    start_game_allowing_incomplete(state)
}

/// Architecture Invariant 9 pre-game check: every object that names a
/// `CardDefinition` in the registry must reference a `Complete` one.
///
/// Scope is deliberately narrow — it fires *only* for a `card_id` that resolves
/// to a **known** but non-`Complete` def. An object whose `card_id` is absent
/// from the registry is out of scope here (that is the `UnknownCard` axis, and
/// the object already carries synthesised characteristics); a naked test object
/// with no `card_id` is not a "card in the game" at all. This keeps the gate
/// precise: it catches exactly the marker-rot that `validate_deck` catches, at
/// every assembly path, and nothing else.
///
/// `pub(crate)` so the script/replay path can share the *exact* check rather than
/// re-deriving it (SR-21): [`crate::testing::replay_harness::build_initial_state_checked`]
/// calls this on a harness-built state.
pub(crate) fn check_all_defs_complete(state: &GameState) -> Result<(), GameStateError> {
    // Deterministic ordering: `state.objects` is an imbl::OrdMap, so iteration is
    // in ObjectId order and the "first" offender reported is stable across runs.
    let mut offenders = state.objects.values().filter_map(|obj| {
        let cid = obj.card_id.as_ref()?;
        let def = state.card_registry.get(cid.clone())?;
        if def.completeness.is_complete() {
            None
        } else {
            Some((
                def.name.clone(),
                def.completeness.kind(),
                def.completeness.note().to_string(),
            ))
        }
    });
    if let Some((first_name, first_kind, first_note)) = offenders.next() {
        let count = 1 + offenders.count();
        return Err(GameStateError::IncompleteCardsInGame {
            count,
            first_name,
            first_kind,
            first_note,
        });
    }
    Ok(())
}

/// Start the game **without** the Architecture Invariant 9 completeness check.
///
/// This is the explicit opt-out for [`start_game`]. Use it only when an
/// incomplete `CardDefinition` in play is intentional — e.g. an engine test that
/// exercises a placeholder def, or a harness fixture that predates the card's
/// implementation. Production game assembly must go through [`start_game`].
pub fn start_game_allowing_incomplete(
    state: GameState,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    let mut events = Vec::new();
    // CR 113.6b: Place opening-hand permanents on the battlefield before game starts.
    place_opening_hand_permanents(&mut state, &mut events)?;
    let active = state.turn.active_player;
    turn_actions::reset_turn_state(&mut state, active);
    // Set to the beginning of the turn
    state.turn.step = crate::state::turn::Step::Untap;
    state.turn.phase = crate::state::turn::Phase::Beginning;
    state.turn.is_first_turn_of_game = true;
    events.push(GameEvent::TurnStarted {
        player: active,
        turn_number: state.turn.turn_number,
    });
    events.push(GameEvent::StepChanged {
        step: crate::state::turn::Step::Untap,
        phase: crate::state::turn::Phase::Beginning,
    });
    // Enter the first step
    let enter_events = enter_step(&mut state)?;
    events.extend(enter_events);
    // Record events in history
    for event in &events {
        state.history.push_back(event.clone());
    }
    Ok((state, events))
}
/// CR 606: Handle activation of a loyalty ability on a planeswalker.
///
/// Validates timing restrictions (CR 606.3), pays the loyalty cost (CR 606.4),
/// and pushes the ability onto the stack.
fn handle_activate_loyalty_ability(
    state: &mut GameState,
    player: PlayerId,
    source: crate::state::game_object::ObjectId,
    ability_index: usize,
    targets: Vec<crate::state::targeting::Target>,
    x_value: Option<u32>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::cards::card_definition::{AbilityDefinition, LoyaltyCost};
    use crate::state::stack::{StackObject, StackObjectKind};
    use crate::state::turn::Step;
    use crate::state::types::CounterType;
    use crate::state::zone::ZoneId;
    let mut events = Vec::new();
    // CR 606.3: activating a loyalty ability requires the player to have priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // CR 606.3: Main phase, stack empty, once per permanent per turn.
    let is_main_phase = matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain);
    if !is_main_phase {
        return Err(GameStateError::InvalidCommand(
            "ActivateLoyaltyAbility: can only activate during a main phase (CR 606.3)".into(),
        ));
    }
    if !state.stack_objects.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "ActivateLoyaltyAbility: stack must be empty (CR 606.3)".into(),
        ));
    }
    // Validate source is on battlefield and controlled by player.
    let obj = state.objects.get(&source).ok_or_else(|| {
        GameStateError::InvalidCommand("ActivateLoyaltyAbility: source not found".into())
    })?;
    if obj.zone != ZoneId::Battlefield {
        return Err(GameStateError::InvalidCommand(
            "ActivateLoyaltyAbility: source not on battlefield".into(),
        ));
    }
    if obj.controller != player {
        return Err(GameStateError::InvalidCommand(
            "ActivateLoyaltyAbility: source not controlled by player".into(),
        ));
    }
    if obj.loyalty_ability_activated_this_turn {
        return Err(GameStateError::InvalidCommand(
            "ActivateLoyaltyAbility: a loyalty ability has already been activated this turn (CR 606.3)".into(),
        ));
    }
    // Look up the card definition to find the loyalty ability.
    let card_id = obj.card_id.clone();
    let Some(cid) = &card_id else {
        return Err(GameStateError::InvalidCommand(
            "ActivateLoyaltyAbility: source has no card_id".into(),
        ));
    };
    let def = state.card_registry.get(cid.clone()).ok_or_else(|| {
        GameStateError::InvalidCommand("ActivateLoyaltyAbility: card not in registry".into())
    })?;
    // Filter loyalty abilities from the card definition.
    let loyalty_abilities: Vec<&AbilityDefinition> = def
        .abilities
        .iter()
        .filter(|a| matches!(a, AbilityDefinition::LoyaltyAbility { .. }))
        .collect();
    let ability = loyalty_abilities.get(ability_index).ok_or_else(|| {
        GameStateError::InvalidCommand(format!(
            "ActivateLoyaltyAbility: ability_index {} out of range (card has {} loyalty abilities)",
            ability_index,
            loyalty_abilities.len()
        ))
    })?;
    let AbilityDefinition::LoyaltyAbility {
        cost,
        effect,
        targets: ability_targets,
    } = ability
    else {
        unreachable!();
    };
    // Clone ability_targets immediately — the `def` borrow (via card_registry) must be
    // dropped before the mutable `state.objects.get_mut` at the loyalty-cost payment below.
    let ability_targets = ability_targets.clone();
    // CR 601.2c: validate declared targets against the ability's TargetRequirements BEFORE
    // paying the loyalty cost, so an illegal activation doesn't burn loyalty.
    // Mirrors the activated-ability path in rules/abilities.rs.
    if !ability_targets.is_empty() {
        // `source` was validated on the battlefield above with no intervening zone move,
        // so calculate_characteristics is total (its only failure is an absent id).
        let source_chars = Some(crate::rules::layers::expect_characteristics(state, source));
        crate::rules::casting::validate_targets_with_source(
            state,
            &targets,
            &ability_targets,
            player,
            source_chars.as_ref(),
            source,
        )?;
    }
    // CR 606.6: Validate sufficient loyalty counters for negative costs.
    // `source` was validated live above; absent Loyalty counter still yields 0.
    let current_loyalty = state
        .expect_object(source)
        .and_then(|o| o.counters.get(&CounterType::Loyalty).copied())
        .unwrap_or(0);
    let effective_cost = match cost {
        LoyaltyCost::Plus(n) => *n as i32,
        LoyaltyCost::Minus(n) => -(*n as i32),
        LoyaltyCost::Zero => 0,
        LoyaltyCost::MinusX => {
            let x = x_value.unwrap_or(0);
            -(x as i32)
        }
    };
    if effective_cost < 0 && current_loyalty < (-effective_cost) as u32 {
        return Err(GameStateError::InvalidCommand(format!(
            "ActivateLoyaltyAbility: insufficient loyalty counters ({} available, {} needed) (CR 606.6)",
            current_loyalty, -effective_cost
        )));
    }
    // Pay the loyalty cost (CR 606.4).
    // `source` is proven live above (loyalty read via expect_object); keep the raw
    // field access so it borrows only `state.objects`, leaving the `state.card_registry`
    // borrow held by `def`/`effect` free (SR-4 disjoint-borrow hazard).
    debug_assert_object_live!(state, source);
    if let Some(obj) = state.objects.get_mut(&source) {
        let new_loyalty = (current_loyalty as i32 + effective_cost) as u32;
        obj.counters.insert(CounterType::Loyalty, new_loyalty);
        // Mark loyalty ability used this turn (CR 606.3).
        obj.loyalty_ability_activated_this_turn = true;
    }
    // Capture the effect for stack resolution.
    let effect_clone = effect.clone();
    // Convert targets to SpellTargets (capture zone at activation time).
    let spell_targets: Vec<crate::state::targeting::SpellTarget> = targets
        .iter()
        .map(|t| match t {
            crate::state::targeting::Target::Player(id) => crate::state::targeting::SpellTarget {
                target: crate::state::targeting::Target::Player(*id),
                zone_at_cast: None,
            },
            crate::state::targeting::Target::Object(id) => {
                // CR 608.2b: a declared target may have left its zone; a stale id captures
                // no zone (zone_at_cast = None), which the SpellTarget explicitly permits.
                let zone = state.fizzle_object(*id).map(|o| o.zone);
                crate::state::targeting::SpellTarget {
                    target: crate::state::targeting::Target::Object(*id),
                    zone_at_cast: zone,
                }
            }
        })
        .collect();
    // Push the ability onto the stack.
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::LoyaltyAbility {
            source_object: source,
            ability_index,
            effect: Box::new(effect_clone),
        },
        targets: spell_targets,
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_warped: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        was_cast_as_adventure: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: x_value.unwrap_or(0),
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_lki: vec![],
        lki_counters: imbl::OrdMap::new(),
        lki_power: None,
        defending_player: None,
    };
    state.stack_objects.push_back(stack_obj);
    // CR 606.1 -> 602.2b -> 601.2i / CR 117.3c: activating a loyalty ability is
    // activating an ability, so the activating player receives priority afterward. The
    // entry guard above proves `priority_holder == Some(player)` already, so this write
    // is a true identity write (same shape as the Group-A AP-gated sites).
    // CR 117.4: reset the pass-round.
    state.turn.players_passed = imbl::OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: source,
        stack_object_id: stack_id,
    });
    Ok(events)
}
/// CR 716.2a: Handle leveling up a Class enchantment.
///
/// Validates: player controls the Class, it's on the battlefield, sorcery timing
/// (empty stack, main phase), Class is at level N-1, and the mana cost can be paid.
/// Then sets the Class's level to N.
fn handle_level_up_class(
    state: &mut GameState,
    player: PlayerId,
    source: crate::state::game_object::ObjectId,
    target_level: u32,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::cards::card_definition::AbilityDefinition;
    let mut events = Vec::new();
    // CR 716.2a: leveling up a Class requires the player to have priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // Validate the source is on the battlefield and controlled by the player.
    let obj = state
        .objects
        .get(&source)
        .ok_or(GameStateError::InvalidCommand("Class not found".into()))?;
    if obj.controller != player {
        return Err(GameStateError::InvalidCommand(
            "Player doesn't control this Class".into(),
        ));
    }
    if obj.zone != crate::state::zone::ZoneId::Battlefield {
        return Err(GameStateError::InvalidCommand(
            "Class is not on the battlefield".into(),
        ));
    }
    // CR 716.2a: "Activate only as a sorcery" — empty stack + main phase.
    if !state.stack_objects.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "Stack must be empty to level up a Class".into(),
        ));
    }
    let is_main_phase = matches!(
        state.turn.step,
        crate::state::turn::Step::PreCombatMain | crate::state::turn::Step::PostCombatMain
    );
    if !is_main_phase {
        return Err(GameStateError::InvalidCommand(
            "Can only level up a Class during a main phase".into(),
        ));
    }
    // CR 716.2a: "Activate only if this Class is level N-1."
    let current_level = obj.class_level.max(1); // CR 716.2d: treat 0 as 1.
    if current_level != target_level - 1 {
        return Err(GameStateError::InvalidCommand(format!(
            "Class is at level {}, must be at level {} to level up to {}",
            current_level,
            target_level - 1,
            target_level
        )));
    }
    // Find the ClassLevel ability for the target level and get the cost.
    let card_id = obj.card_id.clone();
    let registry = state.card_registry.clone();
    let def = card_id
        .as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .ok_or(GameStateError::InvalidCommand(
            "No card definition for Class".into(),
        ))?;
    let level_cost = def
        .abilities
        .iter()
        .find_map(|a| match a {
            AbilityDefinition::ClassLevel { level, cost, .. } if *level == target_level => {
                Some(cost.clone())
            }
            _ => None,
        })
        .ok_or(GameStateError::InvalidCommand(format!(
            "No ClassLevel ability for level {}",
            target_level
        )))?;
    // Check and pay the mana cost from the player's mana pool.
    {
        let player_state = state
            .players
            .get(&player)
            .ok_or(GameStateError::PlayerNotFound(player))?;
        if !crate::rules::casting::can_pay_cost(&player_state.mana_pool, &level_cost) {
            return Err(GameStateError::InsufficientMana);
        }
    }
    {
        let player_state = state
            .players
            .get_mut(&player)
            .ok_or(GameStateError::PlayerNotFound(player))?;
        crate::rules::casting::pay_cost(&mut player_state.mana_pool, &level_cost);
    }
    // CR 716.2a: Push the level-up as a stack object — it's a normal activated ability
    // that uses the stack and can be responded to (Druid Class rulings 2021-09-24).
    let stack_id = state.next_object_id();
    let stack_obj = crate::state::stack::StackObject {
        id: stack_id,
        controller: player,
        kind: crate::state::stack::StackObjectKind::ClassLevelAbility {
            source_object: source,
            target_level,
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_warped: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        was_cast_as_adventure: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_lki: vec![],
        lki_counters: imbl::OrdMap::new(),
        lki_power: None,
        defending_player: None,
    };
    state.stack_objects.push_back(stack_obj);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: source,
        stack_object_id: stack_id,
    });
    // CR 716.2a -> 602.2b -> 601.2i / CR 117.3c: leveling up a Class is activating an
    // ability, so the activating player receives priority afterward. The entry guard
    // above proves `priority_holder == Some(player)` already, so this write is a true
    // identity write (same shape as the Group-A AP-gated sites). CR 117.4: reset
    // the pass-round.
    state.turn.players_passed = imbl::OrdSet::new();
    state.turn.priority_holder = Some(player);
    Ok(events)
}
