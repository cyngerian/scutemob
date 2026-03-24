//! Changeling ability tests (CR 702.73).
//!
//! Changeling is a characteristic-defining ability: "This object is every creature type."
//! It applies in Layer 4 (TypeChange) as a CDA, before any non-CDA Layer 4 effects (CR 613.3).
//! It functions in all zones, not just the battlefield (CR 604.3).

use im::{ordset, OrdSet};
use mtg_engine::{
    calculate_characteristics, CardType, ContinuousEffect, EffectDuration, EffectFilter, EffectId,
    EffectLayer, GameStateBuilder, KeywordAbility, LayerModification, ObjectSpec, PlayerId,
    ProtectionQuality, SubType, ZoneId, ALL_CREATURE_TYPES,
};

fn p1() -> PlayerId {
    PlayerId(1)
}

/// Build a continuous effect with `Indefinite` duration (no source required).
fn indef_effect(
    id: u64,
    layer: EffectLayer,
    filter: EffectFilter,
    modification: LayerModification,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp: 10,
        layer,
        duration: EffectDuration::Indefinite,
        filter,
        modification,
        is_cda: false,
        condition: None,
    }
}

// ---------------------------------------------------------------------------
// Helper: get the first (and only) object ID on the battlefield
// ---------------------------------------------------------------------------

fn battlefield_id(state: &mtg_engine::GameState) -> mtg_engine::ObjectId {
    *state
        .zones
        .get(&ZoneId::Battlefield)
        .expect("battlefield zone must exist")
        .object_ids()
        .first()
        .expect("at least one object on battlefield")
}

// ---------------------------------------------------------------------------
// Test 1: Changeling creature has all creature types on the battlefield
// ---------------------------------------------------------------------------

/// CR 702.73a — "Changeling" means "This object is every creature type."
/// A creature with Changeling should have all 290+ creature types from CR 205.3m
/// after calculate_characteristics.
#[test]
fn test_changeling_has_all_creature_types() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Shapeshifter Creature", 1, 1)
                .with_keyword(KeywordAbility::Changeling),
        )
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let chars = calculate_characteristics(&state, id).unwrap();

    // Must contain representative types from across the full list
    for name in &[
        "Goblin",
        "Elf",
        "Human",
        "Dragon",
        "Sliver",
        "Shapeshifter",
        "Zombie",
    ] {
        assert!(
            chars.subtypes.contains(&SubType(name.to_string())),
            "Changeling creature should be a {name}"
        );
    }

    // Must have the complete list — at least as many types as ALL_CREATURE_TYPES
    assert!(
        chars.subtypes.len() >= ALL_CREATURE_TYPES.len(),
        "Changeling creature should have at least {} creature types, got {}",
        ALL_CREATURE_TYPES.len(),
        chars.subtypes.len()
    );
}

// ---------------------------------------------------------------------------
// Test 2: Changeling + RemoveAllAbilities keeps types (CR 613.3)
// ---------------------------------------------------------------------------

/// CR 613.3 — CDAs apply first within each layer; Layer 4 (type-change) runs before
/// Layer 6 (ability removal). A creature that gains Changeling in Layer 4 and then
/// loses all abilities in Layer 6 (e.g., via Humility or Dress Down) retains all
/// creature types because the Layer 4 CDA already ran.
///
/// Source: Maskwood Nexus ruling 2021-02-05: "If an effect causes a creature with
/// changeling to lose all abilities, it will remain all creature types, even though
/// it will no longer have changeling."
#[test]
fn test_changeling_lose_all_abilities_keeps_types() {
    let remove_all = indef_effect(
        1,
        EffectLayer::Ability,
        EffectFilter::AllCreatures,
        LayerModification::RemoveAllAbilities,
    );

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Shapeshifter Creature", 1, 1)
                .with_keyword(KeywordAbility::Changeling),
        )
        .add_continuous_effect(remove_all)
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let chars = calculate_characteristics(&state, id).unwrap();

    // Layer 6 removed the keyword — Changeling should no longer be in keywords
    assert!(
        !chars.keywords.contains(&KeywordAbility::Changeling),
        "RemoveAllAbilities in Layer 6 should remove the Changeling keyword"
    );

    // But Layer 4 already ran: all creature types should still be present
    assert!(
        chars.subtypes.contains(&SubType("Goblin".to_string())),
        "Creature types from Changeling CDA (Layer 4) must survive Layer 6 ability removal"
    );
    assert!(
        chars.subtypes.contains(&SubType("Elf".to_string())),
        "Creature types from Changeling CDA (Layer 4) must survive Layer 6 ability removal"
    );
    assert!(
        chars.subtypes.len() >= ALL_CREATURE_TYPES.len(),
        "All creature types must still be present after RemoveAllAbilities"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Changeling overridden by SetTypeLine in Layer 4 (CR 613.3)
// ---------------------------------------------------------------------------

/// CR 613.3 + Maskwood Nexus ruling 2021-02-05:
/// "If an effect causes a creature with changeling to become a new creature type,
/// it will be only that new creature type."
///
/// CDAs apply first within Layer 4. A non-CDA SetTypeLine (timestamp 10, later than
/// CDA-first ordering) replaces the full type line, leaving only its specified subtypes.
#[test]
fn test_changeling_overridden_by_set_type_line() {
    let set_type = indef_effect(
        1,
        EffectLayer::TypeChange,
        EffectFilter::AllCreatures,
        LayerModification::SetTypeLine {
            supertypes: OrdSet::new(),
            card_types: ordset![CardType::Creature],
            subtypes: ordset![SubType("Phyrexian".to_string())],
        },
    );

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Shapeshifter Creature", 1, 1)
                .with_keyword(KeywordAbility::Changeling),
        )
        .add_continuous_effect(set_type)
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let chars = calculate_characteristics(&state, id).unwrap();

    // SetTypeLine after CDA within Layer 4 should replace subtypes entirely
    assert!(
        chars.subtypes.contains(&SubType("Phyrexian".to_string())),
        "SetTypeLine subtype should be present"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Goblin".to_string())),
        "Changeling subtypes should be overridden by SetTypeLine (non-CDA applies after CDA within Layer 4)"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Elf".to_string())),
        "Changeling subtypes should be overridden by SetTypeLine"
    );
    assert_eq!(
        chars.subtypes.len(),
        1,
        "Only the SetTypeLine subtype should remain after override"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Changeling and protection from subtype (CR 702.16a + 702.73a)
// ---------------------------------------------------------------------------

/// CR 702.16a: Protection from X means "Can't be damaged, enchanted/equipped,
/// blocked by, or targeted by anything that is X."
/// CR 702.73a: A Changeling creature is every creature type, including Goblin.
///
/// Therefore: a creature with "protection from Goblins" IS protected from a
/// Changeling source, because the Changeling source IS a Goblin per CR 702.73a.
/// Verified by checking that calculate_characteristics populates "Goblin" in
/// the Changeling source's subtypes — the protection logic uses subtypes directly.
#[test]
fn test_changeling_has_goblin_subtype_for_protection_check() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Changeling Attacker", 2, 2)
                .with_keyword(KeywordAbility::Changeling),
        )
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let source_chars = calculate_characteristics(&state, id).unwrap();

    // Protection from Goblins would call: source_chars.subtypes.contains(&SubType("Goblin"))
    let goblin = SubType("Goblin".to_string());
    let quality = ProtectionQuality::FromSubType(goblin.clone());
    let matches = match &quality {
        ProtectionQuality::FromSubType(st) => source_chars.subtypes.contains(st),
        _ => false,
    };
    assert!(
        matches,
        "Changeling creature should match ProtectionQuality::FromSubType(Goblin) \
         because Changeling is every creature type including Goblin"
    );

    // Also check Elf, Human, Dragon to confirm it's not just Goblin
    for name in &["Elf", "Human", "Dragon"] {
        let st = SubType(name.to_string());
        assert!(
            source_chars.subtypes.contains(&st),
            "Changeling should also be a {name} for protection matching"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 5: Changeling CDA works in graveyard (CR 604.3)
// ---------------------------------------------------------------------------

/// CR 604.3: Characteristic-defining abilities function in all zones.
/// CR 702.73a: Changeling is a CDA.
///
/// A Changeling card in the graveyard should still have all creature types when
/// calculate_characteristics is called on it. This matters for effects like
/// "Return target Goblin card from your graveyard to your hand."
#[test]
fn test_changeling_works_in_graveyard() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Changeling Creature", 1, 1)
                .with_keyword(KeywordAbility::Changeling)
                .in_zone(ZoneId::Graveyard(p1())),
        )
        .build()
        .unwrap();

    // Find the object in the graveyard
    let id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Graveyard(p1()) && obj.characteristics.name == "Changeling Creature"
        })
        .map(|(id, _)| *id)
        .expect("Changeling creature should be in p1's graveyard");

    let chars = calculate_characteristics(&state, id).unwrap();

    assert!(
        chars.subtypes.contains(&SubType("Goblin".to_string())),
        "Changeling CDA should work in graveyard — creature should be a Goblin in the GY"
    );
    assert!(
        chars.subtypes.contains(&SubType("Elf".to_string())),
        "Changeling CDA should work in graveyard — creature should be an Elf in the GY"
    );
    assert!(
        chars.subtypes.len() >= ALL_CREATURE_TYPES.len(),
        "All creature types should be present in graveyard via CDA (CR 604.3)"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Non-Changeling creature has only printed subtypes (negative test)
// ---------------------------------------------------------------------------

/// Negative test: a creature without Changeling should NOT gain all creature types.
/// The inline CDA check in calculate_characteristics must be conditional on having
/// the Changeling keyword — other creatures must not be affected.
#[test]
fn test_non_changeling_creature_has_only_printed_subtypes() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Normal Goblin", 1, 1)
                .with_subtypes(vec![SubType("Goblin".to_string())]),
        )
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let chars = calculate_characteristics(&state, id).unwrap();

    // Only the printed Goblin subtype — no other creature types
    assert_eq!(
        chars.subtypes.len(),
        1,
        "Non-Changeling creature should have only its printed subtypes"
    );
    assert!(
        chars.subtypes.contains(&SubType("Goblin".to_string())),
        "Goblin subtype should be present"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Elf".to_string())),
        "Non-Changeling Goblin should NOT be an Elf"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Dragon".to_string())),
        "Non-Changeling Goblin should NOT be a Dragon"
    );
}

// ---------------------------------------------------------------------------
// Test 7: AddAllCreatureTypes LayerModification via ContinuousEffect
// ---------------------------------------------------------------------------

/// CR 702.73a: The `LayerModification::AddAllCreatureTypes` variant can also be
/// used directly via a `ContinuousEffect` (for Maskwood Nexus-style effects).
/// Verify that a non-Changeling creature gains all creature types through a
/// `AddAllCreatureTypes` Layer 4 continuous effect.
#[test]
fn test_add_all_creature_types_modification() {
    let add_all = indef_effect(
        1,
        EffectLayer::TypeChange,
        EffectFilter::AllCreatures,
        LayerModification::AddAllCreatureTypes,
    );

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Regular Warrior", 2, 2))
        .add_continuous_effect(add_all)
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let chars = calculate_characteristics(&state, id).unwrap();

    assert!(
        chars.subtypes.contains(&SubType("Goblin".to_string())),
        "AddAllCreatureTypes effect should add Goblin subtype"
    );
    assert!(
        chars.subtypes.contains(&SubType("Elf".to_string())),
        "AddAllCreatureTypes effect should add Elf subtype"
    );
    assert!(
        chars.subtypes.len() >= ALL_CREATURE_TYPES.len(),
        "AddAllCreatureTypes should add all creature types from CR 205.3m"
    );
}
