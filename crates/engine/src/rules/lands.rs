//! Playing lands (CR 305).
//!
//! Playing a land is a special action — it does not use the stack (CR 115.2a).
//! The player simply puts the land onto the battlefield from hand.
//!
//! Legal conditions (CR 305.1):
//! - It is the player's turn
//! - The current step is a main phase (Precombat or Postcombat)
//! - The stack is empty
//! - The player has at least one land play remaining this turn
//! - The card is a land in the player's hand

use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::turn::Step;
use crate::state::types::{CardType, CounterType, KeywordAbility, SubType};
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::events::GameEvent;

/// Handle a PlayLand command: move a land from hand to battlefield.
///
/// Validates all CR 305.1 conditions. After the land enters the battlefield,
/// `players_passed` is reset (a game action occurred), but the active player
/// retains priority.
pub fn handle_play_land(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Playing a land requires priority (CR 305.1: "whenever they have priority").
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // 2. Playing a land is restricted to the active player (CR 305.1).
    if state.turn.active_player != player {
        return Err(GameStateError::InvalidCommand(
            "can only play a land during your own turn".into(),
        ));
    }

    // 3. Must be a main phase (CR 305.1).
    if !matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain) {
        return Err(GameStateError::NotMainPhase);
    }

    // 4. Stack must be empty (CR 305.1).
    if !state.stack_objects.is_empty() {
        return Err(GameStateError::StackNotEmpty);
    }

    // 5. Player must have land plays remaining.
    let land_plays = state.player(player)?.land_plays_remaining;
    if land_plays == 0 {
        return Err(GameStateError::NoLandPlaysRemaining(player));
    }

    // 6. Fetch card and validate it is in the player's hand.
    let card_obj = state.object(card)?;
    if card_obj.zone != ZoneId::Hand(player) {
        return Err(GameStateError::InvalidCommand(
            "card is not in your hand".into(),
        ));
    }

    // 7. Validate the card is a land.
    if !card_obj
        .characteristics
        .card_types
        .contains(&CardType::Land)
    {
        return Err(GameStateError::InvalidCommand("card is not a land".into()));
    }

    // 8. Player must own (and thereby control) the card in hand.
    //    Cards in hand are always controlled by their owner.
    // MR-M3-12: this is an ownership check, not a controller check — use InvalidCommand.
    if card_obj.owner != player {
        return Err(GameStateError::InvalidCommand(format!(
            "cannot play land {:?}: owned by {:?}, not player {:?}",
            card, card_obj.owner, player
        )));
    }

    // 9. Move the land from Hand to Battlefield (CR 305.1, CR 400.7).
    let (new_land_id, _old_obj) = state.move_object_to_zone(card, ZoneId::Battlefield)?;

    // CR 614.12 / 614.15: Apply ETB replacement effects before emitting LandPlayed.
    // Self-ETB replacements from the card definition apply first (CR 614.15).
    let card_id = state
        .objects
        .get(&new_land_id)
        .and_then(|obj| obj.card_id.clone());
    let registry = state.card_registry.clone();
    let mut events: Vec<GameEvent> = Vec::new();
    events.extend(super::replacement::apply_self_etb_from_definition(
        state,
        new_land_id,
        player,
        card_id.as_ref(),
        &registry,
    ));
    events.extend(super::replacement::apply_etb_replacements(
        state,
        new_land_id,
        player,
    ));

    // CR 702.63a: Place N time counters on a land with Vanishing N as it enters.
    // Lands with Vanishing are extremely rare (Nameless Race et al. are not lands),
    // but the ETB hook must exist at both sites per gotchas-infra.md.
    // CR 702.63b: Vanishing(0) does not place counters.
    // CR 702.63c: Multiple instances of Vanishing each work separately -- sum all N values.
    {
        let total_vanishing: u32 = state
            .objects
            .get(&new_land_id)
            .map(|obj| {
                obj.characteristics
                    .keywords
                    .iter()
                    .filter_map(|kw| {
                        if let KeywordAbility::Vanishing(n) = kw {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .sum()
            })
            .unwrap_or(0);
        if total_vanishing > 0 {
            if let Some(obj) = state.objects.get_mut(&new_land_id) {
                let current = obj.counters.get(&CounterType::Time).copied().unwrap_or(0);
                obj.counters = obj
                    .counters
                    .update(CounterType::Time, current + total_vanishing);
            }
        }
    }

    // CR 702.32a: Place N fade counters on a land with Fading N as it enters.
    // Fading lands are extremely rare, but the ETB hook must exist at both sites
    // (resolution.rs and lands.rs) per gotchas-infra.md.
    {
        let total_fading: u32 = state
            .objects
            .get(&new_land_id)
            .map(|obj| {
                obj.characteristics
                    .keywords
                    .iter()
                    .filter_map(|kw| {
                        if let KeywordAbility::Fading(n) = kw {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .sum()
            })
            .unwrap_or(0);
        if total_fading > 0 {
            if let Some(obj) = state.objects.get_mut(&new_land_id) {
                let current = obj.counters.get(&CounterType::Fade).copied().unwrap_or(0);
                obj.counters = obj
                    .counters
                    .update(CounterType::Fade, current + total_fading);
            }
        }
    }

    // CR 702.58a: Place N +1/+1 counters on a land with Graft N as it enters.
    // Llanowar Reborn is the canonical land with Graft. The counters do nothing
    // on a non-creature land but are important if the land becomes a creature.
    // The ETB hook must exist at both sites (resolution.rs and lands.rs) per
    // gotchas-infra.md.
    // CR 702.58b: Multiple instances each work separately -- sum all N values.
    {
        let total_graft: u32 = state
            .objects
            .get(&new_land_id)
            .map(|obj| {
                obj.characteristics
                    .keywords
                    .iter()
                    .filter_map(|kw| {
                        if let KeywordAbility::Graft(n) = kw {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .sum()
            })
            .unwrap_or(0);
        if total_graft > 0 {
            if let Some(obj) = state.objects.get_mut(&new_land_id) {
                let current = obj
                    .counters
                    .get(&CounterType::PlusOnePlusOne)
                    .copied()
                    .unwrap_or(0);
                obj.counters = obj
                    .counters
                    .update(CounterType::PlusOnePlusOne, current + total_graft);
            }
        }
    }

    // CR 702.38a: Amplify N -- as this object enters, reveal cards from hand that share
    // a creature type with it; enters with N +1/+1 counters per revealed card.
    // CR 702.38b: Multiple instances work separately.
    // Lands never have creature subtypes, so the eligible hand count is always 0.
    // The ETB hook exists here for consistency with resolution.rs (gotchas-infra.md).
    {
        // Collect Amplify instances from the land's keyword set.
        let amplify_instances: Vec<u32> = state
            .objects
            .get(&new_land_id)
            .map(|obj| {
                obj.characteristics
                    .keywords
                    .iter()
                    .filter_map(|kw| {
                        if let KeywordAbility::Amplify(n) = kw {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !amplify_instances.is_empty() {
            // Resolve entering land's subtypes (via layer system -- respects CDAs).
            let entering_subtypes: im::OrdSet<SubType> =
                crate::rules::layers::calculate_characteristics(state, new_land_id)
                    .map(|c| c.subtypes)
                    .unwrap_or_default();

            // Count hand cards sharing a creature subtype with the land.
            // Lands have no creature subtypes, so this is always 0 in practice.
            let hand_obj_ids: Vec<ObjectId> = state
                .objects
                .iter()
                .filter(|(_, obj)| obj.zone == ZoneId::Hand(player) && obj.is_phased_in())
                .map(|(id, _)| *id)
                .collect();

            let eligible_count = hand_obj_ids
                .iter()
                .filter(|&&hand_id| {
                    let hand_subtypes =
                        crate::rules::layers::calculate_characteristics(state, hand_id)
                            .map(|c| c.subtypes)
                            .unwrap_or_default();
                    !entering_subtypes.is_empty()
                        && !entering_subtypes
                            .clone()
                            .intersection(hand_subtypes)
                            .is_empty()
                })
                .count() as u32;

            let mut total_amplify_counters: u32 = 0;
            for n in &amplify_instances {
                total_amplify_counters += n * eligible_count;
            }

            if total_amplify_counters > 0 {
                if let Some(obj) = state.objects.get_mut(&new_land_id) {
                    let current = obj
                        .counters
                        .get(&CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters = obj.counters.update(
                        CounterType::PlusOnePlusOne,
                        current + total_amplify_counters,
                    );
                }
                events.push(GameEvent::CounterAdded {
                    object_id: new_land_id,
                    counter: CounterType::PlusOnePlusOne,
                    count: total_amplify_counters,
                });
            }
        }
    }

    // CR 702.54a: Bloodthirst N -- "If an opponent was dealt damage this turn,
    // this permanent enters with N +1/+1 counters on it."
    // Lands with Bloodthirst are not printed in Magic, but the ETB hook exists here
    // for consistency with resolution.rs (completeness over the rule, not over printed cards).
    // CR 702.54c: Multiple instances work separately.
    // CR 800.4a: Eliminated/conceded players are not opponents.
    {
        let bloodthirst_instances: Vec<u32> = state
            .objects
            .get(&new_land_id)
            .map(|obj| {
                obj.characteristics
                    .keywords
                    .iter()
                    .filter_map(|kw| {
                        if let KeywordAbility::Bloodthirst(n) = kw {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !bloodthirst_instances.is_empty() {
            // Check if any opponent was dealt damage this turn.
            let any_opponent_damaged = state.players.iter().any(|(pid, ps)| {
                *pid != player
                    && !ps.has_lost
                    && !ps.has_conceded
                    && ps.damage_received_this_turn > 0
            });

            if any_opponent_damaged {
                let total_counters: u32 = bloodthirst_instances.iter().sum();
                if total_counters > 0 {
                    if let Some(obj) = state.objects.get_mut(&new_land_id) {
                        let current = obj
                            .counters
                            .get(&CounterType::PlusOnePlusOne)
                            .copied()
                            .unwrap_or(0);
                        obj.counters = obj
                            .counters
                            .update(CounterType::PlusOnePlusOne, current + total_counters);
                    }
                    events.push(GameEvent::CounterAdded {
                        object_id: new_land_id,
                        counter: CounterType::PlusOnePlusOne,
                        count: total_counters,
                    });
                }
            }
        }
    }

    // CR 702.82a: Devour N -- lands with Devour are not printed in Magic.
    // ETB hook exists here for consistency with resolution.rs (gotchas-infra.md).
    // No-op: lands have no StackObject to carry devour_sacrifices.

    // CR 702.104a: Tribute N -- lands with Tribute are not printed in Magic.
    // ETB hook exists here for consistency with resolution.rs (gotchas-infra.md).
    // Bot play: opponent always declines; tribute_was_paid remains false (default).
    // No-op for lands (no StackObject, no opponent selection needed).

    // CR 702.123a: Fabricate N -- lands with Fabricate are not printed in Magic.
    // ETB hook exists in fire_when_enters_triggered_effects (replacement.rs).
    // No-op for lands.

    // CR 702.30a: Mark lands with Echo as pending their echo trigger.
    // "At the beginning of your upkeep, if this permanent came under your
    // control since the beginning of your last upkeep, sacrifice it unless
    // you pay [cost]." Setting echo_pending models the condition.
    // Echo lands are extremely rare, but the ETB hook must exist at both sites
    // (resolution.rs and lands.rs) per gotchas-infra.md.
    if let Some(obj) = state.objects.get_mut(&new_land_id) {
        if obj
            .characteristics
            .keywords
            .iter()
            .any(|kw| matches!(kw, KeywordAbility::Echo(_)))
        {
            obj.echo_pending = true;
        }
    }

    // CR 614: Register global replacement abilities from this land's card definition.
    super::replacement::register_permanent_replacement_abilities(
        state,
        new_land_id,
        player,
        card_id.as_ref(),
        &registry,
    );

    // CR 604 / CR 613: Register static continuous effects from this land's card definition.
    super::replacement::register_static_continuous_effects(
        state,
        new_land_id,
        card_id.as_ref(),
        &registry,
    );

    events.push(GameEvent::LandPlayed {
        player,
        new_land_id,
    });
    // CR 603.2: Emit PermanentEnteredBattlefield so that ETB-sensitive trigger
    // checking (check_triggers) can detect abilities like Hideaway. LandPlayed is
    // consumed by the land-play-count tracker; PermanentEnteredBattlefield is the
    // canonical "object arrived on battlefield" signal for all triggered abilities.
    events.push(GameEvent::PermanentEnteredBattlefield {
        player,
        object_id: new_land_id,
    });

    // CR 603.2: Fire mandatory WhenEntersBattlefield triggered effects from card
    // definition inline (e.g., Rest in Peace ETB exile). Interactive ETB triggers
    // are handled via PendingTrigger.
    events.extend(super::replacement::fire_when_enters_triggered_effects(
        state,
        new_land_id,
        player,
        card_id.as_ref(),
        &registry,
    ));

    // 10. Decrement land plays for this turn.
    {
        let player_state = state.player_mut(player)?;
        player_state.land_plays_remaining -= 1;
    }

    // 11. Reset players_passed — a game action occurred, so the priority round
    //     starts fresh. The active player retains priority (CR 117.3b).
    state.turn.players_passed = im::OrdSet::new();

    Ok(events)
}
