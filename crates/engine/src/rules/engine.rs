//! Engine integration: command processing and game loop (CR 500-514).
//!
//! `process_command` is the single public entry point. It takes an immutable
//! GameState and a Command, produces a new GameState and a list of events.
//! State module = data, rules module = behavior.

use crate::state::error::GameStateError;
use crate::state::player::PlayerId;
use crate::state::GameState;

use super::abilities;
use super::casting;
use super::combat;
use super::command::Command;
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
        } => {
            validate_player_active(&state, player)?;
            let events = mana::handle_tap_for_mana(&mut state, player, source, ability_index)?;
            all_events.extend(events);
        }
        Command::PlayLand { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: playing a land is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = lands::handle_play_land(&mut state, player, card)?;
            // CR 603.3: Check for triggered abilities arising from the land entering
            // (e.g., Hideaway ETB trigger, Landfall). Mirrors CastSpell / ActivateAbility.
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
            all_events.extend(events);
        }
        Command::CastSpell {
            player,
            card,
            targets,
            convoke_creatures,
            improvise_artifacts,
            delve_cards,
            kicker_times,
            alt_cost,
            escape_exile_cards,
            retrace_discard_land,
            jump_start_discard,
            prototype,
            bargain_sacrifice,
            emerge_sacrifice,
            casualty_sacrifice,
            assist_player,
            assist_amount,
            replicate_count,
            splice_cards,
            entwine_paid,
            escalate_modes,
        } => {
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
                escape_exile_cards,
                retrace_discard_land,
                jump_start_discard,
                prototype,
                bargain_sacrifice,
                emerge_sacrifice,
                casualty_sacrifice,
                assist_player,
                assist_amount,
                replicate_count,
                splice_cards,
                entwine_paid,
                escalate_modes,
            )?;
            // CR 603.3: Check for triggered abilities arising from casting this spell
            // (e.g., "Whenever an opponent casts a spell" — Rhystic Study).
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
            all_events.extend(events);
        }
        Command::ActivateAbility {
            player,
            source,
            ability_index,
            targets,
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
            )?;
            // CR 603.3: Check for triggered abilities arising from activating this ability
            // (e.g., Ward — "Whenever this permanent becomes the target of an ability an
            // opponent controls"). Mirrors the same pattern used for CastSpell.
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
            all_events.extend(events);
        }
        Command::DeclareAttackers {
            player,
            attackers,
            enlist_choices,
        } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: declaring attackers is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let events =
                combat::handle_declare_attackers(&mut state, player, attackers, enlist_choices)?;
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
            // CR 603.2: Check for triggers after forecast activation.
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
            all_events.extend(events);
        }

        // ── Cycling (CR 702.29) ───────────────────────────────────────────
        Command::CycleCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: cycling is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_cycle_card(&mut state, player, card)?;
            // CR 603.2: Check for triggers after cycling (including "when you cycle" triggers).
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
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
            // CR 603.2: Check for triggers after dredge (milled cards may trigger effects;
            // the dredge card returning to hand is not an ETB, so no ETB triggers fire).
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
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
            // CR 603.3: Check for triggered abilities arising from miracle reveal.
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
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
            // CR 603.3: Check for triggered abilities arising from crewing.
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
            all_events.extend(events);
        }

        // ── Foretell (CR 702.143) ─────────────────────────────────────────
        Command::ForetellCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: foretelling is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let events = foretell::handle_foretell_card(&mut state, player, card)?;
            all_events.extend(events);
        }

        // ── Plot (CR 702.170) ─────────────────────────────────────────────
        Command::PlotCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: plotting is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let events = plot::handle_plot_card(&mut state, player, card)?;
            all_events.extend(events);
            // CR 116.3: Special action => player receives priority afterward.
            // Priority is already set to the player since they have priority.
        }

        // ── Suspend (CR 702.62) ───────────────────────────────────────────
        Command::SuspendCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: suspending is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let events = suspend::handle_suspend_card(&mut state, player, card)?;
            all_events.extend(events);
        }

        Command::UnearthCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: unearth is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_unearth_card(&mut state, player, card)?;
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
            all_events.extend(events);
        }

        // ── Embalm (CR 702.128) ──────────────────────────────────────────────
        Command::EmbalmCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: embalm is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_embalm_card(&mut state, player, card)?;
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
            all_events.extend(events);
        }

        // ── Eternalize (CR 702.129) ──────────────────────────────────────────
        Command::EternalizeCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: eternalize is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_eternalize_card(&mut state, player, card)?;
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
            all_events.extend(events);
        }

        // ── Encore (CR 702.141) ─────────────────────────────────────────────
        Command::EncoreCard { player, card } => {
            validate_player_active(&state, player)?;
            // CR 104.4b: encore is a meaningful player choice; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = abilities::handle_encore_card(&mut state, player, card)?;
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
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
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
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
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
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
            // CR 603.3: Check for triggered abilities arising from echo resolution.
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
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
            // CR 603.3: Check for triggered abilities arising from recover resolution.
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
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
            // CR 603.3: Check for triggered abilities arising from CU resolution.
            let new_triggers = abilities::check_triggers(&state, &events);
            for t in new_triggers {
                state.pending_triggers.push_back(t);
            }
            let trigger_events = abilities::flush_pending_triggers(&mut state);
            events.extend(trigger_events);
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
/// CR 701.17a) and `echo_pending` is cleared.
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
            Some((obj.owner, obj.controller, obj.counters.clone()))
        } else {
            None
        }
    });

    let Some((owner, controller, pre_death_counters)) = source_info else {
        // Permanent left the battlefield since the trigger resolved; nothing to do.
        return Ok(events);
    };

    // CR 702.30a: Clear the echo_pending flag regardless of pay/sacrifice.
    if let Some(obj) = state.objects.get_mut(&permanent) {
        obj.echo_pending = false;
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
        if let Some(p) = state.players.get_mut(&player) {
            casting::pay_cost(&mut p.mana_pool, &echo_cost);
        }

        events.push(GameEvent::EchoPaid { player, permanent });
    } else {
        // CR 702.30a: Player declines -- sacrifice the permanent (CR 701.17a: bypasses indestructible).
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

    // Grant priority to the active player.
    state.turn.players_passed = im::OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    Ok(events)
}

/// CR 702.24a: Handle the player's cumulative upkeep payment choice.
///
/// If `pay` is true, deducts the total cost (per_counter_cost x age_count) from
/// the player's mana pool (mana variant) or life total (life variant) and the
/// permanent stays. If `pay` is false, the permanent is sacrificed (bypassing
/// indestructible, CR 701.17a).
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
            Some((obj.owner, obj.controller, obj.counters.clone()))
        } else {
            None
        }
    });

    let Some((owner, controller, pre_death_counters)) = source_info else {
        // Permanent left the battlefield since the trigger resolved; nothing to do.
        return Ok(events);
    };

    // Count age counters (already incremented during trigger resolution).
    let age_count = state
        .objects
        .get(&permanent)
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
                if let Some(p) = state.players.get_mut(&player) {
                    casting::pay_cost(&mut p.mana_pool, &total_cost);
                }
            }
            CumulativeUpkeepCost::Life(amount) => {
                // CR 702.24a: Pay amount * age_count life.
                let total_life = amount * age_count;
                if let Some(p) = state.players.get_mut(&player) {
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
        // CR 702.24a: Player declines -- sacrifice the permanent (CR 701.17a: bypasses indestructible).
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

    // Grant priority to the active player.
    state.turn.players_passed = im::OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    Ok(events)
}

/// Multiply a mana cost by a scalar, used for cumulative upkeep cost calculation.
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
        if let Some(p) = state.players.get_mut(&player) {
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
        // CR 702.59a: Player declines -- exile the card from the graveyard.
        let (new_exile_id, _old) = state.move_object_to_zone(recover_card, ZoneId::Exile)?;
        events.push(GameEvent::RecoverDeclined {
            player,
            recover_card,
            new_exile_id,
        });
    }

    // CR 704.3: Check SBAs after recover resolution.
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);

    // Grant priority to the active player.
    state.turn.players_passed = im::OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

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
    } else {
        // Stack is empty — advance step or turn.

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
                        if let Some(player) = state.players.get_mut(&p) {
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

            // CR 104.4b / CR 726: After each mandatory SBA + trigger batch,
            // check for a recurring board state indicating a mandatory infinite loop.
            if !trigger_events.is_empty() {
                if let Some(loop_event) = loop_detection::check_for_mandatory_loop(state) {
                    events.push(loop_event);
                    // All active players lose — game is a draw.
                    let active_players: Vec<_> = state.active_players();
                    for p in active_players {
                        if let Some(player) = state.players.get_mut(&p) {
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
                    state.turn.players_passed = im::OrdSet::new();
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

    events.push(GameEvent::PlayerConceded { player });

    // Check game over
    let game_over_events = check_game_over(state);
    events.extend(game_over_events);

    if !is_game_over(state) {
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

    Ok(events)
}

/// Check if the game is over (one or fewer active players).
/// Returns GameOver event if applicable.
fn check_game_over(state: &GameState) -> Vec<GameEvent> {
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
            .zones
            .get(&ZoneId::Hand(player_id))
            .map(|z| z.object_ids())
            .unwrap_or_default();

        let hand_entries: Vec<(
            crate::state::game_object::ObjectId,
            Option<crate::state::player::CardId>,
        )> = hand_ids
            .into_iter()
            .map(|obj_id| {
                let card_id = state.objects.get(&obj_id).and_then(|o| o.card_id.clone());
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
                );
            }
        }
    }

    Ok(())
}

/// Start the game: set up the first turn and enter the first step.
/// Call this after building the initial state to begin gameplay.
pub fn start_game(state: GameState) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
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
