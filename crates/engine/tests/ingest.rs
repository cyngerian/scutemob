//! Ingest keyword ability enforcement tests (CR 702.115).
//!
//! Ingest is a triggered ability: "Whenever this creature deals combat damage
//! to a player, that player exiles the top card of their library."
//!
//! Tests:
//! - Basic trigger fires on unblocked damage to player; top card exiled (CR 702.115a)
//! - Trigger does NOT fire when creature is blocked (CR 702.115a + CR 510.1c)
//! - Empty library is a safe no-op when trigger resolves (ruling 2015-08-25)
//! - Multiple instances trigger separately (CR 702.115b)
//! - Multiplayer: each trigger targets the specific damaged player (CR 702.115a)

use mtg_engine::{
    process_command, AbilityDefinition, AttackTarget, CardDefinition, CardId, CardRegistry,
    CardType, Command, GameEvent, GameStateBuilder, KeywordAbility, ObjectSpec, PlayerId, Step,
    TypeLine, ZoneId,
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

// ── Helper: count objects in exile ──────────────────────────────────────────

fn count_in_exile(state: &mtg_engine::GameState) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile)
        .count()
}

// ── Helper: library size for a player ───────────────────────────────────────

fn library_size(state: &mtg_engine::GameState, player: PlayerId) -> usize {
    state
        .zones
        .get(&ZoneId::Library(player))
        .map(|z| z.len())
        .unwrap_or(0)
}

// ── CR 702.115: Ingest ────────────────────────────────────────────────────────

#[test]
/// CR 702.115a — basic trigger: Whenever this creature deals combat damage to a
/// player, that player exiles the top card of their library.
/// An unblocked creature with Ingest deals damage and the trigger resolves to
/// exile the top card of the defending player's library.
fn test_702_115_ingest_basic_exiles_top_card() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Ingest Creature", 2, 2).with_keyword(KeywordAbility::Ingest),
        )
        // P2 has one card in their library.
        .object(ObjectSpec::card(p2, "Library Card 1").in_zone(ZoneId::Library(p2)))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Ingest Creature");

    assert_eq!(
        library_size(&state, p2),
        1,
        "setup: p2 should start with 1 card in library"
    );
    assert_eq!(
        count_in_exile(&state),
        0,
        "setup: exile should be empty before combat"
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

    // Advance through CombatDamage step — damage is dealt, Ingest trigger fires.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // P2 took 2 combat damage (life 40 - 2 = 38).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "CR 702.115a: p2 should have taken 2 combat damage"
    );

    // AbilityTriggered event should have been emitted for the ingest source.
    let triggered = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        triggered,
        "CR 702.115a: AbilityTriggered event should fire for ingest creature"
    );

    // The ingest trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.115a: Ingest trigger should be on the stack"
    );

    // Resolve the trigger (both players pass priority).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved event should have been emitted.
    let resolved = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityResolved { .. }));
    assert!(
        resolved,
        "CR 702.115a: AbilityResolved event should fire after trigger resolves"
    );

    // The top card of P2's library is now in exile.
    assert_eq!(
        count_in_exile(&state),
        1,
        "CR 702.115a: 1 card should be in exile after Ingest trigger resolves"
    );

    // P2's library should have 0 cards remaining.
    assert_eq!(
        library_size(&state, p2),
        0,
        "CR 702.115a: p2's library should be empty after Ingest exiles top card"
    );

    // Stack is empty after resolution.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after Ingest trigger resolves"
    );
}

#[test]
/// CR 702.115a + CR 510.1c — Ingest does NOT trigger when the creature is
/// blocked (because a blocked creature without trample deals no damage to the
/// defending player).
fn test_702_115_ingest_does_not_trigger_when_blocked() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Ingest Creature", 2, 2).with_keyword(KeywordAbility::Ingest),
        )
        .object(ObjectSpec::creature(p2, "Big Blocker", 3, 3))
        // P2 has a library card — should remain untouched.
        .object(ObjectSpec::card(p2, "Library Card 1").in_zone(ZoneId::Library(p2)))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Ingest Creature");
    let blocker_id = find_object(&state, "Big Blocker");

    assert_eq!(library_size(&state, p2), 1, "setup: p2 has 1 library card");
    assert_eq!(count_in_exile(&state), 0, "setup: exile is empty");

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

    // Advance through CombatDamage step.
    let (state, combat_events) = pass_all(state, &[p1, p2]);

    // No AbilityTriggered event for the ingest creature.
    let triggered = combat_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        !triggered,
        "CR 702.115a: Ingest should NOT trigger when creature is blocked (no player damage)"
    );

    // Stack is empty — no ingest trigger.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty — Ingest did not trigger (blocked creature)"
    );

    // P2's library is unchanged.
    assert_eq!(
        library_size(&state, p2),
        1,
        "CR 702.115a: p2's library should be unchanged when ingest does not trigger"
    );

    // Exile zone unchanged.
    assert_eq!(
        count_in_exile(&state),
        0,
        "exile should still be empty when ingest does not trigger"
    );
}

#[test]
/// CR 702.115a + ruling 2015-08-25 — "If the player has no cards in their
/// library when the ingest ability resolves, nothing happens."
/// The trigger resolves without error and no card is exiled.
fn test_702_115_ingest_empty_library_is_noop() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P2's library is empty (no cards added).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Ingest Creature", 1, 1).with_keyword(KeywordAbility::Ingest),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Ingest Creature");

    assert_eq!(
        library_size(&state, p2),
        0,
        "setup: p2 should have 0 cards in library"
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
        "Ingest trigger should be on the stack even with empty library"
    );

    // Resolve the trigger — should not panic.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved event should fire (the trigger resolved as a no-op).
    let resolved = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityResolved { .. }));
    assert!(
        resolved,
        "ruling 2015-08-25: AbilityResolved should fire even when library is empty"
    );

    // Nothing exiled.
    assert_eq!(
        count_in_exile(&state),
        0,
        "ruling 2015-08-25: nothing should be exiled when library is empty"
    );

    // P2's library is still empty.
    assert_eq!(
        library_size(&state, p2),
        0,
        "ruling 2015-08-25: p2's library should still be empty after no-op resolution"
    );

    // P2 is still alive (not forced to draw from empty library).
    assert!(
        state.players.get(&p2).is_some(),
        "ruling 2015-08-25: p2 should still be in the game after empty-library no-op"
    );
    assert!(
        state.players.get(&p2).unwrap().life_total > 0,
        "p2 should still have positive life total"
    );
}

#[test]
/// CR 702.115a — Multiple Ingest creatures each trigger separately.
/// Two separate 1/1 creatures with Ingest attack the same player.
/// Each creature generates its own trigger, resulting in two triggers total,
/// each exiling one card from the damaged player's library.
fn test_702_115a_ingest_two_creatures_each_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Ingest Creature A", 1, 1)
                .with_keyword(KeywordAbility::Ingest),
        )
        .object(
            ObjectSpec::creature(p1, "Ingest Creature B", 1, 1)
                .with_keyword(KeywordAbility::Ingest),
        )
        // P2 has 2 cards in their library.
        .object(ObjectSpec::card(p2, "Library Card 1").in_zone(ZoneId::Library(p2)))
        .object(ObjectSpec::card(p2, "Library Card 2").in_zone(ZoneId::Library(p2)))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_a = find_object(&state, "Ingest Creature A");
    let attacker_b = find_object(&state, "Ingest Creature B");

    assert_eq!(library_size(&state, p2), 2, "setup: p2 has 2 library cards");
    assert_eq!(count_in_exile(&state), 0, "setup: exile is empty");

    // Both creatures attack P2.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (attacker_a, AttackTarget::Player(p2)),
                (attacker_b, AttackTarget::Player(p2)),
            ],
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

    // Advance through CombatDamage step — both triggers fire.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // Two AbilityTriggered events (one per ingest creature).
    let triggered_count = damage_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_a || *source_object_id == attacker_b
            )
        })
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.115a: two separate triggers should fire for two Ingest creatures"
    );

    // Two items on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.115a: 2 ingest triggers should be on the stack"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.stack_objects.len(),
        1,
        "one trigger should remain after resolving the first"
    );
    assert_eq!(
        count_in_exile(&state),
        1,
        "1 card should be in exile after first trigger resolves"
    );

    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    // Both library cards are now in exile.
    assert_eq!(
        count_in_exile(&state),
        2,
        "CR 702.115a: 2 cards should be in exile after both triggers resolve (one per creature)"
    );

    // P2's library is empty.
    assert_eq!(
        library_size(&state, p2),
        0,
        "CR 702.115a: p2's library should be empty after 2 Ingest triggers"
    );
}

#[test]
/// CR 702.115b — If a creature has multiple instances of ingest, each triggers
/// separately. A single creature with two AbilityDefinition::Keyword(Ingest)
/// entries in its CardDefinition should generate TWO triggers when it deals
/// combat damage to a player (via keyword_count in abilities.rs).
fn test_702_115b_ingest_single_creature_multiple_instances() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build a CardDefinition with TWO Ingest keyword entries.
    // The `abilities` Vec (not the runtime OrdSet) is what keyword_count reads.
    let double_ingest_def = CardDefinition {
        card_id: CardId("double-ingest-creature".to_string()),
        name: "Double Ingest Creature".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Ingest\nIngest".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ingest),
            AbilityDefinition::Keyword(KeywordAbility::Ingest),
        ],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    };

    let registry = CardRegistry::new(vec![double_ingest_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        // Single creature with two Ingest instances via card_id.
        .object(
            ObjectSpec::creature(p1, "Double Ingest Creature", 1, 1)
                .with_keyword(KeywordAbility::Ingest)
                .with_card_id(CardId("double-ingest-creature".to_string())),
        )
        // P2 has 2 cards in their library.
        .object(ObjectSpec::card(p2, "Library Card 1").in_zone(ZoneId::Library(p2)))
        .object(ObjectSpec::card(p2, "Library Card 2").in_zone(ZoneId::Library(p2)))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Ingest Creature");

    assert_eq!(library_size(&state, p2), 2, "setup: p2 has 2 library cards");
    assert_eq!(count_in_exile(&state), 0, "setup: exile is empty");

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

    // Advance through CombatDamage step — TWO ingest triggers should fire
    // from the single creature with two Ingest entries.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // Two AbilityTriggered events from the SAME creature (CR 702.115b).
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
        "CR 702.115b: a single creature with two Ingest entries should fire TWO triggers"
    );

    // Two items on the stack from a single attacker.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.115b: 2 ingest triggers on the stack from a single creature"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.stack_objects.len(),
        1,
        "one trigger should remain after resolving the first"
    );
    assert_eq!(
        count_in_exile(&state),
        1,
        "1 card should be in exile after first trigger resolves"
    );

    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    // Both library cards are now in exile — both triggered from one creature.
    assert_eq!(
        count_in_exile(&state),
        2,
        "CR 702.115b: 2 cards exiled by a single creature with double Ingest"
    );

    // P2's library is empty.
    assert_eq!(
        library_size(&state, p2),
        0,
        "CR 702.115b: p2's library should be empty after 2 triggers from one creature"
    );
}

#[test]
/// CR 702.115a — In a multiplayer game, each Ingest trigger targets the specific
/// player that was dealt combat damage.
/// Two Ingest creatures attack different players. Each trigger exiles from the
/// correct player's library. The third player's library is untouched.
fn test_702_115_ingest_multiplayer_targets_correct_player() {
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
            ObjectSpec::creature(p1, "Ingest Creature A", 1, 1)
                .with_keyword(KeywordAbility::Ingest),
        )
        .object(
            ObjectSpec::creature(p1, "Ingest Creature B", 1, 1)
                .with_keyword(KeywordAbility::Ingest),
        )
        // Each of P2, P3, P4 has 1 card in their library.
        .object(ObjectSpec::card(p2, "P2 Library Card").in_zone(ZoneId::Library(p2)))
        .object(ObjectSpec::card(p3, "P3 Library Card").in_zone(ZoneId::Library(p3)))
        .object(ObjectSpec::card(p4, "P4 Library Card").in_zone(ZoneId::Library(p4)))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_a = find_object(&state, "Ingest Creature A");
    let attacker_b = find_object(&state, "Ingest Creature B");

    assert_eq!(library_size(&state, p2), 1, "setup: p2 has 1 card");
    assert_eq!(library_size(&state, p3), 1, "setup: p3 has 1 card");
    assert_eq!(library_size(&state, p4), 1, "setup: p4 has 1 card");

    // Creature A attacks P2; Creature B attacks P3.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (attacker_a, AttackTarget::Player(p2)),
                (attacker_b, AttackTarget::Player(p3)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // No blockers declared.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("p2 declare blockers failed");

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![],
        },
    )
    .expect("p3 declare blockers failed");

    // Advance through CombatDamage step — both triggers fire.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Two triggers on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.115a: 2 ingest triggers should fire (one for P2, one for P3)"
    );

    // Resolve both triggers (2 passes each time for 4 players).
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    // P2's library lost 1 card (attacked by Creature A).
    assert_eq!(
        library_size(&state, p2),
        0,
        "CR 702.115a: p2's library should lose 1 card (attacked by ingest creature)"
    );

    // P3's library lost 1 card (attacked by Creature B).
    assert_eq!(
        library_size(&state, p3),
        0,
        "CR 702.115a: p3's library should lose 1 card (attacked by ingest creature)"
    );

    // P4's library is untouched (P4 was not attacked).
    assert_eq!(
        library_size(&state, p4),
        1,
        "CR 702.115a: p4's library should be untouched (p4 was not attacked)"
    );

    // 2 cards total in exile (from P2 and P3).
    assert_eq!(
        count_in_exile(&state),
        2,
        "CR 702.115a: 2 cards should be in exile total (one from P2, one from P3)"
    );
}
