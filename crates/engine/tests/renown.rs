//! Renown keyword ability enforcement tests (CR 702.112).
//!
//! Renown is a triggered ability: "When this creature deals combat damage to a
//! player, if it isn't renowned, put N +1/+1 counters on it and it becomes
//! renowned."
//!
//! Key rules tested:
//! - CR 702.112a: Basic trigger fires on unblocked damage; counters placed and
//!   creature becomes renowned.
//! - CR 702.112a: Renown N > 1 places the correct number of counters.
//! - CR 702.112a + CR 603.4: Trigger does NOT fire when creature is already
//!   renowned (intervening-if fails at trigger time).
//! - CR 702.112b + CR 400.7: Renowned designation resets on zone change.
//! - CR 702.112c + CR 603.4: Multiple Renown instances: first trigger resolves,
//!   subsequent triggers are countered by the intervening-if at resolution.
//! - Ruling 2015-06-22: If the source leaves battlefield before resolution,
//!   the trigger does nothing.
//! - CR 702.112a: Multiplayer -- trigger fires when dealing damage to any player.

use mtg_engine::{
    process_command, AbilityDefinition, AttackTarget, CardDefinition, CardId, CardRegistry,
    CardType, Command, CounterType, GameEvent, GameStateBuilder, KeywordAbility, ObjectSpec,
    PlayerId, Step, TypeLine, ZoneId,
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

// ── Helper: pass all priority ─────────────────────────────────────────────────

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

// ── Helper: get +1/+1 counter count on a named object ────────────────────────

fn plus_counters(state: &mtg_engine::GameState, name: &str) -> u32 {
    let id = find_object(state, name);
    state
        .objects
        .get(&id)
        .and_then(|obj| obj.counters.get(&CounterType::PlusOnePlusOne))
        .copied()
        .unwrap_or(0)
}

// ── Helper: is_renowned for a named object ───────────────────────────────────

fn is_renowned(state: &mtg_engine::GameState, name: &str) -> bool {
    let id = find_object(state, name);
    state
        .objects
        .get(&id)
        .map(|obj| {
            obj.designations
                .contains(mtg_engine::Designations::RENOWNED)
        })
        .unwrap_or(false)
}

// ── CR 702.112: Renown ────────────────────────────────────────────────────────

#[test]
/// CR 702.112a — basic trigger: When this creature deals combat damage to a
/// player, if it isn't renowned, put 1 +1/+1 counter on it and it becomes
/// renowned. An unblocked creature with Renown 1 attacks; trigger fires and
/// resolves; creature gets 1 +1/+1 counter and is_renowned becomes true.
fn test_702_112a_renown_basic_counters_and_renowned() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Renown Creature", 2, 2)
                .with_keyword(KeywordAbility::Renown(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Renown Creature");

    assert_eq!(
        plus_counters(&state, "Renown Creature"),
        0,
        "setup: no +1/+1 counters before combat"
    );
    assert!(
        !is_renowned(&state, "Renown Creature"),
        "setup: creature should not be renowned before combat"
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

    // Advance through CombatDamage step — damage dealt, trigger fires.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // Attacker dealt 2 damage (creature is 2/2).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "CR 702.112a: p2 should have taken 2 combat damage"
    );

    // AbilityTriggered event should fire for the renown creature.
    let triggered = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        triggered,
        "CR 702.112a: AbilityTriggered event should fire for renown creature"
    );

    // Renown trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.112a: Renown trigger should be on the stack"
    );

    // Resolve the trigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved event should fire.
    let resolved = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityResolved { .. }));
    assert!(
        resolved,
        "CR 702.112a: AbilityResolved event should fire after trigger resolves"
    );

    // Stack is empty.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after Renown trigger resolves"
    );

    // Creature has 1 +1/+1 counter.
    assert_eq!(
        plus_counters(&state, "Renown Creature"),
        1,
        "CR 702.112a: renown creature should have 1 +1/+1 counter after trigger resolves"
    );

    // Creature is now renowned.
    assert!(
        is_renowned(&state, "Renown Creature"),
        "CR 702.112a: creature should be renowned after trigger resolves"
    );
}

#[test]
/// CR 702.112a — Renown N > 1: A creature with Renown 2 deals combat damage
/// to a player; the trigger resolves and 2 +1/+1 counters are placed.
fn test_702_112a_renown_n2_places_two_counters() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Renown 2 Creature", 4, 4)
                .with_keyword(KeywordAbility::Renown(2)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Renown 2 Creature");

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

    // Advance through CombatDamage step.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Trigger on stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.112a: Renown trigger should be on the stack"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature has 2 +1/+1 counters.
    assert_eq!(
        plus_counters(&state, "Renown 2 Creature"),
        2,
        "CR 702.112a: Renown 2 should place 2 +1/+1 counters"
    );

    // Creature is now renowned.
    assert!(
        is_renowned(&state, "Renown 2 Creature"),
        "CR 702.112a: creature should be renowned after Renown 2 trigger resolves"
    );
}

#[test]
/// CR 702.112a + CR 603.4 — Trigger does NOT fire when the creature is already
/// renowned (intervening-if "if it isn't renowned" fails at trigger time).
/// A creature is manually set as renowned; when it deals combat damage, no
/// trigger fires and no additional counters are placed.
fn test_702_112a_renown_no_trigger_when_already_renowned() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Already Renowned", 2, 2)
                .with_keyword(KeywordAbility::Renown(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    // Manually set the creature as renowned (simulating a previous trigger).
    let attacker_id = find_object(&state, "Already Renowned");
    let mut state = state;
    if let Some(obj) = state.objects.get_mut(&attacker_id) {
        obj.designations.insert(mtg_engine::Designations::RENOWNED);
    }

    assert!(
        is_renowned(&state, "Already Renowned"),
        "setup: creature should start as renowned"
    );
    assert_eq!(
        plus_counters(&state, "Already Renowned"),
        0,
        "setup: no counters before combat"
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

    // Advance through CombatDamage step.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // P2 took 2 damage (combat damage still happens).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "CR 702.112a: combat damage still applies even when Renown doesn't trigger"
    );

    // No AbilityTriggered event for the renown creature (intervening-if failed).
    let triggered = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        !triggered,
        "CR 702.112a + CR 603.4: Renown should NOT trigger when creature is already renowned"
    );

    // Stack is empty — no renown trigger.
    assert!(
        state.stack_objects.is_empty(),
        "CR 603.4: stack should be empty — Renown did not trigger (creature already renowned)"
    );

    // No additional counters placed.
    assert_eq!(
        plus_counters(&state, "Already Renowned"),
        0,
        "CR 603.4: no additional counters should be placed on already-renowned creature"
    );
}

#[test]
/// CR 702.112b + CR 400.7 — Renowned designation resets on zone change.
/// After a creature becomes renowned, it is bounced back to hand and re-enters
/// the battlefield. The new permanent is NOT renowned.
fn test_702_112b_renown_resets_on_zone_change() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Zone Change Renown", 2, 2)
                .with_keyword(KeywordAbility::Renown(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    // Manually set is_renowned = true and add a +1/+1 counter (simulating
    // a resolved Renown trigger).
    let creature_id = find_object(&state, "Zone Change Renown");
    let mut state = state;
    if let Some(obj) = state.objects.get_mut(&creature_id) {
        obj.designations.insert(mtg_engine::Designations::RENOWNED);
        obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, 1);
    }

    assert!(
        is_renowned(&state, "Zone Change Renown"),
        "setup: creature should be renowned"
    );
    assert_eq!(
        plus_counters(&state, "Zone Change Renown"),
        1,
        "setup: creature should have 1 counter"
    );

    // Simulate zone change: move the creature to its owner's hand.
    // move_object_to_zone resets all CR 400.7 flags including is_renowned.
    let (new_id, _) = state
        .move_object_to_zone(creature_id, ZoneId::Hand(p1))
        .expect("move_object_to_zone failed");

    // The new object (in hand) should NOT be renowned (CR 400.7 reset).
    let new_obj = state.objects.get(&new_id).expect("new object not found");
    assert!(
        !new_obj
            .designations
            .contains(mtg_engine::Designations::RENOWNED),
        "CR 702.112b + CR 400.7: renowned designation should reset on zone change"
    );

    // The new object should NOT have the old counters (CR 400.7).
    assert_eq!(
        new_obj
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "CR 400.7: +1/+1 counters should reset on zone change"
    );

    // Move the creature back to the battlefield.
    let (bf_id, _) = state
        .move_object_to_zone(new_id, ZoneId::Battlefield)
        .expect("move to battlefield failed");

    // The re-entered creature is NOT renowned.
    let bf_obj = state
        .objects
        .get(&bf_id)
        .expect("battlefield object not found");
    assert!(
        !bf_obj
            .designations
            .contains(mtg_engine::Designations::RENOWNED),
        "CR 702.112b: creature re-entering the battlefield should not be renowned"
    );
}

#[test]
/// CR 702.112c + CR 603.4 — Multiple Renown instances on the same creature.
/// A creature with two Renown(1) keyword entries deals combat damage. Two
/// triggers fire and go on the stack. The first trigger resolves: creature gets
/// 1 +1/+1 counter and becomes renowned. The second trigger resolves: the
/// intervening-if at resolution fails (creature is already renowned), so no
/// additional counters are placed.
fn test_702_112c_renown_multiple_instances_first_resolves() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build a CardDefinition with TWO Renown(1) keyword entries.
    let double_renown_def = CardDefinition {
        card_id: CardId("double-renown-creature".to_string()),
        name: "Double Renown Creature".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Renown 1\nRenown 1".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Renown(1)),
            AbilityDefinition::Keyword(KeywordAbility::Renown(1)),
        ],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    };

    let registry = CardRegistry::new(vec![double_renown_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::creature(p1, "Double Renown Creature", 2, 2)
                .with_keyword(KeywordAbility::Renown(1))
                .with_card_id(CardId("double-renown-creature".to_string())),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Renown Creature");

    assert!(
        !is_renowned(&state, "Double Renown Creature"),
        "setup: not renowned"
    );
    assert_eq!(
        plus_counters(&state, "Double Renown Creature"),
        0,
        "setup: no counters"
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

    // Advance through CombatDamage step — TWO renown triggers fire.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Two triggers on the stack (CR 702.112c: each instance triggers separately).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.112c: 2 Renown triggers should be on the stack"
    );

    // Resolve first trigger — creature gets 1 counter and becomes renowned.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.stack_objects.len(),
        1,
        "one trigger should remain after resolving the first"
    );
    assert_eq!(
        plus_counters(&state, "Double Renown Creature"),
        1,
        "CR 702.112c: first trigger should place 1 +1/+1 counter"
    );
    assert!(
        is_renowned(&state, "Double Renown Creature"),
        "CR 702.112c: creature should be renowned after first trigger resolves"
    );

    // Resolve second trigger — intervening-if fails at resolution (already renowned).
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    // Still only 1 counter — second trigger did nothing.
    assert_eq!(
        plus_counters(&state, "Double Renown Creature"),
        1,
        "CR 702.112c + CR 603.4: second trigger should not place additional counters (already renowned)"
    );

    // Still renowned (unchanged).
    assert!(
        is_renowned(&state, "Double Renown Creature"),
        "CR 702.112c: creature should still be renowned after second trigger does nothing"
    );
}

#[test]
/// Ruling 2015-06-22 + CR 603.4 — If a renown ability triggers, but the creature
/// leaves the battlefield before that ability resolves, the creature doesn't
/// become renowned and no counters are placed.
fn test_702_112_renown_creature_leaves_before_resolution() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Renown Then Dies", 2, 2)
                .with_keyword(KeywordAbility::Renown(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Renown Then Dies");

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

    // Advance through CombatDamage step — trigger fires and goes on the stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Renown trigger should be on the stack before creature is removed"
    );

    // Remove the creature from the battlefield before the trigger resolves
    // (simulate it dying, bouncing, etc.) by moving it to the graveyard.
    let mut state = state;
    let (_, _) = state
        .move_object_to_zone(attacker_id, ZoneId::Graveyard(p1))
        .expect("move_object_to_zone failed");

    // Verify the creature is no longer on the battlefield.
    assert!(
        !state
            .objects
            .values()
            .any(|o| o.characteristics.name == "Renown Then Dies" && o.zone == ZoneId::Battlefield),
        "setup: creature should be off battlefield before trigger resolves"
    );

    // Resolve the trigger — should do nothing (source not on battlefield).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved should still fire (trigger resolves, just does nothing).
    let resolved = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityResolved { .. }));
    assert!(
        resolved,
        "ruling 2015-06-22: AbilityResolved should fire even when source left battlefield"
    );

    // Stack is empty.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after trigger resolves (doing nothing)"
    );

    // The creature is in the graveyard with no counters (new object after zone change).
    let graveyard_obj = state
        .objects
        .values()
        .find(|o| {
            o.characteristics.name == "Renown Then Dies" && matches!(o.zone, ZoneId::Graveyard(_))
        })
        .expect("creature should be in graveyard");

    assert!(
        !graveyard_obj
            .designations
            .contains(mtg_engine::Designations::RENOWNED),
        "ruling 2015-06-22: creature should NOT be renowned (was off battlefield at resolution)"
    );
    assert_eq!(
        graveyard_obj
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        0,
        "ruling 2015-06-22: no counters should be placed when source left battlefield"
    );
}

#[test]
/// CR 702.112a — In a 4-player game, a creature deals combat damage to one of
/// the opponent players. Renown triggers correctly for that damage event.
fn test_702_112a_renown_multiplayer_specific_player() {
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
            ObjectSpec::creature(p1, "Renown Multiplayer", 2, 2)
                .with_keyword(KeywordAbility::Renown(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Renown Multiplayer");
    let p3_initial_life = state.players.get(&p3).unwrap().life_total;

    // Attacker declares against P3 (not P2 or P4).
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p3))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P3 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P3 took 2 damage.
    assert_eq!(
        state.players.get(&p3).unwrap().life_total,
        p3_initial_life - 2,
        "CR 702.112a: p3 should have taken 2 combat damage"
    );

    // P2 and P4 untouched.
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        p3_initial_life, // all players start with same life
        "CR 702.112a: p2 should be untouched (not attacked)"
    );
    assert_eq!(
        state.players.get(&p4).unwrap().life_total,
        p3_initial_life,
        "CR 702.112a: p4 should be untouched (not attacked)"
    );

    // Renown trigger on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.112a: Renown trigger should be on the stack (multiplayer)"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Creature is now renowned with 1 counter.
    assert!(
        is_renowned(&state, "Renown Multiplayer"),
        "CR 702.112a: creature should be renowned after combat damage to p3 in 4-player game"
    );
    assert_eq!(
        plus_counters(&state, "Renown Multiplayer"),
        1,
        "CR 702.112a: 1 +1/+1 counter placed in 4-player game"
    );
}
