//! Riot keyword ability tests (CR 702.136).
//!
//! Riot is a static ability that functions as a replacement effect (CR 614.1c).
//! "Riot" means "You may have this permanent enter with an additional +1/+1
//! counter on it. If you don't, it gains haste." (CR 702.136a)
//!
//! Multiple instances of Riot each work separately (CR 702.136b).
//!
//! Key rules verified:
//! - Riot creature enters with +1/+1 counter (default deterministic choice, CR 702.136a).
//! - Counter is reflected in P/T via Layer 7c (CR 613.4c).
//! - KeywordAbility::Riot is present on the permanent after ETB.
//! - Multiple Riot instances (from card def) each add a +1/+1 counter (CR 702.136b).
//! - CounterAdded events are emitted for each Riot instance.
//! - Non-Riot creature is unaffected (negative case).

use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, CounterType, GameEvent, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_on_battlefield(
    state: &mtg_engine::GameState,
    name: &str,
) -> Option<mtg_engine::ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Riot Test Creature: Creature {1}{R} 2/2 with Riot.
/// Simple Goblin used for basic Riot validation.
fn riot_test_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("riot-test-creature".to_string()),
        name: "Riot Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Riot".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Riot)],
        ..Default::default()
    }
}

/// Double Riot Creature: Creature {2}{R}{G} 3/3 with two instances of Riot.
/// Used to verify CR 702.136b (multiple instances work separately).
fn double_riot_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("double-riot-creature".to_string()),
        name: "Double Riot Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Riot, Riot".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Riot),
            AbilityDefinition::Keyword(KeywordAbility::Riot),
        ],
        ..Default::default()
    }
}

/// Vanilla 2/2 with no abilities — used for negative (non-Riot) tests.
fn vanilla_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vanilla-creature".to_string()),
        name: "Vanilla Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}

// ── Test 1: Riot creature enters with +1/+1 counter (default choice) ──────────

/// CR 702.136a — "You may have this permanent enter with an additional +1/+1
/// counter on it." Engine defaults to choosing the counter for deterministic
/// testing. The permanent should have exactly 1 +1/+1 counter after ETB.
#[test]
fn test_riot_enters_with_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![riot_test_creature_def()]);

    let creature_spec = ObjectSpec::card(p1, "Riot Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("riot-test-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Riot)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
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

    // Pay {1}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Riot Test Creature");

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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));

    // Resolve the spell (both players pass priority).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Creature should be on the battlefield.
    let bf_id = find_object_on_battlefield(&state, "Riot Test Creature")
        .expect("CR 702.136a: Riot creature should be on the battlefield after resolution");

    // Verify: creature has exactly 1 +1/+1 counter.
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.136a: Riot creature should have 1 +1/+1 counter after ETB (default choice)"
    );

    // Verify: CounterAdded event was emitted.
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
        "CR 702.136a: CounterAdded event should be emitted when Riot chooses counter"
    );
}

// ── Test 2: Riot creature has correct P/T after ETB (base + counter) ──────────

/// CR 702.136a / CR 613.4c — After entering with a Riot +1/+1 counter,
/// the creature's effective P/T should be base_power+1 / base_toughness+1.
/// A 2/2 Riot creature becomes a 3/3 in practice (via Layer 7c counter application).
#[test]
fn test_riot_creature_has_correct_stats() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![riot_test_creature_def()]);

    // Use creature() to set base P/T (2/2), then chain card_id + zone overrides.
    // calculate_characteristics starts from obj.characteristics, so power/toughness
    // must be set on the ObjectSpec for the Layer 7c counter addition to be visible.
    let creature_spec = ObjectSpec::creature(p1, "Riot Test Creature", 2, 2)
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("riot-test-creature".to_string()))
        .with_keyword(KeywordAbility::Riot)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
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
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Riot Test Creature");

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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Riot Test Creature")
        .expect("Riot creature should be on battlefield");

    // calculate_characteristics applies Layer 7c (counters add to P/T).
    let chars = calculate_characteristics(&state, bf_id)
        .expect("calculate_characteristics should return Some for a battlefield object");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.136a / CR 613.4c: 2/2 Riot creature should have effective power 3 after +1/+1 counter"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.136a / CR 613.4c: 2/2 Riot creature should have effective toughness 3 after +1/+1 counter"
    );
}

// ── Test 3: KeywordAbility::Riot is present on the permanent ──────────────────

/// CR 702.136a — The permanent that entered via a Riot card definition should
/// have KeywordAbility::Riot in its keywords set on the battlefield.
#[test]
fn test_riot_keyword_present_on_permanent() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![riot_test_creature_def()]);

    let creature_spec = ObjectSpec::card(p1, "Riot Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("riot-test-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Riot)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
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
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Riot Test Creature");

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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Riot Test Creature")
        .expect("Riot creature should be on battlefield");

    assert!(
        state.objects[&bf_id]
            .characteristics
            .keywords
            .contains(&KeywordAbility::Riot),
        "CR 702.136a: Riot keyword should be present on the permanent after ETB"
    );
}

// ── Test 4: Multiple Riot instances each add a +1/+1 counter ─────────────────

/// CR 702.136b — "If a permanent has multiple instances of riot, each works separately."
/// A card definition with two Keyword(Riot) entries results in 2 +1/+1 counters.
#[test]
fn test_riot_multiple_instances_each_add_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![double_riot_creature_def()]);

    let creature_spec = ObjectSpec::card(p1, "Double Riot Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("double-riot-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Riot)
        .with_mana_cost(ManaCost {
            generic: 2,
            red: 1,
            green: 1,
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

    // Pay {2}{R}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Double Riot Creature");

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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));

    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Double Riot Creature")
        .expect("CR 702.136b: Double Riot creature should be on battlefield");

    // Two instances of Riot → 2 +1/+1 counters.
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 702.136b: creature with two Riot instances should enter with 2 +1/+1 counters"
    );

    // Two CounterAdded events emitted.
    let counter_events: Vec<_> = resolve_events
        .iter()
        .filter(|ev| {
            matches!(
                ev,
                GameEvent::CounterAdded {
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                    ..
                }
            )
        })
        .collect();
    assert_eq!(
        counter_events.len(),
        2,
        "CR 702.136b: two CounterAdded events should be emitted for two Riot instances"
    );
}

// ── Test 5: Non-Riot creature is unaffected ────────────────────────────────────

/// Negative test: A creature without Riot should enter with no +1/+1 counters
/// and no CounterAdded event from Riot processing.
#[test]
fn test_riot_no_counters_on_non_riot_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![vanilla_creature_def()]);

    let creature_spec = ObjectSpec::creature(p1, "Vanilla Creature", 2, 2)
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("vanilla-creature".to_string()))
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
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
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Vanilla Creature");

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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Vanilla Creature")
        .expect("Vanilla creature should be on battlefield");

    // No counters.
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "Non-Riot creature should enter with 0 +1/+1 counters"
    );

    // P/T is exactly base (2/2).
    let chars = calculate_characteristics(&state, bf_id)
        .expect("calculate_characteristics should return Some for a battlefield object");
    assert_eq!(chars.power, Some(2), "Non-Riot 2/2 should have power 2");
    assert_eq!(
        chars.toughness,
        Some(2),
        "Non-Riot 2/2 should have toughness 2"
    );
}
