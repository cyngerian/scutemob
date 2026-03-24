//! Static grant filter tests: EffectFilter::CreaturesYouControl,
//! OtherCreaturesYouControl, OtherCreaturesYouControlWithSubtype (CR 604.2).
//!
//! These filters resolve the source's controller dynamically at layer-application
//! time, enabling CardDef static abilities like "Creatures you control have haste."

use mtg_engine::{
    calculate_characteristics, ContinuousEffect, EffectDuration, EffectFilter, EffectId,
    EffectLayer, GameStateBuilder, KeywordAbility, LayerModification, ObjectId, ObjectSpec,
    PlayerId, SubType, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// Find objects by name on the battlefield.
fn find_on_battlefield(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    let bf = state.zones.get(&ZoneId::Battlefield).unwrap();
    *bf.object_ids()
        .iter()
        .find(|id| {
            state
                .objects
                .get(id)
                .map(|o| o.characteristics.name == name)
                .unwrap_or(false)
        })
        .unwrap_or_else(|| panic!("object '{}' not found on battlefield", name))
}

// ---------------------------------------------------------------------------
// CreaturesYouControl tests
// ---------------------------------------------------------------------------

/// CR 604.2: "Creatures you control have haste" grants haste only to
/// the source's controller's creatures, not the opponent's.
#[test]
fn test_creatures_you_control_grants_keyword_to_own_creatures_only() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::enchantment(p1(), "Fervor Source"))
        .object(ObjectSpec::creature(p1(), "P1 Bear", 2, 2))
        .object(ObjectSpec::creature(p2(), "P2 Bear", 2, 2))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Fervor Source");
    let p1_bear = find_on_battlefield(&state, "P1 Bear");
    let p2_bear = find_on_battlefield(&state, "P2 Bear");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
        is_cda: false,
        condition: None,
    });

    // P1's creature gets haste
    let p1_chars = calculate_characteristics(&state, p1_bear).unwrap();
    assert!(
        p1_chars.keywords.contains(&KeywordAbility::Haste),
        "P1's creature should have haste"
    );

    // P2's creature does NOT get haste
    let p2_chars = calculate_characteristics(&state, p2_bear).unwrap();
    assert!(
        !p2_chars.keywords.contains(&KeywordAbility::Haste),
        "P2's creature should NOT have haste"
    );
}

/// CR 604.2: CreaturesYouControl does not affect non-creature permanents.
#[test]
fn test_creatures_you_control_excludes_non_creatures() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::land(p1(), "P1 Land"))
        .object(ObjectSpec::enchantment(p1(), "Source"))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Source");
    let land_id = find_on_battlefield(&state, "P1 Land");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
        is_cda: false,
        condition: None,
    });

    let land_chars = calculate_characteristics(&state, land_id).unwrap();
    assert!(
        !land_chars.keywords.contains(&KeywordAbility::Haste),
        "Land should NOT get haste from CreaturesYouControl"
    );
}

// ---------------------------------------------------------------------------
// OtherCreaturesYouControl tests
// ---------------------------------------------------------------------------

/// CR 604.2: "Other creatures you control have haste" grants to own
/// creatures but excludes the source itself.
#[test]
fn test_other_creatures_you_control_excludes_source() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Dragon Lord", 6, 5))
        .object(ObjectSpec::creature(p1(), "P1 Goblin", 1, 1))
        .object(ObjectSpec::creature(p2(), "P2 Elf", 1, 1))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Dragon Lord");
    let p1_goblin = find_on_battlefield(&state, "P1 Goblin");
    let p2_elf = find_on_battlefield(&state, "P2 Elf");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::OtherCreaturesYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
        is_cda: false,
        condition: None,
    });

    // P1's other creature gets haste
    let goblin_chars = calculate_characteristics(&state, p1_goblin).unwrap();
    assert!(
        goblin_chars.keywords.contains(&KeywordAbility::Haste),
        "P1's other creature should have haste"
    );

    // Source creature does NOT get haste
    let source_chars = calculate_characteristics(&state, source_id).unwrap();
    assert!(
        !source_chars.keywords.contains(&KeywordAbility::Haste),
        "Source creature should NOT have haste from OtherCreaturesYouControl"
    );

    // P2's creature does NOT get haste
    let elf_chars = calculate_characteristics(&state, p2_elf).unwrap();
    assert!(
        !elf_chars.keywords.contains(&KeywordAbility::Haste),
        "P2's creature should NOT have haste"
    );
}

// ---------------------------------------------------------------------------
// OtherCreaturesYouControlWithSubtype tests
// ---------------------------------------------------------------------------

/// CR 604.2: "Other Vampires you control get +1/+1" — subtype filter
/// grants only to matching subtypes, excludes source, excludes opponent.
#[test]
fn test_other_creatures_with_subtype_filters_correctly() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Vampire Lord", 2, 2)
                .with_subtypes(vec![SubType("Vampire".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "P1 Vampire", 1, 1)
                .with_subtypes(vec![SubType("Vampire".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "P1 Goblin", 1, 1)
                .with_subtypes(vec![SubType("Goblin".to_string())]),
        )
        .object(
            ObjectSpec::creature(p2(), "P2 Vampire", 1, 1)
                .with_subtypes(vec![SubType("Vampire".to_string())]),
        )
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Vampire Lord");
    let p1_vamp = find_on_battlefield(&state, "P1 Vampire");
    let p1_goblin = find_on_battlefield(&state, "P1 Goblin");
    let p2_vamp = find_on_battlefield(&state, "P2 Vampire");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Vampire".to_string())),
        modification: LayerModification::ModifyBoth(1),
        is_cda: false,
        condition: None,
    });

    // P1's other Vampire gets +1/+1
    let vamp_chars = calculate_characteristics(&state, p1_vamp).unwrap();
    assert_eq!(vamp_chars.power, Some(2), "P1 Vampire should be 2/2");
    assert_eq!(vamp_chars.toughness, Some(2), "P1 Vampire should be 2/2");

    // Source Vampire does NOT get the bonus
    let source_chars = calculate_characteristics(&state, source_id).unwrap();
    assert_eq!(
        source_chars.power,
        Some(2),
        "Vampire Lord should remain 2/2"
    );
    assert_eq!(
        source_chars.toughness,
        Some(2),
        "Vampire Lord should remain 2/2"
    );

    // P1 Goblin does NOT get the bonus (wrong subtype)
    let goblin_chars = calculate_characteristics(&state, p1_goblin).unwrap();
    assert_eq!(goblin_chars.power, Some(1), "Goblin should remain 1/1");
    assert_eq!(goblin_chars.toughness, Some(1), "Goblin should remain 1/1");

    // P2 Vampire does NOT get the bonus (wrong controller)
    let p2_chars = calculate_characteristics(&state, p2_vamp).unwrap();
    assert_eq!(p2_chars.power, Some(1), "P2 Vampire should remain 1/1");
    assert_eq!(p2_chars.toughness, Some(1), "P2 Vampire should remain 1/1");
}

/// Edge case: effect with no source (source = None) should match nothing.
#[test]
fn test_creatures_you_control_no_source_matches_nothing() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Bear", 2, 2))
        .build()
        .unwrap();

    let bear_id = find_on_battlefield(&state, "Bear");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: None,
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
        is_cda: false,
        condition: None,
    });

    let chars = calculate_characteristics(&state, bear_id).unwrap();
    assert!(
        !chars.keywords.contains(&KeywordAbility::Flying),
        "Effect with no source should not grant anything"
    );
}

/// Multiple static grants from different controllers: each grants only to their own.
#[test]
fn test_multiple_controllers_grant_independently() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::enchantment(p1(), "P1 Enchantment"))
        .object(ObjectSpec::enchantment(p2(), "P2 Enchantment"))
        .object(ObjectSpec::creature(p1(), "P1 Soldier", 2, 2))
        .object(ObjectSpec::creature(p2(), "P2 Soldier", 2, 2))
        .build()
        .unwrap();

    let p1_src = find_on_battlefield(&state, "P1 Enchantment");
    let p2_src = find_on_battlefield(&state, "P2 Enchantment");
    let p1_soldier = find_on_battlefield(&state, "P1 Soldier");
    let p2_soldier = find_on_battlefield(&state, "P2 Soldier");

    // P1's enchantment grants haste
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(p1_src),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
        is_cda: false,
        condition: None,
    });

    // P2's enchantment grants flying
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(101),
        source: Some(p2_src),
        timestamp: 11,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
        is_cda: false,
        condition: None,
    });

    // P1's soldier: haste (from P1 enchantment), no flying
    let p1_chars = calculate_characteristics(&state, p1_soldier).unwrap();
    assert!(p1_chars.keywords.contains(&KeywordAbility::Haste));
    assert!(!p1_chars.keywords.contains(&KeywordAbility::Flying));

    // P2's soldier: flying (from P2 enchantment), no haste
    let p2_chars = calculate_characteristics(&state, p2_soldier).unwrap();
    assert!(p2_chars.keywords.contains(&KeywordAbility::Flying));
    assert!(!p2_chars.keywords.contains(&KeywordAbility::Haste));
}
