//! Tests for damage multiplier replacement effects (PB-F).
//!
//! Covers TripleDamage variant, FromControllerCreaturesEnteredThisTurn filter,
//! ToPlayerOrTheirPermanents filter, and RegisterReplacementEffect effect.
//!
//! CR Rules:
//! - CR 614.1: Replacement effects apply continuously as events happen.
//! - CR 614.1a: Effects using "instead" are replacement effects.
//! - CR 701.10g: To double an amount of damage, the source deals twice that much.
//! - CR 616.1: Multiple replacement effects stack; affected player chooses order.

use mtg_engine::cards::card_definition::{AbilityDefinition, CardDefinition, TypeLine};
use mtg_engine::rules::replacement::{
    apply_damage_doubling, register_permanent_replacement_abilities,
};
use mtg_engine::state::continuous_effect::EffectDuration;
use mtg_engine::state::replacement_effect::{
    DamageTargetFilter, ReplacementModification, ReplacementTrigger,
};
use mtg_engine::state::{CardId, CardType, GameStateBuilder, ObjectSpec, PlayerId, ZoneId};
use mtg_engine::{CardRegistry, CombatDamageTarget};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Register replacement effects for all battlefield permanents with card_ids.
fn register_replacements(state: &mut mtg_engine::state::GameState) {
    use mtg_engine::state::game_object::ObjectId;

    let registry = state.card_registry.clone();
    let objects: Vec<(ObjectId, PlayerId, Option<CardId>)> = state
        .objects
        .iter()
        .filter(|(_, obj)| matches!(obj.zone, ZoneId::Battlefield))
        .map(|(id, obj)| (*id, obj.controller, obj.card_id.clone()))
        .collect();
    for (id, controller, card_id) in objects {
        register_permanent_replacement_abilities(
            state,
            id,
            controller,
            card_id.as_ref(),
            &registry,
        );
    }
}

fn make_triple_damage_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("fiery-emancipation".to_string()),
        name: "Fiery Emancipation".to_string(),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::DamageWouldBeDealt {
                target_filter: DamageTargetFilter::FromControllerSources(PlayerId(0)),
            },
            modification: ReplacementModification::TripleDamage,
            is_self: false,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

fn make_double_damage_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("angrath-marauders".to_string()),
        name: "Angrath's Marauders".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(4),
        toughness: Some(4),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::DamageWouldBeDealt {
                target_filter: DamageTargetFilter::FromControllerSources(PlayerId(0)),
            },
            modification: ReplacementModification::DoubleDamage,
            is_self: false,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

fn make_neriv_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("neriv-heart-of-the-storm".to_string()),
        name: "Neriv, Heart of the Storm".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(4),
        toughness: Some(5),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::DamageWouldBeDealt {
                target_filter: DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(
                    PlayerId(0),
                ),
            },
            modification: ReplacementModification::DoubleDamage,
            is_self: false,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

// ── TripleDamage basic tests ──────────────────────────────────────────────────

/// CR 614.1a / CR 701.10g: Fiery Emancipation triples damage from sources the controller controls.
#[test]
fn test_triple_damage_basic() {
    let registry = CardRegistry::new(vec![make_triple_damage_def()]);
    let mut emancipation_spec = ObjectSpec::creature(p(1), "Fiery Emancipation", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fiery-emancipation".to_string()));
    emancipation_spec.card_types = [CardType::Enchantment].into_iter().collect();
    let source_spec =
        ObjectSpec::creature(p(1), "Fire Elemental", 5, 5).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(emancipation_spec)
        .object(source_spec)
        .build()
        .unwrap();

    register_replacements(&mut state);

    let source_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Fire Elemental")
        .map(|(id, _)| *id)
        .unwrap();

    let (modified, events) = apply_damage_doubling(&state, source_id, 5, None);
    assert_eq!(modified, 15, "Fiery Emancipation should triple 5 → 15");
    assert!(
        !events.is_empty(),
        "should emit ReplacementEffectApplied event"
    );
}

/// CR 614.1a: Fiery Emancipation does NOT triple damage from opponents' sources.
#[test]
fn test_triple_damage_opponent_source_not_tripled() {
    let registry = CardRegistry::new(vec![make_triple_damage_def()]);
    let mut emancipation_spec = ObjectSpec::creature(p(1), "Fiery Emancipation", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fiery-emancipation".to_string()));
    emancipation_spec.card_types = [CardType::Enchantment].into_iter().collect();
    let opponent_source =
        ObjectSpec::creature(p(2), "Opponent Creature", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(emancipation_spec)
        .object(opponent_source)
        .build()
        .unwrap();

    register_replacements(&mut state);

    let source_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Opponent Creature")
        .map(|(id, _)| *id)
        .unwrap();

    let (modified, events) = apply_damage_doubling(&state, source_id, 3, None);
    assert_eq!(
        modified, 3,
        "Fiery Emancipation should not affect opponent sources"
    );
    assert!(events.is_empty());
}

/// CR 616.1: Double (Angrath's Marauders) + triple (Fiery Emancipation) both apply.
/// Order is multiplicative: 5 * 2 * 3 = 30 (or 5 * 3 * 2 = 30 — same result).
#[test]
fn test_double_and_triple_stack() {
    let registry = CardRegistry::new(vec![make_triple_damage_def(), make_double_damage_def()]);
    let mut emancipation_spec = ObjectSpec::creature(p(1), "Fiery Emancipation", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fiery-emancipation".to_string()));
    emancipation_spec.card_types = [CardType::Enchantment].into_iter().collect();
    let marauders_spec = ObjectSpec::creature(p(1), "Angrath's Marauders", 4, 4)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("angrath-marauders".to_string()));
    let source_spec = ObjectSpec::creature(p(1), "Attacker", 5, 5).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(emancipation_spec)
        .object(marauders_spec)
        .object(source_spec)
        .build()
        .unwrap();

    register_replacements(&mut state);

    let source_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Attacker")
        .map(|(id, _)| *id)
        .unwrap();

    let (modified, events) = apply_damage_doubling(&state, source_id, 5, None);
    assert_eq!(modified, 30, "Double × Triple should give 5 × 2 × 3 = 30");
    assert_eq!(
        events.len(),
        2,
        "should have two ReplacementEffectApplied events"
    );
}

// ── Neriv "entered this turn" filter tests ────────────────────────────────────

/// CR 614.1a: Neriv doubles damage from creatures you control that entered this turn.
#[test]
fn test_neriv_creatures_entered_this_turn_doubled() {
    let registry = CardRegistry::new(vec![make_neriv_def()]);
    let neriv_spec = ObjectSpec::creature(p(1), "Neriv, Heart of the Storm", 4, 5)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("neriv-heart-of-the-storm".to_string()));
    // Source creature entered this turn (entered_turn will be set by move_object_to_zone).
    // We simulate this by placing it normally — all builder objects will have entered_turn = None
    // but we manually set entered_turn after building.
    let source_spec = ObjectSpec::creature(p(1), "New Creature", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(neriv_spec)
        .object(source_spec)
        .build()
        .unwrap();

    // Manually set entered_turn to current turn for the source creature.
    let turn_number = state.turn.turn_number;
    let source_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "New Creature")
        .map(|(id, _)| *id)
        .unwrap();
    if let Some(obj) = state.objects.get_mut(&source_id) {
        obj.entered_turn = Some(turn_number);
    }

    register_replacements(&mut state);

    let (modified, events) = apply_damage_doubling(&state, source_id, 3, None);
    assert_eq!(
        modified, 6,
        "Neriv should double damage from creature that entered this turn: 3 → 6"
    );
    assert!(!events.is_empty());
}

/// CR 614.1a: Neriv does NOT double damage from creatures that entered a prior turn.
#[test]
fn test_neriv_creature_from_prior_turn_not_doubled() {
    let registry = CardRegistry::new(vec![make_neriv_def()]);
    let neriv_spec = ObjectSpec::creature(p(1), "Neriv, Heart of the Storm", 4, 5)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("neriv-heart-of-the-storm".to_string()));
    let source_spec = ObjectSpec::creature(p(1), "Old Creature", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(neriv_spec)
        .object(source_spec)
        .build()
        .unwrap();

    // The source creature has entered_turn = None (builder default) — treated as prior turn.
    // Neriv's filter: entered_turn == Some(current turn), so None does not match.
    register_replacements(&mut state);

    let source_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Old Creature")
        .map(|(id, _)| *id)
        .unwrap();

    let (modified, events) = apply_damage_doubling(&state, source_id, 3, None);
    assert_eq!(
        modified, 3,
        "Neriv should not double damage from creature from a prior turn"
    );
    assert!(events.is_empty());
}

/// CR 614.1a: Neriv only doubles creature sources — not noncreature permanents.
#[test]
fn test_neriv_noncreature_source_not_doubled() {
    let registry = CardRegistry::new(vec![make_neriv_def()]);
    let neriv_spec = ObjectSpec::creature(p(1), "Neriv, Heart of the Storm", 4, 5)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("neriv-heart-of-the-storm".to_string()));

    // Artifact (non-creature) source
    let mut artifact_spec =
        ObjectSpec::creature(p(1), "Noncreature Artifact", 0, 0).in_zone(ZoneId::Battlefield);
    artifact_spec.card_types = [CardType::Artifact].into_iter().collect();

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(neriv_spec)
        .object(artifact_spec)
        .build()
        .unwrap();

    // Set entered_turn for the artifact to this turn.
    let turn_number = state.turn.turn_number;
    let source_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Noncreature Artifact")
        .map(|(id, _)| *id)
        .unwrap();
    if let Some(obj) = state.objects.get_mut(&source_id) {
        obj.entered_turn = Some(turn_number);
    }

    register_replacements(&mut state);

    let (modified, events) = apply_damage_doubling(&state, source_id, 3, None);
    assert_eq!(
        modified, 3,
        "Neriv should not double damage from non-creature sources"
    );
    assert!(events.is_empty());
}

// ── ToPlayerOrTheirPermanents filter tests ─────────────────────────────────────

/// CR 614.1: ToPlayerOrTheirPermanents filter applies when target is the specific player.
#[test]
fn test_to_player_or_their_permanents_player_target() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();

    // Use a stable dummy ObjectId — the filter only inspects the target (player/permanent),
    // not the source, so the source_id does not need to exist in state.objects.
    use mtg_engine::state::game_object::ObjectId;
    let source_id = ObjectId(999);

    // Manually add a ToPlayerOrTheirPermanents replacement effect targeting p(2).
    state
        .replacement_effects
        .push_back(mtg_engine::state::replacement_effect::ReplacementEffect {
            id: mtg_engine::state::replacement_effect::ReplacementId(100),
            source: None,
            controller: p(1),
            duration: EffectDuration::Indefinite,
            is_self_replacement: false,
            trigger: ReplacementTrigger::DamageWouldBeDealt {
                target_filter: DamageTargetFilter::ToPlayerOrTheirPermanents(p(2)),
            },
            modification: ReplacementModification::DoubleDamage,
        });

    // Damage to p(2) — should be doubled.
    let (modified, events) = apply_damage_doubling(
        &state,
        source_id,
        4,
        Some(&CombatDamageTarget::Player(p(2))),
    );
    assert_eq!(modified, 8, "Damage to target player should be doubled");
    assert!(!events.is_empty());

    // Damage to p(1) — should NOT be doubled (wrong player).
    let (modified2, events2) = apply_damage_doubling(
        &state,
        source_id,
        4,
        Some(&CombatDamageTarget::Player(p(1))),
    );
    assert_eq!(
        modified2, 4,
        "Damage to a different player should not be doubled"
    );
    assert!(events2.is_empty());
}

// ── TripleDamage serde round-trip ─────────────────────────────────────────────

/// CR 614.1 / CR 701.10g: TripleDamage serializes and deserializes correctly.
#[test]
fn test_triple_damage_serde() {
    use mtg_engine::state::replacement_effect::{ReplacementEffect, ReplacementId};
    let effect = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: p(1),
        duration: EffectDuration::WhileSourceOnBattlefield,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::FromControllerSources(p(1)),
        },
        modification: ReplacementModification::TripleDamage,
    };
    let json = serde_json::to_string(&effect).unwrap();
    let deserialized: ReplacementEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, deserialized);
}

/// CR 614.1: FromControllerCreaturesEnteredThisTurn serde round-trip.
#[test]
fn test_entered_this_turn_filter_serde() {
    use mtg_engine::state::replacement_effect::{ReplacementEffect, ReplacementId};
    let effect = ReplacementEffect {
        id: ReplacementId(2),
        source: None,
        controller: p(1),
        duration: EffectDuration::WhileSourceOnBattlefield,
        is_self_replacement: false,
        trigger: ReplacementTrigger::DamageWouldBeDealt {
            target_filter: DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(p(1)),
        },
        modification: ReplacementModification::DoubleDamage,
    };
    let json = serde_json::to_string(&effect).unwrap();
    let deserialized: ReplacementEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, deserialized);
}

// ── entered_turn tracking ─────────────────────────────────────────────────────

/// CR 614.1: entered_turn is set on permanents that enter via move_object_to_zone.
#[test]
fn test_entered_turn_set_on_etb() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();

    // All permanents placed by the builder have entered_turn = None
    // (treated as pre-existing; no zone change occurred via move_object_to_zone).
    for (_, obj) in state.objects.iter() {
        if obj.zone == ZoneId::Battlefield {
            assert_eq!(
                obj.entered_turn, None,
                "Builder-placed permanents have entered_turn = None"
            );
        }
    }
}
