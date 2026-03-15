//! Tests for counter-placement replacement effects (CR 122.6, CR 614.1).
//!
//! PB-12: DoubleCounters, HalveCounters, AddExtraCounter replacement
//! modifications on WouldPlaceCounters trigger.

use mtg_engine::cards::card_definition::{AbilityDefinition, CardDefinition, TypeLine};
use mtg_engine::state::game_object::ManaCost;
use mtg_engine::state::replacement_effect::{
    ObjectFilter, PlayerFilter, ReplacementModification, ReplacementTrigger,
};
use mtg_engine::state::{
    CardId, CardType, CounterType, GameStateBuilder, ObjectSpec, PlayerId, ZoneId,
};
use mtg_engine::CardRegistry;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn make_vorinclex_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vorinclex-test".to_string()),
        name: "Vorinclex Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // Double counters placed by controller
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldPlaceCounters {
                    placer_filter: PlayerFilter::Specific(PlayerId(0)),
                    receiver_filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::DoubleCounters,
                is_self: false,
                unless_condition: None,
            },
            // Halve counters placed by opponents
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldPlaceCounters {
                    placer_filter: PlayerFilter::OpponentsOf(PlayerId(0)),
                    receiver_filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::HalveCounters,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

fn make_pir_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("pir-test".to_string()),
        name: "Pir Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldPlaceCounters {
                placer_filter: PlayerFilter::Any,
                receiver_filter: ObjectFilter::ControlledBy(PlayerId(0)),
            },
            modification: ReplacementModification::AddExtraCounter,
            is_self: false,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

fn make_target_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("target-creature".to_string()),
        name: "Target Creature".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Register replacement effects for all battlefield permanents with card_ids.
/// The builder doesn't do this automatically — it happens at ETB time in the engine.
fn register_replacement_effects(state: &mut mtg_engine::state::GameState) {
    use mtg_engine::state::game_object::ObjectId;
    use mtg_engine::state::zone::ZoneId;

    let registry = state.card_registry.clone();
    let battlefield_objects: Vec<(
        ObjectId,
        PlayerId,
        Option<mtg_engine::state::player::CardId>,
    )> = state
        .objects
        .iter()
        .filter(|(_, obj)| matches!(obj.zone, ZoneId::Battlefield))
        .map(|(id, obj)| (*id, obj.controller, obj.card_id.clone()))
        .collect();

    for (obj_id, controller, card_id) in &battlefield_objects {
        mtg_engine::rules::replacement::register_permanent_replacement_abilities(
            state,
            *obj_id,
            *controller,
            card_id.as_ref(),
            &registry,
        );
    }
}

fn build_vorinclex_state() -> (mtg_engine::state::GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![make_vorinclex_def(), make_target_creature_def()]);

    let mut vorinclex_spec =
        ObjectSpec::creature(p1, "Vorinclex Test", 6, 6).in_zone(ZoneId::Battlefield);
    vorinclex_spec.card_id = Some(CardId("vorinclex-test".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(vorinclex_spec)
        .object(ObjectSpec::creature(p1, "Target Creature", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    register_replacement_effects(&mut state);
    (state, p1, p2)
}

fn build_pir_state() -> (mtg_engine::state::GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![make_pir_def(), make_target_creature_def()]);

    let mut pir_spec = ObjectSpec::creature(p1, "Pir Test", 1, 1).in_zone(ZoneId::Battlefield);
    pir_spec.card_id = Some(CardId("pir-test".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(pir_spec)
        .object(ObjectSpec::creature(p1, "Target Creature", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    register_replacement_effects(&mut state);
    (state, p1, p2)
}

fn find_target(state: &mtg_engine::state::GameState) -> mtg_engine::state::game_object::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Target Creature")
        .map(|(id, _)| *id)
        .unwrap()
}

/// CR 122.6 — Vorinclex doubles counters placed by its controller
#[test]
fn test_vorinclex_doubles_controller_counters() {
    let (state, p1, _) = build_vorinclex_state();
    let target_id = find_target(&state);

    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target_id,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 6, "Vorinclex should double 3 → 6 counters");
    assert!(!events.is_empty(), "Should emit ReplacementEffectApplied");
}

/// CR 122.6 — Vorinclex halves counters placed by opponents (round down)
#[test]
fn test_vorinclex_halves_opponent_counters() {
    let (state, _, p2) = build_vorinclex_state();
    let target_id = find_target(&state);

    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p2,
        target_id,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 1, "Vorinclex should halve 3 → 1 (round down)");
    assert!(!events.is_empty());
}

/// CR 122.6 — Vorinclex halves 1 counter to 0 (round down)
#[test]
fn test_vorinclex_halves_single_counter_to_zero() {
    let (state, _, p2) = build_vorinclex_state();
    let target_id = find_target(&state);

    let (modified, _) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p2,
        target_id,
        &CounterType::PlusOnePlusOne,
        1,
    );
    assert_eq!(modified, 0, "Vorinclex should halve 1 → 0");
}

/// CR 122.6 — Pir adds one extra counter to permanents you control
#[test]
fn test_pir_adds_extra_counter() {
    let (state, p1, _) = build_pir_state();
    let target_id = find_target(&state);

    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target_id,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 4, "Pir should add one extra: 3 → 4");
    assert!(!events.is_empty());
}

/// CR 122.6 — Pir does NOT add extra counter to opponent's permanents
#[test]
fn test_pir_no_extra_counter_on_opponent_permanent() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![make_pir_def(), make_target_creature_def()]);

    let mut pir_spec = ObjectSpec::creature(p1, "Pir Test", 1, 1).in_zone(ZoneId::Battlefield);
    pir_spec.card_id = Some(CardId("pir-test".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(pir_spec)
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    register_replacement_effects(&mut state);
    let target_id = find_target(&state);

    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target_id,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 3, "Pir should not affect opponent's permanents");
    assert!(events.is_empty());
}

/// CR 122.6 — No replacement when no relevant effects registered
#[test]
fn test_no_counter_replacement_without_effects() {
    let registry = CardRegistry::new(vec![make_target_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(ObjectSpec::creature(p(1), "Target Creature", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let target_id = find_target(&state);
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p(1),
        target_id,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 3, "No replacement effects → count unchanged");
    assert!(events.is_empty());
}

/// CR 122.6 — Zero counters remain zero regardless of replacements
#[test]
fn test_counter_replacement_zero_stays_zero() {
    let (state, p1, _) = build_vorinclex_state();
    let target_id = find_target(&state);

    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target_id,
        &CounterType::PlusOnePlusOne,
        0,
    );
    assert_eq!(modified, 0, "Zero counters should remain zero");
    assert!(events.is_empty());
}

/// CR 614.1 — Both Vorinclex and Pir stacking on same creature:
/// controller places 3 counters → Vorinclex doubles (6), Pir adds 1 (7)
#[test]
fn test_vorinclex_and_pir_stack() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![
        make_vorinclex_def(),
        make_pir_def(),
        make_target_creature_def(),
    ]);

    let mut vorinclex_spec =
        ObjectSpec::creature(p1, "Vorinclex Test", 6, 6).in_zone(ZoneId::Battlefield);
    vorinclex_spec.card_id = Some(CardId("vorinclex-test".to_string()));

    let mut pir_spec = ObjectSpec::creature(p1, "Pir Test", 1, 1).in_zone(ZoneId::Battlefield);
    pir_spec.card_id = Some(CardId("pir-test".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(vorinclex_spec)
        .object(pir_spec)
        .object(ObjectSpec::creature(p1, "Target Creature", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    register_replacement_effects(&mut state);
    let target_id = find_target(&state);

    // P1 places 3 counters on own creature.
    // Both Vorinclex (double) and Pir (+1) apply.
    // Order: Vorinclex doubles 3→6, then Pir adds 1→7
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target_id,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 7, "Vorinclex (×2) + Pir (+1): 3 → 6 → 7");
    assert_eq!(events.len(), 2, "Both replacement effects should fire");
}
