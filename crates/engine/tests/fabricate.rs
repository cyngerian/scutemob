//! Fabricate keyword ability tests (CR 702.123).
//!
//! "Fabricate N" means "When this permanent enters, you may put N +1/+1
//! counters on it. If you don't, create N 1/1 colorless Servo artifact
//! creature tokens." (CR 702.123a)
//!
//! If a permanent has multiple instances of fabricate, each triggers
//! separately (CR 702.123b).
//!
//! Key rules verified:
//! - Bot always chooses counters when permanent is on the battlefield (CR 702.123a).
//! - N +1/+1 counters are placed on the permanent (CR 702.123a).
//! - No Servo tokens are created when bot chooses counters (CR 702.123a).
//! - Fabricate 3 places 3 counters (CR 702.123a).
//! - Multiple Fabricate instances trigger separately (CR 702.123b).
//! - KeywordAbility::Fabricate(n) variant compiles and is present in card definitions.
//! - Non-Fabricate creatures do not receive counters or tokens.
//! - 4-player multiplayer: Fabricate fires correctly.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, GameEvent, GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec,
    PlayerId, Step, SubType, ZoneId,
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

fn count_servos_on_battlefield(state: &mtg_engine::GameState) -> usize {
    state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.characteristics.name == "Servo"
                && obj
                    .characteristics
                    .subtypes
                    .contains(&SubType("Servo".to_string()))
        })
        .count()
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

/// Cast a creature from hand by adding generic mana and calling CastSpell.
fn cast_creature(
    state: mtg_engine::GameState,
    caster: PlayerId,
    card_id: ObjectId,
    generic_cost: u32,
) -> mtg_engine::GameState {
    let mut state = state;
    state
        .players
        .get_mut(&caster)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, generic_cost);
    state.turn.priority_holder = Some(caster);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: caster,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));
    state
}

// ── Card Definitions ──────────────────────────────────────────────────────────

/// Fabricate 2 creature — 0/1, cost {2}{colorless}.
fn fabricate_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("fabricate-2-test".to_string()),
        name: "Fabricate 2 Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fabricate 2 (When this creature enters, put two +1/+1 counters on it or create two 1/1 colorless Servo artifact creature tokens.)"
            .to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Fabricate(2))],
        ..Default::default()
    }
}

/// Fabricate 1 creature — 2/2, cost {2}{colorless}.
fn fabricate_1_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("fabricate-1-test".to_string()),
        name: "Fabricate 1 Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fabricate 1 (When this creature enters, put a +1/+1 counter on it or create a 1/1 colorless Servo artifact creature token.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Fabricate(1))],
        ..Default::default()
    }
}

/// Fabricate 3 creature — 1/1, cost {3}{colorless}.
fn fabricate_3_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("fabricate-3-test".to_string()),
        name: "Fabricate 3 Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fabricate 3".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Fabricate(3))],
        ..Default::default()
    }
}

/// Double Fabricate creature (Fabricate 1 + Fabricate 2) — 1/1, cost {3}{colorless}.
/// CR 702.123b: multiple instances trigger separately → total 3 counters.
fn double_fabricate_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("double-fabricate-test".to_string()),
        name: "Double Fabricate Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fabricate 1, Fabricate 2".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Fabricate(1)),
            AbilityDefinition::Keyword(KeywordAbility::Fabricate(2)),
        ],
        ..Default::default()
    }
}

/// Plain 2/2 creature with no special abilities — used for non-Fabricate isolation tests.
fn plain_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-creature-test".to_string()),
        name: "Plain Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}

// ── Test 1: Basic counters — bot chooses counters ─────────────────────────────

#[test]
/// CR 702.123a — "When this permanent enters, you may put N +1/+1 counters on it.
/// If you don't, create N 1/1 colorless Servo artifact creature tokens."
/// Bot always chooses counters. Fabricate 2 creature enters with 2 +1/+1 counters.
fn test_fabricate_basic_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fabricate_2_def()]);

    let spec = ObjectSpec::card(p1, "Fabricate 2 Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fabricate-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Fabricate(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Fabricate 2 Test Creature");
    let state = cast_creature(state, p1, card_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Fabricate 2 Test Creature")
        .expect("CR 702.123a: Fabricate creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 702.123a: Fabricate 2 creature should have 2 +1/+1 counters (bot chooses counters)"
    );
}

// ── Test 2: No Servo tokens when bot chooses counters ─────────────────────────

#[test]
/// CR 702.123a — When the bot chooses to put +1/+1 counters, no Servo tokens
/// should be created on the battlefield.
fn test_fabricate_no_servo_tokens_when_counters_chosen() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fabricate_2_def()]);

    let spec = ObjectSpec::card(p1, "Fabricate 2 Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fabricate-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Fabricate(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Fabricate 2 Test Creature");
    let state = cast_creature(state, p1, card_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let servo_count = count_servos_on_battlefield(&state);
    assert_eq!(
        servo_count, 0,
        "CR 702.123a: No Servo tokens should exist when bot chose +1/+1 counters"
    );
}

// ── Test 3: KeywordAbility::Fabricate variant present on card ─────────────────

#[test]
/// CR 702.123a — KeywordAbility::Fabricate(n) compiles and is present on the
/// card definition. Verifies the enum variant is correctly attached.
fn test_fabricate_keyword_on_card() {
    let def2 = fabricate_2_def();
    let has_fabricate_2 = def2
        .abilities
        .iter()
        .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Fabricate(2))));
    assert!(
        has_fabricate_2,
        "CR 702.123a: Card definition should contain KeywordAbility::Fabricate(2)"
    );

    let def1 = fabricate_1_def();
    let has_fabricate_1 = def1
        .abilities
        .iter()
        .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Fabricate(1))));
    assert!(
        has_fabricate_1,
        "CR 702.123a: Card definition should contain KeywordAbility::Fabricate(1)"
    );
}

// ── Test 4: Fabricate 3 — N=3 produces 3 counters ────────────────────────────

#[test]
/// CR 702.123a — Fabricate 3 creature enters, bot chooses counters.
/// The permanent should gain exactly 3 +1/+1 counters.
fn test_fabricate_3_places_three_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fabricate_3_def()]);

    let spec = ObjectSpec::card(p1, "Fabricate 3 Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fabricate-3-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Fabricate(3))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Fabricate 3 Test Creature");
    let state = cast_creature(state, p1, card_id, 3);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Fabricate 3 Test Creature")
        .expect("CR 702.123a: Fabricate 3 creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 702.123a: Fabricate 3 creature should have exactly 3 +1/+1 counters"
    );
}

// ── Test 5: Multiple Fabricate instances trigger separately ───────────────────

#[test]
/// CR 702.123b — "If a permanent has multiple instances of fabricate, each
/// triggers separately." A creature with Fabricate 1 and Fabricate 2 should
/// receive 1 + 2 = 3 counters total (bot chooses counters for each).
fn test_fabricate_multiple_instances_trigger_separately() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![double_fabricate_def()]);

    let spec = ObjectSpec::card(p1, "Double Fabricate Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("double-fabricate-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Fabricate(1))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Double Fabricate Test");
    let state = cast_creature(state, p1, card_id, 3);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Double Fabricate Test")
        .expect("CR 702.123b: Double Fabricate creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 702.123b: Fabricate 1 + Fabricate 2 should produce 1+2=3 counters total (each triggers separately)"
    );
}

// ── Test 6: Non-Fabricate creature — no counters, no tokens ──────────────────

#[test]
/// CR 702.123a — A creature without Fabricate should not receive any +1/+1
/// counters or Servo tokens when it enters the battlefield.
fn test_non_fabricate_no_counters_no_tokens() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_creature_def()]);

    let spec = ObjectSpec::card(p1, "Plain Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-creature-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plain Creature");
    let state = cast_creature(state, p1, card_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Plain Creature")
        .expect("Plain creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.123a: Non-Fabricate creature should not receive any +1/+1 counters"
    );

    let servo_count = count_servos_on_battlefield(&state);
    assert_eq!(
        servo_count, 0,
        "CR 702.123a: Non-Fabricate creature should not create any Servo tokens"
    );
}

// ── Test 7: Fabricate 1 — minimal N=1 ────────────────────────────────────────

#[test]
/// CR 702.123a — Fabricate 1 (minimum N) places exactly 1 +1/+1 counter.
/// Bot chooses counters. No Servo tokens created.
fn test_fabricate_1_places_one_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fabricate_1_def()]);

    let spec = ObjectSpec::card(p1, "Fabricate 1 Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fabricate-1-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Fabricate(1))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Fabricate 1 Test Creature");
    let state = cast_creature(state, p1, card_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Fabricate 1 Test Creature")
        .expect("CR 702.123a: Fabricate 1 creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.123a: Fabricate 1 should place exactly 1 +1/+1 counter"
    );

    let servo_count = count_servos_on_battlefield(&state);
    assert_eq!(
        servo_count, 0,
        "CR 702.123a: No Servo tokens should be created when bot chose counters"
    );
}

// ── Test 9: Token fallback — creature left battlefield before Fabricate resolves ─

#[test]
/// CR 702.123a — 2016-09-20 ruling: "If you can't put +1/+1 counters on the
/// creature for any reason as fabricate resolves (for instance, if it's no
/// longer on the battlefield), you just create Servo tokens."
///
/// This test directly exercises the `else` branch in
/// `fire_when_enters_triggered_effects` (replacement.rs) by removing the
/// Fabricate permanent from `state.objects` before calling the function, so
/// `permanent_on_bf` evaluates to false and the token path fires.
///
/// Verifies: N=2 Servo tokens are created; each token is 1/1, colorless,
/// Artifact Creature with "Servo" subtype (CR 702.123a).
fn test_fabricate_token_fallback() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fabricate_2_def()]);

    // Place the creature directly on the battlefield so we can get its ID.
    let spec = ObjectSpec::card(p1, "Fabricate 2 Test Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fabricate-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Fabricate(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let fabricate_id = find_object(&state, "Fabricate 2 Test Creature");

    // Remove the object from state to simulate it having left the battlefield
    // before the Fabricate trigger resolves (ruling 2016-09-20).
    state.objects = state.objects.without(&fabricate_id);
    assert!(
        state.objects.get(&fabricate_id).is_none(),
        "Object must be absent from state for the token fallback path to fire"
    );

    // Call queue_carddef_etb_triggers directly with the now-absent ID.
    // The fabricate block checks `state.objects.get(&new_id)` — because the
    // object is absent, `permanent_on_bf` is false → token path executes.
    let card_id = CardId("fabricate-2-test".to_string());
    let evts = mtg_engine::rules::replacement::queue_carddef_etb_triggers(
        &mut state,
        fabricate_id,
        p1,
        Some(&card_id),
        &registry,
    );

    // Verify 2 Servo tokens now exist on the battlefield.
    let servo_count = count_servos_on_battlefield(&state);
    assert_eq!(
        servo_count, 2,
        "CR 702.123a (ruling 2016-09-20): Fabricate 2 should create 2 Servo tokens when permanent left battlefield"
    );

    // Verify Servo token characteristics: 1/1, colorless, Artifact Creature — Servo.
    let servo_tokens: Vec<_> = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.characteristics.name == "Servo")
        .collect();
    assert_eq!(
        servo_tokens.len(),
        2,
        "Expected exactly 2 Servo tokens on the battlefield"
    );

    for token in &servo_tokens {
        assert_eq!(
            token.characteristics.power,
            Some(1),
            "CR 702.123a: Servo token must be power 1"
        );
        assert_eq!(
            token.characteristics.toughness,
            Some(1),
            "CR 702.123a: Servo token must be toughness 1"
        );
        assert!(
            token.characteristics.colors.is_empty(),
            "CR 702.123a: Servo token must be colorless"
        );
        assert!(
            token
                .characteristics
                .card_types
                .contains(&CardType::Artifact),
            "CR 702.123a: Servo token must be an Artifact"
        );
        assert!(
            token
                .characteristics
                .card_types
                .contains(&CardType::Creature),
            "CR 702.123a: Servo token must be a Creature"
        );
        assert!(
            token
                .characteristics
                .subtypes
                .contains(&SubType("Servo".to_string())),
            "CR 702.123a: Servo token must have 'Servo' subtype"
        );
    }

    // Verify the events include token creation (not counter placement).
    let has_token_event = evts
        .iter()
        .any(|e| matches!(e, GameEvent::TokenCreated { .. }));
    assert!(
        has_token_event,
        "CR 702.123a: GameEvent::TokenCreated should be emitted when token fallback fires"
    );

    // Verify no +1/+1 counter event was emitted (tokens path, not counters path).
    let has_counter_event = evts
        .iter()
        .any(|e| matches!(e, GameEvent::CounterAdded { .. }));
    assert!(
        !has_counter_event,
        "CR 702.123a: No CounterAdded event should be emitted when permanent left battlefield"
    );
}

// ── Test 8: Multiplayer — Fabricate fires correctly in 4-player game ──────────

#[test]
/// CR 702.123a — In a 4-player game, Fabricate triggers for the controller
/// and places N +1/+1 counters on the permanent. Other players are unaffected.
fn test_fabricate_multiplayer_4_players() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![fabricate_2_def()]);

    let spec = ObjectSpec::card(p1, "Fabricate 2 Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fabricate-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Fabricate(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Fabricate 2 Test Creature");
    let state = cast_creature(state, p1, card_id, 2);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let bf_id = find_object_on_battlefield(&state, "Fabricate 2 Test Creature")
        .expect("CR 702.123a: Fabricate creature should be on battlefield in 4-player game");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 702.123a: Fabricate 2 should place 2 counters in a 4-player game"
    );

    let servo_count = count_servos_on_battlefield(&state);
    assert_eq!(
        servo_count, 0,
        "CR 702.123a: No Servo tokens should be created in a 4-player game when bot chose counters"
    );
}
