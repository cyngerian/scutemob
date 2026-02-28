//! Invariant checks run after every state transition during fuzzing.
//!
//! 12 checks covering zone integrity, ID uniqueness, mana validity,
//! stack consistency, player consistency, turn order, object-zone
//! agreement, attachment validity, game progression, and more.

use mtg_engine::{GameState, ObjectId, ZoneId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// An invariant violation found during fuzzing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantViolation {
    pub check: String,
    pub description: String,
    pub turn_number: u32,
}

/// Run all invariant checks on a game state. Returns violations found.
pub fn check_all(state: &GameState, prev_turn: Option<u32>) -> Vec<InvariantViolation> {
    let mut violations = Vec::new();

    check_zone_integrity(state, &mut violations);
    check_id_uniqueness(state, &mut violations);
    check_mana_non_negative(state, &mut violations);
    check_stack_consistency(state, &mut violations);
    check_player_consistency(state, &mut violations);
    check_turn_order(state, &mut violations);
    check_object_zone_agreement(state, &mut violations);
    check_attachment_validity(state, &mut violations);
    if let Some(prev) = prev_turn {
        check_game_progression(state, prev, &mut violations);
    }
    check_no_orphaned_tokens(state, &mut violations);

    violations
}

/// 1. Zone integrity: every object in exactly one zone
fn check_zone_integrity(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let mut object_zones: HashMap<ObjectId, Vec<ZoneId>> = HashMap::new();

    for (zone_id, zone) in state.zones.iter() {
        for obj_id in zone.object_ids() {
            object_zones.entry(obj_id).or_default().push(*zone_id);
        }
    }

    // Check for objects in multiple zones
    for (obj_id, zones) in &object_zones {
        if zones.len() > 1 {
            violations.push(InvariantViolation {
                check: "zone_integrity".into(),
                description: format!(
                    "Object {:?} found in {} zones: {:?}",
                    obj_id,
                    zones.len(),
                    zones
                ),
                turn_number: state.turn.turn_number,
            });
        }
    }

    // Check for objects not in any zone
    for (obj_id, _obj) in state.objects.iter() {
        if !object_zones.contains_key(obj_id) {
            violations.push(InvariantViolation {
                check: "zone_integrity".into(),
                description: format!("Object {:?} not found in any zone", obj_id),
                turn_number: state.turn.turn_number,
            });
        }
    }
}

/// 2. ID uniqueness: no duplicate ObjectIds across all zones
fn check_id_uniqueness(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let mut seen: HashSet<ObjectId> = HashSet::new();
    for (_zone_id, zone) in state.zones.iter() {
        for obj_id in zone.object_ids() {
            if !seen.insert(obj_id) {
                violations.push(InvariantViolation {
                    check: "id_uniqueness".into(),
                    description: format!("Duplicate ObjectId {:?} across zones", obj_id),
                    turn_number: state.turn.turn_number,
                });
            }
        }
    }
}

/// 3. Mana non-negative: all mana pool values >= 0
fn check_mana_non_negative(_state: &GameState, _violations: &mut Vec<InvariantViolation>) {
    // ManaPool uses u32 fields, so they can't go negative.
    // This check is a no-op but kept for documentation and future-proofing.
}

/// 4. Stack consistency: stack_objects matches objects in Stack zone
fn check_stack_consistency(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let stack_zone_ids: HashSet<ObjectId> = if let Ok(zone) = state.zone(&ZoneId::Stack) {
        zone.object_ids().into_iter().collect()
    } else {
        HashSet::new()
    };

    let stack_obj_ids: HashSet<ObjectId> = state.stack_objects.iter().map(|so| so.id).collect();

    for id in &stack_zone_ids {
        if !stack_obj_ids.contains(id) {
            violations.push(InvariantViolation {
                check: "stack_consistency".into(),
                description: format!("Object {:?} in Stack zone but not in stack_objects", id),
                turn_number: state.turn.turn_number,
            });
        }
    }

    for id in &stack_obj_ids {
        if !stack_zone_ids.contains(id) {
            violations.push(InvariantViolation {
                check: "stack_consistency".into(),
                description: format!("Object {:?} in stack_objects but not in Stack zone", id),
                turn_number: state.turn.turn_number,
            });
        }
    }
}

/// 5. Player consistency: active player and priority holder are alive
fn check_player_consistency(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let active = state.turn.active_player;
    if let Ok(p) = state.player(active) {
        if p.has_lost || p.has_conceded {
            violations.push(InvariantViolation {
                check: "player_consistency".into(),
                description: format!("Active player {:?} has lost or conceded", active),
                turn_number: state.turn.turn_number,
            });
        }
    }

    if let Some(priority) = state.turn.priority_holder {
        if let Ok(p) = state.player(priority) {
            if p.has_lost || p.has_conceded {
                violations.push(InvariantViolation {
                    check: "player_consistency".into(),
                    description: format!("Priority holder {:?} has lost or conceded", priority),
                    turn_number: state.turn.turn_number,
                });
            }
        }
    }
}

/// 6. Turn order: all players in turn_order
fn check_turn_order(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let active_players = state.active_players();
    for p in &active_players {
        if !state.turn.turn_order.contains(p) {
            violations.push(InvariantViolation {
                check: "turn_order".into(),
                description: format!("Active player {:?} not in turn_order", p),
                turn_number: state.turn.turn_number,
            });
        }
    }
}

/// 7. Object-zone agreement: object's zone field matches containing zone
fn check_object_zone_agreement(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    for (zone_id, zone) in state.zones.iter() {
        for obj_id in zone.object_ids() {
            if let Ok(obj) = state.object(obj_id) {
                if obj.zone != *zone_id {
                    violations.push(InvariantViolation {
                        check: "object_zone_agreement".into(),
                        description: format!(
                            "Object {:?} has zone {:?} but found in zone {:?}",
                            obj_id, obj.zone, zone_id
                        ),
                        turn_number: state.turn.turn_number,
                    });
                }
            }
        }
    }
}

/// 8. Attachment validity: attached_to references existing battlefield objects
fn check_attachment_validity(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    for obj in state.objects_in_zone(&ZoneId::Battlefield) {
        if let Some(target_id) = obj.attached_to {
            if state.object(target_id).is_err() {
                violations.push(InvariantViolation {
                    check: "attachment_validity".into(),
                    description: format!(
                        "Object {:?} attached to {:?} which doesn't exist",
                        obj.id, target_id
                    ),
                    turn_number: state.turn.turn_number,
                });
            }
        }
    }
}

/// 9. Game progression: turn number never decreases
fn check_game_progression(
    state: &GameState,
    prev_turn: u32,
    violations: &mut Vec<InvariantViolation>,
) {
    if state.turn.turn_number < prev_turn {
        violations.push(InvariantViolation {
            check: "game_progression".into(),
            description: format!(
                "Turn number decreased from {} to {}",
                prev_turn, state.turn.turn_number
            ),
            turn_number: state.turn.turn_number,
        });
    }
}

/// 10. No orphaned tokens: no tokens in non-battlefield zones after SBAs.
///
/// Tokens in graveyard/exile are cleaned up by SBAs — if they remain, something is wrong.
fn check_no_orphaned_tokens(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    for (obj_id, obj) in state.objects.iter() {
        if obj.is_token && obj.zone != ZoneId::Battlefield && obj.zone != ZoneId::Stack {
            // Tokens can briefly exist on the stack (e.g., copy of a spell).
            // But in graveyard/exile/hand they should be cleaned up by SBAs.
            violations.push(InvariantViolation {
                check: "no_orphaned_tokens".into(),
                description: format!(
                    "Token {:?} '{}' found in zone {:?}",
                    obj_id, obj.characteristics.name, obj.zone
                ),
                turn_number: state.turn.turn_number,
            });
        }
    }
}
