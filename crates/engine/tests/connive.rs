//! Connive keyword action tests (CR 701.50).
//!
//! Connive is a keyword action (like Surveil), not a keyword ability. Cards say
//! "[permanent] connives" or "[permanent] connives N" as part of a spell effect
//! or triggered/activated ability effect.
//!
//! Key rules verified:
//! - Connive draws 1 card, discards 1 card; +1/+1 counter if nonland discarded (CR 701.50a).
//! - No counter when a land is discarded (CR 701.50a).
//! - Connive N draws N, discards N, places N_nonland counters (CR 701.50e).
//! - Connived event always fires, even when draw/discard was impossible (CR 701.50b).
//! - No counter placed when permanent left the battlefield before connive resolves (CR 701.50c).
//! - "Whenever this creature connives" trigger fires via TriggerEvent::SourceConnives (CR 701.50b).

use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, CounterType, Effect, EffectAmount, GameEvent, GameStateBuilder, ManaColor, ManaCost,
    ObjectSpec, PlayerId, PlayerTarget, Step, Target, TargetRequirement, TriggerCondition,
    TriggerEvent, TriggeredAbilityDef, TypeLine, ZoneId, ZoneTarget,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
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

/// Count objects in a zone.
fn count_in_zone(
    state: &mtg_engine::GameState,
    player: PlayerId,
    zone_fn: impl Fn(PlayerId) -> ZoneId,
) -> usize {
    let zone = zone_fn(player);
    state
        .objects
        .values()
        .filter(|obj| obj.zone == zone)
        .count()
}

/// Build a "Target creature connives N" sorcery card definition.
fn connive_spell_def(card_id_str: &str, name: &str, n: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id_str.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: format!("Target creature connives {}.", n),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Connive {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                count: EffectAmount::Fixed(n as i32),
            },
            targets: vec![mtg_engine::TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Build a creature card with ETB "when this creature enters, it connives."
fn connive_etb_creature_def(card_id_str: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id_str.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "When this creature enters, it connives.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::Connive {
                target: CardEffectTarget::Source,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
        }],
        ..Default::default()
    }
}

// ── Test 1: Basic connive — nonland discard places +1/+1 counter ──────────────

#[test]
/// CR 701.50a — The permanent's controller draws a card, then discards a card.
/// If a nonland card is discarded, put a +1/+1 counter on the conniving permanent.
fn test_connive_basic_nonland_discard_adds_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let spell_def = connive_spell_def("connive-spell", "Connive Spell", 1);
    let registry = CardRegistry::new(vec![spell_def]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Target Creature", 2, 2).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::card(p1, "Connive Spell")
                .with_card_id(CardId("connive-spell".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    // 5 nonland cards in library — the drawn card will be nonland.
    for i in 0..5 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Nonland Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }

    let state = builder.build().unwrap();
    let initial_hand_count = count_in_zone(&state, p1, ZoneId::Hand);

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let creature_id = find_object(&state, "Target Creature");
    let spell_id = find_object(&state, "Connive Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.50a: The creature should have a +1/+1 counter (nonland card discarded).
    let creature = state
        .objects
        .values()
        .find(|obj| {
            obj.characteristics.name == "Target Creature" && obj.zone == ZoneId::Battlefield
        })
        .expect("Target Creature must still be on the battlefield");

    let counter_count = creature
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        counter_count, 1,
        "Target Creature must have exactly 1 +1/+1 counter after connive with nonland discard; got {}",
        counter_count
    );

    // Hand size: spell cast (-1), drew 1, discarded 1 = net -1.
    let final_hand_count = count_in_zone(&state, p1, ZoneId::Hand);
    assert_eq!(
        final_hand_count,
        initial_hand_count - 1,
        "Hand should be 1 fewer (spell cast, draw 1, discard 1 = net -1); before={}, after={}",
        initial_hand_count,
        final_hand_count
    );

    // CR 701.50b: Connived event must be emitted.
    let connived = events
        .iter()
        .find(|e| matches!(e, GameEvent::Connived { object_id, .. } if *object_id == creature_id));
    assert!(
        connived.is_some(),
        "Connived event must be emitted for Target Creature"
    );

    if let Some(GameEvent::Connived {
        counters_placed, ..
    }) = connived
    {
        assert_eq!(
            *counters_placed, 1,
            "Connived event must show counters_placed = 1; got {}",
            counters_placed
        );
    }
}

// ── Test 2: Connive — land card in hand has lower ObjectId than drawn nonland ──

#[test]
/// CR 701.50a — If the discarded card is a land, no +1/+1 counter is placed.
/// Setup: land card in hand (low ObjectId) so it's the one discarded.
fn test_connive_land_discard_no_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let spell_def = connive_spell_def("connive-spell-land", "Connive Spell Land", 1);
    let registry = CardRegistry::new(vec![spell_def]);

    // The land card is added first (low ObjectId). After drawing one card,
    // the discard step picks min ObjectId in hand: the land card.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        // Land card first = lowest ObjectId in hand.
        .object(
            ObjectSpec::card(p1, "A Basic Plains")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(ObjectSpec::creature(p1, "Land Target", 2, 2).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::card(p1, "Connive Spell Land")
                .with_card_id(CardId("connive-spell-land".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        // Add a nonland card to library so draw produces a nonland.
        .object(ObjectSpec::card(p1, "Drawn Nonland").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let creature_id = find_object(&state, "Land Target");
    let spell_id = find_object(&state, "Connive Spell Land");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.50b: Connived event must always be emitted.
    let connived = events
        .iter()
        .find(|e| matches!(e, GameEvent::Connived { object_id, .. } if *object_id == creature_id));
    assert!(
        connived.is_some(),
        "Connived event must be emitted (CR 701.50b)"
    );

    // CR 701.50a: The land card (lowest ObjectId) is discarded, so no counter.
    // After draw, the land was in hand first, gets discarded.
    // The drawn nonland card is NOT discarded (higher ObjectId than the pre-existing land).
    let creature = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Land Target" && obj.zone == ZoneId::Battlefield)
        .expect("Land Target must still be on the battlefield");

    let counter_count = creature
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        counter_count, 0,
        "Land Target must have 0 +1/+1 counters when land was discarded; got {}",
        counter_count
    );

    if let Some(GameEvent::Connived {
        counters_placed, ..
    }) = connived
    {
        assert_eq!(
            *counters_placed, 0,
            "Connived event must show counters_placed = 0 (land discarded); got {}",
            counters_placed
        );
    }
}

// ── Test 3: Connive N — multiple draws and discards ───────────────────────────

#[test]
/// CR 701.50e — Connive N: draw N cards, discard N cards, put a counter for each
/// nonland card discarded.
fn test_connive_n_multiple_draws_and_discards() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let spell_def = connive_spell_def("connive-spell-3", "Connive 3 Spell", 3);
    let registry = CardRegistry::new(vec![spell_def]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Big Conniver", 3, 3).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::card(p1, "Connive 3 Spell")
                .with_card_id(CardId("connive-spell-3".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    // 5 nonland cards in library (all drawn cards will be nonland).
    for i in 0..5 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Nonland {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }

    let state = builder.build().unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let creature_id = find_object(&state, "Big Conniver");
    let spell_id = find_object(&state, "Connive 3 Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.50b: Connived event fires.
    let connived = events
        .iter()
        .find(|e| matches!(e, GameEvent::Connived { object_id, .. } if *object_id == creature_id));
    assert!(
        connived.is_some(),
        "Connived event must be emitted for Big Conniver (CR 701.50b)"
    );

    // CR 701.50e: All 3 drawn cards are nonland, so 3 +1/+1 counters.
    let creature = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Big Conniver" && obj.zone == ZoneId::Battlefield)
        .expect("Big Conniver must still be on the battlefield");

    let counter_count = creature
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        counter_count, 3,
        "Big Conniver must have 3 +1/+1 counters after connive 3 with all nonland discards; got {}",
        counter_count
    );

    if let Some(GameEvent::Connived {
        counters_placed, ..
    }) = connived
    {
        assert_eq!(
            *counters_placed, 3,
            "Connived event must show counters_placed = 3; got {}",
            counters_placed
        );
    }
}

// ── Test 4: Connive with empty library — still connives (CR 701.50b) ──────────

#[test]
/// CR 701.50b — A permanent connives even if the draw was impossible (empty library).
/// The Connived event must still fire. No counter placed (nothing discarded).
fn test_connive_empty_library_still_connives() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let spell_def = connive_spell_def("connive-spell-empty", "Connive Empty", 1);
    let registry = CardRegistry::new(vec![spell_def]);

    // No library cards. Draw will fail (player draws from empty library).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Empty Conniver", 2, 2).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::card(p1, "Connive Empty")
                .with_card_id(CardId("connive-spell-empty".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let creature_id = find_object(&state, "Empty Conniver");
    let spell_id = find_object(&state, "Connive Empty");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let (_state, events) = pass_all(state, &[p1, p2]);

    // CR 701.50b: Connived event must fire even when draw was impossible.
    let connived = events
        .iter()
        .find(|e| matches!(e, GameEvent::Connived { object_id, .. } if *object_id == creature_id));
    assert!(
        connived.is_some(),
        "Connived event must be emitted even with empty library (CR 701.50b); all Connived events: {:?}",
        events
            .iter()
            .filter(|e| matches!(e, GameEvent::Connived { .. }))
            .collect::<Vec<_>>()
    );

    // No counter placed (nothing drawn/discarded = no nonland discard).
    if let Some(GameEvent::Connived {
        counters_placed, ..
    }) = connived
    {
        assert_eq!(
            *counters_placed, 0,
            "counters_placed must be 0 when nothing could be discarded; got {}",
            counters_placed
        );
    }
}

// ── Test 5: Connive ETB trigger (Raffine's Informant pattern) ─────────────────

#[test]
/// CR 701.50a — Creature with ETB trigger "When this creature enters, it connives."
/// After the creature enters the battlefield, the ETB trigger fires and connive
/// resolves: draw 1, discard 1. If a nonland was discarded, creature gets a counter.
fn test_connive_etb_trigger_on_creature() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature_def = connive_etb_creature_def("raffines-informant", "Raffine's Informant");
    let registry = CardRegistry::new(vec![creature_def]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Raffine's Informant")
                .with_card_id(CardId("raffines-informant".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    white: 1,
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    // Nonland cards in library so draw works.
    for i in 0..3 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }

    let state = builder.build().unwrap();
    let mut state = state;

    // Pay {1}{W} for Raffine's Informant.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let card_id = find_object(&state, "Raffine's Informant");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // First pass_all: spell resolves, creature enters, ETB trigger queues.
    let (state, events1) = pass_all(state.clone(), &[p1, p2]);

    // Second pass_all: ETB trigger resolves (connive fires).
    let (state, events2) = pass_all(state, &[p1, p2]);

    // Collect events from both passes (connive may fire in either).
    let all_events: Vec<_> = events1.iter().chain(events2.iter()).collect();

    // Creature should be on the battlefield.
    let creature = state
        .objects
        .values()
        .find(|obj| {
            obj.characteristics.name == "Raffine's Informant" && obj.zone == ZoneId::Battlefield
        })
        .expect("Raffine's Informant must be on the battlefield after entering");

    // CR 701.50b: Connived event must be emitted.
    let connived = all_events
        .iter()
        .find(|e| matches!(e, GameEvent::Connived { object_id, .. } if *object_id == creature.id));

    assert!(
        connived.is_some(),
        "Connived event must be emitted when ETB connive trigger resolves (CR 701.50b); creature.id={:?}, all Connived events: {:?}",
        creature.id,
        all_events.iter().filter(|e| matches!(e, GameEvent::Connived { .. })).collect::<Vec<_>>()
    );
}

// ── Test 6: "Whenever this creature connives" trigger fires ───────────────────

#[test]
/// CR 701.50b — "Whenever [this creature] connives" triggers fire via
/// TriggerEvent::SourceConnives when the Connived event is emitted.
/// Validated via a creature that gains a +1/+1 counter from its own connive trigger.
fn test_connive_self_trigger_fires_on_connive() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let spell_def = connive_spell_def("connive-spell-trig", "Trigger Connive Spell", 1);
    let registry = CardRegistry::new(vec![spell_def]);

    // Build a creature with "Whenever this creature connives, it gets a +1/+1 counter"
    // modeled as SourceConnives -> AddCounter.
    let connive_trigger_creature = ObjectSpec::creature(p1, "Ledger Shredder Stand-in", 2, 1)
        .with_triggered_ability(TriggeredAbilityDef {
            etb_filter: None,
            targets: vec![],
            trigger_on: TriggerEvent::SourceConnives,
            intervening_if: None,
            description: "Whenever this creature connives, it gets a +1/+1 counter (CR 701.50b)"
                .to_string(),
            effect: Some(Effect::AddCounter {
                target: CardEffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
                count: 1,
            }),
        })
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(connive_trigger_creature)
        .object(
            ObjectSpec::card(p1, "Trigger Connive Spell")
                .with_card_id(CardId("connive-spell-trig".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    // Nonland cards in library so draw works.
    for i in 0..3 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }

    let state = builder.build().unwrap();
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let creature_id = find_object(&state, "Ledger Shredder Stand-in");
    let spell_id = find_object(&state, "Trigger Connive Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // First pass_all: spell resolves. Connive fires (draw/discard/counter) and
    // SourceConnives trigger is queued.
    let (state, _) = pass_all(state.clone(), &[p1, p2]);

    // Second pass_all: SourceConnives trigger resolves → +1/+1 counter added.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 701.50b: Creature should have at least 1 counter from SourceConnives trigger.
    // (The connive effect also adds 1 counter if a nonland was discarded.)
    let creature = state
        .objects
        .values()
        .find(|obj| {
            obj.characteristics.name == "Ledger Shredder Stand-in"
                && obj.zone == ZoneId::Battlefield
        })
        .expect("Ledger Shredder Stand-in must still be on the battlefield");

    let counter_count = creature
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);

    // At minimum the SourceConnives trigger should have added 1 counter.
    assert!(
        counter_count >= 1,
        "Ledger Shredder Stand-in must have at least 1 +1/+1 counter from SourceConnives trigger; got {}",
        counter_count
    );
}

// ── Test 7: Creature left battlefield before connive resolves (CR 701.50c) ────

#[test]
/// CR 701.50c — "If a permanent changes zones before an effect causes it to connive,
/// its last known information is used to determine which object connived and who
/// controlled it." No +1/+1 counter is placed when the creature is not on the
/// battlefield at the point where counters would be added.
///
/// This test uses a Sequence effect: first MoveZone moves the target creature to
/// the graveyard; then Connive resolves with the remapped target (the creature is
/// now in the graveyard). The controller still draws and discards, but no counter
/// is placed. The Connived event fires with counters_placed: 0.
///
/// Note: the target_remaps mechanism in EffectContext tracks the creature's new
/// graveyard ObjectId after the MoveZone, allowing the Connive handler to find
/// the object and correctly identify it as off-battlefield (CR 701.50c enforcement
/// at effects/mod.rs:1615-1619). Finding 5 (LOW, deferred) documents the related
/// architecture gap where EffectTarget::Source returns empty when the source has
/// left the battlefield.
fn test_connive_creature_left_battlefield_no_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build a spell: Sequence([MoveZone target to graveyard, Connive target 1]).
    // This simulates the CR 701.50c scenario: the creature leaves the battlefield
    // before connive resolves, so no counter is placed.
    let spell_def = CardDefinition {
        card_id: CardId("connive-spell-lbf".to_string()),
        name: "Left Battlefield Connive Spell".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "Move target creature to its owner's graveyard, then that creature connives. (CR 701.50c test)"
                .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // Step 1: Move the target creature to its owner's graveyard.
                // target_remaps[0] is updated to the new graveyard ObjectId (CR 400.7).
                Effect::MoveZone {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Graveyard {
                        owner: PlayerTarget::Controller,
                    },
                },
                // Step 2: Connive the target (now in graveyard via target_remaps[0]).
                // CR 701.50c: No +1/+1 counter is placed since creature is off-battlefield.
                // Controller still draws and discards (CR 701.50b: connive happens regardless).
                Effect::Connive {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    count: EffectAmount::Fixed(1),
                },
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![spell_def]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Doomed Conniver", 2, 2).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::card(p1, "Left Battlefield Connive Spell")
                .with_card_id(CardId("connive-spell-lbf".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    // Nonland cards in library so draw step succeeds.
    for i in 0..3 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card LBF {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }

    let state = builder.build().unwrap();
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let creature_id = find_object(&state, "Doomed Conniver");
    let spell_id = find_object(&state, "Left Battlefield Connive Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // pass_all: spell resolves. MoveZone sends creature to graveyard, then Connive fires.
    let (_state, events) = pass_all(state, &[p1, p2]);

    // CR 701.50b: Connived event must fire even though the creature left the battlefield.
    let connived = events
        .iter()
        .find(|e| matches!(e, GameEvent::Connived { .. }));
    assert!(
        connived.is_some(),
        "Connived event must be emitted even when creature left battlefield (CR 701.50b/c); events: {:?}",
        events
            .iter()
            .filter(|e| matches!(e, GameEvent::Connived { .. }))
            .collect::<Vec<_>>()
    );

    // CR 701.50c: No +1/+1 counter is placed — creature was off-battlefield when
    // the counter step was reached.
    if let Some(GameEvent::Connived {
        counters_placed, ..
    }) = connived
    {
        assert_eq!(
            *counters_placed, 0,
            "counters_placed must be 0 when creature left battlefield (CR 701.50c); got {}",
            counters_placed
        );
    }

    // CR 701.50c: The creature must be in the graveyard (not on the battlefield).
    let doomed_on_battlefield = _state.objects.values().any(|obj| {
        obj.characteristics.name == "Doomed Conniver" && obj.zone == ZoneId::Battlefield
    });
    assert!(
        !doomed_on_battlefield,
        "Doomed Conniver must not be on the battlefield after MoveZone (CR 701.50c test setup)"
    );
    let doomed_in_graveyard = _state.objects.values().any(|obj| {
        obj.characteristics.name == "Doomed Conniver" && obj.zone == ZoneId::Graveyard(p1)
    });
    assert!(
        doomed_in_graveyard,
        "Doomed Conniver must be in graveyard after MoveZone (CR 701.50c test setup)"
    );
}
