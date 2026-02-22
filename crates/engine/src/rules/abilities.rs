//! Activated and triggered ability handling (CR 602-603).
//!
//! ## Activated abilities (CR 602)
//!
//! Activated abilities are written as "Cost: Effect." They are NOT mana abilities
//! (those are handled in `rules/mana.rs`). Activating puts a `StackObject` on
//! the stack. The active player receives priority afterward.
//!
//! ## Triggered abilities (CR 603)
//!
//! Triggered abilities begin with "when", "whenever", or "at". When a trigger
//! condition is met:
//! 1. The ability goes into `GameState::pending_triggers`.
//! 2. The next time a player would receive priority, pending triggers are flushed
//!    to the stack in APNAP order (CR 603.3).
//!
//! **Intervening-if (CR 603.4)**: If the ability reads "... if [condition] ...",
//! the condition is checked at trigger time (ability only queues if true) AND at
//! resolution time (ability has no effect if condition became false).

use im::OrdSet;

use crate::state::error::GameStateError;
use crate::state::game_object::{InterveningIf, ObjectId, TriggerEvent};
use crate::state::player::PlayerId;
use crate::state::stack::{StackObject, StackObjectKind};
use crate::state::stubs::PendingTrigger;
use crate::state::targeting::{SpellTarget, Target};
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::casting;
use super::events::GameEvent;

// ---------------------------------------------------------------------------
// Activated ability handler
// ---------------------------------------------------------------------------

/// Handle an ActivateAbility command: validate, pay cost, push onto the stack.
///
/// CR 602.2: To activate an ability, the controller announces it, pays all costs
/// in full, and the ability is placed on the stack. Unlike mana abilities, activated
/// abilities DO use the stack and must be responded to before resolving.
///
/// After activation, the active player receives priority (CR 116.3b).
pub fn handle_activate_ability(
    state: &mut GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
    targets: Vec<Target>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // CR 602.2: Activating requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // Source must be on the battlefield.
    {
        let obj = state.object(source)?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(source));
        }
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: source,
            });
        }
        // Validate the ability index exists.
        if obj
            .characteristics
            .activated_abilities
            .get(ability_index)
            .is_none()
        {
            return Err(GameStateError::InvalidAbilityIndex {
                object_id: source,
                index: ability_index,
            });
        }
    }

    // Clone the cost before mutating state.
    let ability_cost = {
        let obj = state.object(source)?;
        obj.characteristics.activated_abilities[ability_index]
            .cost
            .clone()
    };

    let mut events = Vec::new();

    // Pay tap cost if required (CR 602.2b).
    if ability_cost.requires_tap {
        let obj = state.object(source)?;
        if obj.status.tapped {
            return Err(GameStateError::PermanentAlreadyTapped(source));
        }
        // CR 302.6 / CR 702.10: Summoning sickness prevents using {T} abilities
        // on creatures unless they have haste.
        let is_creature = obj
            .characteristics
            .card_types
            .contains(&crate::state::types::CardType::Creature);
        if is_creature && obj.has_summoning_sickness {
            let has_haste = obj
                .characteristics
                .keywords
                .contains(&crate::state::types::KeywordAbility::Haste);
            if !has_haste {
                return Err(GameStateError::InvalidCommand(format!(
                    "object {:?} has summoning sickness and cannot use abilities with {{T}}",
                    source
                )));
            }
        }
        if let Some(obj) = state.objects.get_mut(&source) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: source,
        });
    }

    // Pay mana cost if required (CR 602.2a).
    if let Some(ref mana_cost) = ability_cost.mana_cost {
        if mana_cost.mana_value() > 0 {
            let player_state = state.player_mut(player)?;
            if !casting::can_pay_cost(&player_state.mana_pool, mana_cost) {
                return Err(GameStateError::InsufficientMana);
            }
            casting::pay_cost(&mut player_state.mana_pool, mana_cost);
            events.push(GameEvent::ManaCostPaid {
                player,
                cost: mana_cost.clone(),
            });
        }
    }

    // CR 602.2c: Validate targets for hexproof and shroud.
    for t in &targets {
        if let Target::Object(id) = t {
            if let Some(obj) = state.objects.get(id) {
                let has_shroud = obj
                    .characteristics
                    .keywords
                    .contains(&crate::state::types::KeywordAbility::Shroud);
                let has_hexproof = obj
                    .characteristics
                    .keywords
                    .contains(&crate::state::types::KeywordAbility::Hexproof);
                if has_shroud {
                    return Err(GameStateError::InvalidTarget(format!(
                        "object {:?} has shroud and cannot be targeted",
                        id
                    )));
                }
                if has_hexproof && obj.controller != player {
                    return Err(GameStateError::InvalidTarget(format!(
                        "object {:?} has hexproof and cannot be targeted by opponents",
                        id
                    )));
                }
            }
        }
    }

    // Snapshot targets (zone recorded at activation time for fizzle check at resolution).
    let spell_targets: Vec<SpellTarget> = targets
        .iter()
        .map(|t| match t {
            Target::Player(id) => SpellTarget {
                target: Target::Player(*id),
                zone_at_cast: None,
            },
            Target::Object(id) => {
                let zone = state.objects.get(id).map(|o| o.zone);
                SpellTarget {
                    target: Target::Object(*id),
                    zone_at_cast: zone,
                }
            }
        })
        .collect();

    // Push the activated ability onto the stack.
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::ActivatedAbility {
            source_object: source,
            ability_index,
        },
        targets: spell_targets,
    };
    state.stack_objects.push_back(stack_obj);

    // CR 602.2e: After activating, the active player receives priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: source,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}

// ---------------------------------------------------------------------------
// Trigger checking
// ---------------------------------------------------------------------------

/// Scan all permanents for triggered abilities that fire in response to `events`.
///
/// Called after any batch of events. Returns `PendingTrigger` entries for each
/// ability that triggered. Does NOT modify state — caller pushes results into
/// `state.pending_triggers`.
///
/// CR 603.2: A triggered ability triggers whenever the trigger event occurs
/// and the trigger condition is met.
/// CR 603.4: If an intervening-if clause is present, the condition is checked
/// at trigger time; the ability only queues if the condition is true.
pub fn check_triggers(state: &GameState, events: &[GameEvent]) -> Vec<PendingTrigger> {
    let mut triggers = Vec::new();

    for event in events {
        match event {
            GameEvent::PermanentEnteredBattlefield { object_id, .. } => {
                // SelfEntersBattlefield: fires on the entering permanent itself.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::SelfEntersBattlefield,
                    Some(*object_id), // Only check this specific object
                );

                // AnyPermanentEntersBattlefield: fires on ALL permanents (including the entering one).
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::AnyPermanentEntersBattlefield,
                    None, // Check all battlefield permanents
                );
            }

            GameEvent::SpellCast { .. } => {
                // AnySpellCast: fires on all permanents that watch for spell casts.
                collect_triggers_for_event(state, &mut triggers, TriggerEvent::AnySpellCast, None);
            }

            GameEvent::PermanentTapped { object_id, .. } => {
                // SelfBecomesTapped: fires on the tapped permanent itself.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::SelfBecomesTapped,
                    Some(*object_id),
                );
            }

            GameEvent::AttackersDeclared { attackers, .. } => {
                // SelfAttacks: fires on each creature that is attacking (CR 603.5).
                for (attacker_id, _) in attackers {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfAttacks,
                        Some(*attacker_id),
                    );
                }
            }

            GameEvent::BlockersDeclared { blockers, .. } => {
                // SelfBlocks: fires on each creature that is blocking (CR 603.5).
                for (blocker_id, _) in blockers {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfBlocks,
                        Some(*blocker_id),
                    );
                }
            }

            _ => {}
        }
    }

    triggers
}

/// Collect triggered abilities of type `event_type` from battlefield permanents.
///
/// If `only_object` is `Some(id)`, only checks that specific object.
/// If `only_object` is `None`, checks all permanents on the battlefield.
fn collect_triggers_for_event(
    state: &GameState,
    triggers: &mut Vec<PendingTrigger>,
    event_type: TriggerEvent,
    only_object: Option<ObjectId>,
) {
    let object_ids: Vec<ObjectId> = if let Some(id) = only_object {
        vec![id]
    } else {
        state
            .objects
            .values()
            .filter(|obj| obj.zone == ZoneId::Battlefield)
            .map(|obj| obj.id)
            .collect()
    };

    for obj_id in object_ids {
        let Some(obj) = state.objects.get(&obj_id) else {
            continue;
        };
        if obj.zone != ZoneId::Battlefield {
            continue;
        }

        for (idx, trigger_def) in obj.characteristics.triggered_abilities.iter().enumerate() {
            if trigger_def.trigger_on != event_type {
                continue;
            }

            // CR 603.4: Check intervening-if at trigger time.
            // If the condition is false, the ability does not trigger.
            if let Some(ref cond) = trigger_def.intervening_if {
                if !check_intervening_if(state, cond, obj.controller) {
                    continue;
                }
            }

            triggers.push(PendingTrigger {
                source: obj_id,
                ability_index: idx,
                controller: obj.controller,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Trigger flushing
// ---------------------------------------------------------------------------

/// Place all pending triggered abilities onto the stack in APNAP order (CR 603.3).
///
/// Called immediately before a player would receive priority. If no pending
/// triggers exist, this is a no-op.
///
/// CR 603.3: "Each time a player would receive priority, the game checks for any
/// triggered abilities that have triggered since the last time a player received
/// priority. If any have triggered, those abilities are put on the stack."
///
/// APNAP ordering (CR 101.4): Active player's triggers go on the stack first
/// (ending up at the bottom), then each non-active player in turn order. The last
/// player's triggers are on top and resolve first.
///
/// Returns events for each ability placed on the stack. Does NOT emit
/// `PriorityGiven` — the caller is responsible for granting priority after.
pub fn flush_pending_triggers(state: &mut GameState) -> Vec<GameEvent> {
    if state.pending_triggers.is_empty() {
        return Vec::new();
    }

    // Drain all pending triggers.
    let pending: Vec<PendingTrigger> = state.pending_triggers.iter().cloned().collect();
    state.pending_triggers = im::Vector::new();

    // Build APNAP order starting from the active player.
    let apnap = apnap_order(state);

    // Stable-sort by controller position in APNAP order.
    let mut sorted = pending;
    sorted.sort_by_key(|t| {
        apnap
            .iter()
            .position(|&p| p == t.controller)
            .unwrap_or(usize::MAX)
    });

    let mut events = Vec::new();

    for trigger in sorted {
        // Push the triggered ability onto the stack as a StackObject.
        let stack_id = state.next_object_id();
        let stack_obj = StackObject {
            id: stack_id,
            controller: trigger.controller,
            kind: StackObjectKind::TriggeredAbility {
                source_object: trigger.source,
                ability_index: trigger.ability_index,
            },
            targets: vec![],
        };
        state.stack_objects.push_back(stack_obj);

        events.push(GameEvent::AbilityTriggered {
            controller: trigger.controller,
            source_object_id: trigger.source,
            stack_object_id: stack_id,
        });
    }

    if !events.is_empty() {
        // Triggers going on the stack is a game action — reset priority pass count.
        state.turn.players_passed = OrdSet::new();
    }

    events
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns player IDs in APNAP order starting from the active player.
///
/// CR 101.4 (APNAP): Active Player, Non-Active Players in turn order.
pub fn apnap_order(state: &GameState) -> Vec<PlayerId> {
    let active = state.turn.active_player;
    let order = &state.turn.turn_order;
    let n = order.len();
    let start = order.iter().position(|&p| p == active).unwrap_or(0);
    (0..n).map(|i| order[(start + i) % n]).collect()
}

/// Evaluate an intervening-if condition against the current game state (CR 603.4).
pub fn check_intervening_if(state: &GameState, cond: &InterveningIf, controller: PlayerId) -> bool {
    match cond {
        InterveningIf::ControllerLifeAtLeast(n) => state
            .players
            .get(&controller)
            .map(|p| p.life_total >= *n as i32)
            .unwrap_or(false),
    }
}
