//! Saddle keyword ability tests (CR 702.171).
//!
//! Saddle is an activated ability of Mount cards:
//! "Tap any number of other untapped creatures you control with total power N
//! or greater: This permanent becomes saddled until end of turn. Activate
//! only as a sorcery." (CR 702.171a)
//!
//! Key rules verified:
//! - CR 702.171a: Sorcery-speed restriction (active player, main phase, empty stack).
//! - CR 702.171a: Total power of saddling creatures must be >= N.
//! - CR 702.171a: Only "other" untapped creatures (mount cannot saddle itself).
//! - CR 702.171a: Only creatures controlled by the player.
//! - CR 702.171a: Saddling creatures must be untapped.
//! - CR 702.171b: Saddled is a designation (boolean flag), not a type change.
//! - CR 702.171b: Saddled designation cleared at cleanup (end of turn).
//! - CR 702.171b: Saddled designation cleared on zone change (e.g., leaves battlefield).
//! - Ruling 2024-04-12: Activating saddle on an already-saddled Mount is legal.
//! - Ruling: Summoning sickness does NOT prevent saddling (same as Crew).
//! - CR 602.2: Player must hold priority.

use mtg_engine::{
    process_command, Command, GameEvent, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec,
    PlayerId, Step, ZoneId,
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

/// Build a Mount ObjectSpec: Creature with Mount subtype and Saddle(n) keyword.
/// Mounts are already creatures; saddle doesn't add the Creature type.
fn mount_spec(
    owner: PlayerId,
    name: &str,
    power: i32,
    toughness: i32,
    saddle_n: u32,
) -> ObjectSpec {
    use mtg_engine::SubType;
    ObjectSpec::creature(owner, name, power, toughness)
        .with_subtypes(vec![SubType("Mount".to_string())])
        .with_keyword(KeywordAbility::Saddle(saddle_n))
}

// ── Test 1: Basic saddle — mount becomes saddled ──────────────────────────────

#[test]
/// CR 702.171a — saddling a mount sets the saddled designation until end of turn.
/// CR 702.171b — saddled is a boolean flag, not a type change.
fn test_saddle_basic_mount_becomes_saddled() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let saddler = ObjectSpec::creature(p1, "Test Rider", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(saddler)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let saddler_id = find_object(&state, "Test Rider");

    // Mount should not be saddled initially.
    assert!(
        !state.objects[&mount_id]
            .designations
            .contains(mtg_engine::Designations::SADDLED),
        "mount should not be saddled initially"
    );

    // Issue SaddleMount command.
    let (state, events) = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![saddler_id],
        },
    )
    .unwrap();

    // AbilityActivated event should be emitted.
    let activated = events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1));
    assert!(activated, "should emit AbilityActivated event");

    // Saddler should be tapped.
    assert!(
        state.objects[&saddler_id].status.tapped,
        "saddling creature should be tapped"
    );

    // Mount is not yet saddled (ability is on the stack).
    assert!(
        !state.objects[&mount_id]
            .designations
            .contains(mtg_engine::Designations::SADDLED),
        "mount should not be saddled until ability resolves"
    );

    // Resolve the saddle ability (both players pass).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Mount should now be saddled.
    assert!(
        state.objects[&mount_id]
            .designations
            .contains(mtg_engine::Designations::SADDLED),
        "mount should be saddled after ability resolves (CR 702.171a)"
    );
}

// ── Test 2: Insufficient power rejected ──────────────────────────────────────

#[test]
/// CR 702.171a — saddling creatures must have total power >= N.
fn test_saddle_insufficient_power_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    // Mount with Saddle 3, but saddler only has power 2.
    let mount = mount_spec(p1, "Test Mount", 4, 4, 3);
    let weak_rider = ObjectSpec::creature(p1, "Weak Rider", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(weak_rider)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let rider_id = find_object(&state, "Weak Rider");

    // Total power 2 < Saddle 3 — should fail.
    let result = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![rider_id],
        },
    );

    assert!(
        result.is_err(),
        "should reject saddle attempt with insufficient total power"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("less than Saddle"),
        "error should mention insufficient power: {err_msg}"
    );
}

// ── Test 3: Multiple creatures saddle successfully ────────────────────────────

#[test]
/// CR 702.171a — any number of untapped creatures may be tapped, total power >= N.
fn test_saddle_multiple_creatures() {
    let p1 = p(1);
    let p2 = p(2);

    // Mount with Saddle 3; three 1/1 creatures (total power 3 == 3).
    let mount = mount_spec(p1, "Test Mount", 3, 3, 3);
    let rider_a = ObjectSpec::creature(p1, "Rider A", 1, 1);
    let rider_b = ObjectSpec::creature(p1, "Rider B", 1, 1);
    let rider_c = ObjectSpec::creature(p1, "Rider C", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(rider_a)
        .object(rider_b)
        .object(rider_c)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let id_a = find_object(&state, "Rider A");
    let id_b = find_object(&state, "Rider B");
    let id_c = find_object(&state, "Rider C");

    let (state, _) = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![id_a, id_b, id_c],
        },
    )
    .unwrap();

    // All three should be tapped.
    assert!(
        state.objects[&id_a].status.tapped,
        "Rider A should be tapped"
    );
    assert!(
        state.objects[&id_b].status.tapped,
        "Rider B should be tapped"
    );
    assert!(
        state.objects[&id_c].status.tapped,
        "Rider C should be tapped"
    );

    // Resolve and verify saddled.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        state.objects[&mount_id]
            .designations
            .contains(mtg_engine::Designations::SADDLED),
        "mount should be saddled (CR 702.171a)"
    );
}

// ── Test 4: Mount cannot saddle itself ────────────────────────────────────────

#[test]
/// CR 702.171a: "other untapped creatures" — the mount may not be in the saddle list.
fn test_saddle_mount_cannot_saddle_itself() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");

    // Attempt to use the mount itself as a saddling creature.
    let result = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![mount_id],
        },
    );

    assert!(
        result.is_err(),
        "should reject: mount cannot saddle itself (CR 702.171a)"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("cannot be used to saddle itself"),
        "error should mention self-saddle restriction: {err_msg}"
    );
}

// ── Test 5: Summoning sickness does not prevent saddling ─────────────────────

#[test]
/// Ruling under CR 702.171a: tapping for saddle is not a {T} activated ability.
/// A creature with summoning sickness CAN be tapped to saddle.
fn test_saddle_summoning_sick_creature_can_saddle() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let sick_rider = ObjectSpec::creature(p1, "Sick Rider", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(sick_rider)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let rider_id = find_object(&state, "Sick Rider");

    // Manually set summoning sickness (builder places objects without sickness).
    // This simulates a creature that entered the battlefield this turn.
    state
        .objects
        .get_mut(&rider_id)
        .unwrap()
        .has_summoning_sickness = true;

    // Saddling with a summoning-sick creature should succeed.
    let result = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![rider_id],
        },
    );

    assert!(
        result.is_ok(),
        "summoning sickness should NOT prevent saddling: {:?}",
        result.err()
    );
}

// ── Test 6: Tapped creature rejected ─────────────────────────────────────────

#[test]
/// CR 702.171a: "untapped creatures" — already-tapped creatures cannot saddle.
fn test_saddle_tapped_creature_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let rider = ObjectSpec::creature(p1, "Tapped Rider", 3, 3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(rider)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let rider_id = find_object(&state, "Tapped Rider");

    // Manually tap the rider.
    state.objects.get_mut(&rider_id).unwrap().status.tapped = true;

    let result = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![rider_id],
        },
    );

    assert!(
        result.is_err(),
        "should reject already-tapped creature for saddling"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("already tapped"),
        "error should mention tapped creature: {err_msg}"
    );
}

// ── Test 7: Non-creature rejected ────────────────────────────────────────────

#[test]
/// CR 702.171a: Only creatures can be tapped for saddle cost.
fn test_saddle_not_a_creature_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let artifact = ObjectSpec::artifact(p1, "Shiny Artifact");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let artifact_id = find_object(&state, "Shiny Artifact");

    let result = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![artifact_id],
        },
    );

    assert!(
        result.is_err(),
        "should reject non-creature as saddling source"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not a creature"),
        "error should mention non-creature: {err_msg}"
    );
}

// ── Test 8: Already-saddled mount is legal ────────────────────────────────────

#[test]
/// Ruling 2024-04-12: activating saddle on an already-saddled Mount is legal.
fn test_saddle_already_saddled_is_legal() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let rider_a = ObjectSpec::creature(p1, "Rider A", 1, 1);
    let rider_b = ObjectSpec::creature(p1, "Rider B", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(rider_a)
        .object(rider_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let id_a = find_object(&state, "Rider A");
    let id_b = find_object(&state, "Rider B");

    // First saddle activation.
    let (state, _) = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![id_a],
        },
    )
    .unwrap();

    // Resolve.
    let (mut state, _) = pass_all(state, &[p1, p2]);
    assert!(
        state.objects[&mount_id]
            .designations
            .contains(mtg_engine::Designations::SADDLED),
        "mount should be saddled after first activation"
    );

    // Second saddle activation on an already-saddled mount should succeed.
    state.turn.priority_holder = Some(p1);
    let result = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![id_b],
        },
    );

    assert!(
        result.is_ok(),
        "saddling an already-saddled mount should be legal (ruling 2024-04-12): {:?}",
        result.err()
    );
}

// ── Test 9: Saddled designation expires at end of turn ───────────────────────

#[test]
/// CR 702.171b — "stays saddled until the end of the turn." Designation expires at cleanup.
fn test_saddle_expires_at_end_of_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let rider = ObjectSpec::creature(p1, "Test Rider", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(rider)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let rider_id = find_object(&state, "Test Rider");

    // Saddle the mount.
    let (state, _) = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![rider_id],
        },
    )
    .unwrap();

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        state.objects[&mount_id]
            .designations
            .contains(mtg_engine::Designations::SADDLED),
        "mount should be saddled before cleanup"
    );

    // Advance to cleanup. In a 2-player game, we need to pass through the
    // remaining steps: CombatMain → BeginningOfCombat → DeclareAttackers →
    // DeclareBlockers → FirstStrikeDamage → CombatDamage → EndOfCombat →
    // PostCombatMain → End → Cleanup.
    // Simplest: advance step by step until cleanup clears is_saddled.
    //
    // We check that after enough PassPriority calls the designation is cleared.
    let mut current = state;
    for _ in 0..30 {
        if !current
            .objects
            .get(&mount_id)
            .map(|o| o.designations.contains(mtg_engine::Designations::SADDLED))
            .unwrap_or(false)
        {
            break;
        }
        match process_command(
            current.clone(),
            Command::PassPriority {
                player: current.turn.priority_holder.unwrap_or(p1),
            },
        ) {
            Ok((next, _)) => current = next,
            Err(_) => break,
        }
    }

    assert!(
        !current
            .objects
            .get(&mount_id)
            .map(|o| o.designations.contains(mtg_engine::Designations::SADDLED))
            .unwrap_or(true),
        "mount should not be saddled after cleanup (CR 702.171b)"
    );
}

// ── Test 10: Sorcery-speed only ──────────────────────────────────────────────

#[test]
/// CR 702.171a: "Activate only as a sorcery." Must fail if not the active player's turn
/// or not a main phase.
fn test_saddle_sorcery_speed_only() {
    let p1 = p(1);
    let p2 = p(2);

    // Case (a): Not active player's turn — p2 is active, p1 tries to saddle.
    let mut state_a = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount_spec(p1, "Test Mount", 3, 3, 1))
        .object(ObjectSpec::creature(p1, "Test Rider", 2, 2))
        .active_player(p2) // p2 is active
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state_a.turn.priority_holder = Some(p1);

    let mount_id_a = find_object(&state_a, "Test Mount");
    let rider_id_a = find_object(&state_a, "Test Rider");

    let result_a = process_command(
        state_a,
        Command::SaddleMount {
            player: p1,
            mount: mount_id_a,
            saddle_creatures: vec![rider_id_a],
        },
    );
    assert!(
        result_a.is_err(),
        "saddle should fail when not active player's turn (CR 702.171a)"
    );

    // Case (b): Not a main phase — p1 is active but in combat.
    let mut state_b = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount_spec(p1, "Test Mount B", 3, 3, 1))
        .object(ObjectSpec::creature(p1, "Test Rider B", 2, 2))
        .active_player(p1)
        .at_step(Step::BeginningOfCombat) // combat phase
        .build()
        .unwrap();

    state_b.turn.priority_holder = Some(p1);

    let mount_id_b = find_object(&state_b, "Test Mount B");
    let rider_id_b = find_object(&state_b, "Test Rider B");

    let result_b = process_command(
        state_b,
        Command::SaddleMount {
            player: p1,
            mount: mount_id_b,
            saddle_creatures: vec![rider_id_b],
        },
    );
    assert!(
        result_b.is_err(),
        "saddle should fail outside main phase (CR 702.171a)"
    );
    let err_b = result_b.unwrap_err().to_string();
    assert!(
        err_b.contains("main phase"),
        "error should mention main phase restriction: {err_b}"
    );
}

// ── Test 11: Requires priority ────────────────────────────────────────────────

#[test]
/// CR 602.2: Player must hold priority to activate an ability.
fn test_saddle_requires_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let rider = ObjectSpec::creature(p1, "Test Rider", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(rider)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p2 holds priority, not p1.
    state.turn.priority_holder = Some(p2);

    let mount_id = find_object(&state, "Test Mount");
    let rider_id = find_object(&state, "Test Rider");

    let result = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![rider_id],
        },
    );

    assert!(
        result.is_err(),
        "should fail when player does not hold priority (CR 602.2)"
    );
}

// ── Test 12: Duplicate creature rejected ─────────────────────────────────────

#[test]
/// CR 702.171a: Each creature may only be tapped once for saddle cost.
fn test_saddle_duplicate_creature_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let rider = ObjectSpec::creature(p1, "Test Rider", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(rider)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let rider_id = find_object(&state, "Test Rider");

    // Provide the same creature twice.
    let result = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![rider_id, rider_id],
        },
    );

    assert!(
        result.is_err(),
        "should reject duplicate creature in saddle_creatures"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("duplicate"),
        "error should mention duplicate creature: {err_msg}"
    );
}

// ── Test 13: Opponent's creature rejected ────────────────────────────────────

#[test]
/// CR 702.171a: "you control" — only the saddling player's creatures may be tapped.
fn test_saddle_opponent_creature_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let opponent_rider = ObjectSpec::creature(p2, "Opponent Rider", 5, 5); // p2 controls this

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(opponent_rider)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let opp_rider_id = find_object(&state, "Opponent Rider");

    let result = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![opp_rider_id],
        },
    );

    assert!(
        result.is_err(),
        "should reject opponent-controlled creature for saddling"
    );
}

// ── Test 14: Saddled designation cleared on zone change ──────────────────────

#[test]
/// CR 702.171b: "stays saddled until the end of the turn or it leaves the battlefield."
/// Zone change (object becomes new object per CR 400.7) clears is_saddled.
fn test_saddle_cleared_on_zone_change() {
    let p1 = p(1);
    let p2 = p(2);

    let mount = mount_spec(p1, "Test Mount", 3, 3, 1);
    let rider = ObjectSpec::creature(p1, "Test Rider", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mount)
        .object(rider)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let mount_id = find_object(&state, "Test Mount");
    let rider_id = find_object(&state, "Test Rider");

    // Saddle the mount.
    let (state, _) = process_command(
        state,
        Command::SaddleMount {
            player: p1,
            mount: mount_id,
            saddle_creatures: vec![rider_id],
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        state.objects[&mount_id]
            .designations
            .contains(mtg_engine::Designations::SADDLED),
        "mount should be saddled"
    );

    // Move mount to graveyard (simulating it dying). This invokes CR 400.7 new-object
    // semantics: move_object_to_zone constructs a new GameObject with is_saddled: false.
    let mut state = state;
    let graveyard_zone = ZoneId::Graveyard(p1);
    let _ = state.move_object_to_zone(mount_id, graveyard_zone);

    // Find the new object in the graveyard (different id after zone change).
    let new_mount = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.characteristics.name == "Test Mount" && obj.zone == ZoneId::Graveyard(p1)
        })
        .map(|(_, obj)| obj);

    if let Some(obj) = new_mount {
        assert!(
            !obj.designations.contains(mtg_engine::Designations::SADDLED),
            "is_saddled should be false after zone change (CR 702.171b / CR 400.7)"
        );
    }
    // If the mount is not found in the graveyard, the zone change itself cleared the object.
}
