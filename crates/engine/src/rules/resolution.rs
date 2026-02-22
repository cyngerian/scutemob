//! Stack resolution (CR 608).
//!
//! When all players pass priority in succession with a non-empty stack,
//! the top of the stack resolves (CR 608.1 / LIFO).
//!
//! Instants and sorceries: card moves to owner's graveyard (CR 608.2n).
//! Permanents: card enters the battlefield under spell's controller (CR 608.3a).
//! After resolution: priority resets to the active player (CR 116.3b).
//!
//! **Fizzle rule (CR 608.2b)**: If ALL targets are illegal at resolution time,
//! the spell is removed from the stack and its card goes to the graveyard without
//! resolving (`SpellFizzled`). If only SOME targets are illegal (partial fizzle),
//! the spell resolves normally; illegal targets are unaffected (M7+).

use im::OrdSet;

use crate::effects::{execute_effect, EffectContext};
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::stack::StackObjectKind;
use crate::state::targeting::{SpellTarget, Target};
use crate::state::types::CardType;
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::abilities;
use super::events::GameEvent;
use super::sba;

/// CR 608.1: Resolve the top object on the stack.
///
/// Called when all players pass priority in succession with a non-empty stack.
/// The top object (last in `stack_objects`) resolves via LIFO ordering.
/// After resolution, the active player receives priority (CR 116.3b).
pub fn resolve_top_of_stack(state: &mut GameState) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // Pop the top of the stack (LIFO — last pushed, first resolved).
    let stack_obj = state
        .stack_objects
        .pop_back()
        .ok_or_else(|| GameStateError::InvalidCommand("stack is empty".into()))?;

    match stack_obj.kind.clone() {
        StackObjectKind::Spell { source_object } => {
            let controller = stack_obj.controller;

            // CR 608.2b: Check target legality before resolving.
            let targets = &stack_obj.targets;
            if !targets.is_empty() {
                let legal_count = targets.iter().filter(|t| is_target_legal(state, t)).count();

                if legal_count == 0 {
                    // CR 608.2b: All targets illegal — fizzle.
                    // Card goes to graveyard without effect (same zone move as normal
                    // instant/sorcery resolution, but emits SpellFizzled, not SpellResolved).
                    let owner = state.object(source_object)?.owner;
                    let (new_id, _old) =
                        state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))?;

                    events.push(GameEvent::SpellFizzled {
                        player: controller,
                        stack_object_id: stack_obj.id,
                        source_object_id: new_id,
                    });

                    // CR 704.3: Check SBAs before granting priority.
                    let sba_evts = sba::check_and_apply_sbas(state);
                    events.extend(sba_evts);

                    // Priority resets to active player after fizzle.
                    state.turn.players_passed = OrdSet::new();
                    let active = state.turn.active_player;
                    state.turn.priority_holder = Some(active);
                    events.push(GameEvent::PriorityGiven { player: active });

                    return Ok(events);
                }
                // Partial fizzle (some targets illegal): spell resolves normally.
                // Illegal targets will be unaffected when effects are implemented (M7+).
            }

            // Determine destination zone based on card type (CR 608.2n vs 608.3).
            let (card_types, owner, card_id) = {
                let card = state.object(source_object)?;
                (
                    card.characteristics.card_types.clone(),
                    card.owner,
                    card.card_id.clone(),
                )
            };

            let is_permanent = card_types.iter().any(|t| {
                matches!(
                    t,
                    CardType::Creature
                        | CardType::Artifact
                        | CardType::Enchantment
                        | CardType::Planeswalker
                        | CardType::Battle
                )
            });

            // CR 608.2: Execute the card's effect before it moves to its final zone.
            // Look up the CardDefinition from the registry (if available) and run its Spell effect.
            {
                let registry = state.card_registry.clone();
                if let Some(cid) = card_id {
                    if let Some(def) = registry.get(cid) {
                        // Find the Spell ability variant.
                        let spell_effect = def.abilities.iter().find_map(|a| {
                            if let crate::cards::card_definition::AbilityDefinition::Spell {
                                effect,
                                ..
                            } = a
                            {
                                Some(effect.clone())
                            } else {
                                None
                            }
                        });
                        if let Some(effect) = spell_effect {
                            let mut ctx = EffectContext::new(
                                controller,
                                source_object,
                                stack_obj.targets.clone(),
                            );
                            let effect_events = execute_effect(state, &effect, &mut ctx);
                            events.extend(effect_events);
                        }
                    }
                }
            }

            if is_permanent {
                // CR 608.3a: Permanent spell — card enters the battlefield under
                // the spell's controller's control.
                let (new_id, _old) =
                    state.move_object_to_zone(source_object, ZoneId::Battlefield)?;

                // CR 608.3a: "under the control of the spell's controller"
                // (move_object_to_zone resets controller to owner; restore it here).
                if let Some(obj) = state.objects.get_mut(&new_id) {
                    obj.controller = controller;
                }

                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: controller,
                    object_id: new_id,
                });
                events.push(GameEvent::SpellResolved {
                    player: controller,
                    stack_object_id: stack_obj.id,
                    source_object_id: new_id,
                });
            } else {
                // CR 608.2n: Instant/sorcery — card moves to owner's graveyard.
                let (new_id, _old) =
                    state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))?;

                events.push(GameEvent::SpellResolved {
                    player: controller,
                    stack_object_id: stack_obj.id,
                    source_object_id: new_id,
                });
            }
        }
        StackObjectKind::ActivatedAbility {
            source_object,
            ability_index,
        } => {
            // CR 608.3b: Activated ability resolves — execute its effect.
            // Look up the ability from the Characteristics (inline abilities) or registry.
            let ability_effect = state
                .objects
                .get(&source_object)
                .and_then(|obj| obj.characteristics.activated_abilities.get(ability_index))
                .and_then(|ab| ab.effect.clone());

            if let Some(effect) = ability_effect {
                let mut ctx = EffectContext::new(
                    stack_obj.controller,
                    source_object,
                    stack_obj.targets.clone(),
                );
                let effect_events = execute_effect(state, &effect, &mut ctx);
                events.extend(effect_events);
            }

            events.push(GameEvent::AbilityResolved {
                controller: stack_obj.controller,
                stack_object_id: stack_obj.id,
            });
        }

        StackObjectKind::TriggeredAbility {
            source_object,
            ability_index,
        } => {
            // CR 603.4: Check intervening-if condition at resolution time.
            // If the condition is false, the ability has no effect (but still resolves).
            let condition_holds = {
                let source_obj = state.objects.get(&source_object);
                match source_obj {
                    Some(obj) => {
                        let ability_def =
                            obj.characteristics.triggered_abilities.get(ability_index);
                        match ability_def {
                            Some(def) => def
                                .intervening_if
                                .as_ref()
                                .map(|cond| {
                                    abilities::check_intervening_if(
                                        state,
                                        cond,
                                        stack_obj.controller,
                                    )
                                })
                                .unwrap_or(true),
                            None => true, // No definition found — resolve without effect
                        }
                    }
                    None => true, // Source gone — ability still resolves (no effect)
                }
            };

            // CR 608.3b: Execute effect if condition holds.
            if condition_holds {
                let triggered_effect = state
                    .objects
                    .get(&source_object)
                    .and_then(|obj| obj.characteristics.triggered_abilities.get(ability_index))
                    .and_then(|ab| ab.effect.clone());

                if let Some(effect) = triggered_effect {
                    let mut ctx = EffectContext::new(
                        stack_obj.controller,
                        source_object,
                        stack_obj.targets.clone(),
                    );
                    let effect_events = execute_effect(state, &effect, &mut ctx);
                    events.extend(effect_events);
                }
            }

            events.push(GameEvent::AbilityResolved {
                controller: stack_obj.controller,
                stack_object_id: stack_obj.id,
            });
        }
    }

    // Check for triggered abilities arising from this resolution.
    let new_triggers = abilities::check_triggers(state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }

    // CR 704.3: Check SBAs before granting priority (happens after each resolution).
    let sba_events = sba::check_and_apply_sbas(state);
    let sba_triggers = abilities::check_triggers(state, &sba_events);
    for t in sba_triggers {
        state.pending_triggers.push_back(t);
    }
    events.extend(sba_events);

    // Flush any pending triggers onto the stack before granting priority (CR 603.3).
    let trigger_events = abilities::flush_pending_triggers(state);
    events.extend(trigger_events);

    // CR 116.3b: After resolution (and trigger flushing), the active player receives priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}

/// CR 608.2b: Check whether a spell target is still legal at resolution time.
///
/// A target is illegal if:
/// - It was an object target and the object is no longer in the same zone it was
///   in when targeted ("A target that's no longer in the zone it was in when it
///   was targeted is illegal." — CR 608.2b).
/// - It was a player target and that player is no longer active (eliminated/conceded).
fn is_target_legal(state: &GameState, spell_target: &SpellTarget) -> bool {
    match &spell_target.target {
        Target::Player(id) => state
            .players
            .get(id)
            .map(|p| !p.has_lost && !p.has_conceded)
            .unwrap_or(false),
        Target::Object(id) => {
            // The object must still be in the same zone it was in at cast time.
            state
                .objects
                .get(id)
                .map(|obj| Some(obj.zone) == spell_target.zone_at_cast)
                .unwrap_or(false)
        }
    }
}

/// Counter a specific stack object without it resolving (CR 608.2b, 701.5).
///
/// Finds the stack object by ID, removes it from the stack, and moves the
/// associated card to its owner's graveyard. After countering, the active
/// player receives priority.
///
/// Used by: the fizzle rule (M3-D), counterspell effects (M3-D/E).
pub fn counter_stack_object(
    state: &mut GameState,
    stack_object_id: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // Find and remove the specified stack object (may not be the top).
    let pos = state
        .stack_objects
        .iter()
        .position(|s| s.id == stack_object_id)
        .ok_or(GameStateError::ObjectNotFound(stack_object_id))?;
    let stack_obj = state.stack_objects.remove(pos);

    match stack_obj.kind.clone() {
        StackObjectKind::Spell { source_object } => {
            let controller = stack_obj.controller;
            let owner = state.object(source_object)?.owner;
            let (new_id, _old) =
                state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))?;

            events.push(GameEvent::SpellCountered {
                player: controller,
                stack_object_id: stack_obj.id,
                source_object_id: new_id,
            });
        }
        StackObjectKind::ActivatedAbility { .. } | StackObjectKind::TriggeredAbility { .. } => {
            // Countering abilities is non-standard; just remove from stack.
        }
    }

    // CR 704.3: Check SBAs before granting priority.
    let sba_evts = sba::check_and_apply_sbas(state);
    events.extend(sba_evts);

    // After countering, the active player receives priority (same as resolution).
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}
