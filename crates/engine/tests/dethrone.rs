//! Dethrone keyword ability tests (CR 702.105).
//!
//! Dethrone is a triggered ability: "Whenever this creature attacks the player
//! with the most life or tied for most life, put a +1/+1 counter on this creature."
//!
//! Key rules verified:
//! - Trigger fires when attacking the player with the most life (CR 702.105a).
//! - Trigger fires when attacking a player TIED for most life (CR 702.105a).
//! - The +1/+1 counter goes on the dethrone creature itself (CR 702.105a).
//! - Does NOT trigger when attacking a player who does not have most life (CR 702.105a).
//! - Does NOT trigger when attacking a planeswalker (ruling 2023-07-28).
//! - Multiple instances each trigger separately (CR 702.105b).
//! - Multiplayer: most life is compared among all active players (CR 702.105a).

use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command, CounterType,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step,
    ZoneId,
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

// ── Test 1: Basic — attacks player with most life ─────────────────────────────

#[test]
/// CR 702.105a — Dethrone fires when attacking the player with the most life.
/// P1 at 20, P2 at 25 (most life). P1's 2/2 dethrone creature attacks P2.
/// After the trigger resolves, the creature has 1 +1/+1 counter and is 3/3.
fn test_dethrone_basic_attacks_player_with_most_life() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dethrone_creature = ObjectSpec::creature(p1, "Dethrone Warrior", 2, 2)
        .with_keyword(KeywordAbility::Dethrone)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_life(p1, 20)
        .player_life(p2, 25)
        .with_registry(CardRegistry::new(vec![]))
        .object(dethrone_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dt_id = find_object(&state, "Dethrone Warrior");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(dt_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // AbilityTriggered event from the dethrone source.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, controller, .. }
            if *source_object_id == dt_id && *controller == p1
        )),
        "CR 702.105a: AbilityTriggered event expected from the dethrone creature"
    );

    // Stack has 1 trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.105a: dethrone trigger should be on the stack"
    );

    // Both players pass — trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature has 1 +1/+1 counter.
    let obj = state
        .objects
        .get(&dt_id)
        .expect("Dethrone Warrior on battlefield");
    let counter_count = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.105a: dethrone creature should have 1 +1/+1 counter after trigger resolves"
    );

    // calculate_characteristics shows power=3, toughness=3.
    let chars = calculate_characteristics(&state, dt_id)
        .expect("Dethrone Warrior should be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.105a: dethrone creature power should be 3 (2+1)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.105a: dethrone creature toughness should be 3 (2+1)"
    );
}

// ── Test 2: Tied for most life ────────────────────────────────────────────────

#[test]
/// CR 702.105a "or tied for most life" — trigger fires when the defending player
/// is tied for the highest life total in the game.
/// P1 at 20, P2 at 20 (tied). P1 attacks P2 — trigger should fire.
fn test_dethrone_tied_for_most_life() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dethrone_creature = ObjectSpec::creature(p1, "Tied Dethrone", 2, 2)
        .with_keyword(KeywordAbility::Dethrone)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_life(p1, 20)
        .player_life(p2, 20)
        .with_registry(CardRegistry::new(vec![]))
        .object(dethrone_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dt_id = find_object(&state, "Tied Dethrone");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(dt_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Trigger fires because P2 is tied for most life.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.105a: dethrone trigger fires when attacking a player tied for most life"
    );

    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.105a: trigger should be on the stack"
    );

    // Resolve trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature has 1 +1/+1 counter; P/T = 3/3.
    let obj = state
        .objects
        .get(&dt_id)
        .expect("Tied Dethrone on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "CR 702.105a: dethrone creature should have 1 +1/+1 counter (tied for most life case)"
    );

    let chars =
        calculate_characteristics(&state, dt_id).expect("Tied Dethrone should be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.105a: power should be 3 after +1/+1 counter"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.105a: toughness should be 3 after +1/+1 counter"
    );
}

// ── Test 3: Does NOT trigger against lower life ───────────────────────────────

#[test]
/// CR 702.105a (negative case) — dethrone does NOT fire when attacking a player
/// who has strictly less life than another player.
/// P1 at 25 (most), P2 at 15. P1 attacks P2 — no trigger.
fn test_dethrone_does_not_trigger_against_lower_life() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dethrone_creature = ObjectSpec::creature(p1, "No Trigger Dethrone", 2, 2)
        .with_keyword(KeywordAbility::Dethrone)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_life(p1, 25)
        .player_life(p2, 15)
        .with_registry(CardRegistry::new(vec![]))
        .object(dethrone_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dt_id = find_object(&state, "No Trigger Dethrone");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(dt_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No AbilityTriggered from dethrone source.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == dt_id
        )),
        "CR 702.105a: dethrone must NOT trigger when attacking a player with less life"
    );

    // Stack is empty.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.105a: stack must be empty when dethrone does not trigger"
    );

    // Creature has 0 counters.
    let obj = state
        .objects
        .get(&dt_id)
        .expect("No Trigger Dethrone on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 702.105a: no +1/+1 counter when dethrone did not trigger"
    );
}

// ── Test 4: Multiplayer — attacks player tied for most life ───────────────────

#[test]
/// CR 702.105a — multiplayer: most life is compared among ALL active players.
/// 4 players: P1=30, P2=40, P3=40, P4=20.
/// P1 attacks P2 (tied for most at 40) — trigger fires.
fn test_dethrone_multiplayer_four_player_most_life() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let dethrone_creature = ObjectSpec::creature(p1, "Commander Dethrone", 2, 2)
        .with_keyword(KeywordAbility::Dethrone)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .player_life(p1, 30)
        .player_life(p2, 40)
        .player_life(p3, 40)
        .player_life(p4, 20)
        .with_registry(CardRegistry::new(vec![]))
        .object(dethrone_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dt_id = find_object(&state, "Commander Dethrone");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            // P2 is tied for most life (40); attack them.
            attackers: vec![(dt_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Trigger fires because P2 is tied for most life.
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.105a: dethrone trigger fires in multiplayer when attacking a player tied for most life"
    );

    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.105a: one dethrone trigger on the stack"
    );

    // All 4 players pass to resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let obj = state
        .objects
        .get(&dt_id)
        .expect("Commander Dethrone on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "CR 702.105a: dethrone creature should get +1/+1 counter in multiplayer"
    );

    let chars = calculate_characteristics(&state, dt_id)
        .expect("Commander Dethrone should be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.105a: power should be 3 after trigger"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.105a: toughness should be 3 after trigger"
    );
}

// ── Test 5: Multiplayer — attacks player NOT with most life ───────────────────

#[test]
/// CR 702.105a (negative, multiplayer) — dethrone does NOT trigger when attacking
/// a player who does not have the most life in a 4-player game.
/// 4 players: P1=30, P2=25, P3=40, P4=35.
/// P1 attacks P2 (25, not most — P3 has 40) — no trigger.
fn test_dethrone_multiplayer_not_most_life() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let dethrone_creature = ObjectSpec::creature(p1, "Quiet Dethrone", 2, 2)
        .with_keyword(KeywordAbility::Dethrone)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .player_life(p1, 30)
        .player_life(p2, 25)
        .player_life(p3, 40)
        .player_life(p4, 35)
        .with_registry(CardRegistry::new(vec![]))
        .object(dethrone_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dt_id = find_object(&state, "Quiet Dethrone");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            // P2 has only 25 life; P3 has 40 (the max). Dethrone should NOT fire.
            attackers: vec![(dt_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No AbilityTriggered from the dethrone creature.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == dt_id
        )),
        "CR 702.105a: dethrone must NOT trigger when attacking a player with less than max life"
    );

    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.105a: stack must be empty — no dethrone trigger in multiplayer non-max case"
    );

    let obj = state
        .objects
        .get(&dt_id)
        .expect("Quiet Dethrone on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 702.105a: no counter placed when dethrone did not trigger"
    );
}

// ── Test 6: Multiple instances trigger separately ─────────────────────────────

#[test]
/// CR 702.105b — "If a creature has multiple instances of dethrone, each triggers
/// separately." A creature with two dethrone instances generates two triggers;
/// after both resolve, the creature has 2 +1/+1 counters.
fn test_dethrone_multiple_instances_trigger_separately() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let double_dethrone = ObjectSpec::creature(p1, "Double Dethrone", 2, 2)
        .with_keyword(KeywordAbility::Dethrone)
        .with_keyword(KeywordAbility::Dethrone)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_life(p1, 20)
        .player_life(p2, 25)
        .with_registry(CardRegistry::new(vec![]))
        .object(double_dethrone)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dt_id = find_object(&state, "Double Dethrone");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(dt_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Two AbilityTriggered events — one per instance.
    let triggered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == dt_id))
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.105b: two dethrone instances should generate two separate triggers"
    );

    // Two triggers on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.105b: two dethrone triggers should be on the stack"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature should have 2 +1/+1 counters.
    let obj = state
        .objects
        .get(&dt_id)
        .expect("Double Dethrone on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        2,
        "CR 702.105b: two dethrone triggers should place 2 +1/+1 counters"
    );

    // P/T should be 4/4 (base 2/2 + 2 counters).
    let chars =
        calculate_characteristics(&state, dt_id).expect("Double Dethrone should be on battlefield");
    assert_eq!(
        chars.power,
        Some(4),
        "CR 702.105b: power should be 4 (2 base + 2 counters)"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 702.105b: toughness should be 4 (2 base + 2 counters)"
    );
}

// ── Test 7: Does NOT trigger on planeswalker attack ───────────────────────────

#[test]
/// Ruling 2023-07-28 — "Dethrone doesn't trigger if the creature attacks a
/// planeswalker, even if its controller has the most life."
/// P2 at 25 (most life) controls a planeswalker token.
/// P1 attacks the planeswalker — no dethrone trigger.
fn test_dethrone_does_not_trigger_on_planeswalker_attack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dethrone_creature = ObjectSpec::creature(p1, "Planeswalker Attacker", 2, 2)
        .with_keyword(KeywordAbility::Dethrone)
        .in_zone(ZoneId::Battlefield);

    // Create a planeswalker token owned by P2. We use ObjectSpec::card to set it up
    // in the Battlefield zone controlled by P2. The planeswalker is a stand-in target.
    let planeswalker = ObjectSpec::card(p2, "Test Planeswalker")
        .with_types(vec![mtg_engine::CardType::Planeswalker])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_life(p1, 15)
        .player_life(p2, 25)
        .with_registry(CardRegistry::new(vec![]))
        .object(dethrone_creature)
        .object(planeswalker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dt_id = find_object(&state, "Planeswalker Attacker");
    let pw_id = find_object(&state, "Test Planeswalker");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            // Attack the planeswalker, NOT the player directly.
            attackers: vec![(dt_id, AttackTarget::Planeswalker(pw_id))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No dethrone trigger fires for a planeswalker attack.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == dt_id
        )),
        "Ruling 2023-07-28: dethrone must NOT trigger when attacking a planeswalker"
    );

    assert_eq!(
        state.stack_objects.len(),
        0,
        "Ruling 2023-07-28: stack must be empty — no dethrone trigger on planeswalker attack"
    );

    let obj = state
        .objects
        .get(&dt_id)
        .expect("Planeswalker Attacker on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "Ruling 2023-07-28: no counter placed when attacking a planeswalker"
    );
}

// ── Test 8: Attacker has most life, attacks player with lower life ─────────────

#[test]
/// CR 702.105a (self-life edge case) — dethrone checks whether the DEFENDING player
/// has the most life, not the attacking player.
/// P1 at 30 (most life), P2 at 20.
/// P1 attacks P2 with dethrone creature — no trigger (P2 does NOT have most life).
fn test_dethrone_attacker_has_most_life_attacks_lower() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dethrone_creature = ObjectSpec::creature(p1, "Self Most Life", 2, 2)
        .with_keyword(KeywordAbility::Dethrone)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_life(p1, 30)
        .player_life(p2, 20)
        .with_registry(CardRegistry::new(vec![]))
        .object(dethrone_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dt_id = find_object(&state, "Self Most Life");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            // P1 has most life (30), but P1 is the ATTACKER, not the defender.
            // P2 only has 20 life — no trigger.
            attackers: vec![(dt_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No AbilityTriggered from the dethrone creature.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == dt_id
        )),
        "CR 702.105a: dethrone does NOT trigger when the attacker has most life but the defender does not"
    );

    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.105a: stack must be empty — dethrone checks defending player's life, not attacker's"
    );

    let obj = state
        .objects
        .get(&dt_id)
        .expect("Self Most Life on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 702.105a: no counter placed — attacker's life is irrelevant to dethrone"
    );
}
