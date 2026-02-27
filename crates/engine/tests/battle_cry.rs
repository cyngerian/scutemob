//! Battle Cry keyword ability tests (CR 702.91).
//!
//! Battle Cry is a triggered ability: "Whenever this creature attacks, each
//! other attacking creature gets +1/+0 until end of turn."
//!
//! Key rules verified:
//! - Trigger fires for the battle cry creature when it attacks (CR 702.91a).
//! - EACH OTHER attacking creature gets +1/+0 — not the battle cry creature itself (CR 702.91a).
//! - Only power is modified, not toughness ("+1/+0") (CR 702.91a).
//! - If the battle cry creature is the only attacker, no bonus is applied (CR 702.91a "each other").
//! - Multiple instances each trigger separately (CR 702.91b).
//! - The bonus expires at end of turn (CR 514.2).
//! - Only the battle cry creature's own attack triggers it; opponent attack does NOT (CR 702.91a).

use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command, GameEvent,
    GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
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

// ── Test 1: Basic — other attackers get +1 power ─────────────────────────────

#[test]
/// CR 702.91a — Battle cry source (3/3) attacks with two other creatures (2/2 each).
/// After the trigger resolves, each other attacker has +1 power (becomes 3/2).
/// The battle cry creature itself does NOT receive the bonus (stays 3/3).
fn test_battle_cry_basic_other_attackers_get_plus_one_power() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let battle_cry_creature = ObjectSpec::creature(p1, "Battle Cry Warrior", 3, 3)
        .with_keyword(KeywordAbility::BattleCry)
        .in_zone(ZoneId::Battlefield);
    let attacker_a = ObjectSpec::creature(p1, "Attacker A", 2, 2).in_zone(ZoneId::Battlefield);
    let attacker_b = ObjectSpec::creature(p1, "Attacker B", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(battle_cry_creature)
        .object(attacker_a)
        .object(attacker_b)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let bc_id = find_object(&state, "Battle Cry Warrior");
    let a_id = find_object(&state, "Attacker A");
    let b_id = find_object(&state, "Attacker B");

    // P1 declares all three as attackers.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (bc_id, AttackTarget::Player(p2)),
                (a_id, AttackTarget::Player(p2)),
                (b_id, AttackTarget::Player(p2)),
            ],
        },
    )
    .expect("DeclareAttackers should succeed");

    // AbilityTriggered event from the battle cry source.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, controller, .. }
            if *source_object_id == bc_id && *controller == p1
        )),
        "CR 702.91a: AbilityTriggered event expected from the battle cry creature"
    );

    // Stack has 1 trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.91a: battle cry trigger should be on the stack"
    );

    // Both players pass — trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Attacker A is now 3/2 (+1 power, same toughness).
    let chars_a =
        calculate_characteristics(&state, a_id).expect("Attacker A should still be on battlefield");
    assert_eq!(
        chars_a.power,
        Some(3),
        "CR 702.91a: Attacker A should gain +1 power (2+1=3)"
    );
    assert_eq!(
        chars_a.toughness,
        Some(2),
        "CR 702.91a: Attacker A toughness must be unchanged (+1/+0 bonus)"
    );

    // Attacker B is now 3/2 (+1 power, same toughness).
    let chars_b =
        calculate_characteristics(&state, b_id).expect("Attacker B should still be on battlefield");
    assert_eq!(
        chars_b.power,
        Some(3),
        "CR 702.91a: Attacker B should gain +1 power (2+1=3)"
    );
    assert_eq!(
        chars_b.toughness,
        Some(2),
        "CR 702.91a: Attacker B toughness must be unchanged (+1/+0 bonus)"
    );

    // Battle Cry Warrior is still 3/3 — does NOT receive its own bonus.
    let chars_bc = calculate_characteristics(&state, bc_id)
        .expect("Battle Cry Warrior should still be on battlefield");
    assert_eq!(
        chars_bc.power,
        Some(3),
        "CR 702.91a: battle cry source should NOT receive the +1 bonus"
    );
    assert_eq!(
        chars_bc.toughness,
        Some(3),
        "CR 702.91a: battle cry source toughness unchanged"
    );
}

// ── Test 2: Source-only attacker — no bonus ───────────────────────────────────

#[test]
/// CR 702.91a "each other" — if the battle cry creature is the only attacker,
/// no creatures receive the bonus. The trigger still fires but ForEach over an
/// empty set is a no-op.
fn test_battle_cry_source_only_attacker_no_bonus() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let battle_cry_creature = ObjectSpec::creature(p1, "Lone Battle Cry", 3, 3)
        .with_keyword(KeywordAbility::BattleCry)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(battle_cry_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let bc_id = find_object(&state, "Lone Battle Cry");

    // P1 declares only the battle cry creature as attacker.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bc_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Trigger still fires (the creature attacked), but no other attackers exist.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.91a: battle cry trigger fires even when source is only attacker"
    );

    // Resolve the trigger (no-op: ForEach over empty set).
    let (state, _) = pass_all(state, &[p1, p2]);

    // No continuous effects from battle cry (ForEach was empty).
    assert_eq!(
        state.continuous_effects.len(),
        0,
        "CR 702.91a: no continuous effects when battle cry creature is the only attacker"
    );

    // Battle Cry Warrior is still 3/3.
    let chars = calculate_characteristics(&state, bc_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.91a: battle cry source unchanged when alone"
    );
}

// ── Test 3: +1/+0 — toughness is unchanged ───────────────────────────────────

#[test]
/// CR 702.91a "+1/+0" — verify that only power is boosted, not toughness.
/// A 2/3 attacker should become 3/3 (power +1, toughness unchanged).
fn test_battle_cry_does_not_affect_toughness() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let battle_cry_creature = ObjectSpec::creature(p1, "Battle Cry Source", 2, 2)
        .with_keyword(KeywordAbility::BattleCry)
        .in_zone(ZoneId::Battlefield);
    // Different power/toughness to make the test meaningful.
    let attacker =
        ObjectSpec::creature(p1, "Asymmetric Attacker", 2, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(battle_cry_creature)
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let bc_id = find_object(&state, "Battle Cry Source");
    let att_id = find_object(&state, "Asymmetric Attacker");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (bc_id, AttackTarget::Player(p2)),
                (att_id, AttackTarget::Player(p2)),
            ],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, att_id)
        .expect("Asymmetric Attacker should be on battlefield");
    assert_eq!(chars.power, Some(3), "CR 702.91a: power should be 2+1=3");
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.91a: toughness must remain 3 (battle cry is +1/+0, not +1/+1)"
    );
}

// ── Test 4: Multiple instances stack additively ───────────────────────────────

#[test]
/// CR 702.91b — "If a creature has multiple instances of battle cry, each triggers
/// separately." A creature with two battle cry instances generates two triggers;
/// after both resolve, each other attacker has +2/+0.
fn test_battle_cry_multiple_instances_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Give the battle cry creature two instances of the keyword.
    let double_battle_cry = ObjectSpec::creature(p1, "Double Battle Cry", 2, 2)
        .with_keyword(KeywordAbility::BattleCry)
        .with_keyword(KeywordAbility::BattleCry)
        .in_zone(ZoneId::Battlefield);
    let attacker_a = ObjectSpec::creature(p1, "Ally A", 1, 1).in_zone(ZoneId::Battlefield);
    let attacker_b = ObjectSpec::creature(p1, "Ally B", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(double_battle_cry)
        .object(attacker_a)
        .object(attacker_b)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let bc_id = find_object(&state, "Double Battle Cry");
    let a_id = find_object(&state, "Ally A");
    let b_id = find_object(&state, "Ally B");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (bc_id, AttackTarget::Player(p2)),
                (a_id, AttackTarget::Player(p2)),
                (b_id, AttackTarget::Player(p2)),
            ],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Two AbilityTriggered events (one per instance).
    let triggered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.91b: two battle cry instances should generate two triggers"
    );

    // Two triggers on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.91b: two battle cry triggers should be on the stack"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Each other attacker should have received +2/+0 (two triggers, +1 each).
    let chars_a = calculate_characteristics(&state, a_id).expect("Ally A should be on battlefield");
    assert_eq!(
        chars_a.power,
        Some(3),
        "CR 702.91b: Ally A should have +2 power from two battle cry triggers (1+2=3)"
    );
    assert_eq!(
        chars_a.toughness,
        Some(1),
        "CR 702.91b: Ally A toughness unchanged"
    );

    let chars_b = calculate_characteristics(&state, b_id).expect("Ally B should be on battlefield");
    assert_eq!(
        chars_b.power,
        Some(3),
        "CR 702.91b: Ally B should have +2 power from two battle cry triggers (1+2=3)"
    );
    assert_eq!(
        chars_b.toughness,
        Some(1),
        "CR 702.91b: Ally B toughness unchanged"
    );

    // Double Battle Cry still has its printed power (NOT receiving the bonus).
    let chars_bc = calculate_characteristics(&state, bc_id)
        .expect("Double Battle Cry should be on battlefield");
    assert_eq!(
        chars_bc.power,
        Some(2),
        "CR 702.91b: battle cry source should NOT receive any bonus from its own triggers"
    );
}

// ── Test 5: Multiplayer — all other attackers benefit ─────────────────────────

#[test]
/// CR 702.91a — In a 4-player Commander game, all other attacking creatures benefit
/// regardless of which opponent they are attacking.
fn test_battle_cry_multiplayer_all_attackers_benefit() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let battle_cry_creature = ObjectSpec::creature(p1, "War Caller", 2, 2)
        .with_keyword(KeywordAbility::BattleCry)
        .in_zone(ZoneId::Battlefield);
    // This attacker targets p2.
    let attacker_to_p2 =
        ObjectSpec::creature(p1, "Attacker To P2", 1, 1).in_zone(ZoneId::Battlefield);
    // This attacker targets p3.
    let attacker_to_p3 =
        ObjectSpec::creature(p1, "Attacker To P3", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(battle_cry_creature)
        .object(attacker_to_p2)
        .object(attacker_to_p3)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let bc_id = find_object(&state, "War Caller");
    let a2_id = find_object(&state, "Attacker To P2");
    let a3_id = find_object(&state, "Attacker To P3");

    // P1 declares: War Caller → P2, Attacker To P2 → P2, Attacker To P3 → P3.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (bc_id, AttackTarget::Player(p2)),
                (a2_id, AttackTarget::Player(p2)),
                (a3_id, AttackTarget::Player(p3)),
            ],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the battle cry trigger. All 4 players pass.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Attacker To P2 gets +1 power despite attacking P2 (same as the battle cry creature).
    let chars_a2 =
        calculate_characteristics(&state, a2_id).expect("Attacker To P2 should be on battlefield");
    assert_eq!(
        chars_a2.power,
        Some(2),
        "CR 702.91a: Attacker To P2 should get +1 power (1+1=2)"
    );
    assert_eq!(
        chars_a2.toughness,
        Some(1),
        "CR 702.91a: Attacker To P2 toughness unchanged"
    );

    // Attacker To P3 gets +1 power even though it attacks a different opponent.
    let chars_a3 =
        calculate_characteristics(&state, a3_id).expect("Attacker To P3 should be on battlefield");
    assert_eq!(
        chars_a3.power,
        Some(2),
        "CR 702.91a: Attacker To P3 should get +1 power (1+1=2) regardless of target"
    );
    assert_eq!(
        chars_a3.toughness,
        Some(1),
        "CR 702.91a: Attacker To P3 toughness unchanged"
    );

    // War Caller itself does NOT receive the bonus.
    let chars_bc =
        calculate_characteristics(&state, bc_id).expect("War Caller should be on battlefield");
    assert_eq!(
        chars_bc.power,
        Some(2),
        "CR 702.91a: War Caller must NOT receive its own bonus"
    );
}

// ── Test 6: Bonus expires at end of turn ──────────────────────────────────────

#[test]
/// CR 514.2, CR 702.91a ("until end of turn") — after expire_end_of_turn_effects,
/// the +1/+0 bonus is removed and the creature returns to its printed power.
fn test_battle_cry_bonus_expires_at_end_of_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let battle_cry_creature = ObjectSpec::creature(p1, "Battle Cry Source", 2, 2)
        .with_keyword(KeywordAbility::BattleCry)
        .in_zone(ZoneId::Battlefield);
    let attacker = ObjectSpec::creature(p1, "Recipient", 1, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(battle_cry_creature)
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let bc_id = find_object(&state, "Battle Cry Source");
    let att_id = find_object(&state, "Recipient");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (bc_id, AttackTarget::Player(p2)),
                (att_id, AttackTarget::Player(p2)),
            ],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Verify the effect is active (Recipient is 2/3).
    let chars = calculate_characteristics(&state, att_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "battle cry effect active — Recipient should be 2/3 before cleanup"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "battle cry effect active — Recipient toughness unchanged (still 3)"
    );

    // Simulate cleanup: expire all UntilEndOfTurn effects.
    let mut state = state;
    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    // After cleanup: Recipient returns to its printed 1/3.
    let chars = calculate_characteristics(&state, att_id).unwrap();
    assert_eq!(
        chars.power,
        Some(1),
        "CR 514.2: battle cry bonus expired — Recipient should return to printed power (1)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 514.2: Recipient toughness unchanged after cleanup (stays 3)"
    );
}

// ── Test 7: Opponent attacking does NOT trigger ───────────────────────────────

#[test]
/// CR 702.91a "whenever THIS creature attacks" — SelfAttacks only fires on the
/// battle cry creature itself. If an opponent declares attackers, P1's battle cry
/// creature does NOT trigger.
fn test_battle_cry_does_not_trigger_on_opponent_attack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 has a battle cry creature (not attacking).
    let p1_battle_cry = ObjectSpec::creature(p1, "P1 Battle Cry", 2, 2)
        .with_keyword(KeywordAbility::BattleCry)
        .in_zone(ZoneId::Battlefield);
    // P2 has a creature that will attack P1.
    let p2_attacker = ObjectSpec::creature(p2, "P2 Attacker", 1, 1).in_zone(ZoneId::Battlefield);
    // P1 also has a creature that could receive a bonus (to verify it doesn't).
    let p1_other = ObjectSpec::creature(p1, "P1 Other", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(p1_battle_cry)
        .object(p2_attacker)
        .object(p1_other)
        .active_player(p2) // P2 is the active player attacking
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let p2_atk_id = find_object(&state, "P2 Attacker");
    let p1_other_id = find_object(&state, "P1 Other");

    // P2 declares their creature attacking P1.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p2,
            attackers: vec![(p2_atk_id, AttackTarget::Player(p1))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // P1's battle cry creature did NOT trigger.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.91a: P1's battle cry must NOT trigger when P2 attacks"
    );
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.91a: stack must be empty — battle cry does not fire on opponent's attack"
    );

    // P1 Other's power is unchanged (no bonus was applied).
    let chars = calculate_characteristics(&state, p1_other_id).unwrap();
    assert_eq!(
        chars.power,
        Some(1),
        "CR 702.91a: P1 Other should have no bonus — battle cry did not trigger"
    );
}
