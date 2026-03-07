//! Living Metal keyword ability tests (CR 702.161).
//!
//! Living Metal is a static ability found on some Vehicles.
//! "During your turn, this permanent is an artifact creature in addition to
//! its other types." (CR 702.161a)
//!
//! Key rules verified:
//! - During controller's turn: Vehicle with Living Metal has the Creature type.
//! - During opponent's turn: Vehicle is NOT a Creature.
//! - Vehicle retains Artifact (and Vehicle subtype) when Creature is added.
//! - Living Metal only functions on the battlefield (CR 611.3b).
//! - Vehicle's printed P/T applies (no special P/T effect from Living Metal).
//! - Multiplayer: only creature during its controller's specific turn.
//! - Non-Living Metal artifacts are unaffected.

use mtg_engine::{
    calculate_characteristics, CardType, GameState, GameStateBuilder, KeywordAbility, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_on_battlefield(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found on battlefield", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Build a synthetic Artifact Vehicle with Living Metal on the battlefield.
/// Types: Artifact. P/T: 3/3. (Creature type is added at Layer 4 by Living Metal.)
fn living_metal_vehicle_spec(owner: PlayerId, zone: ZoneId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, "Test Living Metal Vehicle").in_zone(zone);
    spec.card_types = vec![CardType::Artifact];
    spec.keywords = vec![KeywordAbility::LivingMetal];
    spec.power = Some(3);
    spec.toughness = Some(3);
    spec
}

/// Build a plain artifact (no Living Metal) for negative tests.
fn plain_artifact_spec(owner: PlayerId, zone: ZoneId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, "Test Plain Artifact").in_zone(zone);
    spec.card_types = vec![CardType::Artifact];
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

// ── Test 1: Creature type during controller's turn ────────────────────────────

/// CR 702.161a — During the controller's turn, a Vehicle with Living Metal is
/// an artifact creature in addition to its other types.
#[test]
fn test_living_metal_creature_during_controller_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(living_metal_vehicle_spec(p1, ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Test Living Metal Vehicle");
    let chars = calculate_characteristics(&state, id).expect("object should exist");

    // During p1's turn (p1 controls the Vehicle), it IS a creature.
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "CR 702.161a: Vehicle with Living Metal should be a Creature during controller's turn"
    );
    // Still retains Artifact type.
    assert!(
        chars.card_types.contains(&CardType::Artifact),
        "CR 702.161a: Vehicle with Living Metal should still be an Artifact during controller's turn"
    );
    // P/T comes from printed stats (3/3).
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.161a: power should be printed value (3) during controller's turn"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.161a: toughness should be printed value (3) during controller's turn"
    );
}

// ── Test 2: NOT a Creature during opponent's turn ─────────────────────────────

/// CR 702.161a (negative) — During an opponent's turn, the Vehicle does NOT have
/// the Creature type from Living Metal.
#[test]
fn test_living_metal_not_creature_during_opponent_turn() {
    let p1 = p(1);
    let p2 = p(2);

    // p2 is the active player (opponent's turn), but p1 controls the Vehicle.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(living_metal_vehicle_spec(p1, ZoneId::Battlefield))
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Test Living Metal Vehicle");
    let chars = calculate_characteristics(&state, id).expect("object should exist");

    // During p2's turn, p1's Vehicle is NOT a creature.
    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "CR 702.161a: Vehicle with Living Metal should NOT be a Creature during opponent's turn"
    );
    // Still IS an Artifact.
    assert!(
        chars.card_types.contains(&CardType::Artifact),
        "CR 702.161a: Vehicle should remain an Artifact during opponent's turn"
    );
}

// ── Test 3: Retains other types (Artifact) ────────────────────────────────────

/// CR 702.161a — "in addition to its other types." The Vehicle retains Artifact
/// (and any other printed types) when Creature is added.
#[test]
fn test_living_metal_retains_other_types() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(living_metal_vehicle_spec(p1, ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Test Living Metal Vehicle");
    let chars = calculate_characteristics(&state, id).expect("object should exist");

    assert!(
        chars.card_types.contains(&CardType::Artifact),
        "CR 702.161a: Artifact type must be retained when Creature is added"
    );
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "CR 702.161a: Creature type must be added during controller's turn"
    );
}

// ── Test 4: Only functions on battlefield ─────────────────────────────────────

/// CR 611.3b — Static abilities on permanents only function on the battlefield.
/// A Vehicle with Living Metal in the graveyard should NOT gain Creature.
#[test]
fn test_living_metal_not_active_in_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(living_metal_vehicle_spec(p1, ZoneId::Graveyard(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let id = find_in_zone(&state, "Test Living Metal Vehicle", ZoneId::Graveyard(p1))
        .expect("object should be in graveyard");
    let chars = calculate_characteristics(&state, id).expect("object should exist");

    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "CR 611.3b: Living Metal should NOT add Creature type in the graveyard"
    );
}

// ── Test 5: Printed P/T used during controller's turn ────────────────────────

/// CR 702.161a — The Vehicle's printed P/T is used when it is a creature.
/// Living Metal adds no P/T modification; the card definition supplies it.
#[test]
fn test_living_metal_uses_printed_pt() {
    let p1 = p(1);
    let p2 = p(2);

    // Build a Vehicle with distinct P/T (5/6) to confirm printed values are used.
    let mut vehicle_spec =
        ObjectSpec::card(p1, "Heavy Living Metal Vehicle").in_zone(ZoneId::Battlefield);
    vehicle_spec.card_types = vec![CardType::Artifact];
    vehicle_spec.keywords = vec![KeywordAbility::LivingMetal];
    vehicle_spec.power = Some(5);
    vehicle_spec.toughness = Some(6);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Heavy Living Metal Vehicle");
    let chars = calculate_characteristics(&state, id).expect("object should exist");

    assert!(
        chars.card_types.contains(&CardType::Creature),
        "should be a creature during controller's turn"
    );
    assert_eq!(
        chars.power,
        Some(5),
        "CR 702.161a: printed power (5) should be used, not 0"
    );
    assert_eq!(
        chars.toughness,
        Some(6),
        "CR 702.161a: printed toughness (6) should be used, not 0"
    );
}

// ── Test 6: Multiplayer — only creature during controller's turn ──────────────

/// CR 702.161a — In a 4-player game, the Vehicle is only a creature during its
/// controller's (p1's) turn, not during any other player's turn.
#[test]
fn test_living_metal_multiplayer_only_controller_turn() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // p1 controls the Vehicle; test with each player as the active player.
    for (active, expected_creature) in [
        (p1, true),  // controller's turn -> creature
        (p2, false), // opponent's turn -> not creature
        (p3, false), // opponent's turn -> not creature
        (p4, false), // opponent's turn -> not creature
    ] {
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .add_player(p3)
            .add_player(p4)
            .object(living_metal_vehicle_spec(p1, ZoneId::Battlefield))
            .active_player(active)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let id = find_on_battlefield(&state, "Test Living Metal Vehicle");
        let chars = calculate_characteristics(&state, id).expect("object should exist");

        assert_eq!(
            chars.card_types.contains(&CardType::Creature),
            expected_creature,
            "CR 702.161a: during {:?}'s turn, p1's Vehicle should be creature={} (active={:?})",
            active,
            expected_creature,
            active
        );
    }
}

// ── Test 7: Non-Living Metal artifact not affected ────────────────────────────

/// CR 702.161a (negative) — A plain artifact without Living Metal does NOT
/// gain the Creature type during any player's turn.
#[test]
fn test_living_metal_nonkeyword_artifact_unaffected() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(plain_artifact_spec(p1, ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let id = find_on_battlefield(&state, "Test Plain Artifact");
    let chars = calculate_characteristics(&state, id).expect("object should exist");

    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "CR 702.161a: artifact without Living Metal should NOT be a Creature during active player's turn"
    );
    assert!(
        chars.card_types.contains(&CardType::Artifact),
        "artifact without Living Metal should remain an Artifact"
    );
}
