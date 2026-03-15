//! Poisonous keyword ability enforcement tests (CR 702.70).
//!
//! Poisonous is a triggered ability: "Whenever this creature deals combat damage
//! to a player, that player gets N poison counters."
//!
//! Unlike Infect (which replaces damage with poison counters), Poisonous is additive:
//! the creature deals its normal combat damage AND the trigger gives poison counters.
//!
//! Tests:
//! - Basic trigger fires on unblocked damage to player; N poison counters given (CR 702.70a)
//! - Amount is fixed (N), not based on damage dealt (Virulent Sliver ruling 2021-03-19)
//! - Trigger does NOT fire when creature is blocked (CR 702.70a + CR 510.1c)
//! - Multiple instances trigger separately, each giving their N (CR 702.70b)
//! - 10+ poison counters triggers SBA loss (CR 104.3d / CR 704.5c)
//! - Multiplayer: trigger targets the specific damaged player (CR 702.70a)

use mtg_engine::{
    process_command, AbilityDefinition, AttackTarget, CardDefinition, CardId, CardRegistry,
    CardType, Command, GameEvent, GameStateBuilder, KeywordAbility, ObjectSpec, PlayerId, Step,
    TypeLine,
};

// ── Helper: find object ID by name ───────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

// ── Helper: pass all priority (2 players) ────────────────────────────────────

fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &p in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: p })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", p, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

// ── Helper: poison counter count for a player ────────────────────────────────

fn poison_counters(state: &mtg_engine::GameState, player: PlayerId) -> u32 {
    state
        .players
        .get(&player)
        .map(|p| p.poison_counters)
        .unwrap_or_else(|| panic!("player {:?} not found", player))
}

// ── CR 702.70: Poisonous ─────────────────────────────────────────────────────

#[test]
/// CR 702.70a — basic trigger: "Whenever this creature deals combat damage to a
/// player, that player gets N poison counters."
///
/// An unblocked 2/2 creature with Poisonous 1 deals 2 combat damage to P2.
/// P2 loses 2 life (normal combat damage) AND gets 1 poison counter (trigger).
/// This verifies Poisonous is additive, not a replacement like Infect.
fn test_702_70a_poisonous_basic_gives_poison_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Poisonous Viper", 2, 2)
                .with_keyword(KeywordAbility::Poisonous(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Poisonous Viper");

    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        40,
        "setup: p2 starts at 40 life"
    );
    assert_eq!(
        poison_counters(&state, p2),
        0,
        "setup: p2 starts with 0 poison counters"
    );

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step — damage is dealt, Poisonous trigger fires.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // CR 702.70a: P2 took 2 combat damage (life 40 - 2 = 38).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "CR 702.70a: p2 should have taken 2 combat damage (life 40 → 38)"
    );

    // AbilityTriggered event should have been emitted for the poisonous source.
    let triggered = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        triggered,
        "CR 702.70a: AbilityTriggered event should fire for poisonous creature"
    );

    // The poisonous trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.70a: Poisonous trigger should be on the stack"
    );

    // Resolve the trigger (both players pass priority).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved event should have been emitted.
    let resolved = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityResolved { .. }));
    assert!(
        resolved,
        "CR 702.70a: AbilityResolved event should fire after trigger resolves"
    );

    // P2 now has 1 poison counter.
    assert_eq!(
        poison_counters(&state, p2),
        1,
        "CR 702.70a: p2 should have 1 poison counter after Poisonous 1 trigger resolves"
    );

    // PoisonCountersGiven event should have been emitted.
    let poison_given = resolve_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::PoisonCountersGiven { player, amount, .. }
            if *player == p2 && *amount == 1
        )
    });
    assert!(
        poison_given,
        "CR 702.70a: PoisonCountersGiven event should fire with player=p2, amount=1"
    );

    // Stack is empty after resolution.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after Poisonous trigger resolves"
    );

    // P1 is unaffected.
    assert_eq!(
        poison_counters(&state, p1),
        0,
        "CR 702.70a: p1 (attacker) should have 0 poison counters"
    );
}

#[test]
/// CR 702.70a + Virulent Sliver ruling 2021-03-19 — The N value is fixed,
/// regardless of how much combat damage was dealt.
///
/// A 5/5 creature with Poisonous 1 deals 5 combat damage to P2.
/// P2 loses 5 life BUT gets exactly 1 poison counter (not 5).
fn test_702_70a_poisonous_amount_independent_of_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Giant Snake", 5, 5)
                .with_keyword(KeywordAbility::Poisonous(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Giant Snake");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step — trigger fires.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Verify life loss (5 damage from the 5/5).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        35,
        "CR 702.70a: p2 should have taken 5 combat damage (life 40 → 35)"
    );

    // Trigger is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Poisonous 1 trigger should be on the stack"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 gets exactly 1 poison counter (N=1), NOT 5.
    assert_eq!(
        poison_counters(&state, p2),
        1,
        "Virulent Sliver ruling 2021-03-19: p2 should get exactly 1 poison counter \
         (Poisonous 1), not 5 (the damage amount)"
    );
}

#[test]
/// CR 702.70a + CR 510.1c — Poisonous does NOT trigger when the creature is
/// blocked (because a blocked creature without trample deals no damage to the
/// defending player).
fn test_702_70a_poisonous_blocked_no_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Poisonous Creature", 2, 2)
                .with_keyword(KeywordAbility::Poisonous(1)),
        )
        .object(ObjectSpec::creature(p2, "Big Blocker", 3, 3))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Poisonous Creature");
    let blocker_id = find_object(&state, "Big Blocker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with the 3/3.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step — all damage goes to blocker.
    let (state, combat_events) = pass_all(state, &[p1, p2]);

    // No AbilityTriggered event for the poisonous creature.
    let triggered = combat_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        !triggered,
        "CR 702.70a: Poisonous should NOT trigger when creature is blocked \
         (no combat damage dealt to the player)"
    );

    // Stack is empty — no poisonous trigger.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty — Poisonous did not trigger (blocked creature)"
    );

    // P2 has 0 poison counters.
    assert_eq!(
        poison_counters(&state, p2),
        0,
        "CR 702.70a: p2 should have 0 poison counters when poisonous creature is blocked"
    );

    // P2's life total is unchanged (the attack was fully blocked).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        40,
        "p2's life should be unchanged (attack was fully blocked)"
    );
}

#[test]
/// CR 702.70b — Multiple instances of Poisonous trigger separately.
///
/// A single creature with both Poisonous(1) and Poisonous(2) in its
/// CardDefinition generates TWO triggers when it deals combat damage to a
/// player. After both resolve, the player has 3 poison counters (1 + 2).
fn test_702_70b_poisonous_multiple_instances_trigger_separately() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build a CardDefinition with TWO Poisonous keyword entries.
    let double_poison_def = CardDefinition {
        card_id: CardId("double-poison-creature".to_string()),
        name: "Double Poison Creature".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Poisonous 1\nPoisonous 2".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Poisonous(1)),
            AbilityDefinition::Keyword(KeywordAbility::Poisonous(2)),
        ],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    };

    let registry = CardRegistry::new(vec![double_poison_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        // Creature with Poisonous 1 + Poisonous 2 via card_id.
        // The keyword on ObjectSpec enables the fallback lookup; card_id enables
        // the primary lookup from card def (two entries).
        .object(
            ObjectSpec::creature(p1, "Double Poison Creature", 1, 1)
                .with_keyword(KeywordAbility::Poisonous(1))
                .with_card_id(CardId("double-poison-creature".to_string())),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Poison Creature");

    assert_eq!(
        poison_counters(&state, p2),
        0,
        "setup: p2 starts with 0 poison counters"
    );

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step — TWO triggers fire (one for Poisonous 1,
    // one for Poisonous 2).
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // Two AbilityTriggered events from the SAME creature (CR 702.70b).
    let triggered_count = damage_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_id
            )
        })
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.70b: a single creature with Poisonous 1 + Poisonous 2 should fire \
         TWO separate triggers"
    );

    // Two triggers on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.70b: 2 poisonous triggers should be on the stack"
    );

    // Resolve first trigger (Poisonous 2, LIFO order — last in, first out).
    let (state, first_resolve) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.stack_objects.len(),
        1,
        "one trigger should remain after resolving the first"
    );

    // First trigger gave some poison counters (either 1 or 2, LIFO).
    let poison_after_first = poison_counters(&state, p2);
    assert!(
        poison_after_first == 1 || poison_after_first == 2,
        "CR 702.70b: first trigger should give either 1 or 2 poison counters (LIFO order)"
    );

    // PoisonCountersGiven event should have fired.
    let poison_given_first = first_resolve
        .iter()
        .any(|e| matches!(e, GameEvent::PoisonCountersGiven { player, .. } if *player == p2));
    assert!(
        poison_given_first,
        "CR 702.70b: PoisonCountersGiven event should fire for first trigger"
    );

    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    // After both triggers resolve, P2 should have 1 + 2 = 3 poison counters total.
    assert_eq!(
        poison_counters(&state, p2),
        3,
        "CR 702.70b: p2 should have 3 poison counters (1 + 2) after both triggers resolve"
    );
}

#[test]
/// CR 104.3d + CR 704.5c — A player with 10 or more poison counters loses the
/// game as a state-based action.
///
/// P2 starts with 9 poison counters. P1's Poisonous 1 creature deals combat
/// damage to P2. The trigger fires and resolves: P2 gets 1 more poison counter
/// (total = 10). The SBA check marks P2 as having lost.
fn test_702_70a_poisonous_kills_via_sba() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        // P2 starts with 9 poison counters.
        .player_poison(p2, 9)
        .object(
            ObjectSpec::creature(p1, "Lethal Viper", 1, 1)
                .with_keyword(KeywordAbility::Poisonous(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Lethal Viper");

    assert_eq!(
        poison_counters(&state, p2),
        9,
        "setup: p2 starts with 9 poison counters"
    );
    assert!(
        !state.players.get(&p2).unwrap().has_lost,
        "setup: p2 has not yet lost"
    );

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step — trigger fires.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Trigger is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Poisonous 1 trigger should be on the stack"
    );

    // Resolve the trigger — P2 gets 1 more poison counter (total = 10).
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 now has 10 poison counters.
    assert_eq!(
        poison_counters(&state, p2),
        10,
        "CR 104.3d: p2 should have 10 poison counters after Poisonous 1 trigger resolves"
    );

    // SBA check should have marked P2 as having lost.
    assert!(
        state.players.get(&p2).unwrap().has_lost,
        "CR 704.5c: p2 should have lost (has_lost == true) after reaching 10 poison counters"
    );
}

#[test]
/// CR 702.70a — In a multiplayer game, each Poisonous trigger targets the
/// specific player who was dealt combat damage. Other players are unaffected.
///
/// P1 controls a Poisonous 1 creature and attacks P3. P2 and P4 are uninvolved.
/// After the trigger resolves, only P3 has 1 poison counter; P2 and P4 have 0.
fn test_702_70a_poisonous_multiplayer_correct_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(
            ObjectSpec::creature(p1, "Multiplayer Viper", 1, 1)
                .with_keyword(KeywordAbility::Poisonous(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Multiplayer Viper");

    assert_eq!(poison_counters(&state, p2), 0, "setup: p2 has 0 poison");
    assert_eq!(poison_counters(&state, p3), 0, "setup: p3 has 0 poison");
    assert_eq!(poison_counters(&state, p4), 0, "setup: p4 has 0 poison");

    // P1 attacks P3 (not P2 or P4).
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p3))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers (all 4 players pass).
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P3 is the defending player — she declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![],
        },
    )
    .expect("p3 declare blockers failed");

    // Advance through CombatDamage step — trigger fires targeting P3.
    let (state, damage_events) = pass_all(state, &[p1, p2, p3, p4]);

    // One AbilityTriggered event (for the Poisonous creature against P3).
    let triggered = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        triggered,
        "CR 702.70a: AbilityTriggered should fire for poisonous creature attacking p3"
    );

    // One trigger on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.70a: exactly 1 poisonous trigger should be on the stack"
    );

    // Resolve the trigger (all 4 players pass priority).
    let (state, resolve_events) = pass_all(state, &[p1, p2, p3, p4]);

    // AbilityResolved event should fire.
    let resolved = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityResolved { .. }));
    assert!(
        resolved,
        "CR 702.70a: AbilityResolved should fire after trigger resolves"
    );

    // Only P3 should have poison counters.
    assert_eq!(
        poison_counters(&state, p2),
        0,
        "CR 702.70a: p2 should have 0 poison counters (was not attacked)"
    );
    assert_eq!(
        poison_counters(&state, p3),
        1,
        "CR 702.70a: p3 should have 1 poison counter (was dealt combat damage)"
    );
    assert_eq!(
        poison_counters(&state, p4),
        0,
        "CR 702.70a: p4 should have 0 poison counters (was not attacked)"
    );

    // Stack is empty.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after trigger resolves"
    );
}
