//! Craft ability tests (CR 702.167).
//!
//! Craft is an activated ability that exiles the permanent + materials as cost,
//! then returns the card to the battlefield transformed.
//! Key rules verified:
//! - CR 702.167a: Cost is [cost] + exile self + exile [materials].
//! - CR 702.167a: Returns to battlefield transformed ("back face up").
//! - CR 702.167a: "Activate only as a sorcery."
//! - CR 702.167b: Materials can be permanents on battlefield OR cards in graveyard.
//! - CR 702.167c: An ability may refer to the exiled cards used to craft it.
//! - Ruling: If the card isn't a DFC, it stays in exile (no return to battlefield).

use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardFace,
    CardId, CardRegistry, CardType, Command, CraftMaterials, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step,
    TypeLine, ZoneId,
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
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Mock Craft card: "Braided Net" (front) / "Braided Quipu" (back).
/// Front: {1}{G} Artifact. Craft with 2 artifacts {2}{G}.
/// Back: Artifact creature 3/3 with reach.
fn braided_net_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-braided-net".to_string()),
        name: "Braided Net".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Craft with artifacts 2 {2}{G}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Craft),
            AbilityDefinition::Craft {
                cost: ManaCost {
                    green: 1,
                    generic: 2,
                    ..Default::default()
                },
                materials: CraftMaterials::Artifacts(2),
            },
        ],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Braided Quipu".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Artifact, CardType::Creature]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
            oracle_text: "Reach. When Braided Quipu enters, draw a card for each artifact exiled to craft it.".to_string(),
            abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Reach)],
            power: Some(3),
            toughness: Some(3),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

/// A simple artifact card (used as craft material).
fn mock_artifact_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-artifact".to_string()),
        name: "Mock Artifact".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

/// A non-DFC card (no back face) with Craft — used to test "stays in exile" ruling.
fn non_dfc_craft_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-non-dfc-craft".to_string()),
        name: "Non-DFC Craft".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Craft with artifacts 1 {2}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Craft),
            AbilityDefinition::Craft {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
                materials: CraftMaterials::Artifacts(1),
            },
        ],
        color_indicator: None,
        back_face: None, // Not a DFC
        ..Default::default()
    }
}

fn net_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Braided Net")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-braided-net".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_keyword(KeywordAbility::Craft)
}

fn artifact_on_battlefield(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-artifact".to_string()))
        .with_types(vec![CardType::Artifact])
}

// ── Test 1: Basic craft — exile self + materials, return transformed ───────────

/// CR 702.167a: Craft exiles self + materials as cost, then returns to battlefield transformed.
#[test]
fn test_craft_basic_exile_and_transform() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![braided_net_def(), mock_artifact_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(net_on_battlefield(p1))
        .object(artifact_on_battlefield(p1, "Artifact A"))
        .object(artifact_on_battlefield(p1, "Artifact B"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay craft cost {2}{G}.
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

    let net_id = find_in_zone(&state, "Braided Net", ZoneId::Battlefield)
        .expect("Braided Net should be on battlefield");
    let art_a_id = find_object(&state, "Artifact A");
    let art_b_id = find_object(&state, "Artifact B");

    let (state, events) = process_command(
        state,
        Command::ActivateCraft {
            player: p1,
            source: net_id,
            material_ids: vec![art_a_id, art_b_id],
        },
    )
    .expect("ActivateCraft should succeed");

    // Source came back to battlefield as back face (Braided Quipu), transformed.
    // obj.characteristics.name still shows "Braided Net" (front face name set at creation),
    // so we must search by card_id + is_transformed to find the returned DFC.

    // CR 702.167a: The card (DFC) returns transformed.
    let transformed_id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.card_id == Some(CardId("mock-braided-net".to_string()))
                && obj.is_transformed
        })
        .map(|(id, _)| *id);

    assert!(
        transformed_id.is_some(),
        "Braided Quipu (back face) should be on the battlefield with is_transformed=true (CR 702.167a)"
    );

    // CraftActivated event should have been emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CraftActivated { player, .. } if *player == p1)),
        "CraftActivated event should be emitted"
    );

    // PermanentEnteredBattlefield event should have been emitted for the returned DFC.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })),
        "PermanentEnteredBattlefield should be emitted when DFC returns transformed (CR 702.167a)"
    );
}

// ── Test 2: Craft returns back face — characteristics come from back face ──────

/// CR 702.167a + CR 712.8e: After craft, the permanent shows back face characteristics.
/// The back face mana value still uses front face cost (CR 712.8e).
#[test]
fn test_craft_back_face_characteristics() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![braided_net_def(), mock_artifact_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(net_on_battlefield(p1))
        .object(artifact_on_battlefield(p1, "Artifact A"))
        .object(artifact_on_battlefield(p1, "Artifact B"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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

    let net_id = find_in_zone(&state, "Braided Net", ZoneId::Battlefield).unwrap();
    let art_a_id = find_object(&state, "Artifact A");
    let art_b_id = find_object(&state, "Artifact B");

    let (state, _) = process_command(
        state,
        Command::ActivateCraft {
            player: p1,
            source: net_id,
            material_ids: vec![art_a_id, art_b_id],
        },
    )
    .expect("ActivateCraft should succeed");

    // Find the transformed permanent.
    let quipu_id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.card_id == Some(CardId("mock-braided-net".to_string()))
        })
        .map(|(id, _)| *id)
        .expect("Braided Quipu should be on battlefield");

    // Back face characteristics via the layer system.
    let chars = calculate_characteristics(&state, quipu_id).expect("should have characteristics");

    assert_eq!(
        chars.name, "Braided Quipu",
        "back face name should be Braided Quipu (CR 702.167a + layer system)"
    );
    assert_eq!(chars.power, Some(3), "back face P/T should be 3/3");
    assert_eq!(chars.toughness, Some(3));
    assert!(
        chars.keywords.contains(&KeywordAbility::Reach),
        "back face should have Reach"
    );

    // Back face mana_cost in characteristics is None (no mana cost on back face).
    // Front face mana value {1}{G} = 2 is stored in the registry.
    assert!(
        chars.mana_cost.is_none(),
        "back face has no mana cost in characteristics (CR 712.8e: MV uses front face from registry)"
    );
    let def = state
        .card_registry
        .get(CardId("mock-braided-net".to_string()))
        .expect("definition should be in registry");
    let front_mv = def
        .mana_cost
        .as_ref()
        .map(|mc| mc.mana_value())
        .unwrap_or(0);
    assert_eq!(front_mv, 2, "front face mana value should be 2 (CR 712.8e)");
}

// ── Test 3: Non-DFC stays in exile ────────────────────────────────────────────

/// CR 702.167a ruling: If the permanent that activated craft isn't a DFC,
/// it stays in exile and doesn't return to the battlefield.
#[test]
fn test_craft_non_dfc_stays_exiled() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![non_dfc_craft_def(), mock_artifact_def()]);

    let non_dfc_spec = ObjectSpec::card(p1, "Non-DFC Craft")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-non-dfc-craft".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_keyword(KeywordAbility::Craft);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(non_dfc_spec)
        .object(artifact_on_battlefield(p1, "Artifact A"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let source_id = find_in_zone(&state, "Non-DFC Craft", ZoneId::Battlefield).unwrap();
    let art_a_id = find_object(&state, "Artifact A");

    let (state, events) = process_command(
        state,
        Command::ActivateCraft {
            player: p1,
            source: source_id,
            material_ids: vec![art_a_id],
        },
    )
    .expect("ActivateCraft on non-DFC should succeed (it just stays in exile)");

    // CraftActivated should be emitted (the ability did activate).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CraftActivated { .. })),
        "CraftActivated event should be emitted"
    );

    // The non-DFC card should be in exile, not on the battlefield.
    assert!(
        find_in_zone(&state, "Non-DFC Craft", ZoneId::Battlefield).is_none(),
        "non-DFC craft source should NOT be on battlefield (stays in exile, CR 702.167a ruling)"
    );
    let in_exile = state
        .objects
        .iter()
        .any(|(_, obj)| obj.characteristics.name == "Non-DFC Craft" && obj.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "non-DFC craft source should remain in exile (CR 702.167a ruling)"
    );

    // No PermanentEnteredBattlefield event for the non-DFC.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })),
        "no PermanentEnteredBattlefield for non-DFC craft (stays in exile)"
    );
}

// ── Test 4: Craft tracks exiled materials (CR 702.167c) ──────────────────────

/// CR 702.167c: "An ability of a permanent may refer to the exiled cards used to craft it."
/// Verify craft_exiled_cards is populated on the returned permanent.
#[test]
fn test_craft_tracks_exiled_materials() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![braided_net_def(), mock_artifact_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(net_on_battlefield(p1))
        .object(artifact_on_battlefield(p1, "Artifact A"))
        .object(artifact_on_battlefield(p1, "Artifact B"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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

    let net_id = find_in_zone(&state, "Braided Net", ZoneId::Battlefield).unwrap();
    let art_a_id = find_object(&state, "Artifact A");
    let art_b_id = find_object(&state, "Artifact B");

    let (state, _) = process_command(
        state,
        Command::ActivateCraft {
            player: p1,
            source: net_id,
            material_ids: vec![art_a_id, art_b_id],
        },
    )
    .expect("ActivateCraft should succeed");

    // Find the returned permanent.
    let quipu_id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.card_id == Some(CardId("mock-braided-net".to_string()))
        })
        .map(|(id, _)| *id)
        .expect("Braided Quipu should be on battlefield");

    // craft_exiled_cards should contain the 2 exiled material IDs.
    let exiled_materials = &state.objects[&quipu_id].craft_exiled_cards;
    assert_eq!(
        exiled_materials.len(),
        2,
        "craft_exiled_cards should contain exactly 2 material IDs (CR 702.167c)"
    );
}

// ── Test 5: Craft is sorcery-speed only ───────────────────────────────────────

/// CR 702.167a: "Activate only as a sorcery."
/// Attempting to craft at instant speed (non-empty stack) should be rejected.
#[test]
fn test_craft_sorcery_speed_only() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![braided_net_def(), mock_artifact_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(net_on_battlefield(p1))
        .object(artifact_on_battlefield(p1, "Artifact A"))
        .object(artifact_on_battlefield(p1, "Artifact B"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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

    let net_id = find_in_zone(&state, "Braided Net", ZoneId::Battlefield).unwrap();
    let art_a_id = find_object(&state, "Artifact A");
    let art_b_id = find_object(&state, "Artifact B");

    // Simulate a non-empty stack by pushing a dummy stack object.
    // We can't put real spells without casting, but we can test by setting step to Upkeep.
    // CR 702.167a: "Activate only as a sorcery" = main phase, empty stack, active player.
    // Move to upkeep (not a main phase) to trigger the sorcery-speed check.
    let mut state2 = state.clone();
    state2.turn.step = Step::Upkeep;
    state2.turn.phase = mtg_engine::Phase::Beginning;
    state2.turn.priority_holder = Some(p1);

    let result = process_command(
        state2,
        Command::ActivateCraft {
            player: p1,
            source: net_id,
            material_ids: vec![art_a_id, art_b_id],
        },
    );

    assert!(
        result.is_err(),
        "craft should be rejected during upkeep (sorcery speed only, CR 702.167a)"
    );
}

// ── Test 6: Craft materials from graveyard ────────────────────────────────────

/// CR 702.167b: Materials can be "permanents on the battlefield OR cards in a graveyard."
/// Verify that craft accepts materials from the graveyard.
#[test]
fn test_craft_materials_from_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![braided_net_def(), mock_artifact_def()]);

    // Place one material in the graveyard and one on the battlefield.
    let art_in_graveyard = ObjectSpec::card(p1, "Artifact In GY")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("mock-artifact".to_string()))
        .with_types(vec![CardType::Artifact]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(net_on_battlefield(p1))
        .object(art_in_graveyard)
        .object(artifact_on_battlefield(p1, "Artifact BF"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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

    let net_id = find_in_zone(&state, "Braided Net", ZoneId::Battlefield).unwrap();
    let art_gy_id = find_in_zone(&state, "Artifact In GY", ZoneId::Graveyard(p1))
        .expect("Artifact should be in graveyard");
    let art_bf_id = find_object(&state, "Artifact BF");

    // Craft using one graveyard card + one battlefield artifact as materials.
    let (state, events) = process_command(
        state,
        Command::ActivateCraft {
            player: p1,
            source: net_id,
            material_ids: vec![art_gy_id, art_bf_id],
        },
    )
    .expect("ActivateCraft with graveyard material should succeed (CR 702.167b)");

    // CraftActivated should be emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CraftActivated { .. })),
        "CraftActivated should be emitted"
    );

    // The DFC should have returned to the battlefield transformed.
    let quipu_on_bf = state.objects.iter().any(|(_, obj)| {
        obj.zone == ZoneId::Battlefield
            && obj.card_id == Some(CardId("mock-braided-net".to_string()))
            && obj.is_transformed
    });
    assert!(
        quipu_on_bf,
        "Braided Quipu should be on battlefield transformed after craft (CR 702.167b)"
    );
}
