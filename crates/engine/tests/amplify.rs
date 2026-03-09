//! Amplify keyword ability tests (CR 702.38).
//!
//! Amplify is a static ability that functions as an ETB replacement effect.
//! "As this object enters, reveal any number of cards from your hand that share
//! a creature type with it. This permanent enters with N +1/+1 counters on it
//! for each card revealed this way." (CR 702.38a)
//!
//! Key rules verified:
//! - ETB: creature enters with N * (eligible hand count) +1/+1 counters (CR 702.38a).
//! - Zero eligible hand cards → 0 counters (CR 702.38a).
//! - N multiplier: Amplify 3 × 2 cards = 6 counters (CR 702.38a).
//! - Multiple Amplify instances work separately; same hand cards count for each (CR 702.38b).
//! - Non-matching creature types in hand do not count (CR 702.38a).
//! - Changeling in hand counts for any entering creature type (CR 702.73a + CR 702.38a).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, GameEvent, GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec,
    PlayerId, Step, SubType, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn subtype(s: &str) -> SubType {
    SubType(s.to_string())
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

/// Cast spell helper: casts the named card from hand (must already be on the stack's
/// owning player's hand). Adds generic mana to the pool first.
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

/// Amplify 1 Soldier creature (like Daru Stinger). 1/1 Creature - Soldier.
fn amplify_1_soldier_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("amplify-1-soldier-test".to_string()),
        name: "Amplify Soldier One".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [subtype("Soldier")].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Amplify 1".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Amplify(1))],
        ..Default::default()
    }
}

/// Amplify 3 Dragon creature (like Kilnmouth Dragon). 5/5 Creature - Dragon.
fn amplify_3_dragon_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("amplify-3-dragon-test".to_string()),
        name: "Amplify Dragon Three".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [subtype("Dragon")].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Amplify 3".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Amplify(3))],
        ..Default::default()
    }
}

/// Creature with both Amplify 1 and Amplify 2 (artificial test case). 1/1 Creature - Goblin.
fn amplify_dual_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("amplify-dual-test".to_string()),
        name: "Amplify Dual Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [subtype("Goblin")].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Amplify 1, Amplify 2".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Amplify(1)),
            AbilityDefinition::Keyword(KeywordAbility::Amplify(2)),
        ],
        ..Default::default()
    }
}

/// 2/2 Soldier creature (hand fodder matching Soldier type).
fn soldier_card_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("soldier-fodder-test".to_string()),
        name: "Soldier Fodder".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [subtype("Soldier")].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}

/// 2/2 Dragon creature (hand fodder matching Dragon type).
fn dragon_card_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dragon-fodder-test".to_string()),
        name: "Dragon Fodder".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [subtype("Dragon")].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}

/// Goblin creature (hand fodder matching Goblin type).
fn goblin_card_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("goblin-fodder-test".to_string()),
        name: "Goblin Fodder".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [subtype("Goblin")].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}

/// 1/1 Changeling creature -- shares all creature types (CR 702.73a).
fn changeling_card_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("changeling-test".to_string()),
        name: "Changeling Fodder".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Changeling".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Changeling)],
        ..Default::default()
    }
}

/// 1/1 Elf creature (does NOT match Soldier or Dragon types).
fn elf_card_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("elf-fodder-test".to_string()),
        name: "Elf Fodder".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [subtype("Elf")].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}

// ── Test 1: Amplify basic — one revealed card ──────────────────────────────────

#[test]
/// CR 702.38a — "This permanent enters with N +1/+1 counters on it for each card revealed."
/// Amplify 1 creature enters with 1 matching Soldier card in hand → 1 counter placed.
fn test_amplify_basic_one_revealed() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![amplify_1_soldier_def(), soldier_card_def()]);

    let amplify_spec = ObjectSpec::card(p1, "Amplify Soldier One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("amplify-1-soldier-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")])
        .with_keyword(KeywordAbility::Amplify(1))
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let fodder_spec = ObjectSpec::card(p1, "Soldier Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("soldier-fodder-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(amplify_spec)
        .object(fodder_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let amplify_id = find_object(&state, "Amplify Soldier One");

    // Cast the Amplify creature (Soldier Fodder stays in hand).
    let state = cast_creature(state, p1, amplify_id, 1);

    // Resolve: both players pass priority.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Creature should be on the battlefield.
    let bf_id = find_object_on_battlefield(&state, "Amplify Soldier One")
        .expect("CR 702.38a: Amplify creature should be on the battlefield after resolution");

    // Verify: exactly 1 +1/+1 counter (1 amplify × 1 matching hand card).
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.38a: Amplify 1 creature with 1 matching hand card should have 1 counter"
    );

    // Verify CounterAdded event emitted.
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
        "CR 702.38a: CounterAdded event should be emitted for Amplify ETB counters"
    );
}

// ── Test 2: Amplify — multiple matching cards in hand ──────────────────────────

#[test]
/// CR 702.38a — Amplify 1 creature enters with 3 matching Soldier cards in hand → 3 counters.
fn test_amplify_multiple_revealed() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![amplify_1_soldier_def(), soldier_card_def()]);

    let amplify_spec = ObjectSpec::card(p1, "Amplify Soldier One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("amplify-1-soldier-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")])
        .with_keyword(KeywordAbility::Amplify(1))
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    // Add 3 Soldier fodder cards to hand.
    let fodder1 = ObjectSpec::card(p1, "Soldier Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("soldier-fodder-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")]);
    let fodder2 = ObjectSpec::card(p1, "Soldier Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("soldier-fodder-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")]);
    let fodder3 = ObjectSpec::card(p1, "Soldier Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("soldier-fodder-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(amplify_spec)
        .object(fodder1)
        .object(fodder2)
        .object(fodder3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let amplify_id = find_object(&state, "Amplify Soldier One");
    let state = cast_creature(state, p1, amplify_id, 1);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Amplify Soldier One")
        .expect("Amplify creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 702.38a: Amplify 1 creature with 3 matching hand cards should have 3 counters"
    );
}

// ── Test 3: Amplify — no matching cards in hand ────────────────────────────────

#[test]
/// CR 702.38a — Amplify creature enters with no matching creature types in hand → 0 counters.
/// Hand contains Elf cards; entering creature is a Soldier. No shared type.
fn test_amplify_no_matching_cards() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![amplify_1_soldier_def(), elf_card_def()]);

    let amplify_spec = ObjectSpec::card(p1, "Amplify Soldier One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("amplify-1-soldier-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")])
        .with_keyword(KeywordAbility::Amplify(1))
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let elf_spec = ObjectSpec::card(p1, "Elf Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("elf-fodder-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Elf")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(amplify_spec)
        .object(elf_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let amplify_id = find_object(&state, "Amplify Soldier One");
    let state = cast_creature(state, p1, amplify_id, 1);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Amplify Soldier One")
        .expect("Amplify creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.38a: Amplify creature with no matching hand cards should have 0 counters"
    );
}

// ── Test 4: Amplify N multiplier ──────────────────────────────────────────────

#[test]
/// CR 702.38a — Amplify 3 creature enters with 2 matching Dragon cards in hand → 6 counters
/// (3 counters per revealed card × 2 revealed = 6 total).
fn test_amplify_n_multiplier() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![amplify_3_dragon_def(), dragon_card_def()]);

    let amplify_spec = ObjectSpec::card(p1, "Amplify Dragon Three")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("amplify-3-dragon-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Dragon")])
        .with_keyword(KeywordAbility::Amplify(3))
        .with_mana_cost(ManaCost {
            generic: 5,
            ..Default::default()
        });

    let dragon1 = ObjectSpec::card(p1, "Dragon Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("dragon-fodder-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Dragon")]);
    let dragon2 = ObjectSpec::card(p1, "Dragon Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("dragon-fodder-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Dragon")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(amplify_spec)
        .object(dragon1)
        .object(dragon2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let amplify_id = find_object(&state, "Amplify Dragon Three");
    let state = cast_creature(state, p1, amplify_id, 5);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Amplify Dragon Three")
        .expect("Amplify Dragon should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 6,
        "CR 702.38a: Amplify 3 with 2 matching hand cards should produce 6 counters (3×2)"
    );
}

// ── Test 5: Multiple Amplify instances work separately ─────────────────────────

#[test]
/// CR 702.38b — "If a creature has multiple instances of amplify, each one works separately."
/// Creature with Amplify 1 + Amplify 2 enters with 2 matching Goblin cards in hand.
/// Expected: 1×2 + 2×2 = 2 + 4 = 6 counters total.
fn test_amplify_multiple_instances() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![amplify_dual_def(), goblin_card_def()]);

    let amplify_spec = ObjectSpec::card(p1, "Amplify Dual Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("amplify-dual-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Goblin")])
        .with_keyword(KeywordAbility::Amplify(1))
        .with_keyword(KeywordAbility::Amplify(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let goblin1 = ObjectSpec::card(p1, "Goblin Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("goblin-fodder-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Goblin")]);
    let goblin2 = ObjectSpec::card(p1, "Goblin Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("goblin-fodder-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Goblin")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(amplify_spec)
        .object(goblin1)
        .object(goblin2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let amplify_id = find_object(&state, "Amplify Dual Test");
    let state = cast_creature(state, p1, amplify_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Amplify Dual Test")
        .expect("Amplify dual creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    // Amplify 1 × 2 cards + Amplify 2 × 2 cards = 2 + 4 = 6.
    assert_eq!(
        counter_count, 6,
        "CR 702.38b: Amplify 1 + Amplify 2 with 2 matching hand cards should produce 6 counters"
    );
}

// ── Test 6: Amplify — empty hand ──────────────────────────────────────────────

#[test]
/// CR 702.38a — "Reveal any number of cards" (may be zero). Empty hand → 0 counters.
fn test_amplify_empty_hand() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![amplify_1_soldier_def()]);

    let amplify_spec = ObjectSpec::card(p1, "Amplify Soldier One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("amplify-1-soldier-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")])
        .with_keyword(KeywordAbility::Amplify(1))
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(amplify_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let amplify_id = find_object(&state, "Amplify Soldier One");
    let state = cast_creature(state, p1, amplify_id, 1);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Amplify Soldier One")
        .expect("Amplify creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.38a: Amplify creature with empty hand should have 0 counters"
    );
}

// ── Test 7: Changeling in hand counts for any entering creature type ───────────

#[test]
/// CR 702.73a + CR 702.38a: Changeling has every creature type; it shares a type with
/// any Amplify creature, so it is always eligible to reveal.
fn test_amplify_changeling_in_hand() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![amplify_1_soldier_def(), changeling_card_def()]);

    let amplify_spec = ObjectSpec::card(p1, "Amplify Soldier One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("amplify-1-soldier-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")])
        .with_keyword(KeywordAbility::Amplify(1))
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    // Changeling in hand: no printed subtypes, but Changeling CDA gives every type.
    let changeling_spec = ObjectSpec::card(p1, "Changeling Fodder")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("changeling-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Changeling);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(amplify_spec)
        .object(changeling_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let amplify_id = find_object(&state, "Amplify Soldier One");
    let state = cast_creature(state, p1, amplify_id, 1);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Amplify Soldier One")
        .expect("Amplify creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    // Changeling shares every type including Soldier → counts as 1 eligible card.
    assert_eq!(
        counter_count, 1,
        "CR 702.73a + CR 702.38a: Changeling in hand should count as eligible for Amplify"
    );
}

// ── Test 8: Non-creature cards in hand do not count ───────────────────────────

#[test]
/// CR 702.38a — Only cards that "share a creature type" count. Non-creature cards
/// (instants, lands, etc.) have no creature subtypes and are never eligible.
fn test_amplify_non_creature_in_hand() {
    let p1 = p(1);
    let p2 = p(2);

    // Register only the Amplify creature; the hand will also contain a sorcery card.
    let sorcery_def = CardDefinition {
        card_id: CardId("sorcery-test".to_string()),
        name: "Test Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: None,
        toughness: None,
        abilities: vec![],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![amplify_1_soldier_def(), sorcery_def]);

    let amplify_spec = ObjectSpec::card(p1, "Amplify Soldier One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("amplify-1-soldier-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![subtype("Soldier")])
        .with_keyword(KeywordAbility::Amplify(1))
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    // Non-creature card in hand (no creature subtypes).
    let sorcery_spec = ObjectSpec::card(p1, "Test Sorcery")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("sorcery-test".to_string()))
        .with_types(vec![CardType::Sorcery]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(amplify_spec)
        .object(sorcery_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let amplify_id = find_object(&state, "Amplify Soldier One");
    let state = cast_creature(state, p1, amplify_id, 1);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Amplify Soldier One")
        .expect("Amplify creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.38a: Non-creature cards in hand should not count for Amplify"
    );
}
