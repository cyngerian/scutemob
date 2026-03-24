//! Static grant filter tests: EffectFilter::CreaturesYouControl,
//! OtherCreaturesYouControl, OtherCreaturesYouControlWithSubtype (CR 604.2),
//! and the PB-25 additions: CreaturesOpponentsControl, AttackingCreaturesYouControl,
//! CreaturesYouControlWithSubtype, ArtifactsYouControl, CreaturesYouControlWithSupertype,
//! CreaturesYouControlWithColor, OtherCreaturesYouControlExcludingSubtype,
//! CreaturesYouControlExcludingSubtype, AttackingCreaturesYouControlWithSubtype,
//! AllCreaturesWithSubtype, OtherCreaturesYouControlWithSubtypes.
//!
//! These filters resolve the source's controller dynamically at layer-application
//! time, enabling CardDef static abilities like "Creatures you control have haste."

use im;
use mtg_engine::{
    calculate_characteristics, AttackTarget, Color, CombatState, ContinuousEffect, EffectDuration,
    EffectFilter, EffectId, EffectLayer, GameStateBuilder, KeywordAbility, LayerModification,
    ObjectId, ObjectSpec, PlayerId, SubType, SuperType, ZoneId,
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

// ---------------------------------------------------------------------------
// PB-25 new EffectFilter tests
// ---------------------------------------------------------------------------

/// CR 604.2 / CR 613.4c: "Creatures your opponents control get -2/-2."
/// Source's controller is immune; all opponents' creatures are debuffed.
#[test]
fn test_creatures_opponents_control_debuff() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Elesh Norn", 4, 7))
        .object(ObjectSpec::creature(p1(), "P1 Bear", 2, 2))
        .object(ObjectSpec::creature(p2(), "P2 Bear", 2, 2))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Elesh Norn");
    let p1_bear = find_on_battlefield(&state, "P1 Bear");
    let p2_bear = find_on_battlefield(&state, "P2 Bear");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesOpponentsControl,
        modification: LayerModification::ModifyBoth(-2),
        is_cda: false,
        condition: None,
    });

    // P1's own creature is NOT debuffed (controller immunity)
    let p1_chars = calculate_characteristics(&state, p1_bear).unwrap();
    assert_eq!(p1_chars.power, Some(2), "P1 Bear should remain 2/2");
    assert_eq!(p1_chars.toughness, Some(2), "P1 Bear should remain 2/2");

    // Source itself is also not debuffed
    let src_chars = calculate_characteristics(&state, source_id).unwrap();
    assert_eq!(src_chars.power, Some(4), "Elesh Norn should remain 4/7");

    // P2's creature IS debuffed
    let p2_chars = calculate_characteristics(&state, p2_bear).unwrap();
    assert_eq!(p2_chars.power, Some(0), "P2 Bear should be 0/0 after -2/-2");
    assert_eq!(
        p2_chars.toughness,
        Some(0),
        "P2 Bear should be 0/0 after -2/-2"
    );
}

fn p3() -> PlayerId {
    PlayerId(3)
}
fn p4() -> PlayerId {
    PlayerId(4)
}

/// CR 604.2 / CR 613.4c: "Creatures your opponents control get -2/-2."
/// Multiplayer: all 3 opponents affected in 4-player game, controller immune.
#[test]
fn test_creatures_opponents_control_multiplayer() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .add_player(p3())
        .add_player(p4())
        .object(ObjectSpec::creature(p1(), "Elesh Norn", 4, 7))
        .object(ObjectSpec::creature(p2(), "P2 Bear", 2, 2))
        .object(ObjectSpec::creature(p3(), "P3 Bear", 2, 2))
        .object(ObjectSpec::creature(p4(), "P4 Bear", 2, 2))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Elesh Norn");
    let p2_bear = find_on_battlefield(&state, "P2 Bear");
    let p3_bear = find_on_battlefield(&state, "P3 Bear");
    let p4_bear = find_on_battlefield(&state, "P4 Bear");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesOpponentsControl,
        modification: LayerModification::ModifyBoth(-2),
        is_cda: false,
        condition: None,
    });

    // All three opponents' creatures get -2/-2
    for (bear_id, name) in [(p2_bear, "P2"), (p3_bear, "P3"), (p4_bear, "P4")] {
        let chars = calculate_characteristics(&state, bear_id).unwrap();
        assert_eq!(chars.power, Some(0), "{} Bear should be 0/0", name);
        assert_eq!(chars.toughness, Some(0), "{} Bear should be 0/0", name);
    }
}

/// CR 604.2 / CR 613.1f / CR 611.3a: "Attacking creatures you control have deathtouch."
/// Only creatures in state.combat.attackers get the keyword.
#[test]
fn test_attacking_creatures_you_control_grants_keyword() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Ohran Frostfang Source", 2, 6))
        .object(ObjectSpec::creature(p1(), "Attacking Bear", 2, 2))
        .object(ObjectSpec::creature(p1(), "Sitting Bear", 2, 2))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Ohran Frostfang Source");
    let attacker_id = find_on_battlefield(&state, "Attacking Bear");
    let sitter_id = find_on_battlefield(&state, "Sitting Bear");

    // Set up combat: Attacking Bear is attacking p2
    state.combat = Some(CombatState {
        attacking_player: p1(),
        attackers: [(attacker_id, AttackTarget::Player(p2()))]
            .into_iter()
            .collect(),
        blockers: im::OrdMap::new(),
        damage_assignment_order: im::OrdMap::new(),
        first_strike_participants: im::OrdSet::new(),
        defenders_declared: im::OrdSet::new(),
        forced_blocks: im::OrdMap::new(),
        enlist_pairings: Vec::new(),
        blocked_attackers: im::OrdSet::new(),
    });

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::AttackingCreaturesYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
        is_cda: false,
        condition: None,
    });

    // Attacking creature gets deathtouch
    let attacker_chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert!(
        attacker_chars
            .keywords
            .contains(&KeywordAbility::Deathtouch),
        "Attacking creature should have deathtouch"
    );

    // Non-attacking creature does NOT get deathtouch
    let sitter_chars = calculate_characteristics(&state, sitter_id).unwrap();
    assert!(
        !sitter_chars.keywords.contains(&KeywordAbility::Deathtouch),
        "Non-attacking creature should NOT have deathtouch"
    );
}

/// CR 611.3a: AttackingCreaturesYouControl matches nothing when state.combat is None.
#[test]
fn test_attacking_creatures_filter_outside_combat() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Source", 2, 6))
        .object(ObjectSpec::creature(p1(), "Bear", 2, 2))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Source");
    let bear_id = find_on_battlefield(&state, "Bear");

    // No combat state — state.combat is None
    assert!(state.combat.is_none());

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::AttackingCreaturesYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
        is_cda: false,
        condition: None,
    });

    // Outside combat, no creature should receive the keyword
    let chars = calculate_characteristics(&state, bear_id).unwrap();
    assert!(
        !chars.keywords.contains(&KeywordAbility::Deathtouch),
        "Creature should NOT have deathtouch outside of combat"
    );
}

/// CR 604.2: "Elf creatures you control get +3/+3" (includes source).
/// CreaturesYouControlWithSubtype differs from OtherCreaturesYouControlWithSubtype —
/// the source Elf DOES receive the bonus from its own activated ability.
#[test]
fn test_creatures_you_control_with_subtype_includes_self() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Ezuri", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "P1 Elf", 1, 1)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .object(
            ObjectSpec::creature(p2(), "P2 Elf", 1, 1)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Ezuri");
    let p1_elf = find_on_battlefield(&state, "P1 Elf");
    let p2_elf = find_on_battlefield(&state, "P2 Elf");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::CreaturesYouControlWithSubtype(SubType("Elf".to_string())),
        modification: LayerModification::ModifyBoth(3),
        is_cda: false,
        condition: None,
    });

    // Source Elf (Ezuri) gets +3/+3 (includes self, unlike OtherCreaturesYouControlWithSubtype)
    let src_chars = calculate_characteristics(&state, source_id).unwrap();
    assert_eq!(src_chars.power, Some(5), "Ezuri should be 5/5 (+3/+3)");
    assert_eq!(src_chars.toughness, Some(5), "Ezuri should be 5/5 (+3/+3)");

    // P1's other Elf also gets +3/+3
    let p1_chars = calculate_characteristics(&state, p1_elf).unwrap();
    assert_eq!(p1_chars.power, Some(4), "P1 Elf should be 4/4 (+3/+3)");

    // P2's Elf does NOT get the bonus (wrong controller)
    let p2_chars = calculate_characteristics(&state, p2_elf).unwrap();
    assert_eq!(p2_chars.power, Some(1), "P2 Elf should remain 1/1");
}

/// CR 604.2: "Legendary creatures you control get +1/+0."
/// Only legendary creatures are affected; non-legendary are not.
#[test]
fn test_creatures_you_control_with_supertype_legendary() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::enchantment(p1(), "Rising of the Day"))
        .object(
            ObjectSpec::creature(p1(), "P1 Legend", 3, 3)
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .object(ObjectSpec::creature(p1(), "P1 Commoner", 2, 2))
        .object(
            ObjectSpec::creature(p2(), "P2 Legend", 3, 3)
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Rising of the Day");
    let p1_legend = find_on_battlefield(&state, "P1 Legend");
    let p1_commoner = find_on_battlefield(&state, "P1 Commoner");
    let p2_legend = find_on_battlefield(&state, "P2 Legend");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControlWithSupertype(SuperType::Legendary),
        modification: LayerModification::ModifyPower(1),
        is_cda: false,
        condition: None,
    });

    // P1's legendary gets +1/+0
    let legend_chars = calculate_characteristics(&state, p1_legend).unwrap();
    assert_eq!(legend_chars.power, Some(4), "P1 Legend should be 4/3");
    assert_eq!(
        legend_chars.toughness,
        Some(3),
        "P1 Legend toughness unchanged"
    );

    // P1's non-legendary is NOT affected
    let commoner_chars = calculate_characteristics(&state, p1_commoner).unwrap();
    assert_eq!(commoner_chars.power, Some(2), "Commoner should remain 2/2");

    // P2's legendary is NOT affected (different controller)
    let p2_chars = calculate_characteristics(&state, p2_legend).unwrap();
    assert_eq!(p2_chars.power, Some(3), "P2 Legend should remain 3/3");
}

/// CR 604.2: "Red creatures you control have first strike."
/// Only red creatures controlled by source's controller are affected.
#[test]
fn test_creatures_you_control_with_color_red() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Bloodmark Mentor", 1, 1))
        .object(ObjectSpec::creature(p1(), "P1 Red Bear", 2, 2).with_colors(vec![Color::Red]))
        .object(ObjectSpec::creature(p1(), "P1 Blue Bear", 2, 2).with_colors(vec![Color::Blue]))
        .object(ObjectSpec::creature(p2(), "P2 Red Bear", 2, 2).with_colors(vec![Color::Red]))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Bloodmark Mentor");
    let p1_red = find_on_battlefield(&state, "P1 Red Bear");
    let p1_blue = find_on_battlefield(&state, "P1 Blue Bear");
    let p2_red = find_on_battlefield(&state, "P2 Red Bear");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControlWithColor(Color::Red),
        modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
        is_cda: false,
        condition: None,
    });

    // P1's red creature gets first strike
    let red_chars = calculate_characteristics(&state, p1_red).unwrap();
    assert!(
        red_chars.keywords.contains(&KeywordAbility::FirstStrike),
        "P1 Red Bear should have first strike"
    );

    // P1's blue creature does NOT get first strike
    let blue_chars = calculate_characteristics(&state, p1_blue).unwrap();
    assert!(
        !blue_chars.keywords.contains(&KeywordAbility::FirstStrike),
        "P1 Blue Bear should NOT have first strike"
    );

    // P2's red creature does NOT get first strike (different controller)
    let p2_chars = calculate_characteristics(&state, p2_red).unwrap();
    assert!(
        !p2_chars.keywords.contains(&KeywordAbility::FirstStrike),
        "P2 Red Bear should NOT have first strike"
    );
}

/// CR 604.2: "Artifacts you control have shroud" (Indomitable Archangel Metalcraft).
/// Only artifacts controlled by source's controller get shroud; creatures do not.
#[test]
fn test_artifacts_you_control_grants_shroud() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Indomitable Archangel", 4, 4))
        .object(ObjectSpec::artifact(p1(), "P1 Artifact"))
        .object(ObjectSpec::creature(p1(), "P1 Creature", 2, 2))
        .object(ObjectSpec::artifact(p2(), "P2 Artifact"))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Indomitable Archangel");
    let p1_artifact = find_on_battlefield(&state, "P1 Artifact");
    let p1_creature = find_on_battlefield(&state, "P1 Creature");
    let p2_artifact = find_on_battlefield(&state, "P2 Artifact");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::ArtifactsYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Shroud),
        is_cda: false,
        condition: None,
    });

    // P1's artifact gets shroud
    let art_chars = calculate_characteristics(&state, p1_artifact).unwrap();
    assert!(
        art_chars.keywords.contains(&KeywordAbility::Shroud),
        "P1 Artifact should have shroud"
    );

    // P1's creature does NOT get shroud (not an artifact)
    let cre_chars = calculate_characteristics(&state, p1_creature).unwrap();
    assert!(
        !cre_chars.keywords.contains(&KeywordAbility::Shroud),
        "P1 Creature should NOT have shroud"
    );

    // P2's artifact does NOT get shroud (different controller)
    let p2_chars = calculate_characteristics(&state, p2_artifact).unwrap();
    assert!(
        !p2_chars.keywords.contains(&KeywordAbility::Shroud),
        "P2 Artifact should NOT have shroud"
    );
}

/// CR 604.2: "Other non-Human creatures you control get +1/+1 and have undying."
/// (Mikaeus, the Unhallowed) — excludes Humans and excludes source.
#[test]
fn test_other_creatures_excluding_subtype_non_human() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Mikaeus", 5, 5)
                .with_subtypes(vec![SubType("Zombie".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "P1 Zombie", 2, 2)
                .with_subtypes(vec![SubType("Zombie".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "P1 Human", 2, 2)
                .with_subtypes(vec![SubType("Human".to_string())]),
        )
        .object(
            ObjectSpec::creature(p2(), "P2 Zombie", 2, 2)
                .with_subtypes(vec![SubType("Zombie".to_string())]),
        )
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Mikaeus");
    let p1_zombie = find_on_battlefield(&state, "P1 Zombie");
    let p1_human = find_on_battlefield(&state, "P1 Human");
    let p2_zombie = find_on_battlefield(&state, "P2 Zombie");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::OtherCreaturesYouControlExcludingSubtype(SubType(
            "Human".to_string(),
        )),
        modification: LayerModification::ModifyBoth(1),
        is_cda: false,
        condition: None,
    });

    // P1's non-Human, non-source creature gets +1/+1
    let zombie_chars = calculate_characteristics(&state, p1_zombie).unwrap();
    assert_eq!(zombie_chars.power, Some(3), "P1 Zombie should be 3/3");
    assert_eq!(zombie_chars.toughness, Some(3), "P1 Zombie should be 3/3");

    // Source creature does NOT benefit (excluded as "other")
    let src_chars = calculate_characteristics(&state, source_id).unwrap();
    assert_eq!(src_chars.power, Some(5), "Mikaeus should remain 5/5");

    // P1's Human does NOT get the bonus (excluded by subtype)
    let human_chars = calculate_characteristics(&state, p1_human).unwrap();
    assert_eq!(human_chars.power, Some(2), "P1 Human should remain 2/2");

    // P2's Zombie does NOT get the bonus (different controller)
    let p2_chars = calculate_characteristics(&state, p2_zombie).unwrap();
    assert_eq!(p2_chars.power, Some(2), "P2 Zombie should remain 2/2");
}

/// CR 604.2: "Non-Human creatures you control get +3/+3 until end of turn."
/// (Return of the Wildspeaker mode 2) — CreaturesYouControlExcludingSubtype
/// includes source (spell effect, not "other").
#[test]
fn test_creatures_excluding_subtype_spell_effect() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        // A creature acting as "source" for the spell effect
        .object(
            ObjectSpec::creature(p1(), "P1 Wolf", 3, 3)
                .with_subtypes(vec![SubType("Wolf".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "P1 Human", 2, 2)
                .with_subtypes(vec![SubType("Human".to_string())]),
        )
        .object(
            ObjectSpec::creature(p2(), "P2 Wolf", 3, 3)
                .with_subtypes(vec![SubType("Wolf".to_string())]),
        )
        .build()
        .unwrap();

    let p1_wolf = find_on_battlefield(&state, "P1 Wolf");
    let p1_human = find_on_battlefield(&state, "P1 Human");
    let p2_wolf = find_on_battlefield(&state, "P2 Wolf");

    // Spell effect: source is p1_wolf (the spell's controller's creature); includes itself
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(p1_wolf),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::CreaturesYouControlExcludingSubtype(SubType("Human".to_string())),
        modification: LayerModification::ModifyBoth(3),
        is_cda: false,
        condition: None,
    });

    // P1's Wolf gets +3/+3 (non-Human, not excluded)
    let wolf_chars = calculate_characteristics(&state, p1_wolf).unwrap();
    assert_eq!(wolf_chars.power, Some(6), "P1 Wolf should be 6/6 (+3/+3)");

    // P1's Human does NOT get the bonus (excluded by subtype)
    let human_chars = calculate_characteristics(&state, p1_human).unwrap();
    assert_eq!(human_chars.power, Some(2), "P1 Human should remain 2/2");

    // P2's Wolf does NOT get the bonus (different controller)
    let p2_chars = calculate_characteristics(&state, p2_wolf).unwrap();
    assert_eq!(p2_chars.power, Some(3), "P2 Wolf should remain 3/3");
}

/// CR 611.3a: "Attacking Vampires you control have deathtouch."
/// (Crossway Troublemakers) — only attacking Vampires get the keyword.
#[test]
fn test_attacking_creatures_with_subtype() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Crossway Troublemakers", 5, 5)
                .with_subtypes(vec![SubType("Vampire".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "Attacking Vampire", 2, 2)
                .with_subtypes(vec![SubType("Vampire".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "Sitting Vampire", 2, 2)
                .with_subtypes(vec![SubType("Vampire".to_string())]),
        )
        .object(
            ObjectSpec::creature(p2(), "P2 Attacking Vampire", 2, 2)
                .with_subtypes(vec![SubType("Vampire".to_string())]),
        )
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Crossway Troublemakers");
    let attacking_vamp = find_on_battlefield(&state, "Attacking Vampire");
    let sitting_vamp = find_on_battlefield(&state, "Sitting Vampire");
    let p2_attacking_vamp = find_on_battlefield(&state, "P2 Attacking Vampire");

    // Set up combat: P1's Attacking Vampire + P2's are attacking p2/p1 respectively
    state.combat = Some(CombatState {
        attacking_player: p1(),
        attackers: [
            (attacking_vamp, AttackTarget::Player(p2())),
            (p2_attacking_vamp, AttackTarget::Player(p1())),
        ]
        .into_iter()
        .collect(),
        blockers: im::OrdMap::new(),
        damage_assignment_order: im::OrdMap::new(),
        first_strike_participants: im::OrdSet::new(),
        defenders_declared: im::OrdSet::new(),
        forced_blocks: im::OrdMap::new(),
        enlist_pairings: Vec::new(),
        blocked_attackers: im::OrdSet::new(),
    });

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(SubType(
            "Vampire".to_string(),
        )),
        modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
        is_cda: false,
        condition: None,
    });

    // P1's attacking Vampire gets deathtouch
    let att_chars = calculate_characteristics(&state, attacking_vamp).unwrap();
    assert!(
        att_chars.keywords.contains(&KeywordAbility::Deathtouch),
        "Attacking P1 Vampire should have deathtouch"
    );

    // P1's non-attacking Vampire does NOT get deathtouch
    let sit_chars = calculate_characteristics(&state, sitting_vamp).unwrap();
    assert!(
        !sit_chars.keywords.contains(&KeywordAbility::Deathtouch),
        "Sitting P1 Vampire should NOT have deathtouch"
    );

    // P2's attacking Vampire does NOT get deathtouch (different controller)
    let p2_chars = calculate_characteristics(&state, p2_attacking_vamp).unwrap();
    assert!(
        !p2_chars.keywords.contains(&KeywordAbility::Deathtouch),
        "P2 Attacking Vampire should NOT have deathtouch (different controller)"
    );
}

/// CR 604.2: "Dragon creatures get +1/+1 until end of turn." (Bladewing the Risen)
/// AllCreaturesWithSubtype — no controller restriction, affects all players' Dragons.
#[test]
fn test_all_creatures_with_subtype_no_controller() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Bladewing", 4, 4)
                .with_subtypes(vec![SubType("Dragon".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "P1 Dragon", 5, 5)
                .with_subtypes(vec![SubType("Dragon".to_string())]),
        )
        .object(
            ObjectSpec::creature(p2(), "P2 Dragon", 3, 3)
                .with_subtypes(vec![SubType("Dragon".to_string())]),
        )
        .object(ObjectSpec::creature(p1(), "P1 Goblin", 1, 1))
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Bladewing");
    let p1_dragon = find_on_battlefield(&state, "P1 Dragon");
    let p2_dragon = find_on_battlefield(&state, "P2 Dragon");
    let p1_goblin = find_on_battlefield(&state, "P1 Goblin");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::AllCreaturesWithSubtype(SubType("Dragon".to_string())),
        modification: LayerModification::ModifyBoth(1),
        is_cda: false,
        condition: None,
    });

    // Source Bladewing gets +1/+1 (no controller restriction, no "other" exclusion)
    let src_chars = calculate_characteristics(&state, source_id).unwrap();
    assert_eq!(src_chars.power, Some(5), "Bladewing should be 5/5");

    // P1's other Dragon gets +1/+1
    let p1_chars = calculate_characteristics(&state, p1_dragon).unwrap();
    assert_eq!(p1_chars.power, Some(6), "P1 Dragon should be 6/6");

    // P2's Dragon also gets +1/+1 (no controller restriction!)
    let p2_chars = calculate_characteristics(&state, p2_dragon).unwrap();
    assert_eq!(p2_chars.power, Some(4), "P2 Dragon should be 4/4");

    // Goblin is NOT affected (wrong subtype)
    let goblin_chars = calculate_characteristics(&state, p1_goblin).unwrap();
    assert_eq!(goblin_chars.power, Some(1), "Goblin should remain 1/1");
}

/// CR 604.2: "Other Ninja and Rogue creatures you control get +1/+1."
/// (Silver-Fur Master) — OR semantics: Ninja or Rogue matches; source excluded.
#[test]
fn test_other_creatures_with_subtypes_or() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Silver-Fur Master", 2, 2)
                .with_subtypes(vec![SubType("Ninja".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "P1 Ninja", 2, 2)
                .with_subtypes(vec![SubType("Ninja".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1(), "P1 Rogue", 2, 2)
                .with_subtypes(vec![SubType("Rogue".to_string())]),
        )
        .object(ObjectSpec::creature(p1(), "P1 Goblin", 1, 1))
        .object(
            ObjectSpec::creature(p2(), "P2 Ninja", 2, 2)
                .with_subtypes(vec![SubType("Ninja".to_string())]),
        )
        .build()
        .unwrap();

    let source_id = find_on_battlefield(&state, "Silver-Fur Master");
    let p1_ninja = find_on_battlefield(&state, "P1 Ninja");
    let p1_rogue = find_on_battlefield(&state, "P1 Rogue");
    let p1_goblin = find_on_battlefield(&state, "P1 Goblin");
    let p2_ninja = find_on_battlefield(&state, "P2 Ninja");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(200),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::OtherCreaturesYouControlWithSubtypes(vec![
            SubType("Ninja".to_string()),
            SubType("Rogue".to_string()),
        ]),
        modification: LayerModification::ModifyBoth(1),
        is_cda: false,
        condition: None,
    });

    // P1's Ninja gets +1/+1 (Ninja subtype matches)
    let ninja_chars = calculate_characteristics(&state, p1_ninja).unwrap();
    assert_eq!(ninja_chars.power, Some(3), "P1 Ninja should be 3/3");

    // P1's Rogue also gets +1/+1 (Rogue subtype matches via OR)
    let rogue_chars = calculate_characteristics(&state, p1_rogue).unwrap();
    assert_eq!(rogue_chars.power, Some(3), "P1 Rogue should be 3/3");

    // Source (Silver-Fur Master) does NOT get the bonus (excluded as "other")
    let src_chars = calculate_characteristics(&state, source_id).unwrap();
    assert_eq!(
        src_chars.power,
        Some(2),
        "Silver-Fur Master should remain 2/2"
    );

    // P1's Goblin does NOT get the bonus (no matching subtype)
    let goblin_chars = calculate_characteristics(&state, p1_goblin).unwrap();
    assert_eq!(goblin_chars.power, Some(1), "Goblin should remain 1/1");

    // P2's Ninja does NOT get the bonus (different controller)
    let p2_chars = calculate_characteristics(&state, p2_ninja).unwrap();
    assert_eq!(p2_chars.power, Some(2), "P2 Ninja should remain 2/2");
}
