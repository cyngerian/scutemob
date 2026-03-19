//! Tests for token-doubling, damage-doubling, life-loss-doubling, and
//! search-restriction replacement effects (CR 614.1, PB-12).

use mtg_engine::cards::card_definition::{AbilityDefinition, CardDefinition, TypeLine};
use mtg_engine::state::game_object::ManaCost;
use mtg_engine::state::replacement_effect::{
    DamageTargetFilter, PlayerFilter, ReplacementModification, ReplacementTrigger,
};
use mtg_engine::state::{CardId, CardType, GameStateBuilder, ObjectSpec, PlayerId, ZoneId};
use mtg_engine::CardRegistry;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Register replacement effects for all battlefield permanents with card_ids.
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

// ── Token doubling tests ────────────────────────────────────────────────────

fn make_token_doubler_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("token-doubler".to_string()),
        name: "Token Doubler".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldCreateTokens {
                controller_filter: PlayerFilter::Specific(PlayerId(0)),
            },
            modification: ReplacementModification::DoubleTokens,
            is_self: false,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

/// CR 111.1 / CR 614.1 — token doubling doubles token count
#[test]
fn test_token_doubling_doubles_count() {
    let registry = CardRegistry::new(vec![make_token_doubler_def()]);

    let mut spec = ObjectSpec::creature(p(1), "Token Doubler", 2, 2).in_zone(ZoneId::Battlefield);
    spec.card_id = Some(CardId("token-doubler".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(spec)
        .build()
        .unwrap();

    register_replacement_effects(&mut state);

    let (count, events) =
        mtg_engine::rules::replacement::apply_token_creation_replacement(&state, p(1), 3);
    assert_eq!(count, 6, "Token doubler should double 3 → 6");
    assert!(!events.is_empty());
}

/// CR 111.1 — token doubler does not affect other players
#[test]
fn test_token_doubling_no_effect_on_others() {
    let registry = CardRegistry::new(vec![make_token_doubler_def()]);

    let mut spec = ObjectSpec::creature(p(1), "Token Doubler", 2, 2).in_zone(ZoneId::Battlefield);
    spec.card_id = Some(CardId("token-doubler".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(spec)
        .build()
        .unwrap();

    register_replacement_effects(&mut state);

    let (count, events) =
        mtg_engine::rules::replacement::apply_token_creation_replacement(&state, p(2), 3);
    assert_eq!(
        count, 3,
        "Token doubler only applies to controller's tokens"
    );
    assert!(events.is_empty());
}

// ── Search restriction tests ────────────────────────────────────────────────

fn make_search_restrictor_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("search-restrictor".to_string()),
        name: "Search Restrictor".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldSearchLibrary {
                searcher_filter: PlayerFilter::OpponentsOf(PlayerId(0)),
            },
            modification: ReplacementModification::RestrictSearchTopN(4),
            is_self: false,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

/// CR 701.23 / CR 614.1 — search restriction limits opponent searches
#[test]
fn test_search_restriction_opponents() {
    let registry = CardRegistry::new(vec![make_search_restrictor_def()]);

    let mut spec =
        ObjectSpec::creature(p(1), "Search Restrictor", 2, 1).in_zone(ZoneId::Battlefield);
    spec.card_id = Some(CardId("search-restrictor".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(spec)
        .build()
        .unwrap();

    register_replacement_effects(&mut state);

    // Opponent (P2) searching should be restricted to top 4
    let (restriction, events) =
        mtg_engine::rules::replacement::apply_search_library_replacement(&state, p(2));
    assert_eq!(
        restriction,
        Some(4),
        "Opponent search should be restricted to top 4"
    );
    assert!(!events.is_empty());

    // Controller (P1) searching should NOT be restricted
    let (restriction, events) =
        mtg_engine::rules::replacement::apply_search_library_replacement(&state, p(1));
    assert_eq!(
        restriction, None,
        "Controller search should not be restricted"
    );
    assert!(events.is_empty());
}

// ── Damage doubling tests ────────────────────────────────────────────────────

fn make_damage_doubler_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("damage-doubler".to_string()),
        name: "Damage Doubler".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(3),
        toughness: Some(5),
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

/// CR 614.1 — damage doubling doubles damage from controller's sources
#[test]
fn test_damage_doubling() {
    let registry = CardRegistry::new(vec![make_damage_doubler_def()]);

    let mut spec = ObjectSpec::creature(p(1), "Damage Doubler", 3, 5).in_zone(ZoneId::Battlefield);
    spec.card_id = Some(CardId("damage-doubler".to_string()));

    let source_spec =
        ObjectSpec::creature(p(1), "Damage Source", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(spec)
        .object(source_spec)
        .build()
        .unwrap();

    register_replacement_effects(&mut state);

    let source_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Damage Source")
        .map(|(id, _)| *id)
        .unwrap();

    let (modified, events) =
        mtg_engine::rules::replacement::apply_damage_doubling(&state, source_id, 3, None);
    assert_eq!(modified, 6, "Damage doubler should double 3 → 6");
    assert!(!events.is_empty());
}

/// CR 614.1 — damage doubling does not apply to opponent's sources
#[test]
fn test_damage_doubling_no_effect_on_opponent_sources() {
    let registry = CardRegistry::new(vec![make_damage_doubler_def()]);

    let mut spec = ObjectSpec::creature(p(1), "Damage Doubler", 3, 5).in_zone(ZoneId::Battlefield);
    spec.card_id = Some(CardId("damage-doubler".to_string()));

    let opponent_source =
        ObjectSpec::creature(p(2), "Opponent Source", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(spec)
        .object(opponent_source)
        .build()
        .unwrap();

    register_replacement_effects(&mut state);

    let source_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Opponent Source")
        .map(|(id, _)| *id)
        .unwrap();

    let (modified, events) =
        mtg_engine::rules::replacement::apply_damage_doubling(&state, source_id, 3, None);
    assert_eq!(
        modified, 3,
        "Damage doubler should not affect opponent's sources"
    );
    assert!(events.is_empty());
}

// ── Life loss doubling tests ─────────────────────────────────────────────────

fn make_life_loss_doubler_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("life-loss-doubler".to_string()),
        name: "Life Loss Doubler".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(4),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldLoseLife {
                player_filter: PlayerFilter::OpponentsOf(PlayerId(0)),
            },
            modification: ReplacementModification::DoubleLifeLoss,
            is_self: false,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

/// CR 614.1 — life loss doubling doubles opponent life loss
#[test]
fn test_life_loss_doubling_opponents() {
    let registry = CardRegistry::new(vec![make_life_loss_doubler_def()]);

    let mut spec =
        ObjectSpec::creature(p(1), "Life Loss Doubler", 2, 4).in_zone(ZoneId::Battlefield);
    spec.card_id = Some(CardId("life-loss-doubler".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(spec)
        .build()
        .unwrap();

    register_replacement_effects(&mut state);

    // Opponent life loss should be doubled
    let (modified, events) =
        mtg_engine::rules::replacement::apply_life_loss_doubling(&state, p(2), 5);
    assert_eq!(
        modified, 10,
        "Life loss doubler should double 5 → 10 for opponent"
    );
    assert!(!events.is_empty());

    // Controller's own life loss should NOT be doubled
    let (modified, events) =
        mtg_engine::rules::replacement::apply_life_loss_doubling(&state, p(1), 5);
    assert_eq!(
        modified, 5,
        "Life loss doubler should not affect controller"
    );
    assert!(events.is_empty());
}
