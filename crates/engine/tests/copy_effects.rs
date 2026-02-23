//! Copy effect tests: Layer 1 copiable values and clone chain (CR 707).
//!
//! Session 7 of M9.4 implements `LayerModification::CopyOf` in the layer system,
//! backed by `rules::copy::get_copiable_values` and its recursive clone-chain
//! resolution (CR 707.3).

use mtg_engine::{
    calculate_characteristics, CardType, ContinuousEffect, EffectDuration, EffectFilter, EffectId,
    EffectLayer, GameStateBuilder, KeywordAbility, LayerModification, ObjectId, ObjectSpec,
    PlayerId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}

/// Build a `ContinuousEffect` with Layer 1 Copy semantics.
fn copy_effect_of(
    id: u64,
    copier_id: ObjectId,
    source_id: ObjectId,
    timestamp: u64,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: Some(copier_id),
        timestamp,
        layer: EffectLayer::Copy,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(copier_id),
        modification: LayerModification::CopyOf(source_id),
        is_cda: false,
    }
}

// ── CR 707.2: Clone copies a Bear ────────────────────────────────────────────

/// CR 707.2 — A Clone enters copying a Grizzly Bears.
/// The Clone's calculated characteristics must equal the Bear's: 2/2 creature.
/// The Clone's printed characteristics (e.g., "Clone: 0/0 Creature") are replaced
/// by the Bear's copiable values.
#[test]
fn test_clone_copies_bear() {
    let p = p1();

    // The Bear: a 2/2 creature with subtype "Bear"
    let bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);

    // The Clone: printed as a 0/0 creature (its copiable values will be replaced)
    let clone_spec = ObjectSpec::creature(p, "Clone", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone")
        .map(|(id, _)| *id)
        .unwrap();

    // Apply Layer 1 copy effect: Clone copies the Bear.
    let copy_eff = copy_effect_of(100, clone_id, bear_id, 50);
    state.continuous_effects.push_back(copy_eff);

    let chars = calculate_characteristics(&state, clone_id).unwrap();

    // The Clone should now have the Bear's name and stats.
    assert_eq!(chars.name, "Grizzly Bears", "Clone should have Bear's name");
    assert_eq!(chars.power, Some(2), "Clone should have Bear's power (2)");
    assert_eq!(
        chars.toughness,
        Some(2),
        "Clone should have Bear's toughness (2)"
    );
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "Clone should be a creature"
    );
}

// ── CR 707.3: Clone copying a Clone (clone chain) ────────────────────────────

/// CR 707.3 / CC#5 — A Clone-A copies Clone-B, and Clone-B is copying a Bear.
/// Clone-A must end up with the Bear's characteristics, NOT Clone-B's printed
/// characteristics.  This is the core of the clone chain rule.
#[test]
fn test_clone_copies_clone_chain() {
    let p = p1();

    // The original Bear: 2/2 Green creature with Flying keyword.
    let mut bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);
    bear_spec = bear_spec.with_keyword(KeywordAbility::Flying);

    // Clone-B: enters copying the Bear (its copiable values → Bear's values).
    let clone_b_spec = ObjectSpec::creature(p, "Clone-B", 0, 0);

    // Clone-A: enters copying Clone-B.
    let clone_a_spec = ObjectSpec::creature(p, "Clone-A", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_b_spec)
        .object(clone_a_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_b_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone-B")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_a_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone-A")
        .map(|(id, _)| *id)
        .unwrap();

    // Clone-B copies the Bear (timestamp 10).
    let eff_b = copy_effect_of(101, clone_b_id, bear_id, 10);
    // Clone-A copies Clone-B (timestamp 20, applied after eff_b).
    let eff_a = copy_effect_of(102, clone_a_id, clone_b_id, 20);

    state.continuous_effects.push_back(eff_b);
    state.continuous_effects.push_back(eff_a);

    let chars_a = calculate_characteristics(&state, clone_a_id).unwrap();

    // CR 707.3: Clone-A copying Clone-B sees Clone-B's copiable values AFTER
    // the copy effect on Clone-B is applied. Clone-B's copiable values = Bear's values.
    // So Clone-A should look like the Bear, NOT like Clone-B's printed form (0/0).
    assert_eq!(
        chars_a.name, "Grizzly Bears",
        "Clone-A should have Bear's name (not Clone-B's)"
    );
    assert_eq!(
        chars_a.power,
        Some(2),
        "Clone-A should have Bear's power via clone chain"
    );
    assert_eq!(
        chars_a.toughness,
        Some(2),
        "Clone-A should have Bear's toughness via clone chain"
    );
    assert!(
        chars_a.keywords.contains(&KeywordAbility::Flying),
        "Clone-A should inherit Flying from the Bear via clone chain"
    );

    // Also verify Clone-B independently looks like the Bear.
    let chars_b = calculate_characteristics(&state, clone_b_id).unwrap();
    assert_eq!(chars_b.name, "Grizzly Bears");
    assert_eq!(chars_b.power, Some(2));
}

// ── Layer 1 applies before other layers ──────────────────────────────────────

/// CR 613.1a — Layer 1 (Copy) applies before Layer 6 (Ability) and Layer 7 (P/T).
/// A Clone copies a Bear (Layer 1).  A separate +2/+2 continuous effect (Layer 7c)
/// applies to the Clone after the copy resolves.  The final characteristics must be
/// the Bear's base stats plus the modification — NOT the Clone's printed stats.
#[test]
fn test_copy_effect_layer_1_applies_before_other_layers() {
    use mtg_engine::{ContinuousEffect, EffectDuration, EffectId, EffectLayer, LayerModification};

    let p = p1();

    // Bear: 2/2
    let bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);
    // Clone: printed as 0/0
    let clone_spec = ObjectSpec::creature(p, "Clone", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone")
        .map(|(id, _)| *id)
        .unwrap();

    // Layer 1: Clone copies the Bear (timestamp 10).
    let copy_eff = copy_effect_of(200, clone_id, bear_id, 10);

    // Layer 7c: +2/+2 on the Clone (timestamp 20, applied after Layer 1).
    let pump_eff = ContinuousEffect {
        id: EffectId(201),
        source: None,
        timestamp: 20,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::SingleObject(clone_id),
        modification: LayerModification::ModifyBoth(2),
        is_cda: false,
    };

    state.continuous_effects.push_back(copy_eff);
    state.continuous_effects.push_back(pump_eff);

    let chars = calculate_characteristics(&state, clone_id).unwrap();

    // Layer 1 applies first: Clone gets Bear's 2/2 base.
    // Layer 7c applies after: +2/+2 brings it to 4/4.
    assert_eq!(
        chars.power,
        Some(4),
        "Layer 1 (copy) + Layer 7c (+2) should give 4 power"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "Layer 1 (copy) + Layer 7c (+2) should give 4 toughness"
    );
    assert_eq!(
        chars.name, "Grizzly Bears",
        "Name should be from the Bear (Layer 1 applied)"
    );
}

// ── Copy does not copy counters or status ─────────────────────────────────────

/// CR 707.2 — Copy effects copy COPIABLE VALUES only.
/// Counters, damage marked, status (tapped/summoning sickness), zone, and controller
/// are NOT copied (CR 707.2 exclusions).  This test verifies those exclusions.
#[test]
fn test_copy_does_not_copy_counters_or_status() {
    let p = p1();

    // Bear: 2/2 with two +1/+1 counters (should NOT be copied).
    let bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);

    // Clone: 0/0 entering copying the Bear.
    let clone_spec = ObjectSpec::creature(p, "Clone", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone")
        .map(|(id, _)| *id)
        .unwrap();

    // Add +1/+1 counters to the Bear (should not be copied to Clone).
    {
        use mtg_engine::CounterType;
        let bear = state.objects.get_mut(&bear_id).unwrap();
        bear.counters.insert(CounterType::PlusOnePlusOne, 2);
    }

    // Tap the Bear (status should not be copied).
    {
        let bear = state.objects.get_mut(&bear_id).unwrap();
        bear.status.tapped = true;
    }

    // Apply Layer 1 copy effect.
    let copy_eff = copy_effect_of(300, clone_id, bear_id, 10);
    state.continuous_effects.push_back(copy_eff);

    // Verify Clone characteristics after copy.
    let clone_chars = calculate_characteristics(&state, clone_id).unwrap();

    // CR 707.2: the Clone copies Bear's PRINTED values, not the counters.
    // The Bear's printed P/T is 2/2. Counters are applied in Layer 7c to the Bear,
    // but the Clone's copy effect copies only the printed/copiable values (2/2),
    // not the counter-adjusted value (4/4).
    assert_eq!(
        clone_chars.name, "Grizzly Bears",
        "Name should be copied (copiable)"
    );
    assert_eq!(
        clone_chars.power,
        Some(2),
        "Power should be Bear's printed value (counters are NOT copiable)"
    );
    assert_eq!(
        clone_chars.toughness,
        Some(2),
        "Toughness should be Bear's printed value (counters are NOT copiable)"
    );

    // The Clone's own status (not tapped) should be independent of the Bear's status.
    let clone_obj = state.objects.get(&clone_id).unwrap();
    assert!(
        !clone_obj.status.tapped,
        "Clone's tapped status should be independent of Bear's (status is NOT copiable)"
    );

    // The Bear should still have its counters.
    let bear_chars = calculate_characteristics(&state, bear_id).unwrap();
    assert_eq!(
        bear_chars.power,
        Some(4),
        "Bear should still have its counter-boosted power (4)"
    );
}
