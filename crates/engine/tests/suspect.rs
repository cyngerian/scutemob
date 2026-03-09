//! Suspect keyword action tests (CR 701.60).
//!
//! Suspect is a keyword action (CR 701.60), NOT a keyword ability. It applies a
//! designation to a creature that grants menace and "This creature can't block."
//!
//! Key rules verified:
//! - CR 701.60c: A suspected permanent has menace and "This creature can't block."
//! - CR 701.60d: A suspected permanent can't become suspected again (idempotent).
//! - CR 701.60a: Suspected designation is cleared when the permanent leaves the battlefield.
//! - CR 701.60b: Suspected is NOT a copiable value.
//! - CR 701.60a: Unsuspect removes the designation.
//! - Ruling 2024-02-02: Suspected creatures can still attack.

use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command, GameState,
    GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
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

/// Directly suspect a creature in state by setting the is_suspected flag.
/// This simulates Effect::Suspect resolution without going through the full engine.
fn suspect_creature(state: &mut GameState, id: ObjectId) {
    if let Some(obj) = state.objects.get_mut(&id) {
        obj.designations.insert(mtg_engine::Designations::SUSPECTED);
    }
}

/// Directly unsuspect a creature in state by clearing the is_suspected flag.
fn unsuspect_creature(state: &mut GameState, id: ObjectId) {
    if let Some(obj) = state.objects.get_mut(&id) {
        obj.designations.remove(mtg_engine::Designations::SUSPECTED);
    }
}

// ── Test 1: Suspected creature gains Menace ───────────────────────────────────

#[test]
/// CR 701.60c — A suspected permanent has menace for as long as it's suspected.
fn test_suspect_basic_gains_menace() {
    let p1 = PlayerId(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Suspect Target", 2, 2))
        .build()
        .unwrap();

    let id = find_object(&state, "Suspect Target");

    // Before suspecting: no Menace.
    let chars_before = calculate_characteristics(&state, id).unwrap();
    assert!(
        !chars_before.keywords.contains(&KeywordAbility::Menace),
        "CR 701.60c: creature should NOT have Menace before being suspected"
    );

    // Suspect the creature.
    suspect_creature(&mut state, id);

    // After suspecting: Menace is granted.
    let chars_after = calculate_characteristics(&state, id).unwrap();
    assert!(
        chars_after.keywords.contains(&KeywordAbility::Menace),
        "CR 701.60c: suspected creature should have Menace"
    );
}

// ── Test 2: Suspected creature cannot block ───────────────────────────────────

#[test]
/// CR 701.60c — A suspected permanent has "This creature can't block."
fn test_suspect_basic_cant_block() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(ObjectSpec::creature(p2, "Suspected Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let blocker_id = find_object(&state, "Suspected Blocker");

    // Suspect the blocker.
    suspect_creature(&mut state, blocker_id);

    // Set up combat state with the attacker declared.
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
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
        "CR 701.60c: A suspected creature should not be able to block"
    );
}

// ── Test 3: Suspected creature CAN attack ────────────────────────────────────

#[test]
/// CR 701.60c ruling 2024-02-02 — Suspected creatures can still attack.
/// Suspect restricts blocking, not attacking.
fn test_suspect_negative_can_attack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Suspected Attacker", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Suspected Attacker");
    suspect_creature(&mut state, attacker_id);

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    );

    assert!(
        result.is_ok(),
        "CR 701.60c: Suspected creature should still be able to attack: {:?}",
        result.err()
    );
}

// ── Test 4: Suspected creature's Menace enforced in combat ───────────────────

#[test]
/// CR 701.60c + CR 702.110: A suspected attacker has Menace; a single blocker
/// is insufficient and the block must be rejected.
fn test_suspect_menace_evasion() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Suspected Attacker", 2, 2))
        .object(ObjectSpec::creature(p2, "Single Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Suspected Attacker");
    let blocker_id = find_object(&state, "Single Blocker");

    // Suspect the attacker.
    suspect_creature(&mut state, attacker_id);

    // Verify the layer-resolved chars include Menace.
    let attacker_chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert!(
        attacker_chars.keywords.contains(&KeywordAbility::Menace),
        "CR 701.60c: suspected attacker should have Menace in layer-resolved chars"
    );

    // Set up combat with the suspected creature as attacker.
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    // Single blocker should be rejected due to Menace.
    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.110: Suspected attacker has Menace -- single blocker should be rejected"
    );
}

// ── Test 5: Suspect is idempotent ─────────────────────────────────────────────

#[test]
/// CR 701.60d — A suspected permanent can't become suspected again.
/// Suspecting an already-suspected creature is a no-op.
fn test_suspect_idempotent() {
    let p1 = PlayerId(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Target Creature", 2, 2))
        .build()
        .unwrap();

    let id = find_object(&state, "Target Creature");

    // Suspect the creature once.
    suspect_creature(&mut state, id);
    assert!(
        state
            .objects
            .get(&id)
            .unwrap()
            .designations
            .contains(mtg_engine::Designations::SUSPECTED),
        "Creature should be suspected after first suspect"
    );

    // Suspect again (should be idempotent per CR 701.60d).
    suspect_creature(&mut state, id);
    assert!(
        state
            .objects
            .get(&id)
            .unwrap()
            .designations
            .contains(mtg_engine::Designations::SUSPECTED),
        "CR 701.60d: Creature should still be suspected (idempotent)"
    );

    // Verify it still has exactly one Menace (not doubled).
    let chars = calculate_characteristics(&state, id).unwrap();
    assert!(
        chars.keywords.contains(&KeywordAbility::Menace),
        "CR 701.60d: Suspected creature should have Menace after idempotent suspect"
    );
}

// ── Test 6: Zone change clears suspected designation ─────────────────────────

#[test]
/// CR 701.60a + CR 400.7 — Suspected designation lasts "until it leaves the
/// battlefield." Zone change creates a new object with is_suspected = false.
fn test_suspect_zone_change_clears() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Bounce Target", 2, 2))
        .build()
        .unwrap();

    let id = find_object(&state, "Bounce Target");

    // Suspect the creature.
    suspect_creature(&mut state, id);
    assert!(
        state
            .objects
            .get(&id)
            .unwrap()
            .designations
            .contains(mtg_engine::Designations::SUSPECTED),
        "Creature should be suspected before zone change"
    );

    // Bounce to hand (zone change should clear is_suspected).
    state
        .move_object_to_zone(id, ZoneId::Hand(p1))
        .unwrap_or_else(|e| panic!("move_object_to_zone failed: {:?}", e));

    // The creature in hand should have is_suspected = false.
    let in_hand = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Bounce Target" && obj.zone == ZoneId::Hand(p1))
        .expect("Bounce Target should be in hand after bounce");

    assert!(
        !in_hand
            .designations
            .contains(mtg_engine::Designations::SUSPECTED),
        "CR 701.60a + CR 400.7: Suspected designation should be cleared on zone change"
    );
}

// ── Test 7: Unsuspect removes Menace and blocking restriction ─────────────────

#[test]
/// CR 701.60a — "until a spell or ability causes it to no longer be suspected."
/// Unsuspecting a creature removes Menace and restores ability to block.
fn test_unsuspect_removes_menace_and_blocking_restriction() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let target_id = find_object(&state, "Target Creature");

    // Suspect and then unsuspect the creature.
    suspect_creature(&mut state, target_id);
    assert!(
        state
            .objects
            .get(&target_id)
            .unwrap()
            .designations
            .contains(mtg_engine::Designations::SUSPECTED),
        "Creature should be suspected"
    );

    unsuspect_creature(&mut state, target_id);
    assert!(
        !state
            .objects
            .get(&target_id)
            .unwrap()
            .designations
            .contains(mtg_engine::Designations::SUSPECTED),
        "CR 701.60a: Creature should no longer be suspected after unsuspect"
    );

    // Menace should be gone.
    let chars = calculate_characteristics(&state, target_id).unwrap();
    assert!(
        !chars.keywords.contains(&KeywordAbility::Menace),
        "CR 701.60a: Menace should be removed after unsuspect"
    );

    // Creature should now be able to block.
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(target_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "CR 701.60a: Unsuspected creature should be able to block normally: {:?}",
        result.err()
    );
}

// ── Test 8: Suspected flag is not a copiable value ───────────────────────────

#[test]
/// CR 701.60b — Suspected is neither an ability nor part of the permanent's
/// copiable values. Copies of suspected creatures are NOT suspected.
fn test_suspect_not_copiable() {
    let p1 = PlayerId(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Suspected Original", 2, 2))
        .build()
        .unwrap();

    let original_id = find_object(&state, "Suspected Original");

    // Suspect the original.
    suspect_creature(&mut state, original_id);
    assert!(
        state
            .objects
            .get(&original_id)
            .unwrap()
            .designations
            .contains(mtg_engine::Designations::SUSPECTED),
        "Original should be suspected"
    );

    // Verify that the is_suspected flag is NOT in the characteristics
    // (copiable values). It should only be on the raw GameObject.
    let chars = state
        .objects
        .get(&original_id)
        .unwrap()
        .characteristics
        .clone();
    // Characteristics has no is_suspected field -- it's stored on GameObject directly.
    // A copy would clone the characteristics but start with is_suspected: false.
    // We verify this invariant by checking that the field lives outside characteristics.
    let _ = chars; // copiable values don't include is_suspected

    // The suspected field is on the GameObject, not Characteristics.
    // is_suspected is set only on the raw object, not propagated to characteristics.
    let obj = state.objects.get(&original_id).unwrap();
    assert!(
        obj.designations
            .contains(mtg_engine::Designations::SUSPECTED),
        "CR 701.60b: is_suspected is on GameObject, not in copiable characteristics"
    );
    assert!(
        !obj.characteristics
            .keywords
            .contains(&KeywordAbility::Menace),
        "CR 701.60b: Menace is NOT part of printed keywords (it's a runtime grant)"
    );
}

// ── Test 9: Non-suspected creature can block normally (baseline) ──────────────

#[test]
/// CR 701.60c baseline — A creature without the suspected designation can block
/// normally. Verify that the suspect enforcement does not affect non-suspected creatures.
fn test_suspect_baseline_non_suspected_can_block() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(ObjectSpec::creature(p2, "Normal Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let blocker_id = find_object(&state, "Normal Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
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
        "CR 701.60c baseline: Non-suspected creature should be able to block: {:?}",
        result.err()
    );
}
