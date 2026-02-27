//! Crew keyword ability tests (CR 702.122).
//!
//! Crew is an activated ability of Vehicle cards:
//! "Tap any number of other untapped creatures you control with total power N or
//! greater: This permanent becomes an artifact creature until end of turn."
//! (CR 702.122a)
//!
//! Key rules verified:
//! - CR 702.122a: Vehicle becomes an artifact creature until end of turn.
//! - CR 702.122a: Total power of crew creatures must be >= N.
//! - CR 702.122a: Only "other" untapped creatures (vehicle cannot crew itself).
//! - CR 702.122a: Only creatures controlled by the player.
//! - CR 702.122a: Crew creatures must be untapped.
//! - Summoning sickness does NOT prevent crewing (ruling: "any untapped creature
//!   you control, even one you haven't controlled since the beginning of the turn").
//! - Crewing an already-crewed vehicle is legal (ruling).
//! - Becoming a creature does NOT trigger ETB effects (ruling).
//! - CR 301.7b: Vehicle immediately has its printed P/T when it becomes a creature.
//! - CR 514.2: Crew effect ("until end of turn") expires at cleanup.

use mtg_engine::{
    calculate_characteristics, process_command, CardType, Command, GameEvent, GameStateBuilder,
    KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, SubType,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once (resolves top of stack or advances step).
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
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

/// Build a Vehicle ObjectSpec: Artifact with Vehicle subtype, printed P/T, and Crew(n) keyword.
/// No Creature type — that's added when crewed.
fn vehicle_spec(
    owner: PlayerId,
    name: &str,
    power: i32,
    toughness: i32,
    crew_n: u32,
) -> ObjectSpec {
    let mut spec = ObjectSpec::artifact(owner, name)
        .with_subtypes(vec![SubType("Vehicle".to_string())])
        .with_keyword(KeywordAbility::Crew(crew_n));
    // Set printed P/T directly (no builder method exists; these are public fields).
    // CR 301.7b: these become active when Creature type is added by crew.
    spec.power = Some(power);
    spec.toughness = Some(toughness);
    spec
}

// ── Test 1: Basic crew — vehicle becomes an artifact creature ────────────────

#[test]
/// CR 702.122a — crewing a vehicle makes it an artifact creature until end of turn.
/// CR 301.7b — it immediately has its printed power and toughness.
fn test_crew_basic_vehicle_becomes_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    let crew_member = ObjectSpec::creature(p1, "Test Pilot", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(crew_member)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let pilot_id = find_object(&state, "Test Pilot");

    // Issue CrewVehicle command.
    let (state, events) = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id],
        },
    )
    .unwrap();

    // AbilityActivated event should be emitted.
    let activated = events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1));
    assert!(activated, "should emit AbilityActivated event");

    // Crew creature should be tapped.
    let pilot = state.objects.get(&pilot_id).unwrap();
    assert!(pilot.status.tapped, "crew creature should be tapped");

    // Resolve the crew ability (both players pass).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Vehicle should now be a creature.
    let chars = calculate_characteristics(&state, vehicle_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "vehicle should be a Creature after crew resolves"
    );
    assert!(
        chars.card_types.contains(&CardType::Artifact),
        "vehicle should still be an Artifact"
    );

    // CR 301.7b: Vehicle has its printed P/T.
    assert_eq!(chars.power, Some(3), "vehicle should have printed power 3");
    assert_eq!(
        chars.toughness,
        Some(3),
        "vehicle should have printed toughness 3"
    );
}

// ── Test 2: Insufficient power rejected ──────────────────────────────────────

#[test]
/// CR 702.122a — crew creatures must have total power >= N.
fn test_crew_insufficient_power_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    // Vehicle with Crew 3, but pilot only has power 2.
    let vehicle = vehicle_spec(p1, "Test Copter", 5, 5, 3);
    let weak_pilot = ObjectSpec::creature(p1, "Weak Pilot", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(weak_pilot)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let pilot_id = find_object(&state, "Weak Pilot");

    // Total power 2 < Crew 3 — should fail.
    let result = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id],
        },
    );

    assert!(
        result.is_err(),
        "should reject crew attempt with insufficient total power"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("less than Crew"),
        "error should mention insufficient power: {err_msg}"
    );
}

// ── Test 3: Multiple creatures crew successfully ──────────────────────────────

#[test]
/// CR 702.122a — any number of untapped creatures may be tapped, total power >= N.
fn test_crew_multiple_creatures() {
    let p1 = p(1);
    let p2 = p(2);

    // Vehicle with Crew 3; three 1/1 creatures (total power 3 == 3).
    let vehicle = vehicle_spec(p1, "Test Copter", 4, 4, 3);
    let pilot_a = ObjectSpec::creature(p1, "Pilot A", 1, 1);
    let pilot_b = ObjectSpec::creature(p1, "Pilot B", 1, 1);
    let pilot_c = ObjectSpec::creature(p1, "Pilot C", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(pilot_a)
        .object(pilot_b)
        .object(pilot_c)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let id_a = find_object(&state, "Pilot A");
    let id_b = find_object(&state, "Pilot B");
    let id_c = find_object(&state, "Pilot C");

    // Crew with all three — total power 3 >= Crew 3.
    let (state, _) = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![id_a, id_b, id_c],
        },
    )
    .unwrap();

    // All three should be tapped.
    assert!(
        state.objects[&id_a].status.tapped,
        "Pilot A should be tapped"
    );
    assert!(
        state.objects[&id_b].status.tapped,
        "Pilot B should be tapped"
    );
    assert!(
        state.objects[&id_c].status.tapped,
        "Pilot C should be tapped"
    );

    // Resolve the ability.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Vehicle is now a creature.
    let chars = calculate_characteristics(&state, vehicle_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "vehicle should be a Creature"
    );
}

// ── Test 4: Excess creatures allowed ─────────────────────────────────────────

#[test]
/// Ruling under CR 702.122a — you may tap more power than required.
fn test_crew_excess_creatures_allowed() {
    let p1 = p(1);
    let p2 = p(2);

    // Vehicle with Crew 1; two 3/3 creatures (total power 6 >> 1).
    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    let pilot_a = ObjectSpec::creature(p1, "Big Pilot A", 3, 3);
    let pilot_b = ObjectSpec::creature(p1, "Big Pilot B", 3, 3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(pilot_a)
        .object(pilot_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let id_a = find_object(&state, "Big Pilot A");
    let id_b = find_object(&state, "Big Pilot B");

    // Crew with both — excess power is legal.
    let (state, _) = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![id_a, id_b],
        },
    )
    .unwrap();

    assert!(
        state.objects[&id_a].status.tapped,
        "excess crew creature should be tapped"
    );
    assert!(
        state.objects[&id_b].status.tapped,
        "excess crew creature should be tapped"
    );
}

// ── Test 5: Vehicle cannot crew itself ───────────────────────────────────────

#[test]
/// CR 702.122a: "other untapped creatures" — vehicle may not crew itself.
fn test_crew_vehicle_cannot_crew_itself() {
    let p1 = p(1);
    let p2 = p(2);

    // Vehicle that is also a creature (e.g., already animated by another effect).
    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");

    let result = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![vehicle_id], // vehicle trying to crew itself
        },
    );

    assert!(result.is_err(), "vehicle should not be able to crew itself");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("cannot be used to crew itself"),
        "error should mention self-crew prohibition: {err_msg}"
    );
}

// ── Test 6: Summoning-sick creature CAN crew ─────────────────────────────────

#[test]
/// Ruling under CR 702.122a — summoning sickness only prevents {T} activated abilities,
/// not tapping as part of a crew cost.
fn test_crew_summoning_sick_creature_can_crew() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    // Creature with summoning sickness (just entered battlefield, no Haste).
    let sick_pilot = ObjectSpec::creature(p1, "Sick Pilot", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(sick_pilot)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let pilot_id = find_object(&state, "Sick Pilot");

    // Manually set summoning sickness flag.
    state
        .objects
        .get_mut(&pilot_id)
        .unwrap()
        .has_summoning_sickness = true;

    // Should succeed — summoning sickness does NOT prevent crewing.
    let result = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id],
        },
    );

    assert!(
        result.is_ok(),
        "summoning-sick creature should be able to crew: {:?}",
        result.unwrap_err()
    );
}

// ── Test 7: Tapped creature is rejected ──────────────────────────────────────

#[test]
/// CR 702.122a: "untapped creatures" — a tapped creature cannot be used to crew.
fn test_crew_tapped_creature_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    let tapped_pilot = ObjectSpec::creature(p1, "Tired Pilot", 1, 1).tapped();

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(tapped_pilot)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let pilot_id = find_object(&state, "Tired Pilot");

    let result = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id],
        },
    );

    assert!(
        result.is_err(),
        "tapped creature should not be usable for crew"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("already tapped"),
        "error should mention already tapped: {err_msg}"
    );
}

// ── Test 8: Non-creature cannot crew ─────────────────────────────────────────

#[test]
/// CR 702.122a: "creatures" — only creatures may be tapped for the crew cost.
fn test_crew_not_a_creature_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    // A plain artifact (no creature type).
    let artifact = ObjectSpec::artifact(p1, "Sol Ring");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let artifact_id = find_object(&state, "Sol Ring");

    let result = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![artifact_id],
        },
    );

    assert!(
        result.is_err(),
        "non-creature should not be usable for crew"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not a creature"),
        "error should mention not a creature: {err_msg}"
    );
}

// ── Test 9: Already-crewed vehicle can be crewed again (legal, no-op) ─────────

#[test]
/// Ruling under CR 702.122a — crewing an already-crewed vehicle is legal
/// (it does not change the vehicle's P/T or reset effects).
fn test_crew_already_crewed_vehicle_is_legal() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    let pilot_a = ObjectSpec::creature(p1, "Pilot A", 1, 1);
    let pilot_b = ObjectSpec::creature(p1, "Pilot B", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(pilot_a)
        .object(pilot_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let id_a = find_object(&state, "Pilot A");
    let id_b = find_object(&state, "Pilot B");

    // First crew.
    let (state, _) = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![id_a],
        },
    )
    .unwrap();
    let (mut state, _) = pass_all(state, &[p1, p2]);

    // Vehicle is now a creature.
    let chars = calculate_characteristics(&state, vehicle_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "vehicle should be a creature after first crew"
    );

    // Reset priority for second crew.
    state.turn.priority_holder = Some(p1);
    state.turn.players_passed = im::OrdSet::new();

    // Second crew with Pilot B — this should succeed (already a creature, but legal).
    let result = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![id_b],
        },
    );

    assert!(
        result.is_ok(),
        "crewing an already-crewed vehicle should be legal: {:?}",
        result.unwrap_err()
    );
}

// ── Test 10: Crew effect expires at end of turn ───────────────────────────────

#[test]
/// CR 702.122a ("until end of turn"), CR 514.2 — the crew type-change expires at cleanup.
fn test_crew_effect_expires_at_end_of_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    let pilot = ObjectSpec::creature(p1, "Test Pilot", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(pilot)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let pilot_id = find_object(&state, "Test Pilot");

    // Crew the vehicle.
    let (state, _) = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    // Vehicle is a creature.
    let chars = calculate_characteristics(&state, vehicle_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "vehicle should be a creature after crew"
    );

    // Simulate cleanup: expire UntilEndOfTurn effects.
    let mut state = state;
    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    // Vehicle is no longer a creature.
    let chars = calculate_characteristics(&state, vehicle_id).unwrap();
    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "CR 514.2: vehicle should no longer be a creature after cleanup"
    );
    assert!(
        chars.card_types.contains(&CardType::Artifact),
        "vehicle should still be an artifact after cleanup"
    );
}

// ── Test 11: Crewed vehicle has its printed P/T ───────────────────────────────

#[test]
/// CR 301.7b — a vehicle that becomes a creature immediately has its printed P/T.
fn test_crew_vehicle_has_printed_power_and_toughness() {
    let p1 = p(1);
    let p2 = p(2);

    // Vehicle with printed P/T 5/4.
    let vehicle = vehicle_spec(p1, "Big Hauler", 5, 4, 2);
    let pilot_a = ObjectSpec::creature(p1, "Pilot A", 1, 1);
    let pilot_b = ObjectSpec::creature(p1, "Pilot B", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(pilot_a)
        .object(pilot_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Big Hauler");
    let id_a = find_object(&state, "Pilot A");
    let id_b = find_object(&state, "Pilot B");

    let (state, _) = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![id_a, id_b],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, vehicle_id).unwrap();
    assert!(chars.card_types.contains(&CardType::Creature));
    assert_eq!(chars.power, Some(5), "vehicle should have printed power 5");
    assert_eq!(
        chars.toughness,
        Some(4),
        "vehicle should have printed toughness 4"
    );
}

// ── Test 12: Crew does NOT trigger ETB effects ────────────────────────────────

#[test]
/// Ruling under CR 702.122a — the vehicle was already on the battlefield;
/// crewing it does not cause a new ETB event, so ETB triggers do not fire.
fn test_crew_no_etb_trigger() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    let pilot = ObjectSpec::creature(p1, "Test Pilot", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(pilot)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let pilot_id = find_object(&state, "Test Pilot");

    let (state, _) = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id],
        },
    )
    .unwrap();
    let (state, events) = pass_all(state, &[p1, p2]);

    // No PermanentEnteredBattlefield event should have been emitted for the vehicle.
    let etb_for_vehicle = events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { object_id, .. } if *object_id == vehicle_id));
    assert!(
        !etb_for_vehicle,
        "crewing should not trigger ETB for the vehicle (it was already on the battlefield)"
    );

    // Vehicle IS now a creature (effect resolved correctly).
    let chars = calculate_characteristics(&state, vehicle_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "vehicle should be a creature after crew resolves"
    );
}

// ── Test 13: Duplicate creature in crew_creatures rejected ───────────────────

#[test]
/// CR 702.122a — each creature may only be tapped once for the crew cost.
fn test_crew_duplicate_creature_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 2);
    let pilot = ObjectSpec::creature(p1, "Test Pilot", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(pilot)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let pilot_id = find_object(&state, "Test Pilot");

    // Pass the same creature twice.
    let result = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id, pilot_id],
        },
    );

    assert!(result.is_err(), "duplicate creature should be rejected");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("duplicate"),
        "error should mention duplicate: {err_msg}"
    );
}

// ── Test 14: Opponent's creature cannot be used ───────────────────────────────

#[test]
/// CR 702.122a: "you control" — only the player's own creatures may crew.
fn test_crew_opponent_creature_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    // p2's creature.
    let opp_pilot = ObjectSpec::creature(p2, "Opponent Pilot", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(opp_pilot)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let vehicle_id = find_object(&state, "Test Copter");
    let opp_pilot_id = find_object(&state, "Opponent Pilot");

    let result = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![opp_pilot_id],
        },
    );

    assert!(
        result.is_err(),
        "opponent's creature should not be usable for crew"
    );
    // Should be a NotController error.
    assert!(
        matches!(
            result.unwrap_err(),
            mtg_engine::GameStateError::NotController { .. }
        ),
        "error should be NotController"
    );
}

// ── Test 15: Crew requires priority ──────────────────────────────────────────

#[test]
/// CR 602.2 — activating an ability requires the player to hold priority.
fn test_crew_requires_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p1, "Test Copter", 3, 3, 1);
    let pilot = ObjectSpec::creature(p1, "Test Pilot", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(pilot)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p2 holds priority, NOT p1.
    state.turn.priority_holder = Some(p2);

    let vehicle_id = find_object(&state, "Test Copter");
    let pilot_id = find_object(&state, "Test Pilot");

    let result = process_command(
        state,
        Command::CrewVehicle {
            player: p1,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id],
        },
    );

    assert!(
        result.is_err(),
        "crew should fail when player does not have priority"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            mtg_engine::GameStateError::NotPriorityHolder { .. }
        ),
        "error should be NotPriorityHolder"
    );
}
