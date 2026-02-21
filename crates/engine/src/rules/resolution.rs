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

use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::stack::StackObjectKind;
use crate::state::targeting::{SpellTarget, Target};
use crate::state::types::CardType;
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::events::GameEvent;

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
                let legal_count = targets
                    .iter()
                    .filter(|t| is_target_legal(state, t))
                    .count();

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
            let (card_types, owner) = {
                let card = state.object(source_object)?;
                (card.characteristics.card_types.clone(), card.owner)
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
        StackObjectKind::ActivatedAbility { .. } | StackObjectKind::TriggeredAbility { .. } => {
            // M3-E: Ability resolution deferred. Remove from stack without effect.
        }
    }

    // CR 116.3b: After resolution, the active player receives priority.
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

    // After countering, the active player receives priority (same as resolution).
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}
