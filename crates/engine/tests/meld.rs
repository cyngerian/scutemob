//! Meld mechanic tests (CR 701.42 / CR 712.4 / CR 712.8g).
//!
//! Meld is a keyword action that appears on one card in a meld pair. It exiles
//! both cards and puts them onto the battlefield as a single melded permanent
//! with the combined back face characteristics.
//!
//! Key rules verified:
//! - CR 701.42a: Meld exiles both cards, creates melded permanent with back face.
//! - CR 712.8g: Melded permanent has only combined back face characteristics.
//! - CR 712.4c: Meld cards cannot be transformed.
//! - CR 701.42c: If partner not present, nothing happens.
//! - Zone-change splitting: melded permanent leaving battlefield splits into two cards.

use mtg_engine::{
    calculate_characteristics, enrich_spec_from_def, process_command, AbilityDefinition,
    CardDefinition, CardFace, CardId, CardRegistry, CardType, Command, Effect, GameEvent,
    GameState, GameStateBuilder, KeywordAbility, ManaCost, MeldPair, ObjectId, ObjectSpec,
    PlayerId, SubType, TypeLine, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    // Check raw characteristics first, then try layer-resolved characteristics.
    state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.zone == zone && {
                if obj.characteristics.name == name {
                    true
                } else if let Some(chars) = calculate_characteristics(state, obj.id) {
                    chars.name == name
                } else {
                    false
                }
            }
        })
        .map(|(id, _)| *id)
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

// ── Mock card definitions ────────────────────────────────────────────────────

/// The melded result card definition — only holds back_face.
fn melded_township_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("melded-township".to_string()),
        name: "Melded Township Holder".to_string(),
        mana_cost: None,
        types: TypeLine::default(),
        oracle_text: String::new(),
        abilities: vec![],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Hanweir Township".to_string(),
            mana_cost: None,
            types: TypeLine {
                supertypes: [mtg_engine::SuperType::Legendary].into_iter().collect(),
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Eldrazi".to_string()), SubType("Ooze".to_string())]
                    .into_iter()
                    .collect(),
            },
            oracle_text: "Trample, haste".to_string(),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Trample),
                AbilityDefinition::Keyword(KeywordAbility::Haste),
            ],
            power: Some(7),
            toughness: Some(4),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

/// Mock "Battlements" — the card with the meld ability.
fn battlements_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-battlements".to_string()),
        name: "Battlements".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Meld with Garrison".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: mtg_engine::Cost::Tap,
            effect: Effect::Meld,
            timing_restriction: None,
            targets: vec![],
        }],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
        meld_pair: Some(MeldPair {
            pair_card_id: CardId("mock-garrison".to_string()),
            melded_card_id: CardId("melded-township".to_string()),
        }),
        ..Default::default()
    }
}

/// Mock "Garrison" — the other half of the meld pair.
fn garrison_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-garrison".to_string()),
        name: "Garrison".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Human".to_string()), SubType("Soldier".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "(Melds with Battlements.)".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(3),
        color_indicator: None,
        back_face: None,
        meld_pair: Some(MeldPair {
            pair_card_id: CardId("mock-battlements".to_string()),
            melded_card_id: CardId("melded-township".to_string()),
        }),
        ..Default::default()
    }
}

fn meld_registry() -> std::sync::Arc<CardRegistry> {
    CardRegistry::new(vec![
        battlements_def(),
        garrison_def(),
        melded_township_def(),
    ])
}

fn meld_defs() -> std::collections::HashMap<String, CardDefinition> {
    let mut map = std::collections::HashMap::new();
    let b = battlements_def();
    map.insert(b.name.clone(), b);
    let g = garrison_def();
    map.insert(g.name.clone(), g);
    let m = melded_township_def();
    map.insert(m.name.clone(), m);
    map
}

fn setup_meld_game() -> GameState {
    let registry = meld_registry();
    let defs = meld_defs();
    let b_spec = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Battlements")
            .with_card_id(CardId("mock-battlements".to_string()))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let g_spec = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Garrison")
            .with_card_id(CardId("mock-garrison".to_string()))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    GameStateBuilder::four_player()
        .with_registry(registry)
        .object(b_spec)
        .object(g_spec)
        .build()
        .unwrap()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[test]
/// CR 701.42a — basic meld: exile both cards, melded permanent enters battlefield.
fn test_meld_basic_exile_and_enter() {
    let state = setup_meld_game();
    let battlements_id = find_object(&state, "Battlements");

    // Activate the meld ability (index 0 = the Meld activated ability).
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: battlements_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .expect("meld activation should succeed");

    // Resolve the ability by passing priority.
    let (state, _) = pass_all(state, &[p(1), p(2), p(3), p(4)]);

    // Battlements and Garrison should be in exile.
    assert!(
        find_in_zone(&state, "Battlements", ZoneId::Exile).is_some()
            || find_in_zone(&state, "Battlements", ZoneId::Battlefield).is_none(),
        "Battlements should have left the battlefield"
    );

    // The melded permanent should be on the battlefield with the back face name.
    let melded = find_in_zone(&state, "Hanweir Township", ZoneId::Battlefield);
    assert!(
        melded.is_some(),
        "melded permanent 'Hanweir Township' should be on the battlefield"
    );
}

#[test]
/// CR 712.8g — melded permanent has combined back face characteristics.
fn test_meld_characteristics_from_back_face() {
    let state = setup_meld_game();
    let battlements_id = find_object(&state, "Battlements");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: battlements_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p(1), p(2), p(3), p(4)]);

    let melded_id = find_in_zone(&state, "Hanweir Township", ZoneId::Battlefield)
        .expect("melded permanent should exist");
    let chars = calculate_characteristics(&state, melded_id)
        .expect("characteristics should be available for melded permanent");

    assert_eq!(chars.name, "Hanweir Township", "CR 712.8g: melded name");
    assert_eq!(chars.power, Some(7), "CR 712.8g: melded power = 7");
    assert_eq!(chars.toughness, Some(4), "CR 712.8g: melded toughness = 4");
    assert!(
        chars.keywords.contains(&KeywordAbility::Trample),
        "CR 712.8g: melded has trample"
    );
    assert!(
        chars.keywords.contains(&KeywordAbility::Haste),
        "CR 712.8g: melded has haste"
    );
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "CR 712.8g: melded is a creature"
    );
    assert!(
        chars.supertypes.contains(&mtg_engine::SuperType::Legendary),
        "CR 712.8g: melded is legendary"
    );
}

#[test]
/// CR 701.42c — meld fails if partner is not on the battlefield.
fn test_meld_fails_partner_not_present() {
    let registry = meld_registry();
    // Only Battlements on battlefield, no Garrison.
    let defs = meld_defs();
    let b_spec = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Battlements")
            .with_card_id(CardId("mock-battlements".to_string()))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .object(b_spec)
        .build()
        .unwrap();

    let battlements_id = find_object(&state, "Battlements");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: battlements_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p(1), p(2), p(3), p(4)]);

    // Battlements should still be on the battlefield (meld fizzled).
    // The source was tapped as cost, but the effect didn't exile anything.
    assert!(
        find_in_zone(&state, "Hanweir Township", ZoneId::Battlefield).is_none(),
        "CR 701.42c: melded permanent should not exist when partner is missing"
    );
}

#[test]
/// CR 701.42b — meld fails if partner is controlled by a different player.
fn test_meld_fails_different_controller() {
    let registry = meld_registry();
    let defs = meld_defs();
    let b_spec = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Battlements")
            .with_card_id(CardId("mock-battlements".to_string()))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let g_spec = enrich_spec_from_def(
        ObjectSpec::card(p(2), "Garrison")
            .with_card_id(CardId("mock-garrison".to_string()))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let state = GameStateBuilder::four_player()
        .with_registry(registry)
        .object(b_spec)
        .object(g_spec)
        .build()
        .unwrap();

    let battlements_id = find_object(&state, "Battlements");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: battlements_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p(1), p(2), p(3), p(4)]);

    assert!(
        find_in_zone(&state, "Hanweir Township", ZoneId::Battlefield).is_none(),
        "CR 701.42b: meld should fail when partner has a different controller"
    );
}

#[test]
/// Zone-change splitting: when a melded permanent leaves the battlefield,
/// both cards go to the destination zone as separate objects.
fn test_meld_zone_change_splitting() {
    let state = setup_meld_game();
    let battlements_id = find_object(&state, "Battlements");

    // Meld the pair.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: battlements_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p(1), p(2), p(3), p(4)]);

    // Verify melded permanent exists.
    let melded_id = find_in_zone(&state, "Hanweir Township", ZoneId::Battlefield)
        .expect("melded permanent should exist");

    // Move the melded permanent to graveyard directly (simulates destruction).
    let mut state = state;
    let gy = ZoneId::Graveyard(p(1));
    let _ = state.move_object_to_zone(melded_id, gy);

    // Both original cards should be in the graveyard (zone-change splitting).
    let battlements_in_gy = state
        .objects
        .values()
        .any(|obj| obj.zone == gy && obj.card_id == Some(CardId("mock-battlements".to_string())));
    let garrison_in_gy = state
        .objects
        .values()
        .any(|obj| obj.zone == gy && obj.card_id == Some(CardId("mock-garrison".to_string())));

    assert!(
        battlements_in_gy,
        "Battlements card should be in graveyard after melded permanent leaves battlefield"
    );
    assert!(
        garrison_in_gy,
        "Garrison card should be in graveyard after melded permanent leaves battlefield (zone-change splitting)"
    );

    // Melded permanent should no longer be on the battlefield.
    assert!(
        find_in_zone(&state, "Hanweir Township", ZoneId::Battlefield).is_none(),
        "melded permanent should not be on battlefield after zone change"
    );
}

#[test]
/// CR 712.4c — meld cards cannot be transformed.
fn test_meld_cards_cannot_transform() {
    let state = setup_meld_game();
    let battlements_id = find_object(&state, "Battlements");

    // Try to transform a meld card — should be silently ignored.
    let (state, _events) = process_command(
        state,
        Command::Transform {
            player: p(1),
            permanent: battlements_id,
        },
    )
    .expect("transform command should succeed (silently no-op for meld cards)");

    // Battlements should still be on the battlefield, untransformed.
    let obj = state.objects.get(&battlements_id).unwrap();
    assert!(
        !obj.is_transformed,
        "CR 712.4c: meld cards cannot be transformed"
    );
}

#[test]
/// CR 712.8g — melded permanent's meld_component is set correctly.
fn test_meld_component_tracking() {
    let state = setup_meld_game();
    let battlements_id = find_object(&state, "Battlements");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: battlements_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p(1), p(2), p(3), p(4)]);

    let melded_id = find_in_zone(&state, "Hanweir Township", ZoneId::Battlefield)
        .expect("melded permanent should exist");
    let melded_obj = state.objects.get(&melded_id).unwrap();

    assert_eq!(
        melded_obj.meld_component,
        Some(CardId("mock-garrison".to_string())),
        "melded permanent should track the partner's CardId"
    );
    assert_eq!(
        melded_obj.card_id,
        Some(CardId("mock-battlements".to_string())),
        "melded permanent's card_id should be the initiator's CardId"
    );
}
