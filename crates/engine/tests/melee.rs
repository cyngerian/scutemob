//! Melee keyword ability tests (CR 702.121).
//!
//! Melee is a triggered ability: "Whenever this creature attacks, it gets
//! +1/+1 until end of turn for each opponent you attacked with a creature
//! this combat." (CR 702.121a)
//!
//! Key rules verified:
//! - Trigger fires when the creature attacks (CR 702.121a).
//! - Bonus = number of distinct opponents attacked with creatures (CR 702.121a).
//! - Only opponents (players) count, not planeswalkers (ruling 2016-08-23).
//! - Bonus computed at resolution time (ruling 2016-08-23).
//! - Multiple instances trigger separately (CR 702.121b).
//! - Bonus is +N/+N (both power AND toughness) until end of turn.
//! - No bonus if only attacking planeswalkers (no Player targets).
//! - Multiplayer: attacking 3 opponents gives +3/+3.

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

// ── Test 1: Basic 2-player: attack 1 opponent → +1/+1 ────────────────────────

#[test]
/// CR 702.121a — A 2/2 creature with Melee attacks the only opponent (P2).
/// The Melee trigger fires, resolves, and the creature gets +1/+1 (1 opponent attacked).
/// Final P/T = 3/3.
fn test_702_121a_melee_basic_one_opponent_attacked() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Melee Attacker", 2, 2).with_keyword(KeywordAbility::Melee),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Melee Attacker");

    // Declare attacker targeting P2.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // AbilityTriggered event should fire from the melee creature.
    assert!(
        declare_events.iter().any(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_id
            )
        }),
        "CR 702.121a: AbilityTriggered event should fire from melee creature"
    );

    // Trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.121a: Melee trigger should be on the stack after attackers declared"
    );

    // Resolve the trigger (both players pass).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved event should have been emitted.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "CR 702.121a: AbilityResolved event should fire after trigger resolves"
    );

    // Melee Attacker should now be 3/3 (+1/+1 for 1 opponent attacked).
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.121a: Melee attacked 1 opponent → +1 power (2+1=3)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.121a: Melee attacked 1 opponent → +1 toughness (2+1=3)"
    );

    // Stack should be empty.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after melee trigger resolves"
    );
}

// ── Test 2: Multiplayer, attack 2 opponents → +2/+2 ──────────────────────────

#[test]
/// CR 702.121a — In 4-player Commander, P1 has a 2/2 with Melee attacking P2,
/// and a vanilla creature attacking P3. Two distinct opponents are attacked.
/// Melee creature gets +2/+2, ending at 4/4.
fn test_702_121a_melee_multiplayer_two_opponents() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Melee Attacker", 2, 2).with_keyword(KeywordAbility::Melee),
        )
        .object(ObjectSpec::creature(p1, "Vanilla Helper", 3, 3))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let melee_id = find_object(&state, "Melee Attacker");
    let helper_id = find_object(&state, "Vanilla Helper");

    // P1 attacks: melee creature → P2, vanilla → P3.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (melee_id, AttackTarget::Player(p2)),
                (helper_id, AttackTarget::Player(p3)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Melee trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.121a: Melee trigger on the stack (one melee creature)"
    );

    // All 4 players pass to resolve.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Melee Attacker should be 4/4 (+2/+2 for 2 distinct opponents: P2 and P3).
    let chars = calculate_characteristics(&state, melee_id)
        .expect("melee attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(4),
        "CR 702.121a: Melee attacked 2 opponents → +2 power (2+2=4)"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 702.121a: Melee attacked 2 opponents → +2 toughness (2+2=4)"
    );
}

// ── Test 3: Full multiplayer: attack 3 opponents → +3/+3 ─────────────────────

#[test]
/// CR 702.121a — In 4-player Commander, P1 attacks all 3 opponents.
/// Melee creature is a 2/2; it attacks P2, while vanillas attack P3 and P4.
/// Distinct opponents attacked = 3 (P2, P3, P4).
/// Melee creature gets +3/+3, ending at 5/5.
fn test_702_121a_melee_multiplayer_three_opponents() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Melee Attacker", 2, 2).with_keyword(KeywordAbility::Melee),
        )
        .object(ObjectSpec::creature(p1, "Vanilla A", 2, 2))
        .object(ObjectSpec::creature(p1, "Vanilla B", 2, 2))
        .object(ObjectSpec::creature(p1, "Vanilla C", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let melee_id = find_object(&state, "Melee Attacker");
    let vanilla_a = find_object(&state, "Vanilla A");
    let vanilla_b = find_object(&state, "Vanilla B");
    let vanilla_c = find_object(&state, "Vanilla C");

    // P1 attacks all 3 opponents. Vanilla C also attacks P2 (duplicate — still 3 distinct).
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (melee_id, AttackTarget::Player(p2)),
                (vanilla_a, AttackTarget::Player(p3)),
                (vanilla_b, AttackTarget::Player(p4)),
                (vanilla_c, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Melee trigger on stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.121a: one Melee trigger on the stack"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Melee creature gets +3/+3 (3 distinct opponents: P2, P3, P4). Final P/T = 5/5.
    let chars = calculate_characteristics(&state, melee_id)
        .expect("melee attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(5),
        "CR 702.121a: Melee attacked 3 distinct opponents → +3 power (2+3=5)"
    );
    assert_eq!(
        chars.toughness,
        Some(5),
        "CR 702.121a: Melee attacked 3 distinct opponents → +3 toughness (2+3=5)"
    );
}

// ── Test 4: Attacking a planeswalker does not count toward bonus ───────────────

#[test]
/// Ruling 2016-08-23 — "Melee will trigger if the creature with melee attacks a
/// planeswalker. However, the effect counts only opponents (and not planeswalkers)
/// that you attacked with a creature when determining the bonus."
///
/// P1 has a 2/2 Melee creature that attacks a planeswalker controlled by P2.
/// The Melee trigger fires (creature attacked), but the bonus is +0/+0 because
/// no Player AttackTargets are present.
fn test_702_121a_melee_does_not_count_planeswalker_attacks() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let planeswalker = ObjectSpec::card(p2, "Test Planeswalker")
        .with_types(vec![mtg_engine::CardType::Planeswalker])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Melee Attacker", 2, 2).with_keyword(KeywordAbility::Melee),
        )
        .object(planeswalker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let melee_id = find_object(&state, "Melee Attacker");
    let pw_id = find_object(&state, "Test Planeswalker");

    // Attack the planeswalker (not the player directly).
    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(melee_id, AttackTarget::Planeswalker(pw_id))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Trigger DOES fire (melee triggers on any attack).
    assert!(
        declare_events.iter().any(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == melee_id
            )
        }),
        "CR 702.121a: Melee trigger fires even when attacking a planeswalker"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.121a: trigger should be on the stack"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // No bonus — only planeswalker attacked, not a player. P/T stays 2/2.
    let chars = calculate_characteristics(&state, melee_id)
        .expect("melee attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(2),
        "Ruling 2016-08-23: planeswalker attacks don't count → no bonus (power stays 2)"
    );
    assert_eq!(
        chars.toughness,
        Some(2),
        "Ruling 2016-08-23: planeswalker attacks don't count → no bonus (toughness stays 2)"
    );

    // No continuous effects registered (bonus was 0).
    assert!(
        state.continuous_effects.is_empty(),
        "Ruling 2016-08-23: no continuous effects when only planeswalker attacked"
    );
}

// ── Test 5: Multiple Melee instances trigger separately ───────────────────────

#[test]
/// CR 702.121b — "If a creature has multiple instances of melee, each triggers
/// separately." A 2/2 creature with two Melee keywords attacking P2 (1 opponent).
/// Two triggers fire. Each resolves to +1/+1. Final P/T = 4/4.
fn test_702_121b_melee_multiple_instances() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Double Melee", 2, 2)
                .with_keyword(KeywordAbility::Melee)
                .with_keyword(KeywordAbility::Melee),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let melee_id = find_object(&state, "Double Melee");

    // Declare attacker targeting P2.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(melee_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Two AbilityTriggered events (one per Melee instance, CR 702.121b).
    let triggered_count = declare_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == melee_id
            )
        })
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.121b: two Melee instances should generate two triggers"
    );

    // Two triggers on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.121b: two melee triggers on the stack"
    );

    // Resolve first trigger: creature gains +1/+1 → 3/3.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.stack_objects.len(),
        1,
        "one trigger should remain after first resolves"
    );

    let chars = calculate_characteristics(&state, melee_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.121b: after first Melee trigger: 2+1=3 power"
    );

    // Resolve second trigger: creature gains another +1/+1 → 4/4.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    let chars = calculate_characteristics(&state, melee_id).unwrap();
    assert_eq!(
        chars.power,
        Some(4),
        "CR 702.121b: after both Melee triggers: 2+1+1=4 power"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 702.121b: after both Melee triggers: 2+1+1=4 toughness"
    );
}

// ── Test 6: Source leaves battlefield before trigger resolves ─────────────────

#[test]
/// CR 603.10 + ruling 2016-08-23 — If the source leaves the battlefield before
/// the Melee trigger resolves, no bonus is applied. The trigger still resolves
/// as a no-op, no crash.
fn test_702_121a_melee_source_leaves_battlefield_no_bonus() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Melee Attacker", 2, 2).with_keyword(KeywordAbility::Melee),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Melee Attacker");

    // Declare attacker targeting P2. Trigger goes on stack.
    let (mut state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.121a: melee trigger should be on stack"
    );

    // Destroy the melee creature before the trigger resolves.
    let _ = state.move_object_to_zone(attacker_id, ZoneId::Graveyard(p1));

    // Source is no longer on battlefield — verify.
    assert!(
        state.objects.get(&attacker_id).is_none(),
        "attacker should no longer be at original ObjectId after zone move"
    );

    // Resolve the trigger — should be a no-op, no crash.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Trigger still emits AbilityResolved.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "CR 603.10: trigger resolves to a no-op even if source left battlefield"
    );

    // No continuous effects (source not on battlefield → no bonus applied).
    assert!(
        state.continuous_effects.is_empty(),
        "CR 603.10: no continuous effects when source left battlefield before resolution"
    );
}

// ── Test 7: Attacking alone still counts 1 opponent ──────────────────────────

#[test]
/// CR 702.121a — A sole attacker with Melee attacks the only opponent. This verifies
/// that attacking alone (sole creature) still correctly counts 1 opponent. Unlike
/// Exalted (which triggers on attacking alone), Melee triggers on any attack.
fn test_702_121a_melee_attacking_alone_still_counts() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Lone Melee Attacker", 2, 2)
                .with_keyword(KeywordAbility::Melee),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Lone Melee Attacker");

    // Sole attacker declares attack.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Trigger on stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.121a: Melee trigger fires when attacking alone"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature gets +1/+1 (1 opponent attacked). P/T = 3/3.
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.121a: attacking alone counts 1 opponent → +1 power (2+1=3)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.121a: attacking alone counts 1 opponent → +1 toughness (2+1=3)"
    );
}
