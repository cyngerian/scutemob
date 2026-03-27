//! Engine integration: command processing and game loop (CR 500-514).
//!
//! `process_command` is the single public entry point. It takes an immutable
//! GameState and a Command, produces a new GameState and a list of events.
//! State module = data, rules module = behavior.
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
use crate::state::error::GameStateError;
use crate::state::game_object::Designations;
use crate::state::player::PlayerId;
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
            check_and_flush_triggers(&mut state, &mut events);
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
            prototype,
            modes_chosen,
            x_value,
            hybrid_choices,
            phyrexian_life_payments,
            face_down_kind,
            additional_costs,
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
            )?;
            check_and_flush_triggers(&mut state, &mut events);
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
        } => {
            validate_player_active(&state, player)?;
            // CR 116.2b: Turn face up is a special action; reset loop detection.
            loop_detection::reset_loop_detection(&mut state);
            let mut events = handle_turn_face_up(&mut state, player, permanent, method)?;
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
/// CR 701.27a: Transform a double-faced permanent to its other face.
///
/// No new object is created (CR 712.18). Counters, damage, attachments, and
/// continuous effects all persist through transformation.
fn handle_transform(
    state: &mut GameState,
    player: PlayerId,
    permanent: crate::state::game_object::ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::rules::events::GameEvent;
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
    // CR 712.4c: Meld cards cannot be transformed or converted.
    if let Some(ref cid) = obj.card_id {
        if let Some(def) = state.card_registry.get(cid.clone()) {
            if def.meld_pair.is_some() {
                return Ok(events); // Silently ignore transform instruction
            }
        }
    }
    // CR 701.27c: Only DFCs can transform.
    let card_id = obj.card_id.clone();
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
    let would_transform_to_back = !state
        .objects
        .get(&permanent)
        .map(|o| o.is_transformed)
        .unwrap_or(false);
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
    let to_back_face = if let Some(obj) = state.objects.get_mut(&permanent) {
        obj.is_transformed = !obj.is_transformed;
        obj.last_transform_timestamp = state.timestamp_counter;
        state.timestamp_counter += 1;
        obj.is_transformed // true = now showing back face
    } else {
        return Err(GameStateError::ObjectNotFound(permanent));
    };
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
                    crate::rules::layers::calculate_characteristics(state, *mat_id)
                        .map(|c| c.card_types.contains(&req_type))
                        .unwrap_or_else(|| mat_obj.characteristics.card_types.contains(&req_type))
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
    if let Some(p) = state.players.get_mut(&player) {
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
    let source_card_id = state
        .objects
        .get(&exiled_source_id)
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
        // Set is_transformed = true (back face up) on the new permanent.
        // Also track the exiled materials for CR 702.167c abilities.
        if let Some(obj) = state.objects.get_mut(&battlefield_id) {
            obj.is_transformed = true;
            obj.last_transform_timestamp = state.timestamp_counter;
            state.timestamp_counter += 1;
            obj.craft_exiled_cards = exiled_material_ids.into_iter().collect();
        }
        events.push(GameEvent::PermanentEnteredBattlefield {
            player,
            object_id: battlefield_id,
        });
    }
    // If not a DFC, the card stays in exile (no PermanentEnteredBattlefield emitted).
    // CR 704.3: Check SBAs after craft resolution.
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);
    // Grant priority to the active player after craft.
    state.turn.players_passed = im::OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);
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
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::cards::card_definition::AbilityDefinition;
    use crate::state::types::{FaceDownKind, TurnFaceUpMethod};
    use crate::state::zone::ZoneId;
    let mut events = Vec::new();
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
    {
        let player_state = state
            .players
            .get_mut(&player)
            .ok_or(GameStateError::PlayerNotFound(player))?;
        if !player_state.mana_pool.can_spend(&mana_cost, None) {
            return Err(GameStateError::InvalidCommand(
                "TurnFaceUp: player cannot pay the turn-face-up cost".into(),
            ));
        }
        player_state.mana_pool.spend(&mana_cost, None);
    }
    // Check if this is a Megamorph turned face up via MorphCost (gets +1/+1 counter).
    let is_megamorph_flip =
        face_down_as == FaceDownKind::Megamorph && method == TurnFaceUpMethod::MorphCost;
    // Turn the permanent face up: clear face_down and face_down_as.
    if let Some(obj) = state.objects.get_mut(&permanent) {
        obj.status.face_down = false;
        obj.face_down_as = None;
    }
    // CR 702.37b: Megamorph gets +1/+1 counter when turned face up via megamorph cost.
    if is_megamorph_flip {
        if let Some(obj) = state.objects.get_mut(&permanent) {
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
    // CR 116.2b: Special action; reset priority to active player.
    state.turn.players_passed.clear();
    // CR 704.3: Check SBAs after the special action.
    let sba_events = sba::check_and_apply_sbas(state);
    events.extend(sba_events);
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
    // CR 725.4: If the conceding player had the initiative, transfer it to the
    // next active player in turn order.
    let initiative_events = sba::transfer_initiative_on_player_leave(state, player);
    events.extend(initiative_events);
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
                if let Some(ps) = state.players.get_mut(&player) {
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
                    && crate::rules::layers::calculate_characteristics(state, obj.id)
                        .unwrap_or_else(|| obj.characteristics.clone())
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
        let previous_bearer_id = state.players.get(&player).and_then(|ps| ps.ring_bearer_id);
        // Step 4: Clear RING_BEARER from previous ring-bearer if it's a different creature.
        if let Some(prev_id) = previous_bearer_id {
            if prev_id != chosen_id {
                if let Some(prev_obj) = state.objects.get_mut(&prev_id) {
                    prev_obj.designations.remove(Designations::RING_BEARER);
                }
            }
        }
        // Step 5: Set RING_BEARER on the chosen creature.
        if let Some(chosen_obj) = state.objects.get_mut(&chosen_id) {
            chosen_obj.designations.insert(Designations::RING_BEARER);
        }
        // Update player's ring_bearer_id.
        if let Some(ps) = state.players.get_mut(&player) {
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
    let AbilityDefinition::LoyaltyAbility { cost, effect, .. } = ability else {
        unreachable!();
    };
    // CR 606.6: Validate sufficient loyalty counters for negative costs.
    let current_loyalty = state
        .objects
        .get(&source)
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
                let zone = state.objects.get(id).map(|o| o.zone);
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
    };
    state.stack_objects.push_back(stack_obj);
    // Reset priority since a new object is on the stack.
    state.turn.players_passed = im::OrdSet::new();
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
    };
    state.stack_objects.push_back(stack_obj);
    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: source,
        stack_object_id: stack_id,
    });
    // Reset priority since this is a game action.
    state.turn.players_passed = im::OrdSet::new();
    Ok(events)
}
