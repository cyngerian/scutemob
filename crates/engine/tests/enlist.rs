//! Enlist keyword ability tests (CR 702.154).
//!
//! Enlist is a static ability (optional attack cost) + triggered ability.
//! "As this creature attacks, you may tap an untapped non-attacking creature
//! you control without summoning sickness. When you do, this creature gets
//! +X/+0 until end of turn, where X is the enlisted creature's power."
//!
//! Key rules verified:
//! - Tapping an eligible creature adds its power as +X/+0 (CR 702.154a).
//! - Trigger uses the stack (ruling 2022-09-09).
//! - Enlisted creature must not be attacking (CR 702.154a).
//! - Enlisted creature must not have summoning sickness without haste (CR 702.154a).
//! - Cannot enlist self (CR 702.154c).
//! - A creature can only be enlisted once (ruling 2022-09-09).
//! - Multiple Enlist instances each tap a different creature (CR 702.154d).
//! - No enlist = no trigger (negative case).
//! - Multiplayer works correctly.

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

// ── Test 1: Basic power addition ──────────────────────────────────────────────

#[test]
/// CR 702.154a — Enlisting a 3/3 creature gives the attacker +3/+0 until EOT.
/// P1 has a 2/2 creature with Enlist and a 3/3 vanilla creature.
/// After enlisting the 3/3, the Enlist trigger resolves to give +3/+0.
/// Final power = 5 (2 base + 3 from enlisted creature).
fn test_702_154a_enlist_basic_power_addition() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let enlist_creature = ObjectSpec::creature(p1, "Enlist Creature", 2, 2)
        .with_keyword(KeywordAbility::Enlist)
        .in_zone(ZoneId::Battlefield);
    let vanilla = ObjectSpec::creature(p1, "Vanilla Ally", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enlist_creature)
        .object(vanilla)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let enlist_id = find_object(&state, "Enlist Creature");
    let vanilla_id = find_object(&state, "Vanilla Ally");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(enlist_id, AttackTarget::Player(p2))],
            enlist_choices: vec![(enlist_id, vanilla_id)],
        },
    )
    .expect("DeclareAttackers with enlist should succeed");

    // Vanilla creature is tapped.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTapped { object_id, .. }
            if *object_id == vanilla_id
        )),
        "CR 702.154a: vanilla creature should be tapped as enlist cost"
    );

    // Enlist trigger is on the stack.
    assert!(
        state.stack_objects.len() >= 1,
        "CR 702.154a: EnlistTrigger should be on the stack"
    );

    // Both players pass — trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Enlist creature power = 5 (2 base + 3 from enlisted).
    let chars =
        calculate_characteristics(&state, enlist_id).expect("Enlist Creature still on battlefield");
    assert_eq!(
        chars.power,
        Some(5),
        "CR 702.154a: Enlist creature power should be 5 (2 + 3 from enlisted)"
    );
    // Toughness unchanged.
    assert_eq!(
        chars.toughness,
        Some(2),
        "CR 702.154a: toughness should remain at 2"
    );

    // Vanilla creature is tapped.
    let vanilla_obj = state
        .objects
        .get(&vanilla_id)
        .expect("Vanilla Ally on battlefield");
    assert!(
        vanilla_obj.status.tapped,
        "CR 702.154a: enlisted creature should remain tapped"
    );
}

// ── Test 2: No enlist choice = no trigger ─────────────────────────────────────

#[test]
/// CR 702.154a (negative) — If the player doesn't use the enlist ability,
/// no trigger fires and no creature is tapped.
fn test_702_154a_enlist_no_choice_no_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let enlist_creature = ObjectSpec::creature(p1, "Enlist Creature", 2, 2)
        .with_keyword(KeywordAbility::Enlist)
        .in_zone(ZoneId::Battlefield);
    let vanilla = ObjectSpec::creature(p1, "Vanilla Ally", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enlist_creature)
        .object(vanilla)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let enlist_id = find_object(&state, "Enlist Creature");
    let vanilla_id = find_object(&state, "Vanilla Ally");

    let (state, _events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(enlist_id, AttackTarget::Player(p2))],
            enlist_choices: vec![], // No enlist choice
        },
    )
    .expect("DeclareAttackers with no enlist should succeed");

    // No EnlistTrigger on the stack (there may be other triggers, but no enlist ones).
    use mtg_engine::state::stack::StackObjectKind;
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|obj| { matches!(obj.kind, StackObjectKind::EnlistTrigger { .. }) }),
        "CR 702.154a: no EnlistTrigger should be on stack when enlist choice not made"
    );

    // Vanilla creature is NOT tapped.
    let vanilla_obj = state
        .objects
        .get(&vanilla_id)
        .expect("Vanilla Ally on battlefield");
    assert!(
        !vanilla_obj.status.tapped,
        "CR 702.154a: creature should not be tapped when enlist not used"
    );

    // Enlist creature power is still base 2.
    let chars =
        calculate_characteristics(&state, enlist_id).expect("Enlist Creature still on battlefield");
    assert_eq!(
        chars.power,
        Some(2),
        "CR 702.154a: Enlist creature power should remain 2 when no enlist used"
    );
}

// ── Test 3: Enlisted creature must not be attacking ───────────────────────────

#[test]
/// CR 702.154a — An attacking creature cannot be enlisted.
/// P1 declares both creatures as attackers, then tries to enlist the
/// second creature. This should fail validation.
fn test_702_154a_enlist_enlisted_must_not_be_attacking() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let enlist_creature = ObjectSpec::creature(p1, "Enlist Creature", 2, 2)
        .with_keyword(KeywordAbility::Enlist)
        .in_zone(ZoneId::Battlefield);
    let also_attacking =
        ObjectSpec::creature(p1, "Also Attacking", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enlist_creature)
        .object(also_attacking)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let enlist_id = find_object(&state, "Enlist Creature");
    let also_id = find_object(&state, "Also Attacking");

    // Try to enlist a creature that is also declared as an attacker.
    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (enlist_id, AttackTarget::Player(p2)),
                (also_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![(enlist_id, also_id)],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.154a: enlisting an attacker should be rejected"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("attacker"),
        "Error should mention 'attacker', got: {}",
        err_msg
    );
}

// ── Test 4: Summoning sickness rejected ───────────────────────────────────────

#[test]
/// CR 702.154a — A creature with summoning sickness (and no haste) cannot be
/// enlisted. The command should be rejected.
fn test_702_154a_enlist_summoning_sickness_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let enlist_creature = ObjectSpec::creature(p1, "Enlist Creature", 2, 2)
        .with_keyword(KeywordAbility::Enlist)
        .in_zone(ZoneId::Battlefield);
    let sick_creature =
        ObjectSpec::creature(p1, "Sick Creature", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enlist_creature)
        .object(sick_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let enlist_id = find_object(&state, "Enlist Creature");
    let sick_id = find_object(&state, "Sick Creature");

    // Set summoning sickness on the creature to enlist.
    if let Some(obj) = state.objects.get_mut(&sick_id) {
        obj.has_summoning_sickness = true;
    }

    // Try to enlist the sick creature.
    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(enlist_id, AttackTarget::Player(p2))],
            enlist_choices: vec![(enlist_id, sick_id)],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.154a: enlisting a creature with summoning sickness should be rejected"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.to_lowercase().contains("summoning sickness"),
        "Error should mention 'summoning sickness', got: {}",
        err_msg
    );
}

// ── Test 5: Summoning sickness + haste allowed ────────────────────────────────

#[test]
/// CR 702.154a — A creature with summoning sickness but Haste CAN be enlisted.
/// The creature must have either haste or have been under control continuously.
fn test_702_154a_enlist_summoning_sickness_with_haste_allowed() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let enlist_creature = ObjectSpec::creature(p1, "Enlist Creature", 2, 2)
        .with_keyword(KeywordAbility::Enlist)
        .in_zone(ZoneId::Battlefield);
    let hasty_sick = ObjectSpec::creature(p1, "Hasty Sick", 3, 3)
        .with_keyword(KeywordAbility::Haste)
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enlist_creature)
        .object(hasty_sick)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let enlist_id = find_object(&state, "Enlist Creature");
    let hasty_id = find_object(&state, "Hasty Sick");

    // Set summoning sickness + haste.
    if let Some(obj) = state.objects.get_mut(&hasty_id) {
        obj.has_summoning_sickness = true;
        // Haste is already set via keyword; the engine uses calculate_characteristics
        // to check haste at validation time.
    }

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(enlist_id, AttackTarget::Player(p2))],
            enlist_choices: vec![(enlist_id, hasty_id)],
        },
    )
    .expect("CR 702.154a: enlisting hasty creature with summoning sickness should succeed");

    // Hasty creature is tapped.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTapped { object_id, .. }
            if *object_id == hasty_id
        )),
        "CR 702.154a: hasty sick creature should be tapped as enlist cost"
    );

    // Trigger resolves to +3/+0.
    let (state, _) = pass_all(state, &[p1, p2]);
    let chars =
        calculate_characteristics(&state, enlist_id).expect("Enlist Creature on battlefield");
    assert_eq!(
        chars.power,
        Some(5),
        "CR 702.154a: enlist creature power should be 5 (2 + 3) after trigger"
    );
}

// ── Test 6: Cannot enlist itself ──────────────────────────────────────────────

#[test]
/// CR 702.154c — A creature cannot enlist itself.
fn test_702_154c_enlist_cannot_enlist_self() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let enlist_creature = ObjectSpec::creature(p1, "Enlist Creature", 3, 3)
        .with_keyword(KeywordAbility::Enlist)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enlist_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Enlist Creature");

    // Try to enlist itself.
    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(creature_id, AttackTarget::Player(p2))],
            enlist_choices: vec![(creature_id, creature_id)],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.154c: creature cannot enlist itself"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.to_lowercase().contains("702.154c") || err_msg.to_lowercase().contains("itself"),
        "Error should mention CR 702.154c or 'itself', got: {}",
        err_msg
    );
}

// ── Test 7: Creature can only be enlisted once ─────────────────────────────────

#[test]
/// Ruling 2022-09-09 — A single creature cannot be tapped for more than one
/// enlist ability. Trying to enlist the same creature for two attackers with
/// Enlist is rejected.
fn test_702_154_enlist_creature_used_once_only() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let enlist1 = ObjectSpec::creature(p1, "Enlist Attacker 1", 2, 2)
        .with_keyword(KeywordAbility::Enlist)
        .in_zone(ZoneId::Battlefield);
    let enlist2 = ObjectSpec::creature(p1, "Enlist Attacker 2", 2, 2)
        .with_keyword(KeywordAbility::Enlist)
        .in_zone(ZoneId::Battlefield);
    let big_target = ObjectSpec::creature(p1, "Big Target", 4, 4).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enlist1)
        .object(enlist2)
        .object(big_target)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let e1_id = find_object(&state, "Enlist Attacker 1");
    let e2_id = find_object(&state, "Enlist Attacker 2");
    let big_id = find_object(&state, "Big Target");

    // Both attackers try to enlist the same creature.
    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (e1_id, AttackTarget::Player(p2)),
                (e2_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![(e1_id, big_id), (e2_id, big_id)],
        },
    );

    assert!(
        result.is_err(),
        "Ruling 2022-09-09: same creature cannot be enlisted by two attackers"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.to_lowercase().contains("already enlisted"),
        "Error should mention 'already enlisted', got: {}",
        err_msg
    );
}

// ── Test 8: Multiplayer (4-player) ────────────────────────────────────────────

#[test]
/// CR 702.154a + multiplayer — Enlist works correctly in 4-player games.
/// P1 has 1/1 creature with Enlist and a 5/5 vanilla creature.
/// P1 declares Enlist creature attacking P2, enlists the 5/5.
/// After trigger resolves, Enlist creature's power = 6 (1 + 5).
fn test_702_154a_enlist_multiplayer_four_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let enlist_creature = ObjectSpec::creature(p1, "Enlist Creature", 1, 1)
        .with_keyword(KeywordAbility::Enlist)
        .in_zone(ZoneId::Battlefield);
    let big_ally = ObjectSpec::creature(p1, "Big Ally", 5, 5).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(enlist_creature)
        .object(big_ally)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let enlist_id = find_object(&state, "Enlist Creature");
    let big_id = find_object(&state, "Big Ally");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(enlist_id, AttackTarget::Player(p2))],
            enlist_choices: vec![(enlist_id, big_id)],
        },
    )
    .expect("4-player enlist should succeed");

    // Big ally is tapped.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTapped { object_id, .. }
            if *object_id == big_id
        )),
        "CR 702.154a: big ally should be tapped"
    );

    // All 4 players pass to resolve trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Enlist creature power = 6 (1 + 5).
    let chars =
        calculate_characteristics(&state, enlist_id).expect("Enlist Creature on battlefield");
    assert_eq!(
        chars.power,
        Some(6),
        "CR 702.154a: Enlist creature power should be 6 (1 + 5) in 4-player game"
    );

    // 5/5 creature is tapped.
    let big_obj = state.objects.get(&big_id).expect("Big Ally on battlefield");
    assert!(
        big_obj.status.tapped,
        "CR 702.154a: enlisted creature should be tapped"
    );
}
