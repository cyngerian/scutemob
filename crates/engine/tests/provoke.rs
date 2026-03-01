//! Provoke keyword ability tests (CR 702.39).
//!
//! Provoke is a triggered ability: "Whenever this creature attacks, you may
//! have target creature defending player controls untap and block this creature
//! this combat if able."
//!
//! Key rules verified:
//! - Trigger fires on attacker declared, resolves before DeclareBlockers (CR 702.39a).
//! - Provoked creature is untapped (CR 702.39a: "untap that creature").
//! - Provoked creature must block the provoking attacker if able (CR 509.1c).
//! - Forced-block requirement is waived when the creature can't legally block (CR 509.1b/c).
//! - Multiple instances trigger separately (CR 702.39b).
//! - No trigger when no valid target exists (CR 603.3d).
//! - Multiplayer: only targets creatures the defending player controls (CR 508.5a).

use mtg_engine::{
    process_command, AttackTarget, CardRegistry, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

// ── Test 1: Basic provoke -- untaps tapped creature and forces block ──────────

#[test]
/// CR 702.39a — Basic provoke: P1 attacks with a provoke creature targeting P2's
/// tapped creature. The trigger fires, resolves before DeclareBlockers, untaps
/// P2's creature and forces it to block. P2 satisfies the requirement by blocking.
fn test_702_39a_provoke_basic_untap_and_forced_block() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker_spec = ObjectSpec::creature(p1, "Provoke Attacker", 2, 2)
        .with_keyword(KeywordAbility::Provoke)
        .in_zone(ZoneId::Battlefield);

    // P2 has a tapped creature (should be untapped by provoke).
    let defender_spec = ObjectSpec::creature(p2, "Defender", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .tapped();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker_spec)
        .object(defender_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Provoke Attacker");
    let defender_id = find_object(&state, "Defender");

    // Verify defender starts tapped.
    assert!(
        state.objects.get(&defender_id).unwrap().status.tapped,
        "Defender should start tapped"
    );

    // P1 declares the provoke creature as attacker targeting P2.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Provoke trigger should be on the stack.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.39a: AbilityTriggered event expected from provoke"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.39a: provoke trigger should be on the stack"
    );

    // P2's defender is still tapped (trigger hasn't resolved yet).
    assert!(
        state.objects.get(&defender_id).unwrap().status.tapped,
        "Defender should still be tapped before trigger resolves"
    );

    // Both players pass priority -- provoke trigger resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved should have been emitted.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "CR 702.39a: AbilityResolved expected after trigger resolves"
    );

    // P2's defender should now be untapped.
    assert!(
        !state.objects.get(&defender_id).unwrap().status.tapped,
        "CR 702.39a: Defender should be untapped after provoke trigger resolves"
    );

    // PermanentUntapped event should have been emitted.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentUntapped { object_id, .. } if *object_id == defender_id)),
        "CR 702.39a: PermanentUntapped event expected for provoked creature"
    );

    // Stack is now empty.
    assert!(
        state.stack_objects.is_empty(),
        "Stack should be empty after provoke trigger resolves"
    );

    // Pass priority again for all players to advance from DeclareAttackers to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.turn.step,
        Step::DeclareBlockers,
        "Game should have advanced to DeclareBlockers after all players pass with empty stack"
    );

    // P2 satisfies the forced-block requirement by blocking.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(defender_id, attacker_id)],
        },
    )
    .expect("CR 702.39a: P2 should be able to block with the provoked creature");

    // Verify blocking is recorded.
    let combat = state.combat.as_ref().unwrap();
    assert_eq!(
        combat.blockers.get(&defender_id),
        Some(&attacker_id),
        "CR 702.39a: provoked creature should be blocking the provoke attacker"
    );
}

// ── Test 2: Forced block requirement -- empty declaration rejected ──────────

#[test]
/// CR 509.1c — Provoke creates a blocking requirement. If the provoked creature
/// CAN block the provoking attacker (no evasion restrictions), it MUST.
/// Declaring no blockers when the provoked creature could block is illegal.
fn test_702_39a_provoke_forces_block_requirement() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker_spec = ObjectSpec::creature(p1, "Provoke Attacker", 2, 2)
        .with_keyword(KeywordAbility::Provoke)
        .in_zone(ZoneId::Battlefield);

    // P2 has an untapped creature (no evasion restrictions).
    let defender_spec = ObjectSpec::creature(p2, "Defender", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker_spec)
        .object(defender_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Provoke Attacker");

    // P1 declares attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the provoke trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Pass priority to advance from DeclareAttackers to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.turn.step,
        Step::DeclareBlockers,
        "Game should be in DeclareBlockers"
    );

    // P2 declares NO blockers -- should fail (forced-block requirement).
    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 509.1c: Empty blocker declaration should be rejected when provoke requirement exists"
    );
    // Verify the error is specifically about the provoke requirement, not wrong step.
    let err_str = format!("{:?}", result.unwrap_err());
    assert!(
        err_str.contains("provoke requirement") || err_str.contains("must block"),
        "Error should be about provoke requirement, got: {}",
        err_str
    );
}

// ── Test 3: Tapped creature is untapped by provoke ───────────────────────────

#[test]
/// CR 702.39a — "untap that creature": A tapped creature targeted by provoke
/// is untapped when the trigger resolves. The PermanentUntapped event is emitted.
fn test_702_39a_provoke_tapped_creature_untapped() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker_spec = ObjectSpec::creature(p1, "Provoke Attacker", 2, 2)
        .with_keyword(KeywordAbility::Provoke)
        .in_zone(ZoneId::Battlefield);

    // P2 has a tapped creature.
    let defender_spec = ObjectSpec::creature(p2, "Tapped Defender", 3, 3)
        .in_zone(ZoneId::Battlefield)
        .tapped();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker_spec)
        .object(defender_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Provoke Attacker");
    let defender_id = find_object(&state, "Tapped Defender");

    assert!(
        state.objects.get(&defender_id).unwrap().status.tapped,
        "Tapped Defender should start tapped"
    );

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve provoke trigger.
    let (state, events) = pass_all(state, &[p1, p2]);

    // Creature should be untapped.
    assert!(
        !state.objects.get(&defender_id).unwrap().status.tapped,
        "CR 702.39a: Tapped Defender should be untapped after provoke resolves"
    );

    // PermanentUntapped event emitted.
    let untapped_event = events.iter().any(|e| {
        matches!(e, GameEvent::PermanentUntapped { object_id, .. } if *object_id == defender_id)
    });
    assert!(
        untapped_event,
        "CR 702.39a: PermanentUntapped event should be emitted for the provoked creature"
    );
}

// ── Test 4: Provoke + flying -- forced-block impossible, no requirement ────────

#[test]
/// CR 509.1b/c — Provoke forced-block requirement is waived if the provoked
/// creature cannot legally block the provoking attacker due to evasion restrictions.
/// Here, the provoker has Flying and the provoked creature has neither Flying nor Reach.
/// The untap still happens, but no forced-block requirement is enforced.
fn test_702_39a_provoke_creature_cant_block_flying() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker_spec = ObjectSpec::creature(p1, "Provoke Flier", 2, 2)
        .with_keyword(KeywordAbility::Provoke)
        .with_keyword(KeywordAbility::Flying)
        .in_zone(ZoneId::Battlefield);

    // P2 has a tapped ground creature (no flying, no reach).
    let defender_spec = ObjectSpec::creature(p2, "Ground Defender", 3, 3)
        .in_zone(ZoneId::Battlefield)
        .tapped();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker_spec)
        .object(defender_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Provoke Flier");
    let defender_id = find_object(&state, "Ground Defender");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve provoke trigger -- untap still happens.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is untapped (the untap part of provoke still resolves).
    assert!(
        !state.objects.get(&defender_id).unwrap().status.tapped,
        "CR 702.39a: Ground Defender should be untapped even when it can't block the flier"
    );

    // Pass priority to advance from DeclareAttackers to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.turn.step,
        Step::DeclareBlockers,
        "Game should be in DeclareBlockers"
    );

    // P2 declares NO blockers -- should SUCCEED (can't block a flier without flying/reach).
    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    );

    assert!(
        result.is_ok(),
        "CR 509.1b/c: Empty blocker declaration should be accepted when provoke requirement is impossible (flier)"
    );
}

// ── Test 5: Multiple provoke instances trigger separately ────────────────────

#[test]
/// CR 702.39b — A creature with two instances of provoke triggers twice when it
/// attacks, creating two separate triggers on the stack.
fn test_702_39b_provoke_multiple_instances_trigger_separately() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker_spec = ObjectSpec::creature(p1, "Double Provoke Attacker", 2, 2)
        .with_keyword(KeywordAbility::Provoke)
        .with_keyword(KeywordAbility::Provoke) // Two instances
        .in_zone(ZoneId::Battlefield);

    // P2 has two creatures (one for each provoke trigger to target).
    let defender1_spec = ObjectSpec::creature(p2, "Defender 1", 1, 1).in_zone(ZoneId::Battlefield);
    let defender2_spec = ObjectSpec::creature(p2, "Defender 2", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker_spec)
        .object(defender1_spec)
        .object(defender2_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Provoke Attacker");

    // P1 declares the double-provoke creature as attacker.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Count AbilityTriggered events -- should be 2 (one per Provoke instance).
    let trigger_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        trigger_count, 2,
        "CR 702.39b: Two provoke triggers should fire for a creature with two Provoke instances"
    );

    // Two triggers should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.39b: Two ProvokeTrigger stack objects should be on the stack"
    );

    // CR 702.39b: Each provoke trigger fires separately and should target a
    // DIFFERENT creature. Collect the provoked_creature from each stack object
    // and verify they are distinct.
    let targets: Vec<ObjectId> = state
        .stack_objects
        .iter()
        .filter_map(|so| match so.kind {
            mtg_engine::StackObjectKind::ProvokeTrigger {
                provoked_creature, ..
            } => Some(provoked_creature),
            _ => None,
        })
        .collect();
    assert_eq!(
        targets.len(),
        2,
        "CR 702.39b: Both stack objects should be ProvokeTrigger variants"
    );
    assert_ne!(
        targets[0],
        targets[1],
        "CR 702.39b: Two provoke triggers should target different creatures"
    );
}

// ── Test 6: No valid target -- trigger not placed on stack ────────────────────

#[test]
/// CR 603.3d — If the defending player controls no creatures, no provoke trigger
/// is placed on the stack (no valid target).
fn test_702_39a_provoke_no_valid_target() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker_spec = ObjectSpec::creature(p1, "Provoke Attacker", 2, 2)
        .with_keyword(KeywordAbility::Provoke)
        .in_zone(ZoneId::Battlefield);

    // P2 controls NO creatures.

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Provoke Attacker");

    // P1 declares attacker.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No provoke trigger should be on the stack (no valid target).
    let trigger_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        trigger_count, 0,
        "CR 603.3d: No provoke trigger should fire when defending player controls no creatures"
    );

    assert!(
        state.stack_objects.is_empty(),
        "CR 603.3d: Stack should be empty when no valid provoke target exists"
    );
}

// ── Test 7: Multiplayer -- correct defending player targeted ──────────────────

#[test]
/// CR 508.5a — In multiplayer, provoke only targets creatures the defending
/// player controls. P1 attacks P2 with provoke; P3 has creatures but they
/// must NOT be targeted.
fn test_702_39a_provoke_multiplayer_correct_defender() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let attacker_spec = ObjectSpec::creature(p1, "Provoke Attacker", 2, 2)
        .with_keyword(KeywordAbility::Provoke)
        .in_zone(ZoneId::Battlefield);

    // P2 has a creature (should be targeted).
    let p2_creature_spec =
        ObjectSpec::creature(p2, "P2 Creature", 1, 1).in_zone(ZoneId::Battlefield);

    // P3 has a creature (must NOT be targeted).
    let p3_creature_spec =
        ObjectSpec::creature(p3, "P3 Creature", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker_spec)
        .object(p2_creature_spec)
        .object(p3_creature_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Provoke Attacker");
    let p2_creature_id = find_object(&state, "P2 Creature");

    // P1 attacks P2.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Exactly one trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 508.5a: Exactly one provoke trigger should be on the stack"
    );

    // The trigger's provoked_creature should be P2's creature, not P3's.
    let trigger = &state.stack_objects[0];
    if let mtg_engine::StackObjectKind::ProvokeTrigger {
        provoked_creature, ..
    } = trigger.kind
    {
        assert_eq!(
            provoked_creature, p2_creature_id,
            "CR 508.5a: Provoke must target P2's creature (the defending player's creature)"
        );
    } else {
        panic!("Expected ProvokeTrigger on the stack");
    }
}
