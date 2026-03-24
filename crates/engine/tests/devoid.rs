//! Devoid ability tests (CR 702.114).
//!
//! Devoid is a characteristic-defining ability: "This object is colorless."
//! It applies in Layer 5 (ColorChange) as a CDA, before any non-CDA Layer 5 effects (CR 613.3).
//! It functions in all zones, not just the battlefield (CR 604.3).

use im::OrdSet;
use mtg_engine::{
    calculate_characteristics, Color, ContinuousEffect, EffectDuration, EffectFilter, EffectId,
    EffectLayer, GameStateBuilder, KeywordAbility, LayerModification, ObjectSpec, PlayerId,
    ProtectionQuality, ZoneId,
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
// Test 1: A Devoid creature is colorless (CR 702.114a)
// ---------------------------------------------------------------------------

/// CR 702.114a — "Devoid" means "This object is colorless."
/// A creature with Devoid and a colored mana cost should have an empty colors set
/// after calculate_characteristics, even though the base characteristic has red.
#[test]
fn test_devoid_creature_is_colorless() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Devoid Creature", 3, 2)
                .with_keyword(KeywordAbility::Devoid)
                .with_colors(vec![Color::Red]),
        )
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let chars = calculate_characteristics(&state, id).unwrap();

    assert!(
        chars.colors.is_empty(),
        "Devoid creature should be colorless (empty colors set), got: {:?}",
        chars.colors
    );
}

// ---------------------------------------------------------------------------
// Test 2: Base characteristics still have mana-cost-derived colors (layer-level check)
// ---------------------------------------------------------------------------

/// CR 702.114a: Devoid operates at the layer level, not at definition time.
/// The printed (base) characteristics on the object should still carry the mana-cost
/// colors — Devoid only changes the effective color via Layer 5, it does not modify
/// the stored object characteristics.
#[test]
fn test_devoid_base_colors_unmodified() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Devoid Creature", 3, 2)
                .with_keyword(KeywordAbility::Devoid)
                .with_colors(vec![Color::Red]),
        )
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    // The stored (base) colors should still contain Red
    let base_colors = &state.objects.get(&id).unwrap().characteristics.colors;
    assert!(
        base_colors.contains(&Color::Red),
        "Base characteristics should retain mana-cost color Red before layer calculation"
    );

    // But the effective colors (after layers) should be empty
    let effective_chars = calculate_characteristics(&state, id).unwrap();
    assert!(
        effective_chars.colors.is_empty(),
        "Effective colors after Layer 5 CDA should be empty (colorless)"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Devoid + RemoveAllAbilities still colorless (CR 613.3 + ruling 2015-08-25)
// ---------------------------------------------------------------------------

/// CR 613.3: CDAs apply first within each layer. Layer 5 (ColorChange CDA) runs before
/// Layer 6 (ability removal). A Devoid creature under a RemoveAllAbilities effect
/// should still be colorless, because the Devoid CDA already ran in Layer 5.
///
/// Source: Vile Aggregate ruling 2015-08-25: "If a card loses devoid, it will still
/// be colorless." The keyword is gone, but Layer 5 already applied.
#[test]
fn test_devoid_lose_all_abilities_still_colorless() {
    let remove_all = indef_effect(
        1,
        EffectLayer::Ability,
        EffectFilter::AllCreatures,
        LayerModification::RemoveAllAbilities,
    );

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Devoid Creature", 3, 2)
                .with_keyword(KeywordAbility::Devoid)
                .with_colors(vec![Color::Red]),
        )
        .add_continuous_effect(remove_all)
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let chars = calculate_characteristics(&state, id).unwrap();

    // Layer 6 removed the keyword
    assert!(
        !chars.keywords.contains(&KeywordAbility::Devoid),
        "RemoveAllAbilities in Layer 6 should remove the Devoid keyword"
    );

    // But Layer 5 already ran: the object should still be colorless
    assert!(
        chars.colors.is_empty(),
        "Devoid CDA (Layer 5) ran before RemoveAllAbilities (Layer 6) — creature must still be colorless"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Devoid + color-adding effect — the added color wins (CR 613.3)
// ---------------------------------------------------------------------------

/// CR 613.3 + Vile Aggregate ruling 2015-08-25:
/// "Other cards and abilities can give a card with devoid color. If that happens,
/// it's just the new color, not that color and colorless."
///
/// CDA clears colors first (Layer 5, before non-CDA effects). Then a non-CDA
/// AddColors effect in Layer 5 (timestamp 10, applied after the CDA) gives the
/// object that color. Result: the Devoid card has only the added color.
#[test]
fn test_devoid_color_adding_effect_overrides() {
    let add_blue = indef_effect(
        1,
        EffectLayer::ColorChange,
        EffectFilter::AllCreatures,
        LayerModification::AddColors(OrdSet::from(vec![Color::Blue])),
    );

    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Devoid Creature", 3, 2)
                .with_keyword(KeywordAbility::Devoid)
                .with_colors(vec![Color::Red]),
        )
        .add_continuous_effect(add_blue)
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let chars = calculate_characteristics(&state, id).unwrap();

    // AddColors applied after the Devoid CDA: creature should now be Blue
    assert!(
        chars.colors.contains(&Color::Blue),
        "AddColors (non-CDA Layer 5) should give the Devoid creature Blue"
    );
    // Red from mana cost was cleared by Devoid CDA and not re-added
    assert!(
        !chars.colors.contains(&Color::Red),
        "Red from mana cost should not appear — Devoid cleared it, and only Blue was added"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Devoid works in graveyard (CR 604.3)
// ---------------------------------------------------------------------------

/// CR 604.3: Characteristic-defining abilities function in all zones.
/// CR 702.114a: Devoid is a CDA.
///
/// A Devoid card in the graveyard should be colorless after calculate_characteristics.
/// This matters for effects like "target colorless card in a graveyard."
#[test]
fn test_devoid_works_in_graveyard() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Devoid Creature", 3, 2)
                .with_keyword(KeywordAbility::Devoid)
                .with_colors(vec![Color::Red])
                .in_zone(ZoneId::Graveyard(p1())),
        )
        .build()
        .unwrap();

    // Find the object in the graveyard
    let id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Graveyard(p1()) && obj.characteristics.name == "Devoid Creature"
        })
        .map(|(id, _)| *id)
        .expect("Devoid creature should be in p1's graveyard");

    let chars = calculate_characteristics(&state, id).unwrap();

    assert!(
        chars.colors.is_empty(),
        "Devoid CDA should make the creature colorless in graveyard (CR 604.3)"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Devoid works in hand (CR 604.3)
// ---------------------------------------------------------------------------

/// CR 604.3: Characteristic-defining abilities function in all zones.
/// A Devoid card in hand should be colorless after calculate_characteristics.
#[test]
fn test_devoid_works_in_hand() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Devoid Creature", 3, 2)
                .with_keyword(KeywordAbility::Devoid)
                .with_colors(vec![Color::Red])
                .in_zone(ZoneId::Hand(p1())),
        )
        .build()
        .unwrap();

    // Find the object in hand
    let id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Hand(p1()) && obj.characteristics.name == "Devoid Creature"
        })
        .map(|(id, _)| *id)
        .expect("Devoid creature should be in p1's hand");

    let chars = calculate_characteristics(&state, id).unwrap();

    assert!(
        chars.colors.is_empty(),
        "Devoid CDA should make the creature colorless in hand (CR 604.3)"
    );
}

// ---------------------------------------------------------------------------
// Test 7: Non-Devoid creature retains its colors (negative test)
// ---------------------------------------------------------------------------

/// Negative test: a creature without Devoid should NOT have its colors cleared.
/// The inline CDA check in calculate_characteristics must be conditional on having
/// the Devoid keyword — other creatures must not be affected.
#[test]
fn test_non_devoid_creature_retains_colors() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(ObjectSpec::creature(p1(), "Normal Creature", 3, 2).with_colors(vec![Color::Red]))
        .build()
        .unwrap();
    let id = battlefield_id(&state);

    let chars = calculate_characteristics(&state, id).unwrap();

    assert!(
        chars.colors.contains(&Color::Red),
        "Non-Devoid creature should retain its Red color"
    );
    assert_eq!(
        chars.colors.len(),
        1,
        "Non-Devoid creature should have exactly its printed colors"
    );
}

// ---------------------------------------------------------------------------
// Test 8: Protection from a color does NOT protect from a Devoid source
// ---------------------------------------------------------------------------

/// CR 702.16a + CR 702.114a:
/// Protection from red checks whether the SOURCE has color Red.
/// A Devoid creature with {R} in its mana cost is colorless after Layer 5.
/// Therefore, "protection from red" does NOT protect from a Devoid source.
///
/// We verify this by checking that calculate_characteristics returns an empty
/// color set for the Devoid source — the same value the protection check uses
/// via `source_chars.colors`.
#[test]
fn test_devoid_protection_from_color_does_not_match() {
    // The Devoid creature is our "source" — it has Red in mana cost but Devoid makes it colorless
    let state = GameStateBuilder::new()
        .add_player(p1())
        .object(
            ObjectSpec::creature(p1(), "Devoid Attacker", 3, 2)
                .with_keyword(KeywordAbility::Devoid)
                .with_colors(vec![Color::Red]),
        )
        .build()
        .unwrap();
    let source_id = battlefield_id(&state);

    let source_chars = calculate_characteristics(&state, source_id).unwrap();

    // Protection from Red checks: source_chars.colors.contains(&Color::Red)
    let quality = ProtectionQuality::FromColor(Color::Red);
    let matches = match &quality {
        ProtectionQuality::FromColor(c) => source_chars.colors.contains(c),
        _ => false,
    };
    assert!(
        !matches,
        "Devoid source should NOT match ProtectionQuality::FromColor(Red) — \
         Devoid makes it colorless, so protection from red does not apply"
    );

    // Verify the source is truly colorless
    assert!(
        source_chars.colors.is_empty(),
        "Devoid source should have empty colors after Layer 5 CDA"
    );
}
