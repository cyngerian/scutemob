//! The Ring Tempts You mechanic tests (CR 701.54).
//!
//! "The Ring Tempts You" is a keyword action (CR 701.54a): each time it fires,
//! the controller advances their ring level (1-4) and chooses a creature as their
//! ring-bearer. The ring-bearer gains Legendary (level 1+) and can't be blocked
//! by creatures with greater power (level 1+). Higher ring levels grant additional
//! triggered abilities.
//!
//! Tests cover:
//! - CR 701.54a: Basic ring temptation — level advances, ring-bearer chosen (lowest ObjectId).
//! - CR 701.54c: Level progression capped at 4.
//! - CR 701.54a: No creatures → level still advances, no ring-bearer chosen.
//! - Ruling 2023-06-16: Re-choosing same creature still emits RingBearerChosen.
//! - CR 701.54c level 1: Ring-bearer can't be blocked by creatures with greater power.
//! - CR 701.54c level 1: Equal power CAN block.
//! - CR 701.54c level 1: Ring-bearer gets Legendary supertype via layer system.
//! - CR 701.54a: Control change SBA clears ring-bearer designation.
//! - CR 400.7: Ring-bearer leaving battlefield clears ring_bearer_id.
//! - CR 701.54c level 2: Ring-bearer attacks → loot trigger fires.
//! - CR 701.54c level 4: Ring-bearer deals combat damage → each opponent loses 3 life.
//! - CR 701.54: Multiplayer — each player has independent ring level and ring-bearer.
//! - CR 701.54d: "Whenever the Ring tempts you" triggered ability fires on RingTempted event.

use mtg_engine::{
    calculate_characteristics, handle_ring_tempts_you, process_command, AbilityDefinition,
    AttackTarget, CardDefinition, CardId, CardRegistry, CardType, CombatState, Command,
    Designations, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, ManaCost, ObjectId,
    ObjectSpec, PlayerId, PlayerTarget, Step, SuperType, TriggerCondition, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Directly set the RING_BEARER designation on a creature to simulate ring-bearer assignment.
fn set_ring_bearer(state: &mut GameState, id: ObjectId, player: PlayerId) {
    if let Some(obj) = state.objects.get_mut(&id) {
        obj.designations.insert(Designations::RING_BEARER);
    }
    if let Some(ps) = state.players.get_mut(&player) {
        ps.ring_bearer_id = Some(id);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 701.54a: When the Ring tempts a player, ring level advances to 1 and
/// the creature with the lowest ObjectId is chosen as ring-bearer.
///
/// CR 701.54b: Ring-bearer designation is applied to the chosen creature.
#[test]
fn test_ring_tempts_you_basic_level_1() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Test Creature", 2, 2))
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Test Creature");

    let events = handle_ring_tempts_you(&mut state, p1).expect("ring tempts you should succeed");

    // Ring level should advance to 1.
    let ps = state.players.get(&p1).unwrap();
    assert_eq!(
        ps.ring_level, 1,
        "ring level should be 1 after first temptation"
    );
    assert_eq!(
        ps.ring_bearer_id,
        Some(creature_id),
        "ring_bearer_id should point to the creature"
    );

    // RING_BEARER designation should be set.
    let obj = state.objects.get(&creature_id).unwrap();
    assert!(
        obj.designations.contains(Designations::RING_BEARER),
        "creature should have RING_BEARER designation"
    );

    // RingTempted event should be emitted.
    let ring_tempted = events
        .iter()
        .any(|e| matches!(e, GameEvent::RingTempted { player, new_level: 1 } if *player == p1));
    assert!(
        ring_tempted,
        "RingTempted event with new_level=1 should be emitted"
    );

    // RingBearerChosen event should be emitted.
    let bearer_chosen = events.iter().any(|e| {
        matches!(e, GameEvent::RingBearerChosen { player, creature } if *player == p1 && *creature == creature_id)
    });
    assert!(
        bearer_chosen,
        "RingBearerChosen event should be emitted for the creature"
    );
}

/// CR 701.54c: Ring level advances from 1 to 4 over 4 temptations, then is
/// capped at 4 on additional temptations.
#[test]
fn test_ring_tempts_you_level_progression_capped_at_4() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Bearer", 1, 1))
        .build()
        .unwrap();

    // Tempt 4 times — levels 1, 2, 3, 4.
    for expected_level in 1u8..=4 {
        handle_ring_tempts_you(&mut state, p1).expect("ring tempts you should succeed");
        let ps = state.players.get(&p1).unwrap();
        assert_eq!(
            ps.ring_level, expected_level,
            "ring level should be {}",
            expected_level
        );
    }

    // 5th temptation — level stays at 4.
    let events = handle_ring_tempts_you(&mut state, p1).expect("ring tempts you should succeed");
    let ps = state.players.get(&p1).unwrap();
    assert_eq!(
        ps.ring_level, 4,
        "ring level should be capped at 4 on 5th temptation"
    );

    // RingTempted should still fire with new_level = 4.
    let ring_tempted = events
        .iter()
        .any(|e| matches!(e, GameEvent::RingTempted { player, new_level: 4 } if *player == p1));
    assert!(
        ring_tempted,
        "RingTempted should still fire at capped level 4"
    );
}

/// CR 701.54a + ruling 2023-06-16: Ring tempts a player who controls no creatures.
/// Ring level still advances but no ring-bearer is chosen. ring_bearer_id stays None.
/// "Whenever the Ring tempts you" triggers still fire (CR 701.54d).
#[test]
fn test_ring_tempts_you_no_creatures() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new().add_player(p1).build().unwrap();

    let events = handle_ring_tempts_you(&mut state, p1).expect("ring tempts you should succeed");

    // Ring level should advance.
    let ps = state.players.get(&p1).unwrap();
    assert_eq!(
        ps.ring_level, 1,
        "ring level should advance even with no creatures"
    );
    assert_eq!(
        ps.ring_bearer_id, None,
        "ring_bearer_id should be None with no creatures"
    );

    // RingTempted event should fire.
    let ring_tempted = events
        .iter()
        .any(|e| matches!(e, GameEvent::RingTempted { player, .. } if *player == p1));
    assert!(
        ring_tempted,
        "RingTempted should fire even with no creatures"
    );

    // RingBearerChosen should NOT fire.
    let bearer_chosen = events
        .iter()
        .any(|e| matches!(e, GameEvent::RingBearerChosen { player, .. } if *player == p1));
    assert!(
        !bearer_chosen,
        "RingBearerChosen should NOT fire with no creatures"
    );
}

/// Ruling 2023-06-16: Choosing a creature that is already your ring-bearer still
/// counts as choosing — RingBearerChosen event is still emitted.
#[test]
fn test_ring_tempts_you_rechoose_same_creature_emits_event() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Bearer", 2, 2))
        .build()
        .unwrap();

    // First temptation — assigns ring-bearer.
    handle_ring_tempts_you(&mut state, p1).expect("first temptation should succeed");

    let creature_id = find_object(&state, "Bearer");
    let ps = state.players.get(&p1).unwrap();
    assert_eq!(ps.ring_bearer_id, Some(creature_id), "ring-bearer assigned");

    // Second temptation — same creature chosen again.
    let events = handle_ring_tempts_you(&mut state, p1).expect("second temptation should succeed");

    // RingBearerChosen still fires even though it's the same creature.
    let bearer_chosen = events.iter().any(|e| {
        matches!(e, GameEvent::RingBearerChosen { player, creature } if *player == p1 && *creature == creature_id)
    });
    assert!(
        bearer_chosen,
        "RingBearerChosen should fire even when re-choosing the same creature (ruling 2023-06-16)"
    );
}

/// CR 701.54c level 1: Ring-bearer gains Legendary supertype via the layer system.
/// The RING_BEARER designation causes the layer system to grant Legendary.
#[test]
fn test_ring_bearer_is_legendary() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Simple Creature", 2, 2))
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Simple Creature");

    // Before ring-bearer: not Legendary.
    let chars_before = calculate_characteristics(&state, creature_id).unwrap();
    assert!(
        !chars_before.supertypes.contains(&SuperType::Legendary),
        "creature should NOT be Legendary before becoming ring-bearer"
    );

    // Assign ring-bearer (directly via handle_ring_tempts_you).
    handle_ring_tempts_you(&mut state, p1).expect("ring tempts you should succeed");

    // After ring-bearer: Legendary.
    let chars_after = calculate_characteristics(&state, creature_id).unwrap();
    assert!(
        chars_after.supertypes.contains(&SuperType::Legendary),
        "ring-bearer should be Legendary (CR 701.54c level 1)"
    );
}

/// CR 701.54c level 1: Ring-bearer can't be blocked by creatures with strictly greater power.
/// A 2/2 ring-bearer cannot be blocked by a 3/3.
#[test]
fn test_ring_bearer_blocking_restriction_greater_power() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Ring Bearer", 2, 2))
        .object(ObjectSpec::creature(p2, "Big Blocker", 3, 3))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Ring Bearer");
    let blocker_id = find_object(&state, "Big Blocker");

    // Set ring level and ring-bearer directly.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.ring_level = 1;
    }
    set_ring_bearer(&mut state, attacker_id, p1);

    // Set up combat with ring-bearer attacking.
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "A creature with greater power should not be able to block a ring-bearer (CR 701.54c)"
    );
}

/// CR 701.54c level 1: Ring-bearer CAN be blocked by creatures with equal power.
/// A 2/2 ring-bearer CAN be blocked by a 2/2 (strictly greater, not >=).
#[test]
fn test_ring_bearer_blocking_equal_power_allowed() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Ring Bearer", 2, 2))
        .object(ObjectSpec::creature(p2, "Equal Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Ring Bearer");
    let blocker_id = find_object(&state, "Equal Blocker");

    if let Some(ps) = state.players.get_mut(&p1) {
        ps.ring_level = 1;
    }
    set_ring_bearer(&mut state, attacker_id, p1);

    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "A creature with equal power should be able to block a ring-bearer (CR 701.54c)"
    );
}

/// CR 701.54c level 1: Ring-bearer CAN be blocked by creatures with lesser power.
/// A 2/2 ring-bearer CAN be blocked by a 1/5.
#[test]
fn test_ring_bearer_blocking_lesser_power_allowed() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Ring Bearer", 2, 2))
        .object(ObjectSpec::creature(p2, "Small Blocker", 1, 5))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Ring Bearer");
    let blocker_id = find_object(&state, "Small Blocker");

    if let Some(ps) = state.players.get_mut(&p1) {
        ps.ring_level = 1;
    }
    set_ring_bearer(&mut state, attacker_id, p1);

    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "A creature with lesser power should be able to block a ring-bearer (CR 701.54c)"
    );
}

/// CR 701.54a: When another player gains control of the ring-bearer (simulated by
/// changing controller), the SBA clears ring_bearer_id and RING_BEARER designation.
/// Tests the check_ring_bearer_sba logic via check_and_apply_sbas.
#[test]
fn test_ring_bearer_control_change_clears_designation() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Stolen Bearer", 2, 2))
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Stolen Bearer");

    // Make p1 the ring-bearer controller.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.ring_level = 1;
    }
    set_ring_bearer(&mut state, creature_id, p1);

    // Verify designation is set.
    assert!(
        state
            .objects
            .get(&creature_id)
            .unwrap()
            .designations
            .contains(Designations::RING_BEARER),
        "RING_BEARER should be set before control change"
    );

    // Simulate control change: change the creature's controller to p2.
    if let Some(obj) = state.objects.get_mut(&creature_id) {
        obj.controller = p2;
    }

    // SBA check should clear the ring-bearer.
    mtg_engine::check_and_apply_sbas(&mut state);

    let ps = state.players.get(&p1).unwrap();
    assert_eq!(
        ps.ring_bearer_id, None,
        "ring_bearer_id should be cleared after control change"
    );

    let obj = state.objects.get(&creature_id).unwrap();
    assert!(
        !obj.designations.contains(Designations::RING_BEARER),
        "RING_BEARER designation should be cleared after control change"
    );
}

/// CR 400.7: When the ring-bearer leaves the battlefield (zone changes), the SBA
/// clears ring_bearer_id and RING_BEARER designation. The new object in the
/// graveyard is a different object (CR 400.7 — different ObjectId).
#[test]
fn test_ring_bearer_leaves_battlefield_clears_designation() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Dying Bearer", 2, 2))
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Dying Bearer");

    if let Some(ps) = state.players.get_mut(&p1) {
        ps.ring_level = 1;
    }
    set_ring_bearer(&mut state, creature_id, p1);

    assert!(
        state
            .objects
            .get(&creature_id)
            .unwrap()
            .designations
            .contains(Designations::RING_BEARER),
        "RING_BEARER should be set before creature leaves"
    );

    // Move the creature to the graveyard to simulate death.
    if let Some(obj) = state.objects.get_mut(&creature_id) {
        obj.zone = ZoneId::Graveyard(p1);
    }

    // SBA check should clear the ring-bearer (not on battlefield).
    mtg_engine::check_and_apply_sbas(&mut state);

    let ps = state.players.get(&p1).unwrap();
    assert_eq!(
        ps.ring_bearer_id, None,
        "ring_bearer_id should be None after creature leaves battlefield"
    );
}

/// CR 701.54: Multiplayer — each player independently tracks their ring level
/// and ring-bearer. Tempting one player does not affect others.
#[test]
fn test_ring_tempts_you_multiplayer_independence() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "P1 Bearer", 1, 1))
        .object(ObjectSpec::creature(p2, "P2 Bearer", 2, 2))
        .build()
        .unwrap();

    // Tempt p1 once.
    handle_ring_tempts_you(&mut state, p1).expect("p1 temptation");
    // Tempt p2 twice.
    handle_ring_tempts_you(&mut state, p2).expect("p2 temptation 1");
    handle_ring_tempts_you(&mut state, p2).expect("p2 temptation 2");

    // p1 should be at level 1.
    assert_eq!(
        state.players.get(&p1).unwrap().ring_level,
        1,
        "p1 ring_level"
    );
    // p2 should be at level 2.
    assert_eq!(
        state.players.get(&p2).unwrap().ring_level,
        2,
        "p2 ring_level"
    );
    // p3 and p4 should be at level 0 (untouched).
    assert_eq!(
        state.players.get(&p3).unwrap().ring_level,
        0,
        "p3 ring_level should be 0"
    );
    assert_eq!(
        state.players.get(&p4).unwrap().ring_level,
        0,
        "p4 ring_level should be 0"
    );

    // p1's ring-bearer should be p1's creature; p2's should be p2's.
    let p1_bearer = find_object(&state, "P1 Bearer");
    let p2_bearer = find_object(&state, "P2 Bearer");
    assert_eq!(
        state.players.get(&p1).unwrap().ring_bearer_id,
        Some(p1_bearer)
    );
    assert_eq!(
        state.players.get(&p2).unwrap().ring_bearer_id,
        Some(p2_bearer)
    );
    assert_eq!(state.players.get(&p3).unwrap().ring_bearer_id, None);
    assert_eq!(state.players.get(&p4).unwrap().ring_bearer_id, None);
}

/// CR 701.54c level 2: When ring_level >= 2 and the ring-bearer attacks,
/// a loot trigger (draw then discard) is queued.
/// This test verifies a RingAbility stack object is placed when ring-bearer attacks.
#[test]
fn test_ring_level_2_loot_trigger_fires_on_attack() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Ring Bearer", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Ring Bearer");

    // Set ring level to 2 and assign ring-bearer.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.ring_level = 2;
    }
    set_ring_bearer(&mut state, attacker_id, p1);

    // Declare attackers — this should queue the ring loot trigger.
    let (new_state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // A RingAbility stack object or AbilityTriggered event should be present.
    let has_ring_trigger = events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { controller, .. } if *controller == p1));
    assert!(
        has_ring_trigger || new_state.stack_objects.iter().any(|so| {
            matches!(so.kind, mtg_engine::StackObjectKind::RingAbility { controller, .. } if controller == p1)
        }),
        "RingAbility trigger should be queued when ring-bearer (level 2+) attacks"
    );
}

/// CR 701.54c level 1: Blocking restriction does not apply if creature is NOT the ring-bearer
/// (designation not set). A normal 2/2 can be blocked by a 3/3.
#[test]
fn test_non_ring_bearer_no_blocking_restriction() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(ObjectSpec::creature(p2, "Big Blocker", 3, 3))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let blocker_id = find_object(&state, "Big Blocker");

    // p1 has ring level 1 but NO ring-bearer set — no restriction applies.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.ring_level = 1;
    }
    // NOTE: deliberately NOT calling set_ring_bearer.

    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "Non-ring-bearer should be blockable by any creature regardless of ring level"
    );
}

// ── Additional tests for review findings ──────────────────────────────────────

fn is_on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
}

/// CR 701.54c level 3: Blocker of ring-bearer is sacrificed at END of combat (not immediately).
///
/// This test verifies two things:
/// 1. The blocker is NOT immediately sacrificed when DeclareBlockers is processed.
/// 2. The `ring_block_sacrifice_at_eoc` flag is set on the specific blocker — not on any
///    other permanent controlled by P2 (e.g., "Innocent Bystander").
///
/// The actual EOC sacrifice mechanism (flag check in `end_combat()`) is identical to the
/// Decayed EOC pattern verified in `tests/decayed.rs`. We verify the flag is set correctly
/// and only on the specific blocker.
#[test]
fn test_ring_level3_sacrifice_at_eoc() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Ring Bearer", 2, 2))
        .object(ObjectSpec::creature(p2, "Blocker", 1, 1))
        .object(ObjectSpec::creature(p2, "Innocent Bystander", 3, 3))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let bearer_id = find_object(&state, "Ring Bearer");
    let blocker_id = find_object(&state, "Blocker");
    let bystander_id = find_object(&state, "Innocent Bystander");

    // Set ring level 3 and assign ring-bearer.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.ring_level = 3;
    }
    set_ring_bearer(&mut state, bearer_id, p1);

    // Set up combat with ring-bearer attacking.
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(bearer_id, AttackTarget::Player(p2));
        cs
    });

    // P2 blocks with the 1/1 creature.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, bearer_id)],
        },
    )
    .expect("DeclareBlockers should succeed");

    // After declare-blockers, blocker should STILL be on the battlefield (NOT yet sacrificed).
    // The bug in the old implementation was that the SacrificePermanents effect resolved
    // immediately during the trigger resolution, before end of combat.
    assert!(
        is_on_battlefield(&state, "Blocker"),
        "CR 701.54c: blocker should NOT be sacrificed immediately at declare-blockers time"
    );

    // The ring_block_sacrifice_at_eoc flag should be set on the SPECIFIC blocker.
    let blocker_obj = state
        .objects
        .get(&blocker_id)
        .expect("Blocker should still exist on battlefield");
    assert!(
        blocker_obj.ring_block_sacrifice_at_eoc,
        "CR 701.54c: ring_block_sacrifice_at_eoc flag should be set on the specific blocker"
    );

    // The innocent bystander's flag should NOT be set — only the blocker is tagged.
    let bystander_obj = state
        .objects
        .get(&bystander_id)
        .expect("Innocent Bystander should still exist");
    assert!(
        !bystander_obj.ring_block_sacrifice_at_eoc,
        "CR 701.54c: ring_block_sacrifice_at_eoc should NOT be set on other permanents"
    );

    // The ring-bearer (attacker) flag should also NOT be set.
    let bearer_obj = state
        .objects
        .get(&bearer_id)
        .expect("Ring Bearer should still exist");
    assert!(
        !bearer_obj.ring_block_sacrifice_at_eoc,
        "CR 701.54c: ring_block_sacrifice_at_eoc should NOT be set on the attacker (ring-bearer)"
    );
}

/// CR 701.54c level 4: When ring-bearer deals combat damage to a player,
/// a RingCombatDamage trigger fires (each opponent loses 3 life).
///
/// Setup: P1 has ring level 4, ring-bearer attacks P2 who doesn't block. After
/// CombatDamage is dealt, a RingAbility trigger should be on the stack or queued
/// (controlled by P1, the ring-bearer's controller).
///
/// We verify the trigger is queued by checking that a RingCombatDamage PendingTrigger
/// is generated from the CombatDamageDealt event via check_triggers, which is
/// mechanically the same as verifying the trigger fires in practice.
#[test]
fn test_ring_level4_combat_damage_trigger_fires() {
    let p1 = p(1);
    let p2 = p(2);

    use mtg_engine::rules::events::{CombatDamageAssignment, CombatDamageTarget};
    use mtg_engine::state::stubs::PendingTriggerKind;

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Ring Bearer", 3, 3))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let bearer_id = find_object(&state, "Ring Bearer");

    // Set ring level 4 and assign ring-bearer.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.ring_level = 4;
    }
    set_ring_bearer(&mut state, bearer_id, p1);

    // Set up combat state with ring-bearer attacking P2 (no blockers).
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(bearer_id, AttackTarget::Player(p2));
        cs
    });

    // Simulate the CombatDamageDealt event — this is what triggers the ring level 4 path.
    let damage_event = GameEvent::CombatDamageDealt {
        assignments: vec![CombatDamageAssignment {
            source: bearer_id,
            target: CombatDamageTarget::Player(p2),
            amount: 3,
        }],
    };

    // check_triggers should return a RingCombatDamage PendingTrigger for level >= 4.
    let pending = mtg_engine::rules::abilities::check_triggers(&state, &[damage_event]);

    let ring_trigger = pending
        .iter()
        .find(|t| matches!(t.kind, PendingTriggerKind::RingCombatDamage));
    assert!(
        ring_trigger.is_some(),
        "CR 701.54c level 4: RingCombatDamage PendingTrigger should be generated when ring-bearer deals combat damage to a player"
    );

    let trigger = ring_trigger.unwrap();
    assert_eq!(
        trigger.controller, p1,
        "CR 701.54c level 4: trigger should be controlled by the ring-bearer's controller"
    );
    assert_eq!(
        trigger.source, bearer_id,
        "CR 701.54c level 4: trigger source should be the ring-bearer"
    );
}

/// CR 701.54d: "Whenever the Ring tempts you" triggered ability fires on RingTempted event.
///
/// Setup: P1 controls a permanent with a WheneverRingTemptsYou triggered ability
/// (draw a card). When the ring tempts P1, verify the trigger is queued
/// (a PendingTrigger with Normal kind is added to pending_triggers).
///
/// Note: `handle_ring_tempts_you` emits events but does not process them through
/// `check_triggers`. We call `check_triggers` manually on the returned events to
/// verify that a PendingTrigger is generated for the WheneverRingTemptsYou ability.
#[test]
fn test_whenever_ring_tempts_you_trigger() {
    let p1 = p(1);

    // Build a card definition with WheneverRingTemptsYou triggered ability.
    let ring_trigger_def = CardDefinition {
        card_id: CardId("ring-trigger-test".to_string()),
        name: "Ring Watcher".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Whenever the Ring tempts you, draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverRingTemptsYou,
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],

            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![ring_trigger_def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(registry)
        .object(
            ObjectSpec::creature(p1, "Ring Watcher", 1, 1)
                .with_card_id(CardId("ring-trigger-test".to_string())),
        )
        .build()
        .unwrap();

    // Tempt P1 with the ring — this emits RingTempted and RingBearerChosen events.
    let events =
        handle_ring_tempts_you(&mut state, p1).expect("handle_ring_tempts_you should succeed");

    // Verify the RingTempted event fired.
    let ring_tempted = events
        .iter()
        .any(|e| matches!(e, GameEvent::RingTempted { player, .. } if *player == p1));
    assert!(ring_tempted, "CR 701.54d: RingTempted event should fire");

    // Run check_triggers on the emitted events to verify the WheneverRingTemptsYou
    // trigger would be queued. handle_ring_tempts_you returns events but does not
    // internally call check_triggers — that's done by the process_command caller.
    // We call it directly here to verify the dispatch is wired correctly.
    let pending = mtg_engine::rules::abilities::check_triggers(&state, &events);

    let trigger_queued = pending.iter().any(|t| {
        t.controller == p1 && matches!(t.kind, mtg_engine::state::stubs::PendingTriggerKind::Normal)
    });
    assert!(
        trigger_queued,
        "CR 701.54d: a Normal PendingTrigger should be queued for WheneverRingTemptsYou ability (found {} pending triggers)",
        pending.len()
    );
}
