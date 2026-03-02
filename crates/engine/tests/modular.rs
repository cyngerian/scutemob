//! Modular keyword ability tests (CR 702.43).
//!
//! Modular represents both a static ability (ETB counters) and a triggered ability
//! (dies: transfer +1/+1 counters to target artifact creature).
//!
//! Key rules verified:
//! - ETB: Modular N creature enters with N +1/+1 counters (CR 702.43a).
//! - Multiple Modular instances sum their N values on ETB (CR 702.43b).
//! - Dies trigger fires; target artifact creature receives counters (CR 702.43a).
//! - Counter count from last-known information (pre_death_counters, Arcbound Worker ruling).
//! - No artifact creature target → trigger not placed (CR 603.3d).
//! - Zero pre-death counters → trigger resolves with no effect.

use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, CounterType, GameEvent, GameStateBuilder, KeywordAbility,
    ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_on_battlefield(state: &mtg_engine::GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
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

// ── Card Definitions ──────────────────────────────────────────────────────────

/// Modular 1 creature card definition (Arcbound Worker-like).
/// 0/0 Artifact Creature with Modular 1.
fn modular_1_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("modular-1-test".to_string()),
        name: "Modular Test One".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "Modular 1".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Modular(1))],
        ..Default::default()
    }
}

/// Modular 3 creature card definition.
/// 0/0 Artifact Creature with Modular 3.
fn modular_3_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("modular-3-test".to_string()),
        name: "Modular Test Three".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "Modular 3".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Modular(3))],
        ..Default::default()
    }
}

/// Double Modular creature: Modular 1 + Modular 2 (separate instances).
/// CR 702.43b: Multiple instances sum their N values.
fn modular_1_and_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("modular-1-and-2-test".to_string()),
        name: "Double Modular Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "Modular 1, Modular 2".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Modular(1)),
            AbilityDefinition::Keyword(KeywordAbility::Modular(2)),
        ],
        ..Default::default()
    }
}

// ── Test 1: Modular 1 ETB counter ─────────────────────────────────────────────

#[test]
/// CR 702.43a — "This permanent enters with N +1/+1 counters on it."
/// A creature with Modular 1 cast from hand resolves and enters the battlefield
/// with exactly 1 +1/+1 counter. The CounterAdded event is emitted.
fn test_modular_etb_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![modular_1_def()]);

    let creature_spec = ObjectSpec::card(p1, "Modular Test One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("modular-1-test".to_string()))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_keyword(KeywordAbility::Modular(1))
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Add mana to pay {1}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Modular Test One");

    // Cast the creature.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));

    // Resolve the spell (both players pass priority).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Creature should be on the battlefield.
    let bf_id = find_object_on_battlefield(&state, "Modular Test One")
        .expect("CR 702.43a: Modular creature should be on the battlefield after resolution");

    // Verify: creature has exactly 1 +1/+1 counter.
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.43a: Modular 1 creature should have 1 +1/+1 counter after ETB"
    );

    // Verify: CounterAdded event was emitted with correct count.
    let counter_event = resolve_events.iter().any(|ev| {
        matches!(
            ev,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 1,
                ..
            }
        )
    });
    assert!(
        counter_event,
        "CR 702.43a: CounterAdded event should be emitted when Modular creature enters"
    );
}

// ── Test 2: Modular N ETB counters (N=3) ──────────────────────────────────────

#[test]
/// CR 702.43a — Creature with Modular 3 enters the battlefield with exactly 3
/// +1/+1 counters. The CounterAdded event carries count: 3.
fn test_modular_etb_counters_n() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![modular_3_def()]);

    let creature_spec = ObjectSpec::card(p1, "Modular Test Three")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("modular-3-test".to_string()))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_keyword(KeywordAbility::Modular(3))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Modular Test Three");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));

    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Modular Test Three")
        .expect("Modular 3 creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 702.43a: Modular 3 creature should have exactly 3 +1/+1 counters after ETB"
    );

    let counter_event = resolve_events.iter().any(|ev| {
        matches!(
            ev,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 3,
                ..
            }
        )
    });
    assert!(
        counter_event,
        "CR 702.43a: CounterAdded event with count 3 should be emitted"
    );
}

// ── Test 3: Modular dies trigger transfers counters ───────────────────────────

#[test]
/// CR 702.43a — Creature with Modular 1 (1 +1/+1 counter) dies via lethal damage SBA.
/// Modular trigger fires. Target artifact creature receives 1 +1/+1 counter.
fn test_modular_dies_transfers_counters() {
    let p1 = p(1);
    let p2 = p(2);

    // Modular creature with 1 +1/+1 counter (already on battlefield, pre-placed).
    // 0/0 base + 1 counter = 1/1 effective. Give it lethal damage to trigger SBA.
    let modular_creature = ObjectSpec::creature(p1, "Modular Worker", 0, 0)
        .with_keyword(KeywordAbility::Modular(1))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_counter(CounterType::PlusOnePlusOne, 1) // enters with counter pre-placed
        .with_damage(1) // lethal (effective toughness is 0+1=1, damage=1)
        .in_zone(ZoneId::Battlefield);

    // Target artifact creature to receive counters.
    let target = ObjectSpec::creature(p1, "Steel Target", 2, 2)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(modular_creature)
        .object(target)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pass priority → SBA fires → Modular Worker dies → trigger queued.
    let (state, events) = pass_all(state, &[p1, p2]);

    // Modular Worker should be dead.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.43a: CreatureDied event expected when Modular creature dies"
    );

    // Verify trigger is on the stack (before resolution).
    assert!(
        state.stack_objects.len() == 1,
        "CR 702.43a: Modular trigger should be on the stack"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            mtg_engine::StackObjectKind::ModularTrigger { .. }
        ),
        "stack object should be a ModularTrigger"
    );

    // Verify target has no +1/+1 counters yet.
    let target_id =
        find_object_on_battlefield(&state, "Steel Target").expect("Steel Target should exist");
    let target_counters_before = state.objects[&target_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        target_counters_before, 0,
        "Steel Target should have no counters before trigger resolves"
    );

    // Both players pass → trigger resolves → Steel Target receives 1 +1/+1 counter.
    let (state, trigger_events) = pass_all(state, &[p1, p2]);

    let target_counters_after = state.objects[&target_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        target_counters_after, 1,
        "CR 702.43a: Steel Target should have 1 +1/+1 counter after Modular trigger resolves"
    );

    // CounterAdded event emitted for the target.
    assert!(
        trigger_events.iter().any(|ev| {
            matches!(
                ev,
                GameEvent::CounterAdded {
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                    ..
                }
            )
        }),
        "CR 702.43a: CounterAdded event should be emitted when Modular trigger resolves"
    );
}

// ── Test 4: Extra counters — last-known information rule ──────────────────────

#[test]
/// CR 702.43a / Arcbound Worker ruling 2006-09-25 — When a Modular creature dies,
/// the trigger moves the ACTUAL number of +1/+1 counters (from pre_death_counters),
/// not the static N from the Modular keyword. A Modular 1 creature that accumulated
/// 3 +1/+1 counters transfers all 3.
fn test_modular_dies_extra_counters() {
    let p1 = p(1);
    let p2 = p(2);

    // Modular 1 creature that has accumulated 3 +1/+1 counters (e.g., via ability).
    // 0/0 base + 3 counters = 3/3 effective. Give it 3 damage (lethal).
    let modular_creature = ObjectSpec::creature(p1, "Arcbound Worker", 0, 0)
        .with_keyword(KeywordAbility::Modular(1))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_counter(CounterType::PlusOnePlusOne, 3) // 3 counters total
        .with_damage(3) // lethal
        .in_zone(ZoneId::Battlefield);

    let target = ObjectSpec::creature(p1, "Steel Target", 2, 2)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(modular_creature)
        .object(target)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pass priority → SBA → Arcbound Worker dies → trigger queued with count=3.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Verify trigger carries 3 counters.
    assert!(
        state.stack_objects.len() == 1,
        "Modular trigger should be on the stack"
    );
    if let mtg_engine::StackObjectKind::ModularTrigger { counter_count, .. } =
        state.stack_objects[0].kind
    {
        assert_eq!(
            counter_count, 3,
            "CR 702.43a: ModularTrigger should carry 3 (the pre-death counter count, \
             not the static Modular 1 value)"
        );
    } else {
        panic!("Expected ModularTrigger on stack");
    }

    let target_id =
        find_object_on_battlefield(&state, "Steel Target").expect("Steel Target should exist");

    // Resolve trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let target_counters = state.objects[&target_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        target_counters, 3,
        "Arcbound Worker ruling: Steel Target should have ALL 3 pre-death counters, \
         not just the static Modular N (1)"
    );
}

// ── Test 5: No artifact creature target — trigger not placed ──────────────────

#[test]
/// CR 603.3d — If a triggered ability requires a target and there are no legal targets
/// when the trigger would go on the stack, it is removed without being placed.
/// Modular dies with no artifact creatures on the battlefield: trigger NOT placed.
fn test_modular_dies_no_artifact_creature_target() {
    let p1 = p(1);
    let p2 = p(2);

    // Modular creature dies. Only vanilla creature (non-artifact) on battlefield.
    let modular_creature = ObjectSpec::creature(p1, "Arcbound Worker", 0, 0)
        .with_keyword(KeywordAbility::Modular(1))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .with_damage(1) // lethal
        .in_zone(ZoneId::Battlefield);

    // Vanilla non-artifact creature — cannot be targeted by Modular trigger.
    let vanilla = ObjectSpec::creature(p1, "Vanilla Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(modular_creature)
        .object(vanilla)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // Worker should have died.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied should fire for Modular creature"
    );

    // Trigger should NOT be on the stack (no legal artifact creature target).
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 603.3d: Modular trigger should NOT be placed on the stack when \
         no artifact creature target exists"
    );

    // Vanilla creature should be unaffected.
    let vanilla_id =
        find_object_on_battlefield(&state, "Vanilla Creature").expect("Vanilla should survive");
    let vanilla_counters = state.objects[&vanilla_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        vanilla_counters, 0,
        "Non-artifact creature should receive no counters (not a legal target)"
    );
}

// ── Test 6: Zero counters at death — trigger resolves with no effect ──────────

#[test]
/// CR 702.43a — Edge case: Modular creature had its +1/+1 counter removed before dying
/// (e.g., by effects or annihilation with -1/-1 counters). The trigger fires but
/// counter_count is 0; no counters are moved to the target.
fn test_modular_dies_zero_counters() {
    let p1 = p(1);
    let p2 = p(2);

    // Modular 1 creature with NO +1/+1 counters (counter removed before death).
    // 0/0 base + 0 counters → toughness 0 → SBA kills it immediately.
    let modular_creature = ObjectSpec::creature(p1, "Depleted Worker", 0, 0)
        .with_keyword(KeywordAbility::Modular(1))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        // No PlusOnePlusOne counter — counter was removed. Toughness = 0 → SBA kills.
        .in_zone(ZoneId::Battlefield);

    let target = ObjectSpec::creature(p1, "Steel Target", 2, 2)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(modular_creature)
        .object(target)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // SBA kills 0/0 creature → Modular trigger with counter_count=0 queued (if target exists).
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied should fire for 0-counter Modular creature"
    );

    let target_id =
        find_object_on_battlefield(&state, "Steel Target").expect("Steel Target should exist");

    // If trigger is on stack, resolve it.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Target should have 0 counters (trigger had counter_count=0).
    let target_counters = state.objects[&target_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        target_counters, 0,
        "CR 702.43a: No counters should be moved when Modular creature had 0 +1/+1 counters at death"
    );
}

// ── Test 7: Multiple Modular instances (CR 702.43b) ───────────────────────────

#[test]
/// CR 702.43b — Creature with Modular 1 and Modular 2 (from a card definition).
/// ETB: enters with 1+2=3 +1/+1 counters.
/// On death, two separate Modular triggers fire.
fn test_modular_multiple_instances_etb() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![modular_1_and_2_def()]);

    let creature_spec = ObjectSpec::card(p1, "Double Modular Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("modular-1-and-2-test".to_string()))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_keyword(KeywordAbility::Modular(1))
        .with_keyword(KeywordAbility::Modular(2))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Double Modular Test");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Double Modular Test")
        .expect("Double Modular creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 702.43b: Modular 1 + Modular 2 creature should enter with 3 +1/+1 counters (1+2)"
    );
}

// ── Test 8: 0/0 base stats — does NOT die immediately after ETB counters ──────

#[test]
/// CR 702.43a / SBA timing — A 0/0 artifact creature with Modular 1 enters the
/// battlefield. The ETB counter (+1/+1) is applied before SBAs check, so the
/// creature has effective toughness 1 after ETB and does NOT die immediately.
fn test_modular_0_0_base_stats_survives_etb() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![modular_1_def()]);

    let creature_spec = ObjectSpec::card(p1, "Modular Test One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("modular-1-test".to_string()))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_keyword(KeywordAbility::Modular(1))
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Modular Test One");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));

    let (state, events) = pass_all(state, &[p1, p2]);

    // 0/0 with Modular 1 should be on the battlefield (NOT dead).
    let bf_id = find_object_on_battlefield(&state, "Modular Test One")
        .expect("CR 702.43a: 0/0 Modular 1 creature should be on the battlefield (not SBA-killed)");

    // Should have 1 +1/+1 counter (making it effectively 1/1).
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "0/0 Modular 1 creature should have 1 +1/+1 counter after ETB"
    );

    // Should NOT have a CreatureDied event (ETB counter prevents SBA death).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.43a: 0/0 + Modular 1 counter should prevent SBA death (effective toughness = 1)"
    );

    // The creature is alive on the battlefield with 1 +1/+1 counter.
    // (ObjectSpec::card() produces power/toughness = None from the type system's perspective;
    // the SBA correctly does NOT kill it because the card definition has power:0, toughness:0
    // and the +1/+1 counter makes effective toughness non-lethal.)
    // The key invariant: creature exists on battlefield AND has the counter.
    let _ = calculate_characteristics(&state, bf_id); // no panic
}

// ── Test 9: Multiple Modular death triggers (CR 702.43b) ──────────────────────

#[test]
/// CR 702.43b — "If a creature has multiple instances of modular, each one works separately."
/// A creature with Modular 1 and Modular 2 (3 +1/+1 counters total) dies via lethal damage.
/// Each Modular instance triggers separately, producing exactly 2 ModularTrigger entries
/// on the stack. Each trigger carries counter_count equal to the total pre-death counter
/// count (3), not just the individual N value.
fn test_modular_multiple_instances_death_triggers() {
    let p1 = p(1);
    let p2 = p(2);

    // Dual-Modular creature: Modular 1 + Modular 2, pre-placed on battlefield with 3
    // +1/+1 counters (1+2 from ETB). Give it lethal damage (effective toughness = 3,
    // so damage = 3 is lethal).
    let dual_modular = ObjectSpec::creature(p1, "Double Modular Test", 0, 0)
        .with_keyword(KeywordAbility::Modular(1))
        .with_keyword(KeywordAbility::Modular(2))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_counter(CounterType::PlusOnePlusOne, 3) // 3 counters = effective 3/3
        .with_damage(3) // lethal
        .in_zone(ZoneId::Battlefield);

    // Artifact creature target for the triggers to resolve onto.
    let target = ObjectSpec::creature(p1, "Steel Target", 2, 2)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(dual_modular)
        .object(target)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pass priority → SBA fires → dual-Modular creature dies → 2 triggers queued.
    let (state, events) = pass_all(state, &[p1, p2]);

    // The dual-Modular creature should be dead.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.43b: CreatureDied event expected when dual-Modular creature dies"
    );

    // Exactly 2 ModularTrigger entries should be on the stack (one per Modular instance).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.43b: Two separate Modular triggers should be on the stack (one per instance)"
    );

    // Both stack objects must be ModularTrigger.
    for (i, stack_obj) in state.stack_objects.iter().enumerate() {
        assert!(
            matches!(
                stack_obj.kind,
                mtg_engine::StackObjectKind::ModularTrigger { .. }
            ),
            "CR 702.43b: stack object {} should be a ModularTrigger",
            i
        );
    }

    // Each trigger should carry counter_count = 3 (full pre-death counter count, not
    // the individual N value of 1 or 2).
    for (i, stack_obj) in state.stack_objects.iter().enumerate() {
        if let mtg_engine::StackObjectKind::ModularTrigger { counter_count, .. } = stack_obj.kind {
            assert_eq!(
                counter_count, 3,
                "CR 702.43b: ModularTrigger {} should carry counter_count=3 (all pre-death \
                 counters, not just the individual Modular N value)",
                i
            );
        } else {
            panic!("CR 702.43b: Expected ModularTrigger at index {}", i);
        }
    }
}
