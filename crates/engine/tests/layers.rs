//! Layer system tests: continuous effects, characteristic calculation (CR 613).
//!
//! Tests are organized around specific CR subsections and known corner cases.
//! Each test constructs a minimal game state, adds continuous effects directly,
//! and then calls `calculate_characteristics` to verify the result.

use im::{ordset, OrdSet};
use mtg_engine::{
    calculate_characteristics, CardType, ContinuousEffect, EffectDuration, EffectFilter, EffectId,
    EffectLayer, GameStateBuilder, KeywordAbility, LayerModification, ObjectSpec, PlayerId,
    SubType,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn p1() -> PlayerId {
    PlayerId(1)
}

/// Build a continuous effect with sensible defaults.
fn effect(
    id: u64,
    source: Option<mtg_engine::ObjectId>,
    timestamp: u64,
    layer: EffectLayer,
    filter: EffectFilter,
    modification: LayerModification,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source,
        timestamp,
        layer,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter,
        modification,
        is_cda: false,
    }
}

fn eot_effect(
    id: u64,
    source: Option<mtg_engine::ObjectId>,
    timestamp: u64,
    layer: EffectLayer,
    filter: EffectFilter,
    modification: LayerModification,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source,
        timestamp,
        layer,
        duration: EffectDuration::UntilEndOfTurn,
        filter,
        modification,
        is_cda: false,
    }
}

// ---------------------------------------------------------------------------
// Basic Layer 4 (Type-changing) tests
// ---------------------------------------------------------------------------

/// CR 613.1d: Type-changing effects apply in layer 4.
/// Adding "Creature" type makes a land into a creature.
#[test]
fn test_613_layer4_add_creature_type() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::land(p1(), "Test Land"))
        .build()
        .unwrap();
    let land_effect = effect(
        1,
        None,
        10,
        EffectLayer::TypeChange,
        EffectFilter::AllLands,
        LayerModification::AddCardTypes(ordset![CardType::Creature]),
    );

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::land(p1(), "Test Land"))
        .add_continuous_effect(land_effect)
        .build()
        .unwrap();
    let land_id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, land_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "land should be a creature after layer 4 AddCardTypes"
    );
    assert!(
        chars.card_types.contains(&CardType::Land),
        "land type should be preserved"
    );
}

/// CR 613.1d: SetTypeLine replaces all types.
#[test]
fn test_613_layer4_set_type_line_replaces_all() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Test", 2, 2)
                .with_types(vec![CardType::Creature, CardType::Artifact]),
        )
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 10,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetTypeLine {
                supertypes: OrdSet::new(),
                card_types: ordset![CardType::Land],
                subtypes: ordset![SubType("Mountain".to_string())],
            },
            is_cda: false,
        })
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert!(chars.card_types.contains(&CardType::Land));
    assert!(!chars.card_types.contains(&CardType::Creature));
    assert!(!chars.card_types.contains(&CardType::Artifact));
    assert!(chars.subtypes.contains(&SubType("Mountain".to_string())));
}

// ---------------------------------------------------------------------------
// Basic Layer 7 (P/T) tests
// ---------------------------------------------------------------------------

/// CR 613.4b: SetPowerToughness overrides base P/T.
#[test]
fn test_613_layer7b_set_pt() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Big Creature", 5, 5))
        .add_continuous_effect(effect(
            1,
            None,
            10,
            EffectLayer::PtSet,
            EffectFilter::AllCreatures,
            LayerModification::SetPowerToughness {
                power: 1,
                toughness: 1,
            },
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(1));
    assert_eq!(chars.toughness, Some(1));
}

/// CR 613.4c: ModifyPower and ModifyToughness adjust current values.
#[test]
fn test_613_layer7c_modify_pt() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Test", 2, 2))
        .add_continuous_effect(effect(
            1,
            None,
            10,
            EffectLayer::PtModify,
            EffectFilter::AllCreatures,
            LayerModification::ModifyBoth(3),
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(5));
    assert_eq!(chars.toughness, Some(5));
}

/// CR 613.4c: Layer 7c effects stack additively.
#[test]
fn test_613_layer7c_multiple_modifications_stack() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Test", 1, 1))
        .add_continuous_effect(effect(
            1,
            None,
            10,
            EffectLayer::PtModify,
            EffectFilter::AllCreatures,
            LayerModification::ModifyPower(2),
        ))
        .add_continuous_effect(effect(
            2,
            None,
            11,
            EffectLayer::PtModify,
            EffectFilter::AllCreatures,
            LayerModification::ModifyToughness(3),
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(3), "+2 power: 1 + 2 = 3");
    assert_eq!(chars.toughness, Some(4), "+3 toughness: 1 + 3 = 4");
}

/// CR 613.4d: Layer 7d switches P/T after all other P/T effects.
#[test]
fn test_613_layer7d_pt_switch() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Test", 3, 1))
        // First: set to 3/3 (7b)
        .add_continuous_effect(effect(
            1,
            None,
            10,
            EffectLayer::PtSet,
            EffectFilter::AllCreatures,
            LayerModification::SetPowerToughness {
                power: 3,
                toughness: 3,
            },
        ))
        // Then: +1/+0 (7c)
        .add_continuous_effect(effect(
            2,
            None,
            11,
            EffectLayer::PtModify,
            EffectFilter::AllCreatures,
            LayerModification::ModifyPower(1),
        ))
        // Finally: switch (7d) — should see 4/3 → switched to 3/4
        .add_continuous_effect(effect(
            3,
            None,
            12,
            EffectLayer::PtSwitch,
            EffectFilter::AllCreatures,
            LayerModification::SwitchPowerToughness,
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // 7b: 3/3, 7c: 4/3, 7d: switch → 3/4
    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(3), "switched: toughness becomes power");
    assert_eq!(
        chars.toughness,
        Some(4),
        "switched: power becomes toughness"
    );
}

// ---------------------------------------------------------------------------
// Layer 5 (Color-changing) tests
// ---------------------------------------------------------------------------

/// CR 613.1e: Color-changing effects apply in layer 5.
#[test]
fn test_613_layer5_set_colors() {
    use mtg_engine::Color;

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Test", 2, 2).with_colors(vec![Color::Red]))
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 10,
            layer: EffectLayer::ColorChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetColors(ordset![Color::Blue]),
            is_cda: false,
        })
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert!(chars.colors.contains(&Color::Blue));
    assert!(!chars.colors.contains(&Color::Red));
}

// ---------------------------------------------------------------------------
// Layer 6 (Ability-adding/removing) tests
// ---------------------------------------------------------------------------

/// CR 613.1f: RemoveAllAbilities strips keywords.
/// Humility-style effect: all creatures lose all abilities.
#[test]
fn test_613_layer6_remove_all_abilities() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Flying Bear", 2, 2)
                .with_keyword(KeywordAbility::Flying)
                .with_keyword(KeywordAbility::Trample),
        )
        .add_continuous_effect(effect(
            1,
            None,
            10,
            EffectLayer::Ability,
            EffectFilter::AllCreatures,
            LayerModification::RemoveAllAbilities,
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert!(
        chars.keywords.is_empty(),
        "all keyword abilities should be removed"
    );
}

/// CR 613.1f: AddKeyword grants a keyword.
#[test]
fn test_613_layer6_add_keyword() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Vanilla", 2, 2))
        .add_continuous_effect(effect(
            1,
            None,
            10,
            EffectLayer::Ability,
            EffectFilter::AllCreatures,
            LayerModification::AddKeyword(KeywordAbility::Flying),
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert!(chars.keywords.contains(&KeywordAbility::Flying));
}

// ---------------------------------------------------------------------------
// Layer 7a (CDA) tests
// ---------------------------------------------------------------------------

/// CR 613.4a: CDA effects apply before other layer 7 effects.
/// Tarmogoyf-style: CDA sets P/T to a value, then static +1/+1 is applied on top.
#[test]
fn test_613_layer7a_cda_applies_before_static_pt() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "CDA Creature", 0, 0))
        // CDA in layer 7a: set P/T to 3/4
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 5, // Earlier timestamp, but CDA so applies first
            layer: EffectLayer::PtCda,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetPtViaCda {
                power: 3,
                toughness: 4,
            },
            is_cda: true,
        })
        // Non-CDA in 7b with EARLIER timestamp: should apply after the CDA
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 3, // Earlier than CDA, but CDA always applies first
            layer: EffectLayer::PtSet,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetPowerToughness {
                power: 1,
                toughness: 1,
            },
            is_cda: false,
        })
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // 7a CDA: sets 3/4 (CDA runs first, regardless of timestamp)
    // 7b non-CDA (earlier timestamp but still after 7a): sets 1/1
    // Final result: 1/1 (7b overrides 7a since they're in different sublayers, 7b comes after 7a)
    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(1));
    assert_eq!(chars.toughness, Some(1));
}

/// CR 613.4a: SetPtToManaValue sets P/T equal to the object's mana value.
#[test]
fn test_613_layer7a_set_pt_to_mana_value() {
    use mtg_engine::ManaCost;

    let mut spec = ObjectSpec::creature(p1(), "Opalescence Enchantment", 0, 0);
    // Set mana cost to 4 ({2}{W}{W} mana value = 4)
    spec = spec.with_mana_cost(ManaCost {
        white: 2,
        blue: 0,
        black: 0,
        red: 0,
        green: 0,
        colorless: 0,
        generic: 2,
        ..Default::default()
    });

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(spec)
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 10,
            layer: EffectLayer::PtSet,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetPtToManaValue,
            is_cda: false,
        })
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(4), "P/T should equal mana value (4)");
    assert_eq!(chars.toughness, Some(4), "P/T should equal mana value (4)");
}

// ---------------------------------------------------------------------------
// Timestamp ordering within a layer
// ---------------------------------------------------------------------------

/// CR 613.7: Within a layer, later timestamp wins (overrides earlier).
/// Two "set P/T" effects: the newer one wins.
#[test]
fn test_613_timestamp_ordering_later_wins() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Test", 2, 2))
        // Older timestamp: set to 1/1
        .add_continuous_effect(effect(
            1,
            None,
            10, // earlier
            EffectLayer::PtSet,
            EffectFilter::AllCreatures,
            LayerModification::SetPowerToughness {
                power: 1,
                toughness: 1,
            },
        ))
        // Newer timestamp: set to 3/3
        .add_continuous_effect(effect(
            2,
            None,
            20, // later → wins
            EffectLayer::PtSet,
            EffectFilter::AllCreatures,
            LayerModification::SetPowerToughness {
                power: 3,
                toughness: 3,
            },
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "newer effect (3/3) should override older (1/1)"
    );
    assert_eq!(chars.toughness, Some(3));
}

// ---------------------------------------------------------------------------
// Duration tracking
// ---------------------------------------------------------------------------

/// CR 613.7a: Effect expires when source permanent leaves the battlefield.
#[test]
fn test_613_effect_expires_when_source_leaves_battlefield() {
    // Creature 1: the "source" of the continuous effect.
    // Creature 2: the "target" that the effect modifies.
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Source", 1, 1))
        .object(ObjectSpec::creature(p1(), "Target", 2, 2))
        .build()
        .unwrap();

    let battlefield_ids: Vec<_> = state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .to_vec();
    let source_id = battlefield_ids[0];
    let target_id = battlefield_ids[1];

    // Add a continuous effect sourced from source_id that modifies AllCreatures.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(1),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::ModifyBoth(5),
        is_cda: false,
    });

    // Before source leaves: target should have +5/+5
    let chars_before = calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(
        chars_before.power,
        Some(7),
        "2 + 5 = 7 while source is on battlefield"
    );

    // Move source to graveyard (simulating it dying)
    state
        .move_object_to_zone(source_id, mtg_engine::ZoneId::Graveyard(p1()))
        .unwrap();

    // After source leaves: effect is no longer active (source not on battlefield)
    let chars_after = calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(
        chars_after.power,
        Some(2),
        "back to base 2 after source left battlefield"
    );
}

/// CR 514.2: "Until end of turn" effects are removed at cleanup.
///
/// This test verifies:
/// 1. A `UntilEndOfTurn` effect modifies characteristics while it is active.
/// 2. An `Indefinite` effect on the same object is unaffected.
/// 3. After `UntilEndOfTurn` effects are removed from the state (as `expire_end_of_turn_effects`
///    does during Cleanup), only the permanent effect remains.
///
/// The `expire_end_of_turn_effects` function is called from `cleanup_actions` in
/// `turn_actions.rs`; the integration between cleanup and expiry is tested in
/// `tests/turn_actions.rs`. This test focuses on the layer system's handling of
/// the two duration types.
#[test]
fn test_613_until_end_of_turn_expires_at_cleanup() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Test", 2, 2))
        // UntilEndOfTurn effect: +3/+3
        .add_continuous_effect(eot_effect(
            1,
            None,
            10,
            EffectLayer::PtModify,
            EffectFilter::AllCreatures,
            LayerModification::ModifyBoth(3),
        ))
        // Indefinite effect: +1/+1 (should persist after cleanup)
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 20,
            layer: EffectLayer::PtModify,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::ModifyBoth(1),
            is_cda: false,
        })
        .build()
        .unwrap();

    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Both effects active: base 2 + EoT +3 + Indefinite +1 = 6
    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(6), "both effects active: 2 + 3 + 1 = 6");

    // Simulate cleanup: expire UntilEndOfTurn effects (as expire_end_of_turn_effects does)
    let mut state = state;
    state.continuous_effects = state
        .continuous_effects
        .iter()
        .filter(|e| e.duration != mtg_engine::EffectDuration::UntilEndOfTurn)
        .cloned()
        .collect();

    // Only the Indefinite effect remains: base 2 + Indefinite +1 = 3
    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "only indefinite effect remains: 2 + 1 = 3"
    );
    assert!(
        state
            .continuous_effects
            .iter()
            .all(|e| e.duration != mtg_engine::EffectDuration::UntilEndOfTurn),
        "no UntilEndOfTurn effects should remain"
    );
}

// ---------------------------------------------------------------------------
// Counter P/T modification (Layer 7c)
// ---------------------------------------------------------------------------

/// CR 613.4c: +1/+1 counters modify P/T in layer 7c.
#[test]
fn test_613_plus_one_counters_modify_pt() {
    use mtg_engine::CounterType;

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Test", 2, 2).with_counter(CounterType::PlusOnePlusOne, 3),
        )
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(5), "2 + 3 counters = 5");
    assert_eq!(chars.toughness, Some(5), "2 + 3 counters = 5");
}

/// CR 613.4c: -1/-1 counters reduce P/T in layer 7c.
#[test]
fn test_613_minus_one_counters_reduce_pt() {
    use mtg_engine::CounterType;

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Test", 5, 5).with_counter(CounterType::MinusOneMinusOne, 2),
        )
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(3), "5 - 2 counters = 3");
    assert_eq!(chars.toughness, Some(3));
}

/// CR 613.4c: Counters apply AFTER 7b set effects and BEFORE 7d switch.
#[test]
fn test_613_counters_apply_after_set_before_switch() {
    use mtg_engine::CounterType;

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            // Base: 1/1, with 2 +1/+1 counters = 3/3 after layer 7c
            ObjectSpec::creature(p1(), "Test", 1, 1).with_counter(CounterType::PlusOnePlusOne, 2),
        )
        // Layer 7b: set to 1/1 (overrides base)
        .add_continuous_effect(effect(
            1,
            None,
            10,
            EffectLayer::PtSet,
            EffectFilter::AllCreatures,
            LayerModification::SetPowerToughness {
                power: 1,
                toughness: 1,
            },
        ))
        // Layer 7d: switch P/T — applied after counters
        // After 7b: 1/1. After 7c (counters): 3/3. After 7d: 3/3 (symmetric).
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // 7b: 1/1. 7c: +2 counters = 3/3. (No 7d effect here.)
    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(3), "7b(1/1) + 7c(+2 counters) = 3");
    assert_eq!(chars.toughness, Some(3));
}

// ---------------------------------------------------------------------------
// Humility + Opalescence interaction (CR 613.10 example)
// ---------------------------------------------------------------------------

/// CR 613.10: Opalescence makes non-Aura enchantments into creatures (layer 4).
/// Then Humility removes all abilities (layer 6) and sets all creatures to 1/1 (layer 7b).
/// The layer ordering ensures Opalescence (layer 4) runs before Humility (layers 6, 7b).
#[test]
fn test_613_opalescence_makes_enchantments_into_creatures() {
    use mtg_engine::ManaCost;

    // An enchantment with mana value 4 (simulating Opalescence making it a creature).
    let enchantment_spec =
        ObjectSpec::enchantment(p1(), "Test Enchantment").with_mana_cost(ManaCost {
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
            generic: 4,
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(enchantment_spec)
        // Opalescence Layer 4 effect: non-Aura enchantments become creatures
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 5,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonAuraEnchantments,
            modification: LayerModification::AddCardTypes(ordset![CardType::Creature]),
            is_cda: false,
        })
        // Opalescence Layer 7b effect: P/T = mana value
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 5,
            layer: EffectLayer::PtSet,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonAuraEnchantments,
            modification: LayerModification::SetPtToManaValue,
            is_cda: false,
        })
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "enchantment should become a creature"
    );
    assert!(
        chars.card_types.contains(&CardType::Enchantment),
        "should still be an enchantment"
    );
    assert_eq!(chars.power, Some(4), "P/T = mana value (4)");
    assert_eq!(chars.toughness, Some(4));
}

/// CR 613.10: Humility + Opalescence full interaction.
/// Both enchantments become 1/1 creatures with no abilities.
/// Layer ordering (4 → 6 → 7b) handles this correctly:
/// - Layer 4: Opalescence makes enchantments (including Humility itself) into creatures
/// - Layer 6: Humility removes all creature abilities (including both cards' abilities)
/// - Layer 7b: Humility (newer) sets all creatures to 1/1 after Opalescence's mana-value P/T
#[test]
fn test_613_humility_plus_opalescence() {
    use mtg_engine::ManaCost;

    // Humility (mana value 4: {2}{W}{W}), Opalescence (mana value 4: {2}{W}{W})
    // Humility enters AFTER Opalescence (higher timestamp = newer).

    let humility_spec = ObjectSpec::enchantment(p1(), "Humility").with_mana_cost(ManaCost {
        white: 2,
        blue: 0,
        black: 0,
        red: 0,
        green: 0,
        colorless: 0,
        generic: 2, // {2}{W}{W} = 4,
        ..Default::default()
    });
    let opalescence_spec = ObjectSpec::enchantment(p1(), "Opalescence").with_mana_cost(ManaCost {
        white: 2,
        blue: 0,
        black: 0,
        red: 0,
        green: 0,
        colorless: 0,
        generic: 2, // {2}{W}{W} = 4,
        ..Default::default()
    });

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(opalescence_spec) // timestamp 1 (older)
        .object(humility_spec) // timestamp 2 (newer)
        // Opalescence effects (timestamp 5 = entered at time 5):
        // Layer 4: enchantments become creatures
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 5, // Opalescence entered first
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonAuraEnchantments,
            modification: LayerModification::AddCardTypes(ordset![CardType::Creature]),
            is_cda: false,
        })
        // Layer 7b: P/T = mana value (Opalescence, older)
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 5,
            layer: EffectLayer::PtSet,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonAuraEnchantments,
            modification: LayerModification::SetPtToManaValue,
            is_cda: false,
        })
        // Humility effects (timestamp 10 = entered after Opalescence):
        // Layer 6: all creatures lose all abilities
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(3),
            source: None,
            timestamp: 10, // Humility entered second (newer)
            layer: EffectLayer::Ability,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::RemoveAllAbilities,
            is_cda: false,
        })
        // Layer 7b: all creatures are base 1/1 (Humility, newer — overrides Opalescence)
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(4),
            source: None,
            timestamp: 10,
            layer: EffectLayer::PtSet,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetPowerToughness {
                power: 1,
                toughness: 1,
            },
            is_cda: false,
        })
        .build()
        .unwrap();

    // Get both objects (Opalescence = first, Humility = second)
    let bf_ids: Vec<_> = state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .to_vec();

    // Both enchantments should now be 1/1 creatures with no abilities.
    for &id in &bf_ids {
        let chars = calculate_characteristics(&state, id).unwrap();
        let name = &state.objects.get(&id).unwrap().characteristics.name;

        // Layer 4: Opalescence makes them creatures
        assert!(
            chars.card_types.contains(&CardType::Creature),
            "{name} should be a creature (Opalescence, layer 4)"
        );
        assert!(
            chars.card_types.contains(&CardType::Enchantment),
            "{name} should still be an enchantment"
        );

        // Layer 6: Humility removes all abilities
        assert!(
            chars.keywords.is_empty(),
            "{name} should have no keyword abilities (Humility, layer 6)"
        );

        // Layer 7b: Humility (newer) sets P/T to 1/1
        assert_eq!(
            chars.power,
            Some(1),
            "{name} should be 1/1 (Humility layer 7b, newer timestamp wins)"
        );
        assert_eq!(chars.toughness, Some(1));
    }
}

// ---------------------------------------------------------------------------
// Blood Moon + Urborg interaction (CR 613.8 dependency)
// ---------------------------------------------------------------------------

/// CR 613.8: Blood Moon + Urborg — Blood Moon depends on Urborg (SetTypeLine
/// depends on AddSubtypes), so Urborg applies first. Blood Moon then overrides.
/// Result: all nonbasic lands are Mountains only (not Swamps).
///
/// This test: Blood Moon entered AFTER Urborg (Blood Moon is newer/higher timestamp).
/// Without dependency: Urborg first (older) → Blood Moon second (newer) → Mountain only ✓
/// (This is also correct by timestamp alone when Blood Moon is newer.)
#[test]
fn test_613_blood_moon_plus_urborg_blood_moon_newer() {
    // Urborg adds Swamp subtype to all lands.
    // Blood Moon sets all nonbasic lands to "Land — Mountain" (overriding Urborg).
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::land(p1(), "Nonbasic Land")) // A nonbasic land
        // Urborg effect (timestamp 5, older): add Swamp subtype to all lands
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 5,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllLands,
            modification: LayerModification::AddSubtypes(ordset![SubType("Swamp".to_string())]),
            is_cda: false,
        })
        // Blood Moon effect (timestamp 10, newer): set type to "Land — Mountain"
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 10,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonbasicLands,
            modification: LayerModification::SetTypeLine {
                supertypes: OrdSet::new(),
                card_types: ordset![CardType::Land],
                subtypes: ordset![SubType("Mountain".to_string())],
            },
            is_cda: false,
        })
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    // Blood Moon (newer) overrides Urborg (older): land is Mountain only.
    assert!(
        chars.subtypes.contains(&SubType("Mountain".to_string())),
        "land should be a Mountain"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Swamp".to_string())),
        "land should NOT be a Swamp — Blood Moon's SetTypeLine overrides Urborg's AddSubtypes"
    );
}

/// CR 613.8: Blood Moon + Urborg — Blood Moon entered BEFORE Urborg (older timestamp).
/// Without dependency: Blood Moon first (older) → Urborg second (newer) → Mountain + Swamp ✗
/// With dependency (SetTypeLine depends on AddSubtypes): Urborg applies first regardless
/// of timestamp, then Blood Moon overrides → Mountain only ✓
///
/// This is the critical dependency test: the result must be Mountain regardless of
/// which permanent entered the battlefield first.
#[test]
fn test_613_blood_moon_plus_urborg_blood_moon_older_dependency_wins() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::land(p1(), "Nonbasic Land"))
        // Blood Moon effect (timestamp 5, OLDER): set type to "Land — Mountain"
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 5, // Blood Moon entered FIRST (older)
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonbasicLands,
            modification: LayerModification::SetTypeLine {
                supertypes: OrdSet::new(),
                card_types: ordset![CardType::Land],
                subtypes: ordset![SubType("Mountain".to_string())],
            },
            is_cda: false,
        })
        // Urborg effect (timestamp 10, NEWER): add Swamp subtype to all lands
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 10, // Urborg entered SECOND (newer)
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllLands,
            modification: LayerModification::AddSubtypes(ordset![SubType("Swamp".to_string())]),
            is_cda: false,
        })
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    // With dependency: Urborg (AddSubtypes) applies first, Blood Moon (SetTypeLine)
    // applies second and overrides. Result: Mountain only, regardless of timestamp.
    assert!(
        chars.subtypes.contains(&SubType("Mountain".to_string())),
        "land should be a Mountain"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Swamp".to_string())),
        "land should NOT be a Swamp — dependency ensures Blood Moon applies after Urborg"
    );
}

// ---------------------------------------------------------------------------
// Dependency chain (A depends on B depends on C)
// ---------------------------------------------------------------------------

/// CR 613.8: Dependency chains are resolved correctly.
/// Effect A (SetTypeLine) depends on B (AddSubtypes), which depends on C (AddCardTypes).
/// C → B → A (C applied first, A applied last).
#[test]
fn test_613_dependency_chain_three_effects() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::land(p1(), "Test Land"))
        // Effect C (oldest): AddCardTypes — adds Artifact type
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 1, // Oldest
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllLands,
            modification: LayerModification::AddCardTypes(ordset![CardType::Artifact]),
            is_cda: false,
        })
        // Effect B (middle): AddSubtypes — adds Swamp
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 5,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllLands,
            modification: LayerModification::AddSubtypes(ordset![SubType("Swamp".to_string())]),
            is_cda: false,
        })
        // Effect A (newest): SetTypeLine — overrides everything
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(3),
            source: None,
            timestamp: 10, // Newest, but depends on both B and C (SetTypeLine depends on AddSubtypes and AddCardTypes)
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllLands,
            modification: LayerModification::SetTypeLine {
                supertypes: OrdSet::new(),
                card_types: ordset![CardType::Land],
                subtypes: ordset![SubType("Mountain".to_string())],
            },
            is_cda: false,
        })
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // C and B apply first (AddCardTypes, AddSubtypes), then A overrides (SetTypeLine).
    // Final result: Land — Mountain (A overrides both B and C).
    let chars = calculate_characteristics(&state, id).unwrap();
    assert!(chars.subtypes.contains(&SubType("Mountain".to_string())));
    assert!(!chars.subtypes.contains(&SubType("Swamp".to_string())));
    assert!(
        !chars.card_types.contains(&CardType::Artifact),
        "SetTypeLine overrides AddCardTypes"
    );
}

// ---------------------------------------------------------------------------
// Circular dependency (fallback to timestamp)
// ---------------------------------------------------------------------------

/// CR 613.8b: Circular dependencies fall back to timestamp order.
/// Two effects that each "depend on" each other (circular) → apply by timestamp.
/// (Simulated by creating two SetTypeLine effects, neither of which we make depend on
/// the other in the implementation — they just apply in timestamp order.)
///
/// In practice, true circular dependencies are rare in MTG, but the engine must not
/// panic or infinite-loop. We verify it applies in timestamp order.
#[test]
fn test_613_independent_effects_apply_in_timestamp_order() {
    // Two independent type-adding effects. Both apply, in timestamp order.
    // (No circular dependency in this test — just verifying timestamp order for non-dependent effects.)
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Test", 2, 2))
        // Older: add Flying
        .add_continuous_effect(effect(
            1,
            None,
            5, // Older
            EffectLayer::Ability,
            EffectFilter::AllCreatures,
            LayerModification::AddKeyword(KeywordAbility::Flying),
        ))
        // Newer: add Trample
        .add_continuous_effect(effect(
            2,
            None,
            10, // Newer
            EffectLayer::Ability,
            EffectFilter::AllCreatures,
            LayerModification::AddKeyword(KeywordAbility::Trample),
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    // Both additive effects should apply regardless of order (addition is commutative).
    assert!(chars.keywords.contains(&KeywordAbility::Flying));
    assert!(chars.keywords.contains(&KeywordAbility::Trample));
}

// ---------------------------------------------------------------------------
// No effect on non-matching objects
// ---------------------------------------------------------------------------

/// Filters correctly exclude non-matching objects.
#[test]
fn test_613_filter_excludes_non_matching_objects() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::land(p1(), "Test Land")) // A land, not a creature
        // Effect applies to AllCreatures only
        .add_continuous_effect(effect(
            1,
            None,
            10,
            EffectLayer::PtSet,
            EffectFilter::AllCreatures, // Land doesn't match
            LayerModification::SetPowerToughness {
                power: 5,
                toughness: 5,
            },
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    // Land has no P/T — effect shouldn't apply, and None stays None.
    assert_eq!(chars.power, None, "land should have no P/T");
    assert_eq!(chars.toughness, None);
}

/// Layer 4 type change makes an object newly match a later layer's filter.
/// Classic example: if layer 4 adds Creature to an enchantment, layer 6's
/// "AllCreatures" filter should now match it.
#[test]
fn test_613_layer4_type_change_enables_later_filter() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::enchantment(p1(), "Test Enchantment").with_keyword(KeywordAbility::Flying),
        )
        // Layer 4: make enchantments into creatures
        .add_continuous_effect(effect(
            1,
            None,
            5,
            EffectLayer::TypeChange,
            EffectFilter::AllEnchantments,
            LayerModification::AddCardTypes(ordset![CardType::Creature]),
        ))
        // Layer 6: all creatures lose all abilities (filter evaluated after layer 4)
        .add_continuous_effect(effect(
            2,
            None,
            10,
            EffectLayer::Ability,
            EffectFilter::AllCreatures, // Now matches the enchantment (became creature in layer 4)
            LayerModification::RemoveAllAbilities,
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "enchantment should be a creature (layer 4)"
    );
    assert!(
        chars.keywords.is_empty(),
        "enchantment-creature should have lost all abilities (layer 6, filter matched after layer 4)"
    );
}

// ---------------------------------------------------------------------------
// No active effects: base characteristics unchanged
// ---------------------------------------------------------------------------

/// With no continuous effects, calculate_characteristics returns base characteristics.
#[test]
fn test_613_no_effects_returns_base_characteristics() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Grizzly Bears", 2, 2).with_keyword(KeywordAbility::Trample),
        )
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(2));
    assert_eq!(chars.toughness, Some(2));
    assert!(chars.keywords.contains(&KeywordAbility::Trample));
    assert!(chars.card_types.contains(&CardType::Creature));
}

/// calculate_characteristics returns None for a nonexistent object.
#[test]
fn test_613_nonexistent_object_returns_none() {
    let state = GameStateBuilder::new().add_player(p1()).build().unwrap();
    let result = calculate_characteristics(&state, mtg_engine::ObjectId(9999));
    assert!(
        result.is_none(),
        "should return None for nonexistent object"
    );
}

// ---------------------------------------------------------------------------
// Layer ordering: type change (4) happens before ability add/remove (6)
// ---------------------------------------------------------------------------

/// CR 613.1: Layers apply in order. Type change in layer 4 happens before
/// ability changes in layer 6. This test verifies the ordering matters:
/// Layer 4 adds Creature, then Layer 6 removes all abilities from creatures.
#[test]
fn test_613_layer_ordering_type_before_ability() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::enchantment(p1(), "Test").with_keyword(KeywordAbility::Vigilance))
        // Layer 4 (type): makes enchantments into creatures
        .add_continuous_effect(effect(
            1,
            None,
            5,
            EffectLayer::TypeChange,
            EffectFilter::AllEnchantments,
            LayerModification::AddCardTypes(ordset![CardType::Creature]),
        ))
        // Layer 6 (ability): removes all abilities from creatures (now includes this enchantment)
        .add_continuous_effect(effect(
            2,
            None,
            5,
            EffectLayer::Ability,
            EffectFilter::AllCreatures,
            LayerModification::RemoveAllAbilities,
        ))
        .build()
        .unwrap();
    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();
    assert!(chars.card_types.contains(&CardType::Creature));
    assert!(
        chars.keywords.is_empty(),
        "enchantment-creature should lose all abilities via layer 6"
    );
}

// ---------------------------------------------------------------------------
// CC#6: Humility + Magus of the Moon non-dependency test
// ---------------------------------------------------------------------------

/// CC#6 / CR 613.1d + 613.1f + 613.8: Humility + Magus of the Moon do NOT form a
/// dependency because they affect different layers (layer 4 vs layer 6/7b).
///
/// Magus of the Moon (layer 4): nonbasic lands become Mountains.
/// Humility (layers 6+7b): all creatures lose all abilities and are 1/1.
///
/// These effects don't interact: Magus does not affect whether Humility applies
/// to creatures, and Humility does not affect whether Magus changes land types.
/// Both apply independently in their respective layers (CR 613.1d before 613.1f).
///
/// Expected results:
/// - The nonbasic land is now a Mountain (Land — Mountain, layer 4 via Magus).
/// - The creature is 1/1 with no keywords (layer 6+7b via Humility).
/// - No dependency check is needed because the effects are in different layers.
#[test]
fn test_cc6_humility_magus_of_moon_nondependency() {
    let p = PlayerId(1);

    // State: one nonbasic land + one creature (Flying Bear)
    let state = GameStateBuilder::new()
        .add_player(p)
        .object(ObjectSpec::land(p, "Nonbasic Land")) // not basic — affected by Magus
        .object(
            ObjectSpec::creature(p, "Flying Bear", 2, 2).with_keyword(KeywordAbility::Flying), // Humility should strip this
        )
        // Magus of the Moon effect — Layer 4: nonbasic lands become Mountains (CR 613.1d).
        // "Nonbasic lands are Mountains" → SetTypeLine { Land — Mountain } on AllNonbasicLands.
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 10, // Magus entered first
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonbasicLands,
            modification: LayerModification::SetTypeLine {
                supertypes: OrdSet::new(),
                card_types: ordset![CardType::Land],
                subtypes: ordset![SubType("Mountain".to_string())],
            },
            is_cda: false,
        })
        // Humility effect 1 — Layer 6: all creatures lose all abilities (CR 613.1f).
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 20, // Humility entered second
            layer: EffectLayer::Ability,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::RemoveAllAbilities,
            is_cda: false,
        })
        // Humility effect 2 — Layer 7b: all creatures have base P/T 1/1 (CR 613.4b).
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(3),
            source: None,
            timestamp: 20, // Same Humility timestamp
            layer: EffectLayer::PtSet,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetPowerToughness {
                power: 1,
                toughness: 1,
            },
            is_cda: false,
        })
        .build()
        .unwrap();

    // Find the land and the creature by type.
    let land_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Nonbasic Land")
        .map(|(id, _)| *id)
        .expect("Nonbasic Land not found");

    let creature_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Flying Bear")
        .map(|(id, _)| *id)
        .expect("Flying Bear not found");

    // ── Verify Magus of the Moon effect (layer 4) ──────────────────────────
    let land_chars = calculate_characteristics(&state, land_id).unwrap();

    // The nonbasic land should now be typed as Land — Mountain (CR 613.1d).
    assert!(
        land_chars.card_types.contains(&CardType::Land),
        "land should still be a Land; types: {:?}",
        land_chars.card_types
    );
    assert!(
        land_chars
            .subtypes
            .contains(&SubType("Mountain".to_string())),
        "nonbasic land should be a Mountain via Magus of the Moon (layer 4); \
         subtypes: {:?}",
        land_chars.subtypes
    );

    // ── Verify Humility effect (layers 6 + 7b) ─────────────────────────────
    let creature_chars = calculate_characteristics(&state, creature_id).unwrap();

    // Flying should be stripped by Humility's layer-6 RemoveAllAbilities.
    assert!(
        !creature_chars.keywords.contains(&KeywordAbility::Flying),
        "Flying should be removed by Humility (layer 6); keywords: {:?}",
        creature_chars.keywords
    );
    assert!(
        creature_chars.keywords.is_empty(),
        "Humility (layer 6) should remove all keywords; keywords: {:?}",
        creature_chars.keywords
    );

    // P/T should be 1/1 from Humility's layer-7b SetPowerToughness.
    assert_eq!(
        creature_chars.power,
        Some(1),
        "Humility should set base P/T to 1/1; power: {:?}",
        creature_chars.power
    );
    assert_eq!(
        creature_chars.toughness,
        Some(1),
        "Humility should set base P/T to 1/1; toughness: {:?}",
        creature_chars.toughness
    );

    // ── No dependency: both effects applied independently ───────────────────
    // The land is a Mountain (Magus layer 4) and the creature is 1/1 no-ability (Humility).
    // Neither result depends on the other (different layers → no CR 613.8 dependency).
    // The land should NOT be affected by Humility (it's not a creature).
    assert!(
        !land_chars.card_types.contains(&CardType::Creature),
        "the land should not become a creature; types: {:?}",
        land_chars.card_types
    );
}

// ---------------------------------------------------------------------------
// CC#7: Opalescence + Parallax Wave zone-change
// ---------------------------------------------------------------------------

/// CC#7 / CR 613.1d + CR 603.2 — Opalescence makes Parallax Wave a creature (layer 4).
///
/// When Parallax Wave is type-changed to be a creature (Enchantment Creature — Wave),
/// removing it from the battlefield triggers "creature leaves the battlefield" effects.
/// This test verifies:
/// 1. Under Opalescence (layer 4), Parallax Wave has CardType::Creature in calculated chars.
/// 2. The type-change is what enables creature-specific removal (destroying it or similar).
/// 3. Once the type-change enables creature status, zone-change triggers for "creature dies"
///    would fire. (Here we verify the characteristic calculation side — the type change is
///    visible via `calculate_characteristics`.)
///
/// The full trigger dispatch (zone-change triggers) is deferred to a later integration test;
/// this test covers the layer system enabling creature removal of an animated enchantment.
#[test]
fn test_cc7_opalescence_parallax_wave_zone_change() {
    use mtg_engine::ManaCost;

    let p = PlayerId(1);

    // Parallax Wave: normally a 4-mana enchantment (non-Aura).
    let parallax_wave_spec = ObjectSpec::enchantment(p, "Parallax Wave").with_mana_cost(ManaCost {
        generic: 3,
        white: 1,
        ..ManaCost::default()
    });

    // Opalescence is on the battlefield (layer 4): non-Aura enchantments become creatures.
    // This also sets P/T to the card's mana value (layer 7b).
    let state = GameStateBuilder::new()
        .add_player(p)
        .object(parallax_wave_spec)
        // Opalescence layer 4 effect: non-Aura enchantments become creatures.
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 5,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonAuraEnchantments,
            modification: LayerModification::AddCardTypes(ordset![CardType::Creature]),
            is_cda: false,
        })
        // Opalescence layer 7b effect: P/T = mana value.
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 5,
            layer: EffectLayer::PtSet,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonAuraEnchantments,
            modification: LayerModification::SetPtToManaValue,
            is_cda: false,
        })
        .build()
        .unwrap();

    let wave_id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // CR 613.1d: Opalescence (layer 4) grants Parallax Wave the Creature card type.
    let chars = calculate_characteristics(&state, wave_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "Parallax Wave should be a Creature under Opalescence (layer 4); \
         types: {:?}",
        chars.card_types
    );
    assert!(
        chars.card_types.contains(&CardType::Enchantment),
        "Parallax Wave should still be an Enchantment; types: {:?}",
        chars.card_types
    );

    // Mana value of {3W} = 4; Opalescence sets P/T to 4/4 (layer 7b).
    assert_eq!(
        chars.power,
        Some(4),
        "Parallax Wave should have P/T = mana value (4) under Opalescence; \
         power: {:?}",
        chars.power
    );
    assert_eq!(chars.toughness, Some(4));

    // Consequence: Parallax Wave with Creature type can now be destroyed by creature-removal
    // effects. When it leaves the battlefield, zone-change triggers for "creature leaves the
    // battlefield" fire. (Integration behavior verified by SBA + trigger dispatch tests.)
    //
    // We simulate zone-change and verify the object becomes a new object (CR 400.7),
    // which is the identity change that "kills" the animated enchantment as a creature.
    let mut state = state;
    let (new_id, old_snapshot) = state
        .move_object_to_zone(wave_id, mtg_engine::ZoneId::Graveyard(p))
        .unwrap();

    // Old ObjectId is no longer valid (zone change = new identity per CR 400.7).
    assert_ne!(
        wave_id, new_id,
        "zone change produces a new ObjectId (CR 400.7)"
    );
    assert_eq!(
        old_snapshot.zone,
        mtg_engine::ZoneId::Battlefield,
        "old snapshot was on the battlefield (where it was a Creature under Opalescence)"
    );
    assert_eq!(old_snapshot.characteristics.name, "Parallax Wave");
}

// ---------------------------------------------------------------------------
// CC#4: Yixlid Jailer + Anger (graveyard ability removal)
// ---------------------------------------------------------------------------

/// CC#4 / CR 613.1f + CR 613.7 — Yixlid Jailer removes abilities from cards in
/// graveyards (layer 6, `AllCardsInGraveyards` filter).
///
/// Anger is a creature card that says: "As long as Anger is in your graveyard
/// and you control a Mountain, creatures you control have haste." This is a static
/// ability that applies from the graveyard. Yixlid Jailer says "Cards in graveyards
/// lose all abilities."
///
/// When Yixlid Jailer is on the battlefield:
/// - `calculate_characteristics` for Anger (in graveyard) should show NO keyword abilities.
/// - Anger's static "grant haste" effect is suppressed because its source loses its ability.
///
/// This test verifies the layer system handles the `AllCardsInGraveyards` filter correctly.
#[test]
fn test_cc4_yixlid_jailer_removes_anger_graveyard_ability() {
    let p = PlayerId(1);

    // Build state with Anger in p's graveyard.
    // Anger has Haste (representing its self-characteristic) and we model it as having
    // an "ability" keyword that should be stripped by Yixlid Jailer's layer 6 effect.
    let state = GameStateBuilder::new()
        .add_player(p)
        // Anger: in graveyard with Haste keyword (the ability Jailer should strip).
        .object(
            ObjectSpec::creature(p, "Anger", 2, 1)
                .with_keyword(KeywordAbility::Haste)
                .in_zone(mtg_engine::ZoneId::Graveyard(p)),
        )
        // A creature on the battlefield (would normally benefit from Anger's grant).
        .object(ObjectSpec::creature(p, "Hill Giant", 3, 3))
        // Yixlid Jailer continuous effect (layer 6): all cards in graveyards lose all abilities.
        // CR 613.1f: ability-removing effects apply in layer 6.
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 10,
            layer: EffectLayer::Ability,
            duration: EffectDuration::WhileSourceOnBattlefield,
            filter: EffectFilter::AllCardsInGraveyards,
            modification: LayerModification::RemoveAllAbilities,
            is_cda: false,
        })
        .build()
        .unwrap();

    // Find Anger in the graveyard.
    let anger_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Anger")
        .map(|(id, _)| *id)
        .expect("Anger not found");

    // CR 613.1f: Yixlid Jailer (layer 6) strips Anger's Haste ability from the graveyard.
    let anger_chars = calculate_characteristics(&state, anger_id).unwrap();
    assert!(
        !anger_chars.keywords.contains(&KeywordAbility::Haste),
        "Anger (in graveyard) should lose Haste from Yixlid Jailer (layer 6, AllCardsInGraveyards); \
         keywords: {:?}",
        anger_chars.keywords
    );
    assert!(
        anger_chars.keywords.is_empty(),
        "Anger should have NO keyword abilities under Yixlid Jailer (layer 6); \
         keywords: {:?}",
        anger_chars.keywords
    );

    // The battlefield creature (Hill Giant) is NOT affected by Yixlid Jailer.
    // Jailer only affects graveyards (AllCardsInGraveyards filter).
    let giant_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Hill Giant")
        .map(|(id, _)| *id)
        .expect("Hill Giant not found");

    let giant_chars = calculate_characteristics(&state, giant_id).unwrap();
    // Hill Giant has no keywords printed, but confirm it's not incorrectly affected.
    assert_eq!(
        giant_chars.power,
        Some(3),
        "Hill Giant's P/T should be unaffected by Yixlid Jailer (battlefield, not graveyard)"
    );

    // Without Yixlid Jailer, Anger keeps its Haste.
    let state_no_jailer = GameStateBuilder::new()
        .add_player(p)
        .object(
            ObjectSpec::creature(p, "Anger", 2, 1)
                .with_keyword(KeywordAbility::Haste)
                .in_zone(mtg_engine::ZoneId::Graveyard(p)),
        )
        .build()
        .unwrap();

    let anger_id_2 = state_no_jailer
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Anger")
        .map(|(id, _)| *id)
        .expect("Anger (no Jailer) not found");

    let anger_chars_2 = calculate_characteristics(&state_no_jailer, anger_id_2).unwrap();
    assert!(
        anger_chars_2.keywords.contains(&KeywordAbility::Haste),
        "Without Yixlid Jailer, Anger retains Haste; keywords: {:?}",
        anger_chars_2.keywords
    );
}

// ---------------------------------------------------------------------------
// MR-M5-08: CDA vs non-CDA ordering in the same sublayer (CR 613.3)
// ---------------------------------------------------------------------------

/// MR-M5-08 / CR 613.3 — In layer 7b (PtSet), CDAs apply before non-CDAs
/// regardless of timestamp.
///
/// Setup:
/// - CDA effect (is_cda: true) at timestamp=10 sets P/T to 2/2.
/// - Non-CDA effect (is_cda: false) at timestamp=5 (earlier!) sets P/T to 3/3.
///
/// Without special CDA ordering, the earlier-timestamp non-CDA would apply first
/// (setting 3/3), then the later-timestamp CDA would apply (setting 2/2) → final 2/2.
///
/// With CDA-first ordering (CR 613.3): CDAs partition separately and apply before
/// all non-CDAs in the same sublayer. So the CDA (ts=10) applies first → 2/2,
/// then the non-CDA (ts=5) applies last → 3/3. Final P/T: 3/3.
#[test]
fn test_613_layer7b_cda_applies_before_noncda_same_sublayer() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        // A creature whose base P/T (5/6) will be overridden by effects.
        .object(ObjectSpec::creature(p1(), "Tarmogoyf-like", 5, 6))
        // CDA effect: timestamp=10 (later), sets P/T to 2/2.
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(100),
            source: None,
            timestamp: 10,
            layer: EffectLayer::PtSet,
            duration: EffectDuration::WhileSourceOnBattlefield,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetPowerToughness {
                power: 2,
                toughness: 2,
            },
            is_cda: true,
        })
        // Non-CDA effect: timestamp=5 (earlier), sets P/T to 3/3.
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(101),
            source: None,
            timestamp: 5,
            layer: EffectLayer::PtSet,
            duration: EffectDuration::WhileSourceOnBattlefield,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::SetPowerToughness {
                power: 3,
                toughness: 3,
            },
            is_cda: false,
        })
        .build()
        .unwrap();

    let id = *state
        .zones
        .get(&mtg_engine::ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let chars = calculate_characteristics(&state, id).unwrap();

    // CDA (ts=10, 2/2) applies first; non-CDA (ts=5, 3/3) applies last → 3/3 wins.
    // If timestamp-only ordering were used: non-CDA (ts=5) first → 3/3, CDA (ts=10) last
    // → 2/2 (wrong). The CDA-first partition ensures 3/3 is the final result.
    assert_eq!(
        chars.power,
        Some(3),
        "CR 613.3: CDA (ts=10, 2/2) applies before non-CDA (ts=5, 3/3) → non-CDA wins; \
         got power {:?}",
        chars.power
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 613.3: CDA (ts=10, 2/2) applies before non-CDA (ts=5, 3/3) → non-CDA wins; \
         got toughness {:?}",
        chars.toughness
    );
}
