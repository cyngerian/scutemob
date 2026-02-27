//! Annihilator keyword ability tests (CR 702.86).
//!
//! Annihilator is a triggered ability: "Whenever this creature attacks,
//! defending player sacrifices N permanents."
//!
//! Key rules verified:
//! - Trigger fires on attacker declared (CR 702.86a, CR 508.1m).
//! - The DEFENDING player sacrifices (not the attacker's controller) (CR 702.86a).
//! - Sacrifice ignores indestructible (CR 701.17a).
//! - Fewer permanents than N: sacrifice all controlled (CR 701.17a).
//! - Multiple instances trigger separately (CR 702.86b).
//! - Multiplayer: each annihilator trigger targets its own defending player (CR 508.5a).
//! - Attacking a planeswalker: defending player is the planeswalker's controller (CR 508.5).
//! - Zero permanents: no error — sacrifice is a no-op (edge case).

use mtg_engine::{
    process_command, AttackTarget, CardRegistry, CardType, Command, GameEvent, GameState,
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

/// Count permanents on the battlefield controlled by `player`.
fn battlefield_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == player)
        .count()
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

// ── Test 1: Basic sacrifice on attack ────────────────────────────────────────

#[test]
/// CR 702.86a — Annihilator 2: whenever this creature attacks, defending player
/// sacrifices 2 permanents. Trigger fires at DeclareAttackers, resolves before
/// DeclareBlockers. P2 should go from 3 permanents to 1 permanent after resolution.
fn test_annihilator_basic_sacrifice_on_attack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1: annihilator creature
    let attacker = ObjectSpec::creature(p1, "Annihilator Creature", 4, 4)
        .with_keyword(KeywordAbility::Annihilator(2))
        .in_zone(ZoneId::Battlefield);

    // P2: 3 permanents (2 will be sacrificed)
    let perm_a = ObjectSpec::creature(p2, "Defender A", 1, 1).in_zone(ZoneId::Battlefield);
    let perm_b = ObjectSpec::creature(p2, "Defender B", 1, 1).in_zone(ZoneId::Battlefield);
    let perm_c = ObjectSpec::creature(p2, "Defender C", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(perm_a)
        .object(perm_b)
        .object(perm_c)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Annihilator Creature");

    // P1 declares the annihilator creature as attacker targeting P2.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Annihilator trigger should be on the stack.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.86a: AbilityTriggered event expected from annihilator"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.86a: annihilator trigger should be on the stack"
    );

    // P2 has 3 permanents before trigger resolves.
    assert_eq!(
        battlefield_count(&state, p2),
        3,
        "P2 should still have 3 permanents before trigger resolves"
    );

    // Both players pass priority — annihilator trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 should have 1 permanent remaining (2 sacrificed).
    assert_eq!(
        battlefield_count(&state, p2),
        1,
        "CR 702.86a: P2 should have 1 permanent remaining after Annihilator 2 resolves"
    );

    // P1's annihilator creature is unaffected (still on battlefield).
    assert_eq!(
        battlefield_count(&state, p1),
        1,
        "P1's annihilator creature should still be on the battlefield"
    );
}

// ── Test 2: Defending player sacrifices, not attacker's controller ────────────

#[test]
/// CR 702.86a — "defending player sacrifices N permanents." The attacker's
/// controller's permanents are NOT sacrificed. Only the defending player's.
fn test_annihilator_defending_player_sacrifices_not_attacker_controller() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1: annihilator creature + 3 permanents
    let attacker = ObjectSpec::creature(p1, "Annihilator Creature", 4, 4)
        .with_keyword(KeywordAbility::Annihilator(2))
        .in_zone(ZoneId::Battlefield);
    let p1_perm_a = ObjectSpec::creature(p1, "P1 Perm A", 1, 1).in_zone(ZoneId::Battlefield);
    let p1_perm_b = ObjectSpec::creature(p1, "P1 Perm B", 1, 1).in_zone(ZoneId::Battlefield);

    // P2: 2 permanents (should lose 2)
    let p2_perm_a = ObjectSpec::creature(p2, "P2 Perm A", 1, 1).in_zone(ZoneId::Battlefield);
    let p2_perm_b = ObjectSpec::creature(p2, "P2 Perm B", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(p1_perm_a)
        .object(p1_perm_b)
        .object(p2_perm_a)
        .object(p2_perm_b)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Annihilator Creature");
    let p1_before = battlefield_count(&state, p1);
    assert_eq!(p1_before, 3, "P1 starts with 3 permanents");

    // P1 attacks P2.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the annihilator trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 should have 0 permanents (both sacrificed).
    assert_eq!(
        battlefield_count(&state, p2),
        0,
        "CR 702.86a: P2 should have 0 permanents (both sacrificed by Annihilator 2)"
    );

    // P1 should still have 3 permanents — P1's permanents are NOT sacrificed.
    assert_eq!(
        battlefield_count(&state, p1),
        3,
        "CR 702.86a: P1's permanents should NOT be sacrificed (P2 is the defending player)"
    );
}

// ── Test 3: Fewer permanents than N — sacrifice all ───────────────────────────

#[test]
/// CR 701.17a edge case: If the defending player controls fewer permanents than N,
/// they sacrifice all permanents they control (not an error).
fn test_annihilator_fewer_permanents_than_n() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Annihilator Creature", 4, 4)
        .with_keyword(KeywordAbility::Annihilator(3))
        .in_zone(ZoneId::Battlefield);

    // P2: only 1 permanent (annihilator 3 would sacrifice 3, but P2 only has 1).
    let p2_perm = ObjectSpec::creature(p2, "P2 Lone Perm", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(p2_perm)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Annihilator Creature");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 sacrificed their only permanent.
    assert_eq!(
        battlefield_count(&state, p2),
        0,
        "CR 701.17a: P2 should sacrifice all permanents when they control fewer than N"
    );

    // No error — engine handles fewer-than-N gracefully.
    assert_eq!(
        battlefield_count(&state, p1),
        1,
        "P1's annihilator creature should still be on the battlefield"
    );
}

// ── Test 4: Multiple instances trigger separately ─────────────────────────────

#[test]
/// CR 702.86b — If a creature has multiple instances of annihilator, each triggers
/// separately. A creature with Annihilator(2) and Annihilator(1) generates 2 triggers.
/// After both resolve, P2 sacrifices 3 permanents total (2 + 1).
fn test_annihilator_multiple_instances_trigger_separately() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Creature with TWO annihilator instances (via two .with_keyword calls).
    let attacker = ObjectSpec::creature(p1, "Double Annihilator", 8, 8)
        .with_keyword(KeywordAbility::Annihilator(2))
        .with_keyword(KeywordAbility::Annihilator(1))
        .in_zone(ZoneId::Battlefield);

    // P2: 5 permanents.
    let p2_perms: Vec<_> = (0..5)
        .map(|i| {
            ObjectSpec::creature(p2, &format!("P2 Perm {i}"), 1, 1).in_zone(ZoneId::Battlefield)
        })
        .collect();

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker);
    for perm in p2_perms {
        builder = builder.object(perm);
    }
    let state = builder
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Annihilator");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Two AbilityTriggered events (one per annihilator instance).
    let triggered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.86b: two annihilator instances should generate two separate triggers"
    );
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.86b: two annihilator triggers should be on the stack"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 sacrificed 2 + 1 = 3 permanents, leaving 2.
    assert_eq!(
        battlefield_count(&state, p2),
        2,
        "CR 702.86b: P2 should have 2 permanents remaining after two annihilator triggers (5 - 3)"
    );
}

// ── Test 5: Multiplayer — annihilator targets correct defending player ─────────

#[test]
/// CR 508.5a — In multiplayer, "defending player" = the specific player being attacked.
/// If P1 attacks P2 with an annihilator creature and attacks P3 with a non-annihilator
/// creature, only P2's permanents are sacrificed.
fn test_annihilator_multiplayer_correct_defending_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // P1: annihilator creature attacks P2; plain creature attacks P3.
    let anni_attacker = ObjectSpec::creature(p1, "Annihilator Creature", 4, 4)
        .with_keyword(KeywordAbility::Annihilator(2))
        .in_zone(ZoneId::Battlefield);
    let plain_attacker =
        ObjectSpec::creature(p1, "Plain Attacker", 2, 2).in_zone(ZoneId::Battlefield);

    // P2: 3 permanents (2 will be sacrificed).
    let p2_perm_a = ObjectSpec::creature(p2, "P2 Perm A", 1, 1).in_zone(ZoneId::Battlefield);
    let p2_perm_b = ObjectSpec::creature(p2, "P2 Perm B", 1, 1).in_zone(ZoneId::Battlefield);
    let p2_perm_c = ObjectSpec::creature(p2, "P2 Perm C", 1, 1).in_zone(ZoneId::Battlefield);

    // P3: 3 permanents (should NOT be sacrificed).
    let p3_perm_a = ObjectSpec::creature(p3, "P3 Perm A", 1, 1).in_zone(ZoneId::Battlefield);
    let p3_perm_b = ObjectSpec::creature(p3, "P3 Perm B", 1, 1).in_zone(ZoneId::Battlefield);
    let p3_perm_c = ObjectSpec::creature(p3, "P3 Perm C", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(anni_attacker)
        .object(plain_attacker)
        .object(p2_perm_a)
        .object(p2_perm_b)
        .object(p2_perm_c)
        .object(p3_perm_a)
        .object(p3_perm_b)
        .object(p3_perm_c)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let anni_id = find_object(&state, "Annihilator Creature");
    let plain_id = find_object(&state, "Plain Attacker");

    // P1 declares: annihilator attacks P2, plain creature attacks P3.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (anni_id, AttackTarget::Player(p2)),
                (plain_id, AttackTarget::Player(p3)),
            ],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Exactly one AbilityTriggered (from the annihilator creature only).
    let triggered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 1,
        "CR 508.5a: only the annihilator creature should generate a trigger"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P2 sacrificed 2 permanents.
    assert_eq!(
        battlefield_count(&state, p2),
        1,
        "CR 508.5a: P2 (the annihilator's defending player) should have 1 permanent remaining"
    );

    // P3 is unaffected — not the annihilator's defending player.
    assert_eq!(
        battlefield_count(&state, p3),
        3,
        "CR 508.5a: P3 (attacked by plain creature) should NOT lose permanents to annihilator"
    );
}

// ── Test 6: Sacrifice ignores indestructible ──────────────────────────────────

#[test]
/// CR 701.17a — "To sacrifice a permanent, its controller moves it from the
/// battlefield directly to its owner's graveyard." Sacrifice bypasses indestructible.
fn test_annihilator_sacrifice_ignores_indestructible() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Annihilator Creature", 4, 4)
        .with_keyword(KeywordAbility::Annihilator(1))
        .in_zone(ZoneId::Battlefield);

    // P2 has one indestructible permanent — it should still be sacrificed.
    let indestructible_perm = ObjectSpec::creature(p2, "Indestructible Perm", 2, 2)
        .with_keyword(KeywordAbility::Indestructible)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(indestructible_perm)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Annihilator Creature");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Indestructible permanent was sacrificed despite being indestructible.
    assert_eq!(
        battlefield_count(&state, p2),
        0,
        "CR 701.17a: sacrifice should bypass indestructible — P2 should have 0 permanents"
    );

    // The permanent should be in P2's graveyard.
    let in_graveyard = state.objects.values().any(|obj| {
        obj.characteristics.name == "Indestructible Perm" && obj.zone == ZoneId::Graveyard(p2)
    });
    assert!(
        in_graveyard,
        "CR 701.17a: the indestructible permanent should be in P2's graveyard after sacrifice"
    );
}

// ── Test 7: Attacking a planeswalker — defending player is its controller ──────

#[test]
/// CR 508.5 — If a creature with annihilator attacks a planeswalker, the defending
/// player is the controller of that planeswalker (not the planeswalker itself).
fn test_annihilator_attacking_planeswalker_defending_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Annihilator Creature", 4, 4)
        .with_keyword(KeywordAbility::Annihilator(2))
        .in_zone(ZoneId::Battlefield);

    // P2 controls a planeswalker.
    let planeswalker = ObjectSpec::card(p2, "Test Planeswalker")
        .with_types(vec![CardType::Planeswalker])
        .in_zone(ZoneId::Battlefield);

    // P2 also has 2 other permanents that may be sacrificed.
    let p2_perm_a = ObjectSpec::creature(p2, "P2 Extra Perm A", 1, 1).in_zone(ZoneId::Battlefield);
    let p2_perm_b = ObjectSpec::creature(p2, "P2 Extra Perm B", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(planeswalker)
        .object(p2_perm_a)
        .object(p2_perm_b)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Annihilator Creature");
    let pw_id = find_object(&state, "Test Planeswalker");

    // P1 attacks the planeswalker (not P2 directly).
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Planeswalker(pw_id))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Annihilator trigger should fire.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 508.5: annihilator should trigger when attacking a planeswalker"
    );

    let p2_before = battlefield_count(&state, p2);

    // Resolve the trigger — P2 (controller of the planeswalker) sacrifices 2 permanents.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 should have lost 2 permanents.
    assert_eq!(
        battlefield_count(&state, p2),
        p2_before - 2,
        "CR 508.5: P2 (planeswalker controller) should sacrifice 2 permanents"
    );

    // P1 is unaffected.
    assert_eq!(
        battlefield_count(&state, p1),
        1,
        "P1's permanents should not be affected"
    );
}

// ── Test 8: Zero permanents — no error ───────────────────────────────────────

#[test]
/// Edge case: If the defending player controls no permanents when annihilator resolves,
/// no error occurs — sacrificing 0 of 0 is a valid no-op.
fn test_annihilator_zero_permanents_no_error() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Annihilator Creature", 4, 4)
        .with_keyword(KeywordAbility::Annihilator(2))
        .in_zone(ZoneId::Battlefield);

    // P2 controls NO permanents.

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Annihilator Creature");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Trigger still fires.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "annihilator trigger should fire even if defending player has no permanents"
    );

    // Resolve — no error, P2 simply has nothing to sacrifice.
    let result = pass_all(state, &[p1, p2]);
    let (state, _) = result;

    // P2 still has 0 permanents — no error, no negative permanents.
    assert_eq!(
        battlefield_count(&state, p2),
        0,
        "Edge case: P2 has 0 permanents — sacrifice is a no-op, no error"
    );

    // P1's annihilator creature is still on the battlefield.
    assert_eq!(
        battlefield_count(&state, p1),
        1,
        "P1's annihilator creature should still be on the battlefield"
    );
}
