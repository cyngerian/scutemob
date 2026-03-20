//! Mutate keyword ability tests (CR 702.140 / CR 729).
//!
//! Mutate is an alternative cost (CR 118.9a) that allows a creature spell to merge
//! with a non-Human creature you own on the battlefield instead of entering as a
//! separate permanent.
//!
//! Key rules verified:
//! - CR 702.140a: Mutate targets a non-Human creature the caster owns.
//! - CR 702.140a: Cannot mutate onto a Human creature.
//! - CR 702.140b: If the target becomes illegal before resolution, the spell resolves
//!   as a normal creature spell (enters battlefield separately).
//! - CR 702.140c: On legal merge, controller chooses to place on top or underneath.
//! - CR 729.2a: Topmost component's characteristics become the merged permanent's
//!   base characteristics (name, P/T, types, etc.).
//! - CR 702.140e / CR 729.3: Merged permanent has ALL abilities from ALL components.
//! - CR 702.140d: "Whenever this creature mutates" trigger fires after merge.
//! - CR 729.3: When merged permanent leaves battlefield, each component becomes
//!   a separate object in the destination zone.
//! - CR 729.2c: ETB triggers do NOT fire on merge (same object, not new entry).

use mtg_engine::state::game_object::{Characteristics, TriggerEvent, TriggeredAbilityDef};
use mtg_engine::state::types::{AltCostKind, FaceDownKind};
use mtg_engine::state::CardType;
use mtg_engine::AdditionalCost;
use mtg_engine::{
    calculate_characteristics, enrich_spec_from_def, process_command, CardDefinition, CardId,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor,
    ManaCost, MergedComponent, ObjectId, ObjectSpec, PlayerId, Step, SubType, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

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

fn is_on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
}

fn is_in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Graveyard(owner))
}

/// Pass priority for all listed players once.
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

/// A mock mutating creature (Gemrazer-like): 4/4 Beast with Mutate and Reach.
fn mock_mutating_beast_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-mutating-beast".to_string()),
        name: "Mock Mutating Beast".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Beast".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Mutate {1}{G}{G}\nReach".to_string(),
        abilities: vec![
            mtg_engine::AbilityDefinition::Keyword(KeywordAbility::Mutate),
            // CR 702.140a: Mutate cost {1}{G}{G}.
            mtg_engine::AbilityDefinition::MutateCost {
                cost: ManaCost {
                    generic: 1,
                    green: 2,
                    ..Default::default()
                },
            },
            mtg_engine::AbilityDefinition::Keyword(KeywordAbility::Reach),
        ],
        power: Some(4),
        toughness: Some(4),
        ..Default::default()
    }
}

/// A 2/3 Wolf (non-Human, good mutate target).
fn mock_wolf_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-wolf".to_string()),
        name: "Mock Wolf".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Wolf".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(3),
        ..Default::default()
    }
}

/// A 2/2 Human (illegal mutate target).
fn mock_human_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-human".to_string()),
        name: "Mock Human".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Human".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

// ── Test 1: Basic mutate cast validation ──────────────────────────────────────

#[test]
/// CR 702.140a: A mutating creature spell can target a non-Human creature you own.
/// After resolution, the target permanent has the spell's data in merged_components,
/// and the spell's source object no longer exists separately.
fn test_mutate_resolution_basic_merge() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_wolf_def()]);

    // Beast in hand (use ObjectSpec::card then set power/toughness directly).
    let mut mutating_beast = ObjectSpec::card(p1, "Mock Mutating Beast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_keyword(KeywordAbility::Reach)
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        });
    mutating_beast.power = Some(4);
    mutating_beast.toughness = Some(4);

    // Wolf on battlefield.
    let mut wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mutating_beast)
        .object(wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay mana for the mutate cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let beast_id = find_object(&state, "Mock Mutating Beast");
    let wolf_id = find_object(&state, "Mock Wolf");

    // CR 702.140a: Cast the beast for its mutate cost targeting the Wolf.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: beast_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate {
                target: wolf_id,
                on_top: true,
            }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with mutate failed: {:?}", e));

    // Spell should be on the stack as a MutatingCreatureSpell.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "mutating spell should be on stack"
    );
    let stack_obj = &state.stack_objects[0];
    assert!(
        matches!(
            &stack_obj.kind,
            mtg_engine::state::stack::StackObjectKind::MutatingCreatureSpell { target, .. }
            if *target == wolf_id
        ),
        "CR 702.140a: stack object should be MutatingCreatureSpell targeting the wolf"
    );
    assert!(
        stack_obj
            .additional_costs
            .iter()
            .any(|c| matches!(c, AdditionalCost::Mutate { on_top: true, .. })),
        "mutate_on_top should be propagated to stack object via additional_costs"
    );

    // Resolve: all players pass priority.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 729.2: After resolution, the wolf's ObjectId should still be on the battlefield.
    // The merged permanent uses the wolf's ObjectId (CR 729.2c). Its displayed name is the
    // topmost component's name (beast), since mutate_on_top=true.
    assert!(
        state
            .objects
            .get(&wolf_id)
            .map(|obj| obj.zone == ZoneId::Battlefield)
            .unwrap_or(false),
        "CR 729.2: wolf's ObjectId should still be on battlefield after merge (CR 729.2c)"
    );

    // CR 729.2b: The mutating beast's source object should no longer exist separately.
    // After merge, there should be exactly one object with the beast's name
    // (the merged permanent itself, whose base name is now the beast's).
    let beast_objects_count = state
        .objects
        .values()
        .filter(|obj| obj.characteristics.name == "Mock Mutating Beast")
        .count();
    assert_eq!(
        beast_objects_count,
        1,
        "CR 729.2b: only one object named 'Mock Mutating Beast' should exist (the merged permanent)"
    );

    // CR 729.2a / CR 729.2: Wolf permanent should have merged_components with 2 entries.
    let wolf_obj = state
        .objects
        .get(&wolf_id)
        .expect("wolf should still exist with same ObjectId (CR 729.2c)");
    assert_eq!(
        wolf_obj.merged_components.len(),
        2,
        "CR 729.2: merged permanent should have 2 components (beast on top, wolf on bottom)"
    );

    // CR 729.2a: Topmost component (index 0) is the beast (mutate_on_top=true).
    assert_eq!(
        wolf_obj.merged_components[0].characteristics.name, "Mock Mutating Beast",
        "CR 729.2a: topmost component should be the mutating beast (mutate_on_top=true)"
    );
    assert_eq!(
        wolf_obj.merged_components[1].characteristics.name, "Mock Wolf",
        "CR 729.2: bottom component should be the wolf"
    );

    // CR 729.2a: Via the layer system, the merged permanent should use beast's P/T.
    let merged_chars = calculate_characteristics(&state, wolf_id)
        .expect("merged permanent should have characteristics");
    assert_eq!(
        merged_chars.power,
        Some(4),
        "CR 729.2a: merged permanent should have beast's power (4)"
    );
    assert_eq!(
        merged_chars.toughness,
        Some(4),
        "CR 729.2a: merged permanent should have beast's toughness (4)"
    );

    // CR 702.140e: Merged permanent should have beast's Reach keyword.
    assert!(
        merged_chars.keywords.contains(&KeywordAbility::Reach),
        "CR 702.140e: merged permanent should have beast's Reach keyword"
    );

    // CR 729.2c: Stack should be empty — no ETB triggers fired.
    assert!(
        state.stack_objects.is_empty(),
        "CR 729.2c: no ETB triggers should fire (merged, not new entry)"
    );
}

// ── Test 2: Cannot mutate onto a Human creature ──────────────────────────────

#[test]
/// CR 702.140a: Mutate cannot target a Human creature. Casting must be rejected.
fn test_mutate_validation_rejects_human_target() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_human_def()]);

    let mut mutating_beast = ObjectSpec::card(p1, "Mock Mutating Beast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        });
    mutating_beast.power = Some(4);
    mutating_beast.toughness = Some(4);

    let mut human = ObjectSpec::card(p1, "Mock Human")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-human".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Human".to_string())]);
    human.power = Some(2);
    human.toughness = Some(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mutating_beast)
        .object(human)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let beast_id = find_object(&state, "Mock Mutating Beast");
    let human_id = find_object(&state, "Mock Human");

    // CR 702.140a: Should be rejected — Human is not a valid mutate target.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: beast_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate {
                target: human_id,
                on_top: true,
            }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.140a: mutate onto a Human should be rejected"
    );
}

// ── Test 3: Illegal target fallback — spell resolves as normal creature ───────

#[test]
/// CR 702.140b: If the mutate target becomes illegal before resolution (the creature
/// left the battlefield), the spell resolves as a normal creature spell (enters
/// the battlefield separately, not as a merge).
fn test_mutate_resolution_illegal_target_fallback() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_wolf_def()]);

    let mut mutating_beast = ObjectSpec::card(p1, "Mock Mutating Beast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_keyword(KeywordAbility::Reach)
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        });
    mutating_beast.power = Some(4);
    mutating_beast.toughness = Some(4);

    let mut wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mutating_beast)
        .object(wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let beast_id = find_object(&state, "Mock Mutating Beast");
    let wolf_id = find_object(&state, "Mock Wolf");

    // Cast with mutate targeting the wolf.
    let (mut state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: beast_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate {
                target: wolf_id,
                on_top: true,
            }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with mutate failed: {:?}", e));

    // Now simulate the wolf leaving the battlefield before resolution.
    let _ = state
        .move_object_to_zone(wolf_id, ZoneId::Graveyard(p1))
        .unwrap();

    // Resolve: all players pass priority.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.140b: The mutating beast should have entered the battlefield normally.
    assert!(
        is_on_battlefield(&state, "Mock Mutating Beast"),
        "CR 702.140b: beast should enter battlefield normally when target is illegal at resolution"
    );

    // The wolf should be in the graveyard (we put it there manually).
    assert!(
        is_in_graveyard(&state, "Mock Wolf", p1),
        "wolf should be in graveyard (we moved it there)"
    );

    // CR 729.2c: The beast entered normally — it should have NO merged_components.
    let beast_on_field = state
        .objects
        .values()
        .find(|obj| {
            obj.characteristics.name == "Mock Mutating Beast" && obj.zone == ZoneId::Battlefield
        })
        .expect("beast should be on battlefield");
    assert!(
        beast_on_field.merged_components.is_empty(),
        "CR 702.140b: when entering normally (fallback), should have no merged_components"
    );
}

// ── Test 4: Zone-change splits merged permanent into components ────────────────

#[test]
/// CR 729.3: When a merged permanent leaves the battlefield, all components
/// become separate objects in the destination zone. Each component card appears
/// individually in the graveyard (or other zone).
fn test_mutate_zone_change_splits_components() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_wolf_def()]);

    // Start with a wolf on the battlefield.
    let mut wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let wolf_id = find_object(&state, "Mock Wolf");

    // Manually inject merged_components to simulate a merged permanent.
    // Beast is on top (index 0), Wolf is index 1.
    let beast_characteristics = Characteristics {
        name: "Mock Mutating Beast".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Beast".to_string())].into_iter().collect(),
        keywords: [KeywordAbility::Reach].into_iter().collect(),
        power: Some(4),
        toughness: Some(4),
        ..Default::default()
    };
    let wolf_characteristics = Characteristics {
        name: "Mock Wolf".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Wolf".to_string())].into_iter().collect(),
        power: Some(2),
        toughness: Some(3),
        ..Default::default()
    };

    {
        let wolf_obj = state.objects.get_mut(&wolf_id).unwrap();
        wolf_obj.merged_components = im::vector![
            MergedComponent {
                card_id: Some(CardId("mock-mutating-beast".to_string())),
                characteristics: beast_characteristics,
                is_token: false,
            },
            MergedComponent {
                card_id: Some(CardId("mock-wolf".to_string())),
                characteristics: wolf_characteristics,
                is_token: false,
            }
        ];
    }

    // Move the merged permanent to the graveyard (simulates dying).
    let _ = state
        .move_object_to_zone(wolf_id, ZoneId::Graveyard(p1))
        .expect("move to graveyard should succeed");

    // CR 729.3: Each component should be a separate object in the graveyard.
    let graveyard_cards: Vec<_> = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Graveyard(p1))
        .collect();

    assert_eq!(
        graveyard_cards.len(),
        2,
        "CR 729.3: both components should be in graveyard as separate objects (got {})",
        graveyard_cards.len()
    );

    // CR 729.3: Each component object should have empty merged_components (fresh objects).
    for obj in &graveyard_cards {
        assert!(
            obj.merged_components.is_empty(),
            "CR 729.3 / CR 400.7: each split component starts with empty merged_components"
        );
    }

    // Verify both component names appear in the graveyard.
    assert!(
        is_in_graveyard(&state, "Mock Mutating Beast", p1),
        "CR 729.3: beast component should be in graveyard"
    );
    assert!(
        is_in_graveyard(&state, "Mock Wolf", p1),
        "CR 729.3: wolf component should be in graveyard"
    );
}

// ── Test 5: "Whenever this creature mutates" trigger fires ────────────────────

#[test]
/// CR 702.140d: After a successful merge, "whenever this creature mutates" triggers
/// fire on the merged permanent. The trigger goes onto the stack.
fn test_mutate_trigger_fires() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_wolf_def()]);

    // Build a mutating beast in hand with a SelfMutates triggered ability.
    let mut mutating_beast = ObjectSpec::card(p1, "Mock Mutating Beast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        })
        // CR 702.140d: "whenever this creature mutates" trigger
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfMutates,
            intervening_if: None,
            description: "Whenever this creature mutates (CR 702.140d)".to_string(),
            effect: None, // No effect needed for trigger-fires test
            etb_filter: None,
            targets: vec![],
        });
    mutating_beast.power = Some(4);
    mutating_beast.toughness = Some(4);

    let mut wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mutating_beast)
        .object(wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let beast_id = find_object(&state, "Mock Mutating Beast");
    let wolf_id = find_object(&state, "Mock Wolf");

    // Cast with mutate targeting the wolf, placing beast on top.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: beast_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate {
                target: wolf_id,
                on_top: true,
            }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with mutate failed: {:?}", e));

    // Resolve the mutating spell (all players pass priority once).
    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 702.140d: CreatureMutated event should have been emitted.
    let mutated_event = events.iter().any(
        |e| matches!(e, GameEvent::CreatureMutated { object_id, .. } if *object_id == wolf_id),
    );
    assert!(
        mutated_event,
        "CR 702.140d: CreatureMutated event should be emitted after successful merge"
    );

    // CR 702.140d: After merge, the SelfMutates trigger should be on the stack.
    // The trigger fires on the merged permanent's "whenever this creature mutates" ability.
    // The trigger beast was placed on top (mutate_on_top=true), so the merged permanent
    // has the trigger beast's triggered abilities (from merged_components[0]).
    assert!(
        !state.stack_objects.is_empty(),
        "CR 702.140d: 'whenever this creature mutates' trigger should be on the stack"
    );

    // The stack should contain a triggered ability from the merged permanent (wolf_id).
    let trigger_on_stack = state.stack_objects.iter().any(|so| {
        matches!(
            &so.kind,
            mtg_engine::state::stack::StackObjectKind::TriggeredAbility { source_object, .. }
            if *source_object == wolf_id
        )
    });
    assert!(
        trigger_on_stack,
        "CR 702.140d: mutate trigger should be from the merged permanent (wolf_id preserved, CR 729.2c)"
    );
}

// ── Test 6: Mutate under (mutate_on_top=false) ────────────────────────────────

#[test]
/// CR 729.2c: When mutate_on_top=false, the mutating spell goes underneath.
/// The existing permanent's characteristics remain on top.
fn test_mutate_under_uses_target_characteristics() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_wolf_def()]);

    let mut mutating_beast = ObjectSpec::card(p1, "Mock Mutating Beast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_keyword(KeywordAbility::Reach)
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        });
    mutating_beast.power = Some(4);
    mutating_beast.toughness = Some(4);

    let mut wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mutating_beast)
        .object(wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let beast_id = find_object(&state, "Mock Mutating Beast");
    let wolf_id = find_object(&state, "Mock Wolf");

    // CR 729.2: mutate_on_top=false — spell goes underneath.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: beast_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate {
                target: wolf_id,
                on_top: false,
            }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with mutate (under) failed: {:?}", e));

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    let wolf_obj = state
        .objects
        .get(&wolf_id)
        .expect("wolf should still exist");
    assert_eq!(
        wolf_obj.merged_components.len(),
        2,
        "should have 2 merged components"
    );

    // CR 729.2: When mutate_on_top=false, the existing permanent (wolf) is on top.
    assert_eq!(
        wolf_obj.merged_components[0].characteristics.name, "Mock Wolf",
        "CR 729.2: wolf should be topmost component (mutate_on_top=false)"
    );
    assert_eq!(
        wolf_obj.merged_components[1].characteristics.name, "Mock Mutating Beast",
        "CR 729.2: beast should be bottom component (mutate_on_top=false)"
    );

    // CR 729.2a: Merged permanent uses wolf's P/T (it's on top).
    let merged_chars = calculate_characteristics(&state, wolf_id)
        .expect("merged permanent should have characteristics");
    assert_eq!(
        merged_chars.power,
        Some(2),
        "CR 729.2a: merged permanent should have wolf's power (2) when wolf is on top"
    );
    assert_eq!(
        merged_chars.toughness,
        Some(3),
        "CR 729.2a: merged permanent should have wolf's toughness (3) when wolf is on top"
    );

    // CR 702.140e: Merged permanent should still have beast's Reach keyword (bottom component).
    assert!(
        merged_chars.keywords.contains(&KeywordAbility::Reach),
        "CR 702.140e: merged permanent should have beast's Reach from bottom component"
    );
}

// ── Session 3 Integration Tests: Card Definitions ─────────────────────────────

/// Build a HashMap of card defs needed by `enrich_spec_from_def`.
fn make_defs(defs: Vec<CardDefinition>) -> std::collections::HashMap<String, CardDefinition> {
    defs.into_iter().map(|d| (d.name.clone(), d)).collect()
}

// ── Test 7: Gemrazer mutates onto a creature, "whenever mutates" trigger queued ─

#[test]
/// CR 702.140d / CR 702.140: Gemrazer (real card def) mutates onto a Wolf, placing
/// beast on top. After merge, the "whenever this creature mutates" trigger goes on
/// the stack. The merged permanent has Reach, Trample, and 4/4 P/T from the beast.
fn test_mutate_gemrazer_trigger_queued_after_merge() {
    let p1 = p(1);
    let p2 = p(2);

    let gemrazer_def = mtg_engine::cards::defs::gemrazer::card();
    let wolf_def = mock_wolf_def();

    let defs = make_defs(vec![gemrazer_def.clone(), wolf_def.clone()]);
    let registry = CardRegistry::new(vec![gemrazer_def, wolf_def]);

    // Gemrazer in hand — enriched from the real card def.
    // card_id must be set to "gemrazer" so get_mutate_cost() can look it up in the registry.
    let gemrazer_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Gemrazer")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(CardId("gemrazer".to_string())),
        &defs,
    );

    // Wolf on battlefield.
    let mut wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(gemrazer_spec)
        .object(wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay mutate cost {1}{G}{G} for Gemrazer.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let gemrazer_id = find_object(&state, "Gemrazer");
    let wolf_id = find_object(&state, "Mock Wolf");

    // CR 702.140a: Cast Gemrazer for its mutate cost targeting the wolf.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: gemrazer_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate {
                target: wolf_id,
                on_top: true,
            }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with mutate (Gemrazer) failed: {:?}", e));

    // Resolve: all players pass priority.
    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 702.140d: CreatureMutated event should have been emitted.
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::CreatureMutated { object_id, .. } if *object_id == wolf_id)
        ),
        "CR 702.140d: CreatureMutated event should fire after successful Gemrazer mutate"
    );

    // CR 702.140d: The "whenever this creature mutates" trigger should be on the stack.
    // The trigger goes on the stack targeting an artifact or enchantment (needs manual target choice).
    assert!(
        !state.stack_objects.is_empty(),
        "CR 702.140d: Gemrazer's 'whenever this creature mutates' trigger should be on the stack"
    );

    // CR 729.2a: Merged permanent should have Gemrazer's P/T (beast on top).
    let merged_chars = calculate_characteristics(&state, wolf_id)
        .expect("merged permanent should have characteristics");
    assert_eq!(
        merged_chars.power,
        Some(4),
        "CR 729.2a: merged permanent should have Gemrazer's power (4)"
    );
    assert_eq!(
        merged_chars.toughness,
        Some(4),
        "CR 729.2a: merged permanent should have Gemrazer's toughness (4)"
    );

    // CR 702.140e: Merged permanent should have Gemrazer's Reach and Trample.
    assert!(
        merged_chars.keywords.contains(&KeywordAbility::Reach),
        "CR 702.140e: merged permanent should have Gemrazer's Reach keyword"
    );
    assert!(
        merged_chars.keywords.contains(&KeywordAbility::Trample),
        "CR 702.140e: merged permanent should have Gemrazer's Trample keyword"
    );
}

// ── Test 8: Three-deep mutate stacking ────────────────────────────────────────

#[test]
/// CR 729.2: A merged permanent can be mutated onto again. Three-deep stacking
/// produces 3 merged components. Topmost characteristics are from the last mutation.
fn test_mutate_stacking_three_deep() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_wolf_def()]);

    // Wolf on battlefield.
    let mut wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let wolf_id = find_object(&state, "Mock Wolf");

    // Inject a two-deep merged permanent (beast on top of wolf).
    let beast_chars = Characteristics {
        name: "Mock Mutating Beast".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Beast".to_string())].into_iter().collect(),
        keywords: [KeywordAbility::Reach].into_iter().collect(),
        power: Some(4),
        toughness: Some(4),
        ..Default::default()
    };
    let wolf_chars = Characteristics {
        name: "Mock Wolf".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Wolf".to_string())].into_iter().collect(),
        power: Some(2),
        toughness: Some(3),
        ..Default::default()
    };
    {
        let wolf_obj = state.objects.get_mut(&wolf_id).unwrap();
        wolf_obj.merged_components = im::vector![
            MergedComponent {
                card_id: Some(CardId("mock-mutating-beast".to_string())),
                characteristics: beast_chars.clone(),
                is_token: false,
            },
            MergedComponent {
                card_id: Some(CardId("mock-wolf".to_string())),
                characteristics: wolf_chars,
                is_token: false,
            }
        ];
        // Also sync obj.characteristics to the topmost component (beast on top).
        wolf_obj.characteristics.name = beast_chars.name.clone();
        wolf_obj.characteristics.power = beast_chars.power;
        wolf_obj.characteristics.toughness = beast_chars.toughness;
    }

    // Simpler approach: manually inject merged_components for a 3-deep test.
    // Rather than going through the full CastSpell flow again, verify 3-component logic directly.
    {
        let wolf_obj = state.objects.get_mut(&wolf_id).unwrap();
        // Add a third component on top.
        let top_chars = Characteristics {
            name: "Mock Top Beast".to_string(),
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Beast".to_string())].into_iter().collect(),
            keywords: [KeywordAbility::Flying].into_iter().collect(),
            power: Some(5),
            toughness: Some(5),
            ..Default::default()
        };
        wolf_obj.merged_components.push_front(MergedComponent {
            card_id: Some(CardId("mock-top-beast".to_string())),
            characteristics: top_chars.clone(),
            is_token: false,
        });
        // Sync obj.characteristics to new topmost component.
        wolf_obj.characteristics.name = top_chars.name.clone();
        wolf_obj.characteristics.power = top_chars.power;
        wolf_obj.characteristics.toughness = top_chars.toughness;
    }

    // Verify 3 components.
    let wolf_obj = state.objects.get(&wolf_id).unwrap();
    assert_eq!(
        wolf_obj.merged_components.len(),
        3,
        "CR 729.2: merged permanent should have 3 components after three-deep stacking"
    );

    // CR 729.2a: Topmost component (index 0) should be the "top beast" (Flying, 5/5).
    let merged_chars = calculate_characteristics(&state, wolf_id)
        .expect("merged permanent should have characteristics");
    assert_eq!(
        merged_chars.power,
        Some(5),
        "CR 729.2a: topmost component's P/T (5/5) should be used"
    );
    assert!(
        merged_chars.keywords.contains(&KeywordAbility::Flying),
        "CR 729.2a: topmost component's Flying keyword should be present"
    );

    // CR 702.140e: All non-topmost components' abilities should be on the merged permanent.
    assert!(
        merged_chars.keywords.contains(&KeywordAbility::Reach),
        "CR 702.140e: Reach from middle component (beast at index 1) should be present"
    );

    // CR 729.3: When merged permanent leaves battlefield, all 3 components become separate.
    let _ = state
        .move_object_to_zone(wolf_id, ZoneId::Graveyard(p1))
        .expect("move to graveyard should succeed");

    let graveyard_count = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Graveyard(p1))
        .count();
    assert_eq!(
        graveyard_count, 3,
        "CR 729.3: all 3 components should become separate objects in graveyard"
    );
}

// ── Test 9: Bounce returns all components to hand ─────────────────────────────

#[test]
/// CR 729.3: When a merged permanent is bounced (returned to hand), all components
/// become separate cards in the owner's hand. Each component is a separate object.
fn test_mutate_bounce_returns_all_cards() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_wolf_def()]);

    // Wolf on battlefield, pre-merged with beast.
    let mut wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let wolf_id = find_object(&state, "Mock Wolf");

    // Inject merged_components: beast on top of wolf (2 components).
    {
        let wolf_obj = state.objects.get_mut(&wolf_id).unwrap();
        wolf_obj.merged_components = im::vector![
            MergedComponent {
                card_id: Some(CardId("mock-mutating-beast".to_string())),
                characteristics: Characteristics {
                    name: "Mock Mutating Beast".to_string(),
                    card_types: [CardType::Creature].into_iter().collect(),
                    subtypes: [SubType("Beast".to_string())].into_iter().collect(),
                    power: Some(4),
                    toughness: Some(4),
                    ..Default::default()
                },
                is_token: false,
            },
            MergedComponent {
                card_id: Some(CardId("mock-wolf".to_string())),
                characteristics: Characteristics {
                    name: "Mock Wolf".to_string(),
                    card_types: [CardType::Creature].into_iter().collect(),
                    subtypes: [SubType("Wolf".to_string())].into_iter().collect(),
                    power: Some(2),
                    toughness: Some(3),
                    ..Default::default()
                },
                is_token: false,
            }
        ];
    }

    // CR 729.3: Bounce the merged permanent to hand.
    let _ = state
        .move_object_to_zone(wolf_id, ZoneId::Hand(p1))
        .expect("bounce to hand should succeed");

    // CR 729.3: Both components should be in hand as separate objects.
    let hand_count = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_count, 2,
        "CR 729.3: both components should be in hand as separate objects after bounce"
    );

    // Beast and wolf should both be in hand.
    assert!(
        state
            .objects
            .values()
            .any(|obj| obj.characteristics.name == "Mock Mutating Beast"
                && obj.zone == ZoneId::Hand(p1)),
        "CR 729.3: beast component should be in hand after bounce"
    );
    assert!(
        state
            .objects
            .values()
            .any(|obj| obj.characteristics.name == "Mock Wolf" && obj.zone == ZoneId::Hand(p1)),
        "CR 729.3: wolf component should be in hand after bounce"
    );

    // CR 400.7 / CR 729.3: Each component in hand should have empty merged_components.
    for obj in state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
    {
        assert!(
            obj.merged_components.is_empty(),
            "CR 400.7 / CR 729.3: each component in hand starts with empty merged_components"
        );
    }
}

// ── Test 10: Mutate onto a face-down creature ─────────────────────────────────

#[test]
/// CR 702.140a / CR 708.2 / CR 729.6: A face-down creature (Morph) IS a legal
/// Mutate target.
///
/// CR 708.2: A face-down permanent has no name, mana cost, color, or type —
/// its subtypes are not visible. Since a face-down creature has no visible Human
/// subtype, the CR 702.140a "non-Human" check passes. The engine checks the
/// CURRENT characteristics (face-down = no subtypes = not Human).
///
/// This behavior is correct per CR: the controller takes the risk that the
/// face-down creature might be an illegal target once turned face-up
/// (but at cast time, the engine evaluates legality from visible characteristics).
///
/// MR-Mutate-02: Verifies that mutating onto a face-down creature is ACCEPTED by
/// the engine (face-down creatures have no visible Human subtype). The spell goes
/// onto the stack successfully.
///
/// Source: CR 702.140a, CR 708.2, CR 729.6
fn test_mutate_onto_face_down_creature_accepted() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_wolf_def()]);

    // Beast in hand (potential mutating spell).
    let mut mutating_beast = ObjectSpec::card(p1, "Mock Mutating Beast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        });
    mutating_beast.power = Some(4);
    mutating_beast.toughness = Some(4);

    // Wolf on battlefield, will be set face-down as a Morph after building state.
    let mut face_down_wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    face_down_wolf.power = Some(2);
    face_down_wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mutating_beast)
        .object(face_down_wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Mark the wolf as face-down (Morph) by setting face_down_as on the GameObject.
    // This makes the wolf's printed subtypes hidden (CR 708.2).
    let wolf_game_id = find_object(&state, "Mock Wolf");
    if let Some(wolf_obj) = state.objects.get_mut(&wolf_game_id) {
        wolf_obj.face_down_as = Some(FaceDownKind::Morph);
    }

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 4);
    state.turn.priority_holder = Some(p1);

    let hand_beast_id = state
        .objects
        .iter()
        .find(|(_, o)| {
            o.zone == ZoneId::Hand(p1) && o.characteristics.name == "Mock Mutating Beast"
        })
        .map(|(id, _)| *id)
        .expect("beast should be in hand");

    // CR 702.140a / CR 708.2: Mutating onto a face-down creature should be ACCEPTED.
    // The face-down permanent has no visible subtypes (CR 708.2), so the non-Human
    // check passes (no Human subtype visible = not Human). The spell goes on the stack.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: hand_beast_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate {
                target: wolf_game_id, // the face-down wolf
                on_top: true,
            }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    // CR 708.2: Face-down creature has no visible Human subtype, so mutate IS legal.
    // The engine checks current visible characteristics, not printed characteristics.
    assert!(
        result.is_ok(),
        "CR 702.140a / CR 708.2: mutating onto a face-down creature should succeed — \
         face-down creatures have no visible Human subtype, passing the non-Human check \
         (engine evaluates current visible characteristics per CR 708.2)"
    );
    let (state, _) = result.unwrap();
    // The spell should be on the stack as a MutatingCreatureSpell targeting the face-down wolf.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 729.6: mutating creature spell targeting face-down permanent should be on stack"
    );
}

// ── Test 11: Copy of a mutating creature spell (documentation) ────────────────

#[test]
/// CR 729.8: If a copy of a mutating creature spell is put onto the stack,
/// the copy is also a mutating creature spell. However, a copy cannot be cast
/// from hand — the copy would resolve as a new creature entering the battlefield
/// (it has no associated card object to merge). The copy targets the same base
/// creature if the original does; if the original's target has become illegal,
/// the copy also fizzles.
///
/// MR-Mutate-01: This test documents the expected behavior per CR 729.8 and
/// verifies that a MutatingCreatureSpell on the stack has an associated target
/// in its additional_costs. Setting up a true copy-of-spell is complex because
/// it requires the copy effect from the rules, so this test validates the
/// data model invariant: a mutating spell on the stack must have a Mutate
/// AdditionalCost entry.
///
/// Source: CR 729.8, CR 706.10
fn test_mutate_stack_object_has_mutate_additional_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_mutating_beast_def(), mock_wolf_def()]);

    let mut mutating_beast = ObjectSpec::card(p1, "Mock Mutating Beast")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        });
    mutating_beast.power = Some(4);
    mutating_beast.toughness = Some(4);

    let mut wolf = ObjectSpec::card(p1, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mutating_beast)
        .object(wolf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 4);
    state.turn.priority_holder = Some(p1);

    let beast_id = find_object(&state, "Mock Mutating Beast");
    let wolf_id = find_object(&state, "Mock Wolf");

    // Cast the beast for its mutate cost targeting the Wolf.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: beast_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate {
                target: wolf_id,
                on_top: true,
            }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with mutate failed: {:?}", e));

    // CR 729.8: The stack object for a mutating spell must record its Mutate target.
    // This invariant must hold for any copy of the spell as well — the copy would
    // inherit the same additional_costs (including the Mutate target).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "mutating spell should be on stack"
    );
    let stack_obj = &state.stack_objects[0];
    let has_mutate_cost = stack_obj
        .additional_costs
        .iter()
        .any(|c| matches!(c, AdditionalCost::Mutate { target, .. } if *target == wolf_id));
    assert!(
        has_mutate_cost,
        "CR 729.8: MutatingCreatureSpell on stack must have AdditionalCost::Mutate \
         recording the target — a copy of this spell would inherit the same cost data"
    );
}
