//! Training keyword ability tests (CR 702.149).
//!
//! Training is a triggered ability: "Whenever this creature and at least one
//! other creature with power greater than this creature's power attack, put a
//! +1/+1 counter on this creature."
//!
//! Key rules verified:
//! - Trigger fires when attacking alongside a creature with greater power (CR 702.149a).
//! - The +1/+1 counter goes on the training creature itself (CR 702.149a).
//! - Does NOT trigger when no co-attacker has greater power (CR 702.149a).
//! - Does NOT trigger when attacking alone (CR 702.149a).
//! - Power comparison is strictly greater, not equal (CR 702.149a).
//! - Multiple instances each trigger separately (CR 702.149b).
//! - Multiple training creatures can each trigger from the same co-attacker.
//! - Multiplayer: co-attacker power check across all declared attackers.

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

// ── Test 1: Basic — attacks alongside creature with greater power ─────────────

#[test]
/// CR 702.149a — Training fires when attacking alongside a creature with
/// strictly greater power.
/// P1 has 1/2 Training creature and a 3/3 vanilla creature. Both attack P2.
/// After the trigger resolves, the Training creature has 1 +1/+1 counter
/// and its P/T is 2/3.
fn test_702_149a_training_basic_attacks_with_greater_power() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let training_creature = ObjectSpec::creature(p1, "Training Rookie", 1, 2)
        .with_keyword(KeywordAbility::Training)
        .in_zone(ZoneId::Battlefield);
    let big_ally = ObjectSpec::creature(p1, "Big Ally", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(training_creature)
        .object(big_ally)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let tr_id = find_object(&state, "Training Rookie");
    let ba_id = find_object(&state, "Big Ally");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (tr_id, AttackTarget::Player(p2)),
                (ba_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // AbilityTriggered event from the training source.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, controller, .. }
            if *source_object_id == tr_id && *controller == p1
        )),
        "CR 702.149a: AbilityTriggered event expected from the training creature"
    );

    // Stack has exactly 1 trigger (only the training creature triggered).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.149a: training trigger should be on the stack"
    );

    // Both players pass — trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Training creature has 1 +1/+1 counter.
    let obj = state
        .objects
        .get(&tr_id)
        .expect("Training Rookie on battlefield");
    let counter_count = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.149a: training creature should have 1 +1/+1 counter after trigger resolves"
    );

    // calculate_characteristics shows power=2, toughness=3.
    let chars =
        calculate_characteristics(&state, tr_id).expect("Training Rookie should be on battlefield");
    assert_eq!(
        chars.power,
        Some(2),
        "CR 702.149a: training creature power should be 2 (1+1)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.149a: training creature toughness should be 3 (2+1)"
    );
}

// ── Test 2: Does NOT trigger when attacking alone ─────────────────────────────

#[test]
/// CR 702.149a (negative: attacking alone) — Training does NOT fire when the
/// creature attacks without any co-attacker.
/// P1 has only 1/2 Training creature. Attacks P2 alone — no trigger.
fn test_702_149a_training_does_not_trigger_alone() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let training_creature = ObjectSpec::creature(p1, "Lone Trainee", 1, 2)
        .with_keyword(KeywordAbility::Training)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(training_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let tr_id = find_object(&state, "Lone Trainee");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(tr_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No AbilityTriggered from training source.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == tr_id
        )),
        "CR 702.149a: Training must NOT trigger when attacking alone"
    );

    // Stack is empty.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.149a: stack must be empty when training does not trigger"
    );

    // Creature has 0 counters.
    let obj = state
        .objects
        .get(&tr_id)
        .expect("Lone Trainee on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 702.149a: no +1/+1 counter when training did not trigger"
    );
}

// ── Test 3: Does NOT trigger when co-attacker has equal power ─────────────────

#[test]
/// CR 702.149a (negative: equal power) — Training requires STRICTLY greater power.
/// P1 has 2/2 Training creature and 2/2 vanilla. Both attack P2.
/// No trigger (2 is not greater than 2).
fn test_702_149a_training_does_not_trigger_equal_power() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let training_creature = ObjectSpec::creature(p1, "Equal Power Trainee", 2, 2)
        .with_keyword(KeywordAbility::Training)
        .in_zone(ZoneId::Battlefield);
    let equal_ally = ObjectSpec::creature(p1, "Equal Ally", 2, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(training_creature)
        .object(equal_ally)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let tr_id = find_object(&state, "Equal Power Trainee");
    let ea_id = find_object(&state, "Equal Ally");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (tr_id, AttackTarget::Player(p2)),
                (ea_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No AbilityTriggered from training source.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == tr_id
        )),
        "CR 702.149a: Training must NOT trigger when co-attacker has equal power (not strictly greater)"
    );

    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.149a: stack must be empty — equal power is not greater than"
    );

    let obj = state
        .objects
        .get(&tr_id)
        .expect("Equal Power Trainee on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 702.149a: no counter placed when co-attacker power equals training creature power"
    );
}

// ── Test 4: Does NOT trigger when co-attacker has lower power ─────────────────

#[test]
/// CR 702.149a (negative: lower power) — Training does NOT fire when all
/// co-attackers have lower or equal power.
/// P1 has 3/3 Training creature and 1/1 vanilla. Both attack.
/// No trigger (1 < 3, not greater than).
fn test_702_149a_training_does_not_trigger_lower_power() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let training_creature = ObjectSpec::creature(p1, "Strong Trainee", 3, 3)
        .with_keyword(KeywordAbility::Training)
        .in_zone(ZoneId::Battlefield);
    let weak_ally = ObjectSpec::creature(p1, "Weak Ally", 1, 1).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(training_creature)
        .object(weak_ally)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let tr_id = find_object(&state, "Strong Trainee");
    let wa_id = find_object(&state, "Weak Ally");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (tr_id, AttackTarget::Player(p2)),
                (wa_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No AbilityTriggered from training source.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == tr_id
        )),
        "CR 702.149a: Training must NOT trigger when co-attacker has lower power"
    );

    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.149a: stack must be empty — co-attacker power is lower than training creature power"
    );

    let obj = state
        .objects
        .get(&tr_id)
        .expect("Strong Trainee on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 702.149a: no counter placed when no co-attacker has greater power"
    );
}

// ── Test 5: Multiple instances trigger separately ─────────────────────────────

#[test]
/// CR 702.149b — "If a creature has multiple instances of training, each triggers
/// separately." A creature with two Training instances generates two triggers;
/// after both resolve, the creature has 2 +1/+1 counters.
fn test_702_149b_training_multiple_instances() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Creature with TWO Training keywords.
    let double_training = ObjectSpec::creature(p1, "Double Trainee", 1, 2)
        .with_keyword(KeywordAbility::Training)
        .with_keyword(KeywordAbility::Training)
        .in_zone(ZoneId::Battlefield);
    let big_ally = ObjectSpec::creature(p1, "Large Mentor", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(double_training)
        .object(big_ally)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dt_id = find_object(&state, "Double Trainee");
    let ba_id = find_object(&state, "Large Mentor");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (dt_id, AttackTarget::Player(p2)),
                (ba_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Two AbilityTriggered events — one per instance.
    let triggered_count = events
        .iter()
        .filter(|e| {
            matches!(e, GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == dt_id)
        })
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.149b: two Training instances should generate two separate triggers"
    );

    // Two triggers on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.149b: two training triggers should be on the stack"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature has 2 +1/+1 counters.
    let obj = state
        .objects
        .get(&dt_id)
        .expect("Double Trainee on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        2,
        "CR 702.149b: two training triggers should place 2 +1/+1 counters total"
    );

    // P/T = 3/4 (base 1/2 + 2 counters).
    let chars =
        calculate_characteristics(&state, dt_id).expect("Double Trainee should be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.149b: power should be 3 (1 base + 2 counters)"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 702.149b: toughness should be 4 (2 base + 2 counters)"
    );
}

// ── Test 6: Two Training creatures both trigger from same co-attacker ──────────

#[test]
/// CR 702.149a — Multiple training creatures can each trigger independently
/// when a single co-attacker with greater power attacks alongside them.
/// P1 has 1/1 "Trainee A" (Training), 2/2 "Trainee B" (Training), and a 4/4 vanilla.
/// All three attack P2.
/// Trainee A triggers (4 > 1), Trainee B triggers (4 > 2). Two triggers on stack.
fn test_702_149a_training_two_training_creatures_both_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let trainee_a = ObjectSpec::creature(p1, "Trainee A", 1, 1)
        .with_keyword(KeywordAbility::Training)
        .in_zone(ZoneId::Battlefield);
    let trainee_b = ObjectSpec::creature(p1, "Trainee B", 2, 2)
        .with_keyword(KeywordAbility::Training)
        .in_zone(ZoneId::Battlefield);
    let powerhouse = ObjectSpec::creature(p1, "Powerhouse", 4, 4).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(trainee_a)
        .object(trainee_b)
        .object(powerhouse)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let a_id = find_object(&state, "Trainee A");
    let b_id = find_object(&state, "Trainee B");
    let p_id = find_object(&state, "Powerhouse");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (a_id, AttackTarget::Player(p2)),
                (b_id, AttackTarget::Player(p2)),
                (p_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Both training creatures triggered.
    let a_triggered = events.iter().any(|e| {
        matches!(e, GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == a_id)
    });
    let b_triggered = events.iter().any(|e| {
        matches!(e, GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == b_id)
    });
    assert!(
        a_triggered,
        "CR 702.149a: Trainee A (1/1) should trigger (Powerhouse 4/4 > 1)"
    );
    assert!(
        b_triggered,
        "CR 702.149a: Trainee B (2/2) should trigger (Powerhouse 4/4 > 2)"
    );

    // Exactly 2 triggers on the stack (one per training creature).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.149a: two training triggers on the stack — one per training creature"
    );

    // Resolve both triggers.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Trainee A: 1 counter, P/T = 2/2.
    let a_counters = state
        .objects
        .get(&a_id)
        .expect("Trainee A on battlefield")
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        a_counters, 1,
        "CR 702.149a: Trainee A should have 1 +1/+1 counter"
    );

    let a_chars =
        calculate_characteristics(&state, a_id).expect("Trainee A should be on battlefield");
    assert_eq!(
        a_chars.power,
        Some(2),
        "CR 702.149a: Trainee A power should be 2 (1+1)"
    );
    assert_eq!(
        a_chars.toughness,
        Some(2),
        "CR 702.149a: Trainee A toughness should be 2 (1+1)"
    );

    // Trainee B: 1 counter, P/T = 3/3.
    let b_counters = state
        .objects
        .get(&b_id)
        .expect("Trainee B on battlefield")
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        b_counters, 1,
        "CR 702.149a: Trainee B should have 1 +1/+1 counter"
    );

    let b_chars =
        calculate_characteristics(&state, b_id).expect("Trainee B should be on battlefield");
    assert_eq!(
        b_chars.power,
        Some(3),
        "CR 702.149a: Trainee B power should be 3 (2+1)"
    );
    assert_eq!(
        b_chars.toughness,
        Some(3),
        "CR 702.149a: Trainee B toughness should be 3 (2+1)"
    );
}

// ── Test 7: Multiplayer — four player training trigger ────────────────────────

#[test]
/// CR 702.149a + multiplayer — Training checks co-attackers' power across all
/// declared attackers in the batch, regardless of which player they're attacking.
/// 4 players. P1 has 1/2 Training creature (attacks P2) and a 3/3 vanilla (attacks P3).
/// The training trigger still fires because 3/3 > 1 and both are in the attack declaration.
fn test_702_149a_training_multiplayer_four_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let training_creature = ObjectSpec::creature(p1, "Commander Trainee", 1, 2)
        .with_keyword(KeywordAbility::Training)
        .in_zone(ZoneId::Battlefield);
    let big_attacker = ObjectSpec::creature(p1, "Big Commander", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(training_creature)
        .object(big_attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let tr_id = find_object(&state, "Commander Trainee");
    let ba_id = find_object(&state, "Big Commander");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            // Training creature attacks P2; big attacker attacks P3.
            // They are in the same declare-attackers batch — power check applies.
            attackers: vec![
                (tr_id, AttackTarget::Player(p2)),
                (ba_id, AttackTarget::Player(p3)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Training trigger fires (3/3 co-attacker is greater than 1/2 training creature).
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, controller, .. }
            if *source_object_id == tr_id && *controller == p1
        )),
        "CR 702.149a: training trigger should fire in multiplayer when co-attacker has greater power"
    );

    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.149a: one training trigger on stack in multiplayer"
    );

    // All 4 players pass to resolve.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let obj = state
        .objects
        .get(&tr_id)
        .expect("Commander Trainee on battlefield");
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "CR 702.149a: training creature should get 1 +1/+1 counter in multiplayer"
    );

    let chars = calculate_characteristics(&state, tr_id)
        .expect("Commander Trainee should be on battlefield");
    assert_eq!(
        chars.power,
        Some(2),
        "CR 702.149a: multiplayer training — power should be 2 (1+1)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.149a: multiplayer training — toughness should be 3 (2+1)"
    );
}
