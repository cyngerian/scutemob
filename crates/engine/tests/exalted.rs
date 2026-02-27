//! Exalted keyword ability tests (CR 702.83).
//!
//! Exalted is a triggered ability: "Whenever a creature you control attacks alone,
//! that creature gets +1/+1 until end of turn."
//!
//! Key rules verified:
//! - Triggers when exactly one creature is declared as an attacker (CR 702.83b).
//! - Does NOT trigger when multiple creatures attack (CR 702.83b).
//! - The +1/+1 goes to the lone attacker, not the permanent with exalted (CR 702.83a).
//! - Exalted on a non-attacker permanent still targets the lone attacker.
//! - Multiple exalted instances stack additively (rulings).
//! - The bonus expires at end of turn (CR 702.83a, CR 514.2).
//! - Only the attacking player's exalted abilities trigger (CR 702.83a "you").
//! - An empty attacker list does NOT trigger exalted (edge case).

use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command, GameEvent,
    GameStateBuilder, KeywordAbility, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once.
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

// ── Test 1: Basic exalted triggers when attacking alone ───────────────────────

#[test]
/// CR 702.83a — Exalted triggers when a creature your control attacks alone,
/// giving the attacking creature (NOT the exalted source) +1/+1 until end of turn.
fn test_exalted_basic_attacks_alone_gives_plus_one() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // p1 has a 2/2 creature with exalted (the exalted source — not the attacker).
    let exalted_source = ObjectSpec::creature(p1, "Exalted Bear", 2, 2)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);

    // p1 has a 1/1 creature that will attack alone.
    let attacker = ObjectSpec::creature(p1, "Lone Attacker", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(exalted_source)
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Lone Attacker");
    let source_id = find_object(&state, "Exalted Bear");

    // p1 declares only the 1/1 as attacker — "attacks alone" (CR 702.83b).
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // AbilityTriggered event from the exalted source.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, controller, .. }
            if *source_object_id == source_id && *controller == p1
        )),
        "CR 702.83a: AbilityTriggered event expected from the exalted source"
    );

    // Stack has 1 trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.83a: exalted trigger should be on the stack"
    );

    // Both players pass — exalted trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Lone Attacker is now 2/2 (1+1 / 1+1).
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("Lone Attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(2),
        "CR 702.83a: attacker should get +1 power from exalted"
    );
    assert_eq!(
        chars.toughness,
        Some(2),
        "CR 702.83a: attacker should get +1 toughness from exalted"
    );

    // Exalted Bear is still 2/2 (bonus does NOT go to the source).
    let bear_chars = calculate_characteristics(&state, source_id)
        .expect("Exalted Bear should still be on battlefield");
    assert_eq!(
        bear_chars.power,
        Some(2),
        "CR 702.83a: exalted source should NOT get the bonus"
    );
    assert_eq!(
        bear_chars.toughness,
        Some(2),
        "CR 702.83a: exalted source should NOT get the bonus"
    );
}

// ── Test 2: Exalted does NOT trigger with multiple attackers ──────────────────

#[test]
/// CR 702.83b — "attacks alone" = exactly one creature declared as attacker.
/// If two creatures attack, exalted does NOT trigger even if one has exalted.
fn test_exalted_does_not_trigger_with_multiple_attackers() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // p1 has two creatures, one with exalted.
    let exalted_creature = ObjectSpec::creature(p1, "Exalted Bear", 2, 2)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);
    let second = ObjectSpec::creature(p1, "Plain Bear", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(exalted_creature)
        .object(second)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let exalted_id = find_object(&state, "Exalted Bear");
    let second_id = find_object(&state, "Plain Bear");

    // p1 declares BOTH as attackers — not "attacks alone" (CR 702.83b).
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (exalted_id, AttackTarget::Player(p2)),
                (second_id, AttackTarget::Player(p2)),
            ],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No exalted trigger.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.83b: exalted must NOT trigger when multiple creatures attack"
    );
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.83b: stack must be empty when two attackers are declared"
    );
}

// ── Test 3: Multiple exalted instances stack additively ───────────────────────

#[test]
/// CR 702.83a, rulings — "If a creature has multiple instances of exalted, each
/// triggers separately." Three permanents with exalted generate three triggers.
/// After all resolve, the attacker has +3/+3.
fn test_exalted_multiple_instances_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // p1 has three permanents with exalted and one creature that will attack alone.
    let exalted1 = ObjectSpec::creature(p1, "Exalted A", 1, 1)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);
    let exalted2 = ObjectSpec::creature(p1, "Exalted B", 1, 1)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);
    let exalted3 = ObjectSpec::creature(p1, "Exalted C", 1, 1)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);
    let attacker = ObjectSpec::creature(p1, "Lone Attacker", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(exalted1)
        .object(exalted2)
        .object(exalted3)
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Lone Attacker");

    // Declare the 1/1 as the lone attacker — three exalted triggers fire.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Exactly 3 AbilityTriggered events (one per exalted source).
    let triggered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 3,
        "CR 702.83a: three exalted permanents should generate three triggers"
    );

    // 3 triggers on the stack.
    assert_eq!(
        state.stack_objects.len(),
        3,
        "CR 702.83a: three exalted triggers on the stack"
    );

    // Resolve all three triggers: both players pass three times.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Attacker is now 4/4 (1 + 3 = 4 power/toughness).
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("Lone Attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(4),
        "CR 702.83a: three exalted triggers should give +3 power total (1+3=4)"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 702.83a: three exalted triggers should give +3 toughness total (1+3=4)"
    );
}

// ── Test 4: Exalted on a non-attacker permanent targets the attacker ──────────

#[test]
/// CR 702.83a — The exalted ability gives +1/+1 to the attacking creature,
/// not to the permanent that has exalted. Exalted on a non-attacking creature
/// still gives +1/+1 to the lone attacker.
fn test_exalted_on_non_attacker_permanent_targets_attacker() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Creature A attacks alone. Creature B has exalted and does NOT attack.
    let attacker = ObjectSpec::creature(p1, "Creature A", 2, 2).in_zone(ZoneId::Battlefield);
    let exalted_non_attacker = ObjectSpec::creature(p1, "Creature B", 1, 1)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(exalted_non_attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Creature A");
    let source_id = find_object(&state, "Creature B");

    // Only Creature A attacks — Creature B's exalted triggers.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // AbilityTriggered from Creature B (the exalted source, which is NOT attacking).
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == source_id
        )),
        "CR 702.83a: exalted on non-attacker Creature B should trigger"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature A (the attacker) is now 3/3 (2+1 / 2+1).
    let chars_a = calculate_characteristics(&state, attacker_id)
        .expect("Creature A should still be on battlefield");
    assert_eq!(
        chars_a.power,
        Some(3),
        "CR 702.83a: attacker Creature A should get +1 power from Creature B's exalted"
    );
    assert_eq!(
        chars_a.toughness,
        Some(3),
        "CR 702.83a: attacker Creature A should get +1 toughness from Creature B's exalted"
    );

    // Creature B (the exalted source) is still 1/1.
    let chars_b = calculate_characteristics(&state, source_id)
        .expect("Creature B should still be on battlefield");
    assert_eq!(
        chars_b.power,
        Some(1),
        "CR 702.83a: Creature B (exalted source) should NOT get the +1 bonus"
    );
    assert_eq!(
        chars_b.toughness,
        Some(1),
        "CR 702.83a: Creature B (exalted source) should NOT get the +1 bonus"
    );
}

// ── Test 5: Opponent attacking alone does NOT trigger player's exalted ─────────

#[test]
/// CR 702.83a — "whenever a creature YOU control attacks alone" — the "you" means
/// only the attacking player's exalted abilities trigger. An opponent's exalted
/// permanent does NOT trigger when that opponent attacks alone.
fn test_exalted_does_not_trigger_on_opponent_attack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // p1 has an exalted creature. p2 attacks alone.
    let p1_exalted = ObjectSpec::creature(p1, "P1 Exalted Bear", 2, 2)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);
    let p2_attacker = ObjectSpec::creature(p2, "P2 Attacker", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(p1_exalted)
        .object(p2_attacker)
        .active_player(p2) // p2 is the active player attacking
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let p2_attacker_id = find_object(&state, "P2 Attacker");

    // p2 declares their creature as the lone attacker.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p2,
            attackers: vec![(p2_attacker_id, AttackTarget::Player(p1))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // p1's exalted permanent does NOT trigger (p2 is the attacking player).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.83a: p1's exalted must NOT trigger when p2 attacks alone"
    );
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.83a: stack must be empty — p1's exalted does not trigger on p2's attack"
    );
}

// ── Test 6: Exalted bonus expires at end of turn ──────────────────────────────

#[test]
/// CR 702.83a ("until end of turn"), CR 514.2 — the +1/+1 from exalted expires
/// during the Cleanup step. After expire_end_of_turn_effects is called, the
/// continuous effect is removed and the creature returns to its printed P/T.
fn test_exalted_bonus_expires_at_end_of_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let exalted_source = ObjectSpec::creature(p1, "Exalted Bear", 2, 2)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);
    let attacker = ObjectSpec::creature(p1, "Lone Attacker", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(exalted_source)
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Lone Attacker");

    // Declare lone attacker — exalted triggers.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the exalted trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Exalted effect active: attacker is now 2/2.
    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "exalted effect active — attacker should be 2/2 before cleanup"
    );
    assert_eq!(
        chars.toughness,
        Some(2),
        "exalted effect active — attacker should be 2/2 before cleanup"
    );

    // Simulate cleanup: expire all UntilEndOfTurn effects.
    let mut state = state;
    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    // After cleanup: attacker is back to 1/1.
    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(1),
        "CR 514.2: exalted +1 power should expire at cleanup"
    );
    assert_eq!(
        chars.toughness,
        Some(1),
        "CR 514.2: exalted +1 toughness should expire at cleanup"
    );
}

// ── Test 7: Multiplayer — only attacking player's exalted triggers ────────────

#[test]
/// CR 702.83a, multiplayer — In a 4-player Commander game, only the attacking
/// player's exalted abilities trigger. Other players' exalted permanents do NOT trigger.
fn test_exalted_multiplayer_only_attacker_controller_triggers() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // p1 has an exalted creature (attacking player).
    let p1_exalted = ObjectSpec::creature(p1, "P1 Exalted", 1, 1)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);

    // p1 also has the lone attacker.
    let p1_attacker = ObjectSpec::creature(p1, "P1 Attacker", 2, 2).in_zone(ZoneId::Battlefield);

    // p3 also has an exalted creature (non-attacking player).
    let p3_exalted = ObjectSpec::creature(p3, "P3 Exalted", 1, 1)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(p1_exalted)
        .object(p1_attacker)
        .object(p3_exalted)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let p1_attacker_id = find_object(&state, "P1 Attacker");
    let p1_exalted_id = find_object(&state, "P1 Exalted");

    // p1 declares only their 2/2 as the lone attacker.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(p1_attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Exactly 1 AbilityTriggered event — only p1's exalted triggers, NOT p3's.
    let triggered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 1,
        "CR 702.83a: only p1's exalted should trigger (1 trigger, not 2)"
    );

    // The triggered ability is controlled by p1 and sourced from p1's exalted permanent.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { controller, source_object_id, .. }
            if *controller == p1 && *source_object_id == p1_exalted_id
        )),
        "CR 702.83a: the exalted trigger should be controlled by p1"
    );

    // Resolve: p1_attacker gets +1/+1.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let chars = calculate_characteristics(&state, p1_attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.83a: p1's attacker should get +1 from p1's exalted (not p3's)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.83a: p1's attacker should get +1 from p1's exalted (not p3's)"
    );
}

// ── Test 8: Empty attacker list does NOT trigger exalted ─────────────────────

#[test]
/// Edge case: If a player declares zero attackers, "attacks alone" is not satisfied
/// (CR 702.83b requires exactly one attacker). Exalted must NOT trigger.
fn test_exalted_with_zero_attackers_does_not_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let exalted_creature = ObjectSpec::creature(p1, "Exalted Bear", 2, 2)
        .with_keyword(KeywordAbility::Exalted)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(exalted_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    // p1 declares no attackers.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![],
        },
    )
    .expect("DeclareAttackers with empty list should succeed");

    // No exalted trigger (zero attackers ≠ "attacks alone").
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.83b: exalted must NOT trigger when zero attackers declared"
    );
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.83b: stack must be empty when no attackers declared"
    );
}
