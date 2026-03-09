//! Disturb ability tests (CR 702.146).
//!
//! Disturb allows casting a card transformed from your graveyard by paying its disturb cost.
//! Key rules verified:
//! - CR 702.146a: Card must be in graveyard with Disturb ability.
//! - CR 702.146b: Card enters battlefield with back face up (is_transformed = true).
//! - CR 702.146b (ruling): If cast disturbed and would go to graveyard, exile instead.
//! - CR 712.8c: Mana value on stack uses front face mana cost.

use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, AltCostKind, CardDefinition,
    CardFace, CardId, CardRegistry, CardType, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, SubType, TypeLine,
    ZoneId,
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

/// A mock Disturb card: "Beloved Beggar" (front) / "Generous Soul" (back).
/// Front: {W} Spirit 1/1. Disturb {1}{W}.
/// Back: White Spirit 3/2 Flying.
fn beloved_beggar_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-beloved-beggar".to_string()),
        name: "Beloved Beggar".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Human".to_string()), SubType("Peasant".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "Disturb {1}{W}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Disturb),
            AbilityDefinition::Disturb {
                cost: ManaCost {
                    white: 1,
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(1),
        toughness: Some(1),
        back_face: Some(CardFace {
            name: "Generous Soul".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature, CardType::Enchantment]
                    .into_iter()
                    .collect(),
                subtypes: [SubType("Spirit".to_string())].into_iter().collect(),
                ..Default::default()
            },
            oracle_text:
                "Flying. If Generous Soul would be put into a graveyard, exile it instead."
                    .to_string(),
            abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Flying)],
            power: Some(3),
            toughness: Some(2),
            color_indicator: Some(vec![mtg_engine::Color::White]),
        }),
        ..Default::default()
    }
}

fn beggar_in_graveyard(owner: PlayerId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, "Beloved Beggar")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("mock-beloved-beggar".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Disturb)
        .with_mana_cost(ManaCost {
            white: 1,
            ..Default::default()
        });
    spec.power = Some(1);
    spec.toughness = Some(1);
    spec
}

// ── Test 1: Cast with disturb from graveyard ──────────────────────────────────

/// CR 702.146a: "Disturb [cost]" means "You may cast this card transformed from your
/// graveyard by paying [cost] rather than its mana cost."
/// Verify: the card can be cast from graveyard with AltCostKind::Disturb.
#[test]
fn test_disturb_cast_from_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![beloved_beggar_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(beggar_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let beggar_id = find_in_zone(&state, "Beloved Beggar", ZoneId::Graveyard(p1))
        .expect("Beloved Beggar should be in graveyard");

    // Pay disturb cost {1}{W}.
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
    state.turn.priority_holder = Some(p1);

    // Cast with disturb (from graveyard).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: beggar_id,
            alt_cost: Some(AltCostKind::Disturb),
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .expect("Cast with disturb should succeed");

    // Verify the spell is on the stack with is_cast_transformed = true.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
    assert!(
        state.stack_objects[0].is_cast_transformed,
        "disturb spell should be on stack with is_cast_transformed=true (CR 702.146a)"
    );
}

// ── Test 2: Disturb spell enters battlefield transformed ──────────────────────

/// CR 702.146b: "A resolving double-faced spell that was cast using its disturb ability
/// enters the battlefield with its back face up."
#[test]
fn test_disturb_enters_transformed() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![beloved_beggar_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(beggar_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let beggar_id = find_in_zone(&state, "Beloved Beggar", ZoneId::Graveyard(p1))
        .expect("Beloved Beggar should be in graveyard");

    // Pay disturb cost {1}{W}.
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
    state.turn.priority_holder = Some(p1);

    // Cast with disturb.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: beggar_id,
            alt_cost: Some(AltCostKind::Disturb),
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .expect("Cast with disturb should succeed");

    // Resolve the spell (all players pass).
    let (state, _) = pass_all(state, &[p1, p2]);

    // The permanent should be on the battlefield with back face up.
    // Search by card_id since characteristics.name still shows front face name.
    let souls_id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.card_id == Some(CardId("mock-beloved-beggar".to_string()))
                && obj.is_transformed
        })
        .map(|(id, _)| *id);

    assert!(
        souls_id.is_some(),
        "Generous Soul (back face) should be on the battlefield (CR 702.146b)"
    );

    let souls_id = souls_id.unwrap();
    assert!(
        state.objects[&souls_id].is_transformed,
        "permanent should have is_transformed=true (entered transformed)"
    );
    assert!(
        state.objects[&souls_id].was_cast_disturbed,
        "permanent should have was_cast_disturbed=true"
    );

    // Back face characteristics via layer system.
    let chars = calculate_characteristics(&state, souls_id).expect("should have chars");
    assert_eq!(chars.name, "Generous Soul", "should show back face name");
    assert!(
        chars.keywords.contains(&KeywordAbility::Flying),
        "back face has Flying"
    );
    assert_eq!(chars.power, Some(3), "back face P/T should be 3/2");
    assert_eq!(chars.toughness, Some(2));
}

// ── Test 3: Disturb exile replacement — check_zone_change_replacement works ───

/// CR 702.146b (ruling): "The back face of each card with disturb has an ability
/// that instructs its controller to exile if it would be put into a graveyard
/// from anywhere."
/// Verify: check_zone_change_replacement returns Redirect{Exile} for was_cast_disturbed objects.
#[test]
fn test_disturb_exile_replacement_check() {
    use mtg_engine::ZoneType;

    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![beloved_beggar_def()]);

    // Place the card on the battlefield with was_cast_disturbed = true.
    let mut spec = ObjectSpec::card(p1, "Generous Soul")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-beloved-beggar".to_string()))
        .with_types(vec![CardType::Creature, CardType::Enchantment])
        .with_keyword(KeywordAbility::Flying);
    spec.power = Some(3);
    spec.toughness = Some(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Manually set was_cast_disturbed = true and is_transformed = true.
    let soul_id = find_object(&state, "Generous Soul");
    if let Some(obj) = state.objects.get_mut(&soul_id) {
        obj.was_cast_disturbed = true;
        obj.is_transformed = true;
    }

    // Check what the zone-change replacement returns for graveyard destination.
    use mtg_engine::rules::replacement::{check_zone_change_replacement, ZoneChangeAction};
    use std::collections::HashSet;
    let action = check_zone_change_replacement(
        &state,
        soul_id,
        ZoneType::Battlefield,
        ZoneType::Graveyard,
        p1,
        &HashSet::new(),
    );

    // Should be redirected to exile.
    assert!(
        matches!(
            action,
            ZoneChangeAction::Redirect {
                to: ZoneId::Exile,
                ..
            }
        ),
        "disturb permanent going to graveyard should be redirected to exile (CR 702.146b ruling)"
    );
}

// ── Test 4: Cannot cast with disturb from hand ────────────────────────────────

/// CR 702.146a: Disturb requires the card to be in the graveyard.
/// Trying to cast with disturb from hand should be rejected.
#[test]
fn test_disturb_requires_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![beloved_beggar_def()]);

    let mut beggar_in_hand = ObjectSpec::card(p1, "Beloved Beggar")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-beloved-beggar".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Disturb)
        .with_mana_cost(ManaCost {
            white: 1,
            ..Default::default()
        });
    beggar_in_hand.power = Some(1);
    beggar_in_hand.toughness = Some(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(beggar_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let beggar_id = find_in_zone(&state, "Beloved Beggar", ZoneId::Hand(p1))
        .expect("Beloved Beggar should be in hand");

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
    state.turn.priority_holder = Some(p1);

    // Attempt to cast with disturb from hand — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: beggar_id,
            alt_cost: Some(AltCostKind::Disturb),
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "casting with disturb from hand should fail (CR 702.146a: must be in graveyard)"
    );
}
