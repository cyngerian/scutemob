//! Combat phase rules handler (CR 506-511).
//!
//! Handles the full combat phase:
//! - DeclareAttackers (CR 508): active player declares attackers and their targets
//! - DeclareBlockers (CR 509): defending players declare blockers
//! - OrderBlockers (CR 509.2): attacker chooses damage assignment order for multiple blockers
//! - Combat damage (CR 510): simultaneous damage, trample, deathtouch, first/double strike
//! - Commander damage tracking (CR 903.10a)

use im::OrdSet;

use crate::state::combat::{AttackTarget, CombatState};
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::{CardId, PlayerId};
use crate::state::turn::Step;
use crate::state::types::{CardType, KeywordAbility};
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::abilities;
use super::events::{CombatDamageAssignment, CombatDamageTarget, GameEvent};
use super::layers::calculate_characteristics;

// ---------------------------------------------------------------------------
// Declare Attackers
// ---------------------------------------------------------------------------

/// Handle a DeclareAttackers command (CR 508.1).
///
/// The active player announces which creatures are attacking and what they
/// attack. Non-Vigilance attackers become tapped (CR 508.1f). After declaring,
/// triggers are flushed and priority is granted to the active player.
pub fn handle_declare_attackers(
    state: &mut GameState,
    player: PlayerId,
    attackers: Vec<(ObjectId, AttackTarget)>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // Must be in the DeclareAttackers step.
    if state.turn.step != Step::DeclareAttackers {
        return Err(GameStateError::InvalidCommand(
            "DeclareAttackers is only valid in the DeclareAttackers step".into(),
        ));
    }

    // Must be the active player.
    if player != state.turn.active_player {
        return Err(GameStateError::InvalidCommand(
            "Only the active player can declare attackers".into(),
        ));
    }

    // Must have priority (CR 508.1 is a turn-based action but requires player to have priority).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // Initialize CombatState if not already set (may be set by BeginningOfCombat action).
    if state.combat.is_none() {
        state.combat = Some(CombatState::new(player));
    }

    // Validate each attacker.
    for (attacker_id, _target) in &attackers {
        let obj = state.object(*attacker_id)?;

        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(*attacker_id));
        }
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: *attacker_id,
            });
        }

        // Must be a creature.
        let chars = calculate_characteristics(state, *attacker_id)
            .ok_or(GameStateError::ObjectNotFound(*attacker_id))?;
        if !chars.card_types.contains(&CardType::Creature) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} is not a creature",
                attacker_id
            )));
        }

        // CR 702.3a: A creature with defender can't attack.
        if chars.keywords.contains(&KeywordAbility::Defender) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} has defender and cannot attack",
                attacker_id
            )));
        }

        let has_vigilance = chars.keywords.contains(&KeywordAbility::Vigilance);
        let has_haste = chars.keywords.contains(&KeywordAbility::Haste);
        let obj = state.object(*attacker_id)?;

        // Must not already be tapped (unless Vigilance).
        if obj.status.tapped && !has_vigilance {
            return Err(GameStateError::PermanentAlreadyTapped(*attacker_id));
        }

        // CR 302.6 / CR 702.10: Summoning sickness prevents attacking unless the
        // creature has haste.
        if obj.has_summoning_sickness && !has_haste {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} has summoning sickness and cannot attack (no haste)",
                attacker_id
            )));
        }
    }

    let mut events = Vec::new();

    // Tap non-Vigilance attackers (CR 508.1f).
    for (attacker_id, _target) in &attackers {
        let has_vigilance = has_keyword(state, *attacker_id, KeywordAbility::Vigilance);
        if !has_vigilance {
            if let Some(obj) = state.objects.get_mut(attacker_id) {
                obj.status.tapped = true;
            }
            events.push(GameEvent::PermanentTapped {
                player,
                object_id: *attacker_id,
            });
        }
    }

    // Record attackers in combat state.
    if let Some(combat) = state.combat.as_mut() {
        for (attacker_id, target) in &attackers {
            combat.attackers.insert(*attacker_id, target.clone());
        }
    }

    events.push(GameEvent::AttackersDeclared {
        attacking_player: player,
        attackers: attackers.clone(),
    });

    // Check and queue triggers from the attack declaration (e.g., SelfAttacks).
    let new_triggers = abilities::check_triggers(state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }

    // Flush triggers before granting priority (CR 603.3).
    let trigger_events = abilities::flush_pending_triggers(state);
    events.extend(trigger_events);

    // Grant priority to the active player (combat actions reset priority).
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::PriorityGiven { player });

    Ok(events)
}

// ---------------------------------------------------------------------------
// Declare Blockers
// ---------------------------------------------------------------------------

/// Handle a DeclareBlockers command (CR 509.1).
///
/// Any defending player may declare blockers during the DeclareBlockers step.
/// Priority is not required — this is a turn-based action for defending players.
/// Multiple defending players each declare independently (CR 509.1a).
pub fn handle_declare_blockers(
    state: &mut GameState,
    player: PlayerId,
    blockers: Vec<(ObjectId, ObjectId)>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // Must be in the DeclareBlockers step.
    if state.turn.step != Step::DeclareBlockers {
        return Err(GameStateError::InvalidCommand(
            "DeclareBlockers is only valid in the DeclareBlockers step".into(),
        ));
    }

    // Must not be the attacking player.
    let combat = state
        .combat
        .as_ref()
        .ok_or_else(|| GameStateError::InvalidCommand("No active combat".into()))?;

    if player == combat.attacking_player {
        return Err(GameStateError::InvalidCommand(
            "The attacking player cannot declare blockers".into(),
        ));
    }

    // Validate each blocker.
    for (blocker_id, attacker_id) in &blockers {
        let obj = state.object(*blocker_id)?;

        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(*blocker_id));
        }
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: *blocker_id,
            });
        }
        if obj.status.tapped {
            return Err(GameStateError::PermanentAlreadyTapped(*blocker_id));
        }

        // Must be a creature.
        let blocker_chars = calculate_characteristics(state, *blocker_id)
            .ok_or(GameStateError::ObjectNotFound(*blocker_id))?;
        if !blocker_chars.card_types.contains(&CardType::Creature) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} is not a creature",
                blocker_id
            )));
        }

        // The attacker must be a declared attacker.
        let combat = state.combat.as_ref().unwrap();
        if !combat.attackers.contains_key(attacker_id) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} is not a declared attacker",
                attacker_id
            )));
        }

        // CR 509.1b / CR 702.9a: A creature without flying or reach cannot block
        // a creature with flying.
        let attacker_chars = calculate_characteristics(state, *attacker_id)
            .ok_or(GameStateError::ObjectNotFound(*attacker_id))?;
        let attacker_has_flying = attacker_chars.keywords.contains(&KeywordAbility::Flying);
        let blocker_has_flying = blocker_chars.keywords.contains(&KeywordAbility::Flying);
        let blocker_has_reach = blocker_chars.keywords.contains(&KeywordAbility::Reach);
        if attacker_has_flying && !blocker_has_flying && !blocker_has_reach {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (attacker has flying, blocker has neither flying nor reach)",
                blocker_id, attacker_id
            )));
        }
    }

    // CR 702.110a: A creature with menace can't be blocked except by two or more creatures.
    // Check that no attacker with menace is being blocked by only one creature.
    {
        // Count how many blockers each attacker in this declaration has (summing over all declarations so far + this one).
        use std::collections::HashMap;
        let mut blocker_count_for_attacker: HashMap<ObjectId, usize> = HashMap::new();

        // Existing blockers already recorded in combat state.
        if let Some(combat) = state.combat.as_ref() {
            for (_, &att) in &combat.blockers {
                *blocker_count_for_attacker.entry(att).or_insert(0) += 1;
            }
        }
        // New blockers being declared now.
        for (_, attacker_id) in &blockers {
            *blocker_count_for_attacker.entry(*attacker_id).or_insert(0) += 1;
        }

        for (attacker_id, count) in &blocker_count_for_attacker {
            if *count == 1 {
                let chars = calculate_characteristics(state, *attacker_id)
                    .ok_or(GameStateError::ObjectNotFound(*attacker_id))?;
                if chars.keywords.contains(&KeywordAbility::Menace) {
                    return Err(GameStateError::InvalidCommand(format!(
                        "Object {:?} has menace and must be blocked by two or more creatures",
                        attacker_id
                    )));
                }
            }
        }
    }

    let mut events = Vec::new();

    // Record blockers in combat state.
    if let Some(combat) = state.combat.as_mut() {
        for (blocker_id, attacker_id) in &blockers {
            combat.blockers.insert(*blocker_id, *attacker_id);
        }
        combat.defenders_declared.insert(player);
    }

    // Always emit BlockersDeclared (even for empty declarations, to mark player done).
    events.push(GameEvent::BlockersDeclared {
        defending_player: player,
        blockers: blockers.clone(),
    });

    // Check and queue triggers from blocker declaration (e.g., SelfBlocks).
    let new_triggers = abilities::check_triggers(state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }

    // Do NOT change priority — defending players declare without requiring priority.
    // Triggers will be flushed the next time enter_step grants priority.

    Ok(events)
}

// ---------------------------------------------------------------------------
// Order Blockers
// ---------------------------------------------------------------------------

/// Handle an OrderBlockers command (CR 509.2).
///
/// When an attacker has multiple blockers, its controller declares the order
/// in which damage is assigned. `order` is the blocker ObjectIds from front
/// (receives damage first) to back.
pub fn handle_order_blockers(
    state: &mut GameState,
    player: PlayerId,
    attacker: ObjectId,
    order: Vec<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    if state.turn.step != Step::DeclareBlockers {
        return Err(GameStateError::InvalidCommand(
            "OrderBlockers is only valid during the DeclareBlockers step".into(),
        ));
    }

    // Must be the attacking player.
    let combat = state
        .combat
        .as_ref()
        .ok_or_else(|| GameStateError::InvalidCommand("No active combat".into()))?;

    if player != combat.attacking_player {
        return Err(GameStateError::InvalidCommand(
            "Only the attacking player can order blockers".into(),
        ));
    }

    // Attacker must be a declared attacker.
    if !combat.attackers.contains_key(&attacker) {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} is not a declared attacker",
            attacker
        )));
    }

    // Validate all ordered blockers are actually blocking this attacker.
    let blocking_this: Vec<ObjectId> = combat
        .blockers
        .iter()
        .filter(|(_, &a)| a == attacker)
        .map(|(&b, _)| b)
        .collect();

    for blocker_id in &order {
        if !blocking_this.contains(blocker_id) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} is not blocking attacker {:?}",
                blocker_id, attacker
            )));
        }
    }

    if let Some(combat) = state.combat.as_mut() {
        combat.damage_assignment_order.insert(attacker, order);
    }

    Ok(Vec::new())
}

// ---------------------------------------------------------------------------
// Combat damage
// ---------------------------------------------------------------------------

/// Apply combat damage for the current step (CR 510).
///
/// `first_strike_step`: true when processing `Step::FirstStrikeDamage`,
/// false when processing `Step::CombatDamage`.
///
/// Creatures with FirstStrike or DoubleStrike deal damage in the first-strike
/// step; creatures with DoubleStrike or neither deal damage in the regular step.
///
/// Damage is assigned simultaneously (CR 510.2), then marked on objects/players
/// all at once. SBAs fire afterward (handled by `enter_step`).
pub fn apply_combat_damage(state: &mut GameState, first_strike_step: bool) -> Vec<GameEvent> {
    let Some(combat) = state.combat.as_ref() else {
        return Vec::new();
    };

    // Clone combat data to avoid borrow conflicts during damage application.
    let attackers = combat.attackers.clone();
    let blockers_map = combat.blockers.clone();
    let damage_order = combat.damage_assignment_order.clone();

    let mut assignments: Vec<CombatDamageAssignment> = Vec::new();

    // --- Attacker damage ---
    for (attacker_id, attack_target) in &attackers {
        if !deals_damage_in_step(state, *attacker_id, first_strike_step) {
            continue;
        }

        let power = get_effective_power(state, *attacker_id);
        if power <= 0 {
            continue;
        }

        let has_trample = has_keyword(state, *attacker_id, KeywordAbility::Trample);
        let has_deathtouch = has_keyword(state, *attacker_id, KeywordAbility::Deathtouch);

        // Get ordered blockers (from damage_assignment_order or default OrdMap order).
        let ordered_blockers: Vec<ObjectId> = if let Some(order) = damage_order.get(attacker_id) {
            order
                .iter()
                .filter(|&&b| {
                    state
                        .objects
                        .get(&b)
                        .map(|o| o.zone == ZoneId::Battlefield)
                        .unwrap_or(false)
                })
                .copied()
                .collect()
        } else {
            blockers_map
                .iter()
                .filter(|(_, &a)| a == *attacker_id)
                .filter(|(&b, _)| {
                    state
                        .objects
                        .get(&b)
                        .map(|o| o.zone == ZoneId::Battlefield)
                        .unwrap_or(false)
                })
                .map(|(&b, _)| b)
                .collect()
        };

        if ordered_blockers.is_empty() {
            // CR 509.1h: a creature remains "blocked" even if all blockers leave.
            // Unblocked = was never blocked during declaration.
            let was_blocked = {
                let c = state.combat.as_ref().unwrap();
                c.is_blocked(*attacker_id)
            };

            if !was_blocked {
                // Truly unblocked — deal damage to attack target.
                push_player_or_pw_damage(
                    &mut assignments,
                    *attacker_id,
                    attack_target,
                    power as u32,
                );
            } else if has_trample {
                // Was blocked but all blockers gone: trample goes to player (CR 702.19b).
                push_player_or_pw_damage(
                    &mut assignments,
                    *attacker_id,
                    attack_target,
                    power as u32,
                );
            }
            // else: blocked (blocker gone), no trample → no player damage.
        } else {
            // Assign damage to blockers in order (CR 510.1c).
            let mut remaining = power;
            let last_idx = ordered_blockers.len() - 1;

            for (i, blocker_id) in ordered_blockers.iter().enumerate() {
                if remaining <= 0 {
                    break;
                }

                let is_last = i == last_idx;

                // Minimum lethal damage for this blocker.
                let lethal = if has_deathtouch {
                    1 // CR 702.2c: deathtouch makes 1 damage lethal for assignment purposes
                } else {
                    let toughness = get_effective_toughness(state, *blocker_id);
                    let already_damaged = state
                        .objects
                        .get(blocker_id)
                        .map(|o| o.damage_marked as i32)
                        .unwrap_or(0);
                    (toughness - already_damaged).max(0)
                };

                if is_last && has_trample {
                    // Last blocker with trample: assign minimum lethal, excess to player.
                    let to_blocker = remaining.min(lethal);
                    if to_blocker > 0 {
                        assignments.push(CombatDamageAssignment {
                            source: *attacker_id,
                            target: CombatDamageTarget::Creature(*blocker_id),
                            amount: to_blocker as u32,
                        });
                    }
                    let trample_amount = remaining - to_blocker;
                    if trample_amount > 0 {
                        push_player_or_pw_damage(
                            &mut assignments,
                            *attacker_id,
                            attack_target,
                            trample_amount as u32,
                        );
                    }
                    remaining = 0;
                } else {
                    // CR 510.1c: for the last blocker (no trample), all remaining power
                    // is assigned to it. For non-last blockers, exactly lethal must be
                    // assigned before moving excess to the next blocker in order.
                    let to_blocker = if is_last || remaining < lethal {
                        remaining
                    } else {
                        lethal
                    };
                    if to_blocker > 0 {
                        assignments.push(CombatDamageAssignment {
                            source: *attacker_id,
                            target: CombatDamageTarget::Creature(*blocker_id),
                            amount: to_blocker as u32,
                        });
                    }
                    remaining -= to_blocker;
                }
            }
        }
    }

    // --- Blocker damage (CR 510.1a: blockers also deal damage to attackers) ---
    for (blocker_id, attacker_id) in &blockers_map {
        let blocker_on_bf = state
            .objects
            .get(blocker_id)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);
        let attacker_on_bf = state
            .objects
            .get(attacker_id)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);

        if !blocker_on_bf || !attacker_on_bf {
            continue;
        }

        if !deals_damage_in_step(state, *blocker_id, first_strike_step) {
            continue;
        }

        let power = get_effective_power(state, *blocker_id);
        if power <= 0 {
            continue;
        }

        assignments.push(CombatDamageAssignment {
            source: *blocker_id,
            target: CombatDamageTarget::Creature(*attacker_id),
            amount: power as u32,
        });
    }

    if assignments.is_empty() {
        return Vec::new();
    }

    // --- Collect application info before mutating state ---
    // Pre-extract per-assignment: (source_deathtouch, source_lifelink, source_controller, commander_info)
    // commander_info = Some((attacking_player_id, card_id)) if source is a commander.
    type DamageAppInfo = (bool, bool, PlayerId, Option<(PlayerId, CardId)>);
    let app_info: Vec<DamageAppInfo> = assignments
        .iter()
        .map(|a| {
            let obj = state.objects.get(&a.source);
            let chars = calculate_characteristics(state, a.source);
            let source_deathtouch = chars
                .as_ref()
                .map(|c| c.keywords.contains(&KeywordAbility::Deathtouch))
                .unwrap_or(false);
            // CR 702.15a: Damage dealt by a source with lifelink causes its controller to gain life.
            let source_lifelink = chars
                .as_ref()
                .map(|c| c.keywords.contains(&KeywordAbility::Lifelink))
                .unwrap_or(false);
            let source_controller = obj.map(|o| o.controller).unwrap_or(PlayerId(0));

            let commander_info = obj.and_then(|o| {
                let controller = o.controller;
                let card_id = o.card_id.clone()?;
                let is_commander = state
                    .players
                    .get(&controller)
                    .map(|p| p.commander_ids.iter().any(|c| *c == card_id))
                    .unwrap_or(false);
                if is_commander {
                    Some((controller, card_id))
                } else {
                    None
                }
            });

            (
                source_deathtouch,
                source_lifelink,
                source_controller,
                commander_info,
            )
        })
        .collect();

    // --- Apply damage and collect lifelink gains ---
    // lifelink_gains: controller → total damage dealt by their lifelink sources this step.
    let mut lifelink_gains: im::OrdMap<PlayerId, u32> = im::OrdMap::new();

    for (assignment, (source_deathtouch, source_lifelink, source_controller, commander_info)) in
        assignments.iter().zip(app_info.iter())
    {
        match &assignment.target {
            CombatDamageTarget::Creature(obj_id) => {
                if let Some(obj) = state.objects.get_mut(obj_id) {
                    obj.damage_marked += assignment.amount;
                    if *source_deathtouch && assignment.amount > 0 {
                        obj.deathtouch_damage = true;
                    }
                }
            }
            CombatDamageTarget::Player(player_id) => {
                // Apply life loss.
                if let Some(player) = state.players.get_mut(player_id) {
                    player.life_total -= assignment.amount as i32;
                }
                // Track commander damage (CR 903.10a).
                if let Some((attacking_player, card_id)) = commander_info {
                    if player_id != attacking_player {
                        // Can't double-borrow players — read current value, then write.
                        let current = state
                            .players
                            .get(player_id)
                            .and_then(|p| p.commander_damage_received.get(attacking_player))
                            .and_then(|m| m.get(card_id))
                            .copied()
                            .unwrap_or(0);
                        let new_val = current + assignment.amount;

                        let inner = state
                            .players
                            .get(player_id)
                            .and_then(|p| p.commander_damage_received.get(attacking_player))
                            .cloned()
                            .unwrap_or_default();
                        let mut new_inner = inner;
                        new_inner.insert(card_id.clone(), new_val);

                        if let Some(target_player) = state.players.get_mut(player_id) {
                            target_player
                                .commander_damage_received
                                .insert(*attacking_player, new_inner);
                        }
                    }
                }
            }
            CombatDamageTarget::Planeswalker(pw_id) => {
                // Planeswalkers receive damage as marked damage; SBA 704.5i
                // handles the loyalty counter check separately.
                if let Some(obj) = state.objects.get_mut(pw_id) {
                    obj.damage_marked += assignment.amount;
                }
            }
        }

        // CR 702.15a: Lifelink — source's controller gains life equal to damage dealt.
        if *source_lifelink && assignment.amount > 0 {
            let entry = lifelink_gains.entry(*source_controller).or_insert(0);
            *entry += assignment.amount;
        }
    }

    let mut events = vec![GameEvent::CombatDamageDealt {
        assignments: assignments.clone(),
    }];

    // Apply lifelink gains and emit LifeGained events.
    for (controller, amount) in &lifelink_gains {
        if let Some(player) = state.players.get_mut(controller) {
            player.life_total += *amount as i32;
        }
        events.push(GameEvent::LifeGained {
            player: *controller,
            amount: *amount,
        });
    }

    events
}

// ---------------------------------------------------------------------------
// First strike step detection
// ---------------------------------------------------------------------------

/// Returns true if any combatant has FirstStrike or DoubleStrike,
/// meaning a separate first-strike damage step must occur (CR 510.4).
pub fn should_have_first_strike_step(state: &GameState) -> bool {
    let Some(combat) = state.combat.as_ref() else {
        return false;
    };

    // Check attackers.
    let attacker_has_fs = combat.attackers.keys().any(|&id| {
        has_keyword(state, id, KeywordAbility::FirstStrike)
            || has_keyword(state, id, KeywordAbility::DoubleStrike)
    });

    // Check blockers.
    let blocker_has_fs = combat.blockers.keys().any(|&id| {
        has_keyword(state, id, KeywordAbility::FirstStrike)
            || has_keyword(state, id, KeywordAbility::DoubleStrike)
    });

    attacker_has_fs || blocker_has_fs
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Returns the effective power of an object using the layer system.
fn get_effective_power(state: &GameState, id: ObjectId) -> i32 {
    calculate_characteristics(state, id)
        .and_then(|c| c.power)
        .unwrap_or(0)
}

/// Returns the effective toughness of an object using the layer system.
fn get_effective_toughness(state: &GameState, id: ObjectId) -> i32 {
    calculate_characteristics(state, id)
        .and_then(|c| c.toughness)
        .unwrap_or(0)
}

/// Returns true if the object has the given keyword (via layer system).
fn has_keyword(state: &GameState, id: ObjectId, keyword: KeywordAbility) -> bool {
    calculate_characteristics(state, id)
        .map(|c| c.keywords.contains(&keyword))
        .unwrap_or(false)
}

/// Returns true if this creature deals damage in the given step.
///
/// CR 702.7: First-strike creatures deal damage only in the first-strike step.
/// CR 702.4: Double-strike creatures deal damage in BOTH steps.
/// Normal creatures deal damage only in the regular step.
fn deals_damage_in_step(state: &GameState, id: ObjectId, first_strike_step: bool) -> bool {
    let has_first = has_keyword(state, id, KeywordAbility::FirstStrike);
    let has_double = has_keyword(state, id, KeywordAbility::DoubleStrike);

    if first_strike_step {
        has_first || has_double
    } else {
        has_double || !has_first
    }
}

/// Push a damage assignment to a player or planeswalker attack target.
fn push_player_or_pw_damage(
    assignments: &mut Vec<CombatDamageAssignment>,
    source: ObjectId,
    target: &AttackTarget,
    amount: u32,
) {
    match target {
        AttackTarget::Player(pid) => {
            assignments.push(CombatDamageAssignment {
                source,
                target: CombatDamageTarget::Player(*pid),
                amount,
            });
        }
        AttackTarget::Planeswalker(pw_id) => {
            assignments.push(CombatDamageAssignment {
                source,
                target: CombatDamageTarget::Planeswalker(*pw_id),
                amount,
            });
        }
    }
}
