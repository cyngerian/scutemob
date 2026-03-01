//! Combat phase rules handler (CR 506-511).
//!
//! Handles the full combat phase:
//! - DeclareAttackers (CR 508): active player declares attackers and their targets
//! - DeclareBlockers (CR 509): defending players declare blockers
//! - OrderBlockers (CR 509.2): attacker chooses damage assignment order for multiple blockers
//! - Combat damage (CR 510): simultaneous damage, trample, deathtouch, first/double strike
//! - Commander damage tracking (CR 903.10a)

use im::{OrdMap, OrdSet};

use crate::state::combat::{AttackTarget, CombatState};
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::{CardId, PlayerId};
use crate::state::turn::Step;
use crate::state::types::{CardType, Color, CounterType, KeywordAbility, LandwalkType, SuperType};
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
    enlist_choices: Vec<(ObjectId, ObjectId)>,
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

    // Validate each attacker and collect vigilance flags for the tapping loop below.
    // MR-M6-12: capture has_vigilance here to avoid a second calculate_characteristics
    //           call in the tapping loop.
    let mut attacker_vigilance: Vec<(ObjectId, bool)> = Vec::with_capacity(attackers.len());
    for (attacker_id, target) in &attackers {
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

        // MR-M6-01: validate attack target (CR 508.1, CR 903.6).
        // A player may only attack opponents or their planeswalkers.
        match target {
            AttackTarget::Player(pid) => {
                if *pid == player {
                    return Err(GameStateError::InvalidAttackTarget(
                        "a player cannot attack themselves".into(),
                    ));
                }
                let target_player = state
                    .players
                    .get(pid)
                    .ok_or(GameStateError::PlayerNotFound(*pid))?;
                if target_player.has_lost || target_player.has_conceded {
                    return Err(GameStateError::InvalidAttackTarget(format!(
                        "player {pid:?} is eliminated"
                    )));
                }
            }
            AttackTarget::Planeswalker(pw_id) => {
                let (pw_zone, pw_controller) = state
                    .objects
                    .get(pw_id)
                    .map(|pw| (pw.zone, pw.controller))
                    .ok_or_else(|| {
                        GameStateError::InvalidAttackTarget(format!(
                            "planeswalker object {pw_id:?} does not exist"
                        ))
                    })?;
                if pw_zone != ZoneId::Battlefield {
                    return Err(GameStateError::InvalidAttackTarget(format!(
                        "planeswalker object {pw_id:?} is not on the battlefield"
                    )));
                }
                if pw_controller == player {
                    return Err(GameStateError::InvalidAttackTarget(format!(
                        "planeswalker {pw_id:?} is controlled by the attacking player"
                    )));
                }
                let pw_chars = calculate_characteristics(state, *pw_id)
                    .ok_or(GameStateError::ObjectNotFound(*pw_id))?;
                if !pw_chars.card_types.contains(&CardType::Planeswalker) {
                    return Err(GameStateError::InvalidAttackTarget(format!(
                        "object {pw_id:?} is not a Planeswalker"
                    )));
                }
            }
        }

        attacker_vigilance.push((*attacker_id, has_vigilance));
    }

    // CR 701.15b: A goaded creature must attack each combat if able.
    // For each creature on the battlefield controlled by the active player
    // that has at least one goading player in goaded_by: if the creature can
    // attack (not tapped without vigilance, no summoning sickness without haste,
    // no Defender), it must be in the attackers list.
    let declared_attacker_ids: OrdSet<ObjectId> = attackers.iter().map(|(id, _)| *id).collect();
    {
        let goaded_ids: Vec<ObjectId> = state
            .objects
            .values()
            .filter(|obj| {
                obj.zone == ZoneId::Battlefield
                    && obj.controller == player
                    && !obj.goaded_by.is_empty()
            })
            .map(|obj| obj.id)
            .collect();

        for goaded_id in goaded_ids {
            if declared_attacker_ids.contains(&goaded_id) {
                continue;
            }
            // Check if the creature is able to attack.
            let chars = match calculate_characteristics(state, goaded_id) {
                Some(c) => c,
                None => continue,
            };
            let obj = match state.objects.get(&goaded_id) {
                Some(o) => o,
                None => continue,
            };
            let has_vigilance = chars.keywords.contains(&KeywordAbility::Vigilance);
            let has_haste = chars.keywords.contains(&KeywordAbility::Haste);
            let has_defender = chars.keywords.contains(&KeywordAbility::Defender);
            let is_tapped = obj.status.tapped;
            let has_sickness = obj.has_summoning_sickness;
            // Creature cannot attack if: tapped and no vigilance, or summoning sickness
            // and no haste, or has Defender.
            let cannot_attack =
                (is_tapped && !has_vigilance) || (has_sickness && !has_haste) || has_defender;
            if !cannot_attack {
                return Err(GameStateError::InvalidCommand(format!(
                    "Goaded creature {:?} must attack (CR 701.15b)",
                    goaded_id
                )));
            }
        }
    }

    // CR 701.15b: A goaded creature must attack a player other than the goading
    // player if able. For each declared attacker that is goaded, if its target is
    // one of the goading players, verify there is no other valid (non-goading) target.
    {
        let opponent_ids: Vec<PlayerId> = state
            .players
            .keys()
            .filter(|pid| **pid != player)
            .filter(|pid| {
                state
                    .players
                    .get(*pid)
                    .map(|p| !p.has_lost && !p.has_conceded)
                    .unwrap_or(false)
            })
            .copied()
            .collect();

        for (attacker_id, target) in &attackers {
            let obj = match state.objects.get(attacker_id) {
                Some(o) => o,
                None => continue,
            };
            if obj.goaded_by.is_empty() {
                continue;
            }
            if let AttackTarget::Player(target_pid) = target {
                if obj.goaded_by.contains(target_pid) {
                    // Check if any non-goading opponent exists.
                    let has_non_goading_target =
                        opponent_ids.iter().any(|pid| !obj.goaded_by.contains(pid));
                    if has_non_goading_target {
                        return Err(GameStateError::InvalidCommand(format!(
                            "Goaded creature {:?} must attack a player other than the goading player if able (CR 701.15b)",
                            attacker_id
                        )));
                    }
                }
            }
        }
    }

    // ---- CR 702.154a / CR 508.1g: Validate enlist choices ----
    //
    // Each (enlisting_attacker_id, enlisted_creature_id) must satisfy:
    //  1. The attacker is in the declared_attacker_ids set.
    //  2. The attacker has the Enlist keyword (layer-aware check).
    //  3. The enlisted creature is on the battlefield, controlled by the player.
    //  4. The enlisted creature is NOT in the declared_attacker_ids set.
    //  5. The enlisted creature is untapped.
    //  6. The enlisted creature is a creature (layer-aware check).
    //  7. The enlisted creature does not have summoning sickness (or has haste).
    //  8. Each enlisted creature appears at most once across ALL enlist choices
    //     (ruling 2022-09-09: "a single creature can't be tapped for more than
    //     one enlist ability").
    //  9. For a given attacker, the number of enlist choices must not exceed
    //     the number of Enlist keyword instances on that attacker (CR 702.154d).
    // 10. The enlisted creature is not the same as the attacker (CR 702.154c).
    {
        let mut enlisted_ids_used: Vec<ObjectId> = Vec::new();
        let mut enlist_used_per_attacker: OrdMap<ObjectId, u32> = OrdMap::new();

        for (attacker_id, enlisted_id) in &enlist_choices {
            // Check 10: cannot enlist itself (CR 702.154c).
            if attacker_id == enlisted_id {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} cannot enlist itself (CR 702.154c)",
                    attacker_id
                )));
            }

            // Check 1: attacker is declared.
            if !declared_attacker_ids.contains(attacker_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} is not a declared attacker",
                    attacker_id
                )));
            }

            // Check 2: attacker has Enlist keyword + check 9: instance count.
            let attacker_chars = calculate_characteristics(state, *attacker_id)
                .ok_or(GameStateError::ObjectNotFound(*attacker_id))?;
            let enlist_count = attacker_chars
                .keywords
                .iter()
                .filter(|kw| matches!(kw, KeywordAbility::Enlist))
                .count() as u32;
            if enlist_count == 0 {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: attacker {:?} does not have the Enlist keyword",
                    attacker_id
                )));
            }
            let used = enlist_used_per_attacker.entry(*attacker_id).or_insert(0);
            *used += 1;
            if *used > enlist_count {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: attacker {:?} has {} Enlist instance(s) but {} choices were made",
                    attacker_id, enlist_count, *used
                )));
            }

            // Check 4: enlisted creature is not attacking.
            if declared_attacker_ids.contains(enlisted_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} is an attacker and cannot be enlisted",
                    enlisted_id
                )));
            }

            // Check 3: on battlefield, controlled by player.
            let enlisted_obj = state.object(*enlisted_id)?;
            if enlisted_obj.zone != ZoneId::Battlefield {
                return Err(GameStateError::ObjectNotOnBattlefield(*enlisted_id));
            }
            if enlisted_obj.controller != player {
                return Err(GameStateError::NotController {
                    player,
                    object_id: *enlisted_id,
                });
            }

            // Check 5: untapped.
            if enlisted_obj.status.tapped {
                return Err(GameStateError::PermanentAlreadyTapped(*enlisted_id));
            }

            // Check 6: is a creature.
            let enlisted_chars = calculate_characteristics(state, *enlisted_id)
                .ok_or(GameStateError::ObjectNotFound(*enlisted_id))?;
            if !enlisted_chars.card_types.contains(&CardType::Creature) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: object {:?} is not a creature",
                    enlisted_id
                )));
            }

            // Check 7: no summoning sickness (or has haste).
            let has_haste = enlisted_chars.keywords.contains(&KeywordAbility::Haste);
            let enlisted_obj_for_sickness = state.object(*enlisted_id)?;
            if enlisted_obj_for_sickness.has_summoning_sickness && !has_haste {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} has summoning sickness and no haste (CR 702.154a)",
                    enlisted_id
                )));
            }

            // Check 8: not already enlisted by another attacker.
            if enlisted_ids_used.contains(enlisted_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} is already enlisted by another attacker \
                     (ruling 2022-09-09)",
                    enlisted_id
                )));
            }
            enlisted_ids_used.push(*enlisted_id);
        }
    }

    let mut events = Vec::new();

    // Tap non-Vigilance attackers (CR 508.1f).
    // Uses pre-computed vigilance flags to avoid a redundant calculate_characteristics call.
    for (attacker_id, has_vigilance) in &attacker_vigilance {
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

    // CR 702.154a / CR 508.1j: Tap enlisted creatures as part of the
    // attack cost payment.
    for (_, enlisted_id) in &enlist_choices {
        if let Some(obj) = state.objects.get_mut(enlisted_id) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: *enlisted_id,
        });
    }

    // Record attackers in combat state.
    if let Some(combat) = state.combat.as_mut() {
        for (attacker_id, target) in &attackers {
            combat.attackers.insert(*attacker_id, target.clone());
        }
    }

    // CR 702.154a: Store enlist pairings for trigger collection in abilities.rs.
    if let Some(combat) = state.combat.as_mut() {
        combat.enlist_pairings = enlist_choices.clone();
    }

    // CR 702.147a: Tag creatures with decayed for EOC sacrifice.
    // "When this creature attacks, sacrifice it at end of combat."
    // Must be tagged here (when state is mutable) rather than in check_triggers
    // (which receives &GameState). The tag persists even if decayed is removed
    // later (ruling 2021-09-24: "Once a creature with decayed attacks, it will be
    // sacrificed at end of combat, even if it no longer has decayed at that time.").
    for (attacker_id, _) in &attackers {
        let has_decayed = calculate_characteristics(state, *attacker_id)
            .map(|c| c.keywords.contains(&KeywordAbility::Decayed))
            .unwrap_or(false);
        if has_decayed {
            if let Some(obj) = state.objects.get_mut(attacker_id) {
                obj.decayed_sacrifice_at_eoc = true;
            }
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

    // Must not be the attacking player and must not have already declared blockers.
    {
        let combat = state
            .combat
            .as_ref()
            .ok_or_else(|| GameStateError::InvalidCommand("No active combat".into()))?;

        if player == combat.attacking_player {
            return Err(GameStateError::InvalidCommand(
                "The attacking player cannot declare blockers".into(),
            ));
        }

        // MR-M6-10: each defending player may only declare blockers once per combat step
        // (CR 509.1a — each defending player declares independently, not repeatedly).
        if combat.defenders_declared.contains(&player) {
            return Err(GameStateError::AlreadyDeclaredBlockers(player));
        }
    }

    // Track blocker IDs seen in this declaration to catch within-batch duplicates.
    let mut seen_blocker_ids: Vec<ObjectId> = Vec::with_capacity(blockers.len());

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

        // CR 702.147a: A creature with decayed can't block.
        if blocker_chars.keywords.contains(&KeywordAbility::Decayed) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} has decayed and cannot block (CR 702.147a)",
                blocker_id
            )));
        }

        // MR-M6-02: a creature can only block one attacker.
        // Check both existing combat.blockers and within-this-declaration duplicates.
        if seen_blocker_ids.contains(blocker_id)
            || state
                .combat
                .as_ref()
                .map(|c| c.blockers.contains_key(blocker_id))
                .unwrap_or(false)
        {
            return Err(GameStateError::DuplicateBlocker(*blocker_id));
        }
        seen_blocker_ids.push(*blocker_id);

        // MR-M6-09: a defending player can only block attackers that are attacking them
        // (or their planeswalker). CR 509.1c.
        // Also validates that the attacker is a declared attacker.
        let attacker_target = state
            .combat
            .as_ref()
            .and_then(|c| c.attackers.get(attacker_id).cloned());
        match attacker_target {
            None => {
                return Err(GameStateError::InvalidCommand(format!(
                    "Object {:?} is not a declared attacker",
                    attacker_id
                )));
            }
            Some(AttackTarget::Player(pid)) if pid == player => {
                // Valid: this attacker is targeting the declaring player directly.
            }
            Some(AttackTarget::Planeswalker(pw_id)) => {
                // Valid only if the planeswalker is controlled by the declaring player.
                let pw_controller = state.objects.get(&pw_id).map(|o| o.controller);
                if pw_controller != Some(player) {
                    return Err(GameStateError::CrossPlayerBlock {
                        blocker: *blocker_id,
                        attacker: *attacker_id,
                    });
                }
            }
            Some(_) => {
                return Err(GameStateError::CrossPlayerBlock {
                    blocker: *blocker_id,
                    attacker: *attacker_id,
                });
            }
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

        // CR 509.1 / KeywordAbility::CantBeBlocked: a creature with this keyword
        // cannot be blocked at all. Applied by Rogue's Passage activated ability.
        if attacker_chars
            .keywords
            .contains(&KeywordAbility::CantBeBlocked)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot be blocked (CantBeBlocked keyword)",
                attacker_id
            )));
        }

        // CR 702.13b: A creature with intimidate can't be blocked except by artifact creatures
        // and/or creatures that share a color with it.
        if attacker_chars
            .keywords
            .contains(&KeywordAbility::Intimidate)
        {
            let blocker_is_artifact_creature =
                blocker_chars.card_types.contains(&CardType::Artifact)
                    && blocker_chars.card_types.contains(&CardType::Creature);
            let shares_a_color = attacker_chars
                .colors
                .iter()
                .any(|c| blocker_chars.colors.contains(c));
            if !blocker_is_artifact_creature && !shares_a_color {
                return Err(GameStateError::InvalidCommand(format!(
                    "Object {:?} cannot block {:?} (attacker has intimidate; \
                     blocker is neither an artifact creature nor shares a color)",
                    blocker_id, attacker_id
                )));
            }
        }

        // CR 702.36b: A creature with fear can't be blocked except by artifact creatures
        // and/or black creatures.
        if attacker_chars.keywords.contains(&KeywordAbility::Fear) {
            let blocker_is_artifact_creature =
                blocker_chars.card_types.contains(&CardType::Artifact)
                    && blocker_chars.card_types.contains(&CardType::Creature);
            let blocker_is_black = blocker_chars.colors.contains(&Color::Black);
            if !blocker_is_artifact_creature && !blocker_is_black {
                return Err(GameStateError::InvalidCommand(format!(
                    "Object {:?} cannot block {:?} (attacker has fear; \
                     blocker is neither an artifact creature nor black)",
                    blocker_id, attacker_id
                )));
            }
        }

        // CR 702.28b: Shadow is a bidirectional evasion ability.
        // A creature with shadow can't be blocked by creatures without shadow,
        // and a creature without shadow can't be blocked by creatures with shadow.
        let attacker_has_shadow = attacker_chars.keywords.contains(&KeywordAbility::Shadow);
        let blocker_has_shadow = blocker_chars.keywords.contains(&KeywordAbility::Shadow);
        if attacker_has_shadow != blocker_has_shadow {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (shadow mismatch: attacker shadow={}, blocker shadow={})",
                blocker_id, attacker_id, attacker_has_shadow, blocker_has_shadow
            )));
        }

        // CR 702.31b: Horsemanship is a unidirectional evasion ability.
        // A creature with horsemanship can't be blocked by creatures without horsemanship.
        // Unlike Shadow, a creature with horsemanship CAN block creatures without horsemanship.
        if attacker_chars
            .keywords
            .contains(&KeywordAbility::Horsemanship)
            && !blocker_chars
                .keywords
                .contains(&KeywordAbility::Horsemanship)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (attacker has horsemanship; \
                 blocker does not have horsemanship)",
                blocker_id, attacker_id
            )));
        }

        // CR 702.118b: Skulk -- a creature with skulk can't be blocked by creatures
        // with greater power. Unlike Shadow, this is one-directional: it only restricts
        // what can block the skulk creature, not what the skulk creature can block.
        // Equal power IS allowed to block (strictly greater than, not greater-or-equal).
        if attacker_chars.keywords.contains(&KeywordAbility::Skulk) {
            let attacker_power = attacker_chars.power.unwrap_or(0);
            let blocker_power = blocker_chars.power.unwrap_or(0);
            if blocker_power > attacker_power {
                return Err(GameStateError::InvalidCommand(format!(
                    "Object {:?} cannot block {:?} (attacker has skulk with power {}; \
                     blocker has greater power {})",
                    blocker_id, attacker_id, attacker_power, blocker_power
                )));
            }
        }

        // CR 702.16f: protection from blocking. A creature with protection from a quality
        // cannot be blocked by creatures that match that quality. The blocker is the source.
        if !super::protection::can_block(&attacker_chars.keywords, &blocker_chars) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (attacker has protection from the blocker)",
                blocker_id, attacker_id
            )));
        }

        // CR 702.14c: A creature with landwalk can't be blocked as long as the defending
        // player controls at least one land with the specified type. Uses
        // `calculate_characteristics` to get post-layer subtypes (handles Blood Moon, etc.).
        for kw in attacker_chars.keywords.iter() {
            if let KeywordAbility::Landwalk(lw_type) = kw {
                let defender_has_matching_land = state.objects.values().any(|obj| {
                    obj.zone == ZoneId::Battlefield && obj.controller == player && {
                        let chars = calculate_characteristics(state, obj.id).unwrap_or_default();
                        chars.card_types.contains(&CardType::Land)
                            && match lw_type {
                                LandwalkType::BasicType(st) => chars.subtypes.contains(st),
                                LandwalkType::Nonbasic => {
                                    !chars.supertypes.contains(&SuperType::Basic)
                                }
                            }
                    }
                });
                if defender_has_matching_land {
                    return Err(GameStateError::InvalidCommand(format!(
                        "Object {:?} cannot block {:?} (attacker has {:?} landwalk; \
                         defending player controls a matching land)",
                        blocker_id, attacker_id, lw_type
                    )));
                }
            }
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

    // CR 702.39a / CR 509.1c: Provoke forced-block requirements.
    //
    // Each provoked creature must block its provoking attacker if able.
    // "If able" means the creature is on the battlefield, untapped, is a creature,
    // can legally block the attacker (no evasion restrictions prevent it), and is
    // controlled by the declaring player. If ALL conditions are met and the creature
    // is NOT in the blocker list blocking its provoking attacker, the declaration is
    // illegal (CR 509.1c -- must maximize obeyed requirements without violating restrictions).
    {
        // Collect forced-block entries for this player (immutable borrow scope).
        let forced: Vec<(ObjectId, ObjectId)> = state
            .combat
            .as_ref()
            .map(|c| c.forced_blocks.iter().map(|(&k, &v)| (k, v)).collect())
            .unwrap_or_default();

        for (provoked_id, must_block_attacker) in forced {
            // Only check if the provoked creature is controlled by this declaring player.
            let provoked_obj = match state.objects.get(&provoked_id) {
                Some(o) if o.controller == player && o.zone == ZoneId::Battlefield => o,
                _ => continue, // Not this player's creature or not on battlefield
            };

            // Check if the creature is tapped (can't block if tapped).
            if provoked_obj.status.tapped {
                continue;
            }

            // Check if the attacker is still a declared attacker.
            let attacker_still_active = state
                .combat
                .as_ref()
                .map(|c| c.attackers.contains_key(&must_block_attacker))
                .unwrap_or(false);
            if !attacker_still_active {
                continue;
            }

            // Compute characteristics for both the provoked creature and the attacker.
            // If either is missing characteristics (no longer a creature, etc.), skip.
            let provoked_chars = match calculate_characteristics(state, provoked_id) {
                Some(c) if c.card_types.contains(&CardType::Creature) => c,
                _ => continue, // No longer a creature -- requirement impossible
            };
            let attacker_chars = match calculate_characteristics(state, must_block_attacker) {
                Some(c) => c,
                None => continue, // Attacker gone -- skip
            };

            // CR 509.1b / CR 702.9a: Flying evasion check.
            let attacker_has_flying = attacker_chars.keywords.contains(&KeywordAbility::Flying);
            let blocker_has_flying = provoked_chars.keywords.contains(&KeywordAbility::Flying);
            let blocker_has_reach = provoked_chars.keywords.contains(&KeywordAbility::Reach);
            if attacker_has_flying && !blocker_has_flying && !blocker_has_reach {
                continue; // Requirement impossible -- skip
            }

            // CR 702.147a: Decayed creatures can't block.
            if provoked_chars.keywords.contains(&KeywordAbility::Decayed) {
                continue; // Requirement impossible -- skip
            }

            // CR 509.1: CantBeBlocked keyword -- creature can't be blocked at all.
            if attacker_chars
                .keywords
                .contains(&KeywordAbility::CantBeBlocked)
            {
                continue; // Requirement impossible -- skip
            }

            // CR 702.13b: Intimidate -- can only be blocked by artifact creatures
            // and/or creatures sharing a color.
            if attacker_chars
                .keywords
                .contains(&KeywordAbility::Intimidate)
            {
                let blocker_is_artifact_creature =
                    provoked_chars.card_types.contains(&CardType::Artifact)
                        && provoked_chars.card_types.contains(&CardType::Creature);
                let shares_a_color = attacker_chars
                    .colors
                    .iter()
                    .any(|c| provoked_chars.colors.contains(c));
                if !blocker_is_artifact_creature && !shares_a_color {
                    continue; // Requirement impossible -- skip
                }
            }

            // CR 702.36b: Fear -- can only be blocked by artifact creatures and/or black.
            if attacker_chars.keywords.contains(&KeywordAbility::Fear) {
                let blocker_is_artifact_creature =
                    provoked_chars.card_types.contains(&CardType::Artifact)
                        && provoked_chars.card_types.contains(&CardType::Creature);
                let blocker_is_black = provoked_chars.colors.contains(&Color::Black);
                if !blocker_is_artifact_creature && !blocker_is_black {
                    continue; // Requirement impossible -- skip
                }
            }

            // CR 702.28b: Shadow mismatch.
            let attacker_has_shadow = attacker_chars.keywords.contains(&KeywordAbility::Shadow);
            let blocker_has_shadow = provoked_chars.keywords.contains(&KeywordAbility::Shadow);
            if attacker_has_shadow != blocker_has_shadow {
                continue; // Requirement impossible -- skip
            }

            // CR 702.31b: Horsemanship.
            if attacker_chars
                .keywords
                .contains(&KeywordAbility::Horsemanship)
                && !provoked_chars
                    .keywords
                    .contains(&KeywordAbility::Horsemanship)
            {
                continue; // Requirement impossible -- skip
            }

            // CR 702.118b: Skulk -- can't be blocked by creatures with greater power.
            if attacker_chars.keywords.contains(&KeywordAbility::Skulk) {
                let attacker_power = attacker_chars.power.unwrap_or(0);
                let blocker_power = provoked_chars.power.unwrap_or(0);
                if blocker_power > attacker_power {
                    continue; // Requirement impossible -- skip
                }
            }

            // CR 702.16f: Protection prevents blocking.
            if !super::protection::can_block(&attacker_chars.keywords, &provoked_chars) {
                continue; // Requirement impossible -- skip
            }

            // CR 702.14c: Landwalk -- can't be blocked if defender controls matching land.
            let mut landwalk_blocks = false;
            for kw in attacker_chars.keywords.iter() {
                if let KeywordAbility::Landwalk(lw_type) = kw {
                    let defender_has_matching_land = state.objects.values().any(|obj| {
                        obj.zone == ZoneId::Battlefield && obj.controller == player && {
                            let chars =
                                calculate_characteristics(state, obj.id).unwrap_or_default();
                            chars.card_types.contains(&CardType::Land)
                                && match lw_type {
                                    LandwalkType::BasicType(st) => chars.subtypes.contains(st),
                                    LandwalkType::Nonbasic => {
                                        !chars.supertypes.contains(&SuperType::Basic)
                                    }
                                }
                        }
                    });
                    if defender_has_matching_land {
                        landwalk_blocks = true;
                        break;
                    }
                }
            }
            if landwalk_blocks {
                continue; // Requirement impossible -- skip
            }

            // The provoked creature CAN block the provoking attacker.
            // Check if it IS blocking it in this declaration.
            let is_blocking_required_attacker = blockers
                .iter()
                .any(|(b, a)| *b == provoked_id && *a == must_block_attacker);
            if !is_blocking_required_attacker {
                return Err(GameStateError::InvalidCommand(format!(
                    "Creature {:?} must block {:?} (provoke requirement, CR 702.39a / CR 509.1c)",
                    provoked_id, must_block_attacker
                )));
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

    // Check and queue triggers from blocker declaration (e.g., SelfBlocks, Flanking).
    let new_triggers = abilities::check_triggers(state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }

    // CR 603.3 / CR 509.3f: Flush any pending triggers (e.g., Flanking CR 702.25a,
    // SelfBlocks) so they appear on the stack before priority is granted.
    // This ensures triggered abilities from blocker declaration (like Flanking's -1/-1)
    // resolve BEFORE combat damage is dealt, which is correct per MTG rules.
    let trigger_events = abilities::flush_pending_triggers(state);
    events.extend(trigger_events);

    // Grant priority to the active player so players can respond to triggers
    // (including Flanking triggers) before combat damage is dealt.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(state.turn.active_player);
    events.push(GameEvent::PriorityGiven {
        player: state.turn.active_player,
    });

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

    // MR-M6-03: the order must include every blocker assigned to this attacker
    // (CR 509.2 — the attacker's controller orders ALL blockers, not a subset).
    if order.len() != blocking_this.len() {
        return Err(GameStateError::IncompleteBlockerOrder {
            provided: order.len(),
            required: blocking_this.len(),
        });
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
    // Pre-extract per-assignment: (source_deathtouch, source_lifelink, source_wither,
    // source_infect, source_toxic_total, source_controller, commander_info)
    // commander_info = Some((attacking_player_id, card_id)) if source is a commander.
    type DamageAppInfo = (
        bool,
        bool,
        bool,
        bool,
        u32,
        PlayerId,
        Option<(PlayerId, CardId)>,
    );
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
            // CR 702.80a: Damage dealt to a creature by a source with wither places
            // -1/-1 counters instead of marking damage.
            let source_wither = chars
                .as_ref()
                .map(|c| c.keywords.contains(&KeywordAbility::Wither))
                .unwrap_or(false);
            // CR 702.90a: Damage dealt by a source with infect to a creature places
            // -1/-1 counters; to a player gives poison counters (CR 120.3b, CR 120.3d).
            let source_infect = chars
                .as_ref()
                .map(|c| c.keywords.contains(&KeywordAbility::Infect))
                .unwrap_or(false);
            // CR 702.164b: Total toxic value is the sum of all Toxic N values on the source.
            // Multiple instances are cumulative (not redundant like Infect).
            // Uses layer-resolved characteristics so ability-removal (Humility, Dress Down)
            // and ability-granting effects are correctly respected (CR 613).
            // NOTE: If two identical Toxic(N) values exist on the same object, OrdSet
            // deduplication means only one is counted. This is a known limitation (LOW);
            // no real-world card combination currently produces this in the engine.
            let source_toxic_total: u32 = chars
                .as_ref()
                .map(|c| {
                    c.keywords
                        .iter()
                        .filter_map(|kw| match kw {
                            KeywordAbility::Toxic(n) => Some(*n),
                            _ => None,
                        })
                        .sum()
                })
                .unwrap_or(0);
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
                source_wither,
                source_infect,
                source_toxic_total,
                source_controller,
                commander_info,
            )
        })
        .collect();

    // --- CR 702.16e + CR 615: Apply protection then dynamic prevention ---
    // apply_damage_prevention checks protection (static) first, then dynamic shields.
    let mut prevention_events: Vec<GameEvent> = Vec::new();
    let final_amounts: Vec<u32> = assignments
        .iter()
        .map(|a| {
            let (final_dmg, pevts) = crate::rules::replacement::apply_damage_prevention(
                state, a.source, &a.target, a.amount,
            );
            prevention_events.extend(pevts);
            final_dmg
        })
        .collect();

    // Build the post-prevention assignment list for the CombatDamageDealt event.
    let final_assignments: Vec<CombatDamageAssignment> = assignments
        .iter()
        .zip(final_amounts.iter())
        .map(|(a, &amt)| CombatDamageAssignment {
            source: a.source,
            target: a.target.clone(),
            amount: amt,
        })
        .collect();

    // --- Apply damage and collect lifelink gains ---
    // lifelink_gains: controller → total damage dealt by their lifelink sources this step.
    let mut lifelink_gains: im::OrdMap<PlayerId, u32> = im::OrdMap::new();
    // Collect wither/infect counter events during the damage application loop.
    // These will be added to the event stream after the loop.
    let mut wither_counter_events: Vec<GameEvent> = Vec::new();
    // Collect PoisonCountersGiven events for infect damage to players.
    let mut poison_events: Vec<GameEvent> = Vec::new();

    for (
        (
            assignment,
            (
                source_deathtouch,
                source_lifelink,
                source_wither,
                source_infect,
                source_toxic_total,
                source_controller,
                commander_info,
            ),
        ),
        &final_dmg,
    ) in assignments
        .iter()
        .zip(app_info.iter())
        .zip(final_amounts.iter())
    {
        if final_dmg == 0 {
            // All damage prevented for this assignment — skip state mutation.
            continue;
        }
        match &assignment.target {
            CombatDamageTarget::Creature(obj_id) => {
                if let Some(obj) = state.objects.get_mut(obj_id) {
                    if *source_wither || *source_infect {
                        // CR 702.80a / CR 702.90c / CR 120.3d: damage to a creature by a
                        // source with wither and/or infect places -1/-1 counters instead
                        // of marking damage. Multiple instances of wither/infect are
                        // redundant (CR 702.80d / CR 702.90f) — this fires at most once.
                        let cur = obj
                            .counters
                            .get(&CounterType::MinusOneMinusOne)
                            .copied()
                            .unwrap_or(0);
                        obj.counters
                            .insert(CounterType::MinusOneMinusOne, cur + final_dmg);
                        wither_counter_events.push(GameEvent::CounterAdded {
                            object_id: *obj_id,
                            counter: CounterType::MinusOneMinusOne,
                            count: final_dmg,
                        });
                    } else {
                        // CR 120.3e: normal damage marking.
                        obj.damage_marked += final_dmg;
                    }
                    if *source_deathtouch {
                        obj.deathtouch_damage = true;
                    }
                }
            }
            CombatDamageTarget::Player(player_id) => {
                if *source_infect {
                    // CR 702.90b / CR 120.3b: infect damage to a player gives poison
                    // counters instead of causing life loss.
                    if let Some(player) = state.players.get_mut(player_id) {
                        player.poison_counters += final_dmg;
                    }
                    poison_events.push(GameEvent::PoisonCountersGiven {
                        player: *player_id,
                        amount: final_dmg,
                        source: assignment.source,
                    });
                } else {
                    // CR 120.3a: normal damage causes life loss.
                    if let Some(player) = state.players.get_mut(player_id) {
                        player.life_total -= final_dmg as i32;
                    }
                }
                // CR 702.164c / CR 120.3g: Toxic -- give poison counters equal to the
                // source's total toxic value, in addition to the damage's other results.
                // Applies regardless of Infect (both can coexist: Infect adds damage-amount
                // poison counters, Toxic adds toxic-value poison counters independently).
                // The final_dmg == 0 guard above ensures we only reach here when damage
                // was actually dealt (CR 120.3g: "combat damage dealt to a player").
                if *source_toxic_total > 0 {
                    if let Some(player) = state.players.get_mut(player_id) {
                        player.poison_counters += *source_toxic_total;
                    }
                    poison_events.push(GameEvent::PoisonCountersGiven {
                        player: *player_id,
                        amount: *source_toxic_total,
                        source: assignment.source,
                    });
                }
                // Track commander damage (CR 903.10a).
                // Commander damage counts COMBAT damage dealt, not life lost — infect
                // damage still counts toward commander damage totals (CR 903.10a).
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
                        let new_val = current + final_dmg;

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
                    obj.damage_marked += final_dmg;
                }
            }
        }

        // CR 702.15a: Lifelink — source's controller gains life equal to damage dealt.
        if *source_lifelink {
            let entry = lifelink_gains.entry(*source_controller).or_insert(0);
            *entry += final_dmg;
        }
    }

    // Prevention events fire before CombatDamageDealt (they modify the event as it happens).
    let mut events = prevention_events;
    // CounterAdded events from wither/infect precede the CombatDamageDealt summary event.
    events.extend(wither_counter_events);
    // PoisonCountersGiven events from infect player damage precede CombatDamageDealt.
    events.extend(poison_events);
    events.push(GameEvent::CombatDamageDealt {
        assignments: final_assignments,
    });

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
