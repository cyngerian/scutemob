//! W3-LC: Layer correctness integration tests.
//!
//! These tests verify that engine code paths (triggers, mana, effects) use
//! layer-resolved characteristics instead of base characteristics. Each test
//! constructs a game state with continuous effects (Humility-style, animation,
//! anthem) and checks that the engine respects them.

use im::ordset;
use mtg_engine::{
    calculate_characteristics, CardType, Command, ContinuousEffect, Effect, EffectAmount,
    EffectDuration, EffectFilter, EffectId, EffectLayer, GameState, GameStateBuilder,
    KeywordAbility, LayerModification, ManaAbility, ManaColor, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, TriggerEvent, TriggeredAbilityDef,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// Helper: build a Humility-style RemoveAllAbilities continuous effect.
fn humility_effect(id: u64, timestamp: u64) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::RemoveAllAbilities,
        is_cda: false,
    }
}

/// Helper: build an animation effect (add Creature type to all lands).
fn animate_lands_effect(id: u64, timestamp: u64) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllLands,
        modification: LayerModification::AddCardTypes(ordset![CardType::Creature]),
        is_cda: false,
    }
}

/// Helper: build a Fervor-style haste-granting effect.
fn fervor_effect(id: u64, timestamp: u64) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
        is_cda: false,
    }
}

/// Helper: build anthem effects (+N/+N to all creatures).
/// Returns two effects: one for power, one for toughness.
fn anthem_effects(
    base_id: u64,
    timestamp: u64,
    power: i32,
    toughness: i32,
) -> (ContinuousEffect, ContinuousEffect) {
    (
        ContinuousEffect {
            id: EffectId(base_id),
            source: None,
            timestamp,
            layer: EffectLayer::PtModify,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::ModifyPower(power),
            is_cda: false,
        },
        ContinuousEffect {
            id: EffectId(base_id + 1),
            source: None,
            timestamp,
            layer: EffectLayer::PtModify,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllCreatures,
            modification: LayerModification::ModifyToughness(toughness),
            is_cda: false,
        },
    )
}

fn find_object_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

// ---------------------------------------------------------------------------
// Fix 1: PowerOf/ToughnessOf must use layer-resolved P/T (effects/mod.rs)
// CR 613.1, CR 613.4
// ---------------------------------------------------------------------------

/// CR 613.4: PowerOf reads layer-resolved power including anthem effects.
/// A 2/2 creature with a +2/+2 anthem should report power=4, not power=2.
/// This verifies that the resolve_amount → EffectAmount::PowerOf path in
/// effects/mod.rs would see the correct value through layers.
#[test]
fn test_w3lc_power_of_uses_layer_resolved_pt() {
    let (anthem_p, anthem_t) = anthem_effects(1, 10, 2, 2);
    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Test Creature", 2, 2))
        .add_continuous_effect(anthem_p)
        .add_continuous_effect(anthem_t)
        .build()
        .unwrap();

    let creature_id = find_object_by_name(&state, "Test Creature");

    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(
        chars.power,
        Some(4),
        "layer-resolved power should be 4 (2 base + 2 anthem)"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "layer-resolved toughness should be 4"
    );
}

/// CR 613.4: ToughnessOf reads layer-resolved toughness including Humility P/T set.
/// A 5/5 creature under Humility (1/1 set) should report toughness=1.
#[test]
fn test_w3lc_toughness_of_uses_layer_resolved_under_humility() {
    let humility_pt = ContinuousEffect {
        id: EffectId(2),
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
    };

    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Big Creature", 5, 5))
        .add_continuous_effect(humility_pt)
        .build()
        .unwrap();

    let creature_id = find_object_by_name(&state, "Big Creature");
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert_eq!(chars.power, Some(1), "Humility should set power to 1");
    assert_eq!(
        chars.toughness,
        Some(1),
        "Humility should set toughness to 1"
    );
}

// ---------------------------------------------------------------------------
// Fix 2: collect_triggers_for_event must use layer-resolved abilities
// CR 613.1f (Layer 6)
// ---------------------------------------------------------------------------

/// CR 613.1f: Humility removes all abilities including triggered abilities.
/// A creature with a triggered ability under Humility should NOT have that
/// trigger fire. This tests that collect_triggers_for_event in abilities.rs
/// uses layer-resolved triggered_abilities.
#[test]
fn test_w3lc_humility_suppresses_triggered_abilities() {
    let trigger = TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfEntersBattlefield,
        description: "When this enters, gain 3 life.".to_string(),
        effect: Some(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(3),
        }),
        intervening_if: None,
        etb_filter: None,
        targets: vec![],
    };

    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Trigger Bear", 2, 2).with_triggered_ability(trigger))
        .add_continuous_effect(humility_effect(1, 10))
        .build()
        .unwrap();

    let creature_id = find_object_by_name(&state, "Trigger Bear");

    // Layer-resolved characteristics should have empty triggered_abilities.
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert!(
        chars.triggered_abilities.is_empty(),
        "Humility should remove all triggered abilities; got {} triggers",
        chars.triggered_abilities.len()
    );

    // Base characteristics should still have the trigger (not removed, just suppressed).
    let obj = state.objects.get(&creature_id).unwrap();
    assert_eq!(
        obj.characteristics.triggered_abilities.len(),
        1,
        "base characteristics should still have the trigger"
    );
}

/// CR 613.1d + 613.1f: ETB filter creature_only check must use layer-resolved
/// card types. An animated land (layer 4: +Creature) entering the battlefield
/// should pass the creature_only ETB filter.
#[test]
fn test_w3lc_etb_filter_uses_layer_resolved_types() {
    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::land(p1(), "Animated Forest"))
        .add_continuous_effect(animate_lands_effect(1, 10))
        .build()
        .unwrap();

    let land_id = find_object_by_name(&state, "Animated Forest");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars.card_types.contains(&CardType::Creature),
        "animated land should be a creature in layer-resolved types"
    );
    assert!(
        chars.card_types.contains(&CardType::Land),
        "animated land should still be a land"
    );
}

// ---------------------------------------------------------------------------
// Fix 3: Summoning sickness must use layer-resolved types and keywords
// CR 302.6 / CR 702.10 / CR 613.1d / CR 613.1f
// ---------------------------------------------------------------------------

/// CR 302.6: An animated land (creature via layer 4) with summoning sickness
/// should not be able to tap for mana if it's a creature with no haste.
#[test]
fn test_w3lc_animated_land_summoning_sickness_blocks_mana() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::land(p1(), "Fresh Forest")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .add_continuous_effect(animate_lands_effect(1, 10))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1());
    state.turn.active_player = p1();

    let land_id = find_object_by_name(&state, "Fresh Forest");
    state
        .objects
        .get_mut(&land_id)
        .unwrap()
        .has_summoning_sickness = true;

    // Attempt to tap for mana should fail — animated land is a creature with
    // summoning sickness and no haste.
    let result = mtg_engine::process_command(
        state.clone(),
        Command::TapForMana {
            player: p1(),
            source: land_id,
            ability_index: 0,
        },
    );

    assert!(
        result.is_err(),
        "animated land with summoning sickness should not tap for mana; got: {:?}",
        result.as_ref().map(|(_, events)| events)
    );
}

/// CR 702.10 + CR 613.1f: Fervor grants haste to all creatures (layer 6).
/// An animated land with summoning sickness but haste from Fervor SHOULD be
/// able to tap for mana.
#[test]
fn test_w3lc_fervor_grants_haste_allows_mana_tap() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::land(p1(), "Hasty Forest")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .add_continuous_effect(animate_lands_effect(1, 10))
        .add_continuous_effect(fervor_effect(2, 11))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1());
    state.turn.active_player = p1();

    let land_id = find_object_by_name(&state, "Hasty Forest");
    state
        .objects
        .get_mut(&land_id)
        .unwrap()
        .has_summoning_sickness = true;

    // Should succeed — Fervor grants haste via layer 6.
    let result = mtg_engine::process_command(
        state.clone(),
        Command::TapForMana {
            player: p1(),
            source: land_id,
            ability_index: 0,
        },
    );

    assert!(
        result.is_ok(),
        "animated land with Fervor-granted haste should tap for mana; error: {:?}",
        result.err()
    );
}

/// CR 302.6: A non-animated land (not a creature) with summoning sickness
/// should still be able to tap for mana. Summoning sickness only affects
/// creatures.
#[test]
fn test_w3lc_non_animated_land_ignores_summoning_sickness() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::land(p1(), "Normal Forest")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1());
    state.turn.active_player = p1();

    let land_id = find_object_by_name(&state, "Normal Forest");
    state
        .objects
        .get_mut(&land_id)
        .unwrap()
        .has_summoning_sickness = true;

    // Should succeed — land is not a creature, summoning sickness doesn't apply.
    let result = mtg_engine::process_command(
        state.clone(),
        Command::TapForMana {
            player: p1(),
            source: land_id,
            ability_index: 0,
        },
    );

    assert!(
        result.is_ok(),
        "non-creature land should ignore summoning sickness; error: {:?}",
        result.err()
    );
}

/// CR 613.1d: Sacrifice creature check in mana ability uses layer-resolved types.
/// An animated artifact (creature via layer 4) that sacrifices itself for mana
/// should emit CreatureDied, not PermanentDestroyed.
#[test]
fn test_w3lc_sacrifice_mana_uses_layer_resolved_types() {
    // Animate all permanents into creatures.
    let animate_all = ContinuousEffect {
        id: EffectId(1),
        source: None,
        timestamp: 10,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllPermanents,
        modification: LayerModification::AddCardTypes(ordset![CardType::Creature]),
        is_cda: false,
    };

    let mut produces = im::OrdMap::new();
    produces.insert(ManaColor::Colorless, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::artifact(p1(), "Mana Rock").with_mana_ability(ManaAbility {
                produces,
                requires_tap: false,
                sacrifice_self: true,
                any_color: false,
                damage_to_controller: 0,
            }),
        )
        .add_continuous_effect(animate_all)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1());
    state.turn.active_player = p1();

    let rock_id = find_object_by_name(&state, "Mana Rock");

    // Verify it's a creature through layers.
    let chars = calculate_characteristics(&state, rock_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "animated artifact should be a creature"
    );

    let (_, events) = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: p1(),
            source: rock_id,
            ability_index: 0,
        },
    )
    .expect("sacrifice-for-mana should succeed");

    // Should emit CreatureDied (not just PermanentDestroyed) because the
    // animated artifact is a creature per layer 4.
    let has_creature_died = events
        .iter()
        .any(|e| matches!(e, mtg_engine::GameEvent::CreatureDied { .. }));
    assert!(
        has_creature_died,
        "animated artifact sacrifice should emit CreatureDied; events: {:?}",
        events
    );
}
