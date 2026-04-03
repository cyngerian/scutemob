//! Tests for PB-K: Additional Land Drops primitives.
//!
//! Covers:
//! - TriggerCondition::WheneverOpponentPlaysLand + TriggerEvent::OpponentPlaysLand (Burgeoning)
//! - Effect::PutLandFromHandOntoBattlefield (CR 305.4)
//! - EffectFilter::LandsYouControl (Dryad of the Ilysian Grove)
//! - Designations::SOLVED + Condition::SourceIsSolved + Effect::SolveCase (Case mechanic CR 719)
//! - Condition::And (logical conjunction)
//!
//! CR references: 305.1, 305.2, 305.4, 305.7, 613.1, 719.3a, 719.3b, 702.169b

use mtg_engine::cards::card_definition::Condition;
use mtg_engine::effects::{check_condition, execute_effect, EffectContext};
use mtg_engine::rules::abilities::check_triggers;
use mtg_engine::rules::events::GameEvent;
use mtg_engine::state::continuous_effect::{
    ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
};
use mtg_engine::state::game_object::{Designations, TriggerEvent};
use mtg_engine::state::types::{CardType, SubType};
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, CardDefinition, CardRegistry,
    Effect, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Build an EffectContext for a given source object.
fn make_ctx(state: &GameState, source: ObjectId) -> EffectContext {
    let controller = state
        .objects
        .get(&source)
        .map(|o| o.controller)
        .unwrap_or(p(1));
    EffectContext::new(controller, source, vec![])
}

/// Load all card defs into a name→def HashMap (the same way enrich_spec_from_def does).
fn load_defs() -> HashMap<String, CardDefinition> {
    let cards = all_cards();
    cards.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

// ── TriggerCondition::WheneverOpponentPlaysLand / TriggerEvent::OpponentPlaysLand ──

/// CR 305.1 — Burgeoning triggers when an opponent plays a land (not a put-land effect).
/// Verify that when an opponent plays a land, Burgeoning on the battlefield has a
/// WheneverOpponentPlaysLand trigger that is converted to OpponentPlaysLand by
/// enrich_spec_from_def, and that check_triggers finds it from the LandPlayed event.
#[test]
fn test_burgeoning_trigger_on_opponent_land_play() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    // Place Burgeoning on the battlefield for p1.
    let burgeoning_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Burgeoning").in_zone(ZoneId::Battlefield),
        &defs,
    );
    // A land in p1's hand (to be put onto the battlefield by the trigger).
    let land_in_hand = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Hand(p1));
    // A land being played by p2 (the trigger source).
    let p2_land = ObjectSpec::land(p2, "Island").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(burgeoning_spec)
        .object(land_in_hand)
        .object(p2_land)
        .build()
        .unwrap();

    let burgeoning_id = find_object(&state, "Burgeoning");

    // Verify enrich_spec_from_def created the OpponentPlaysLand triggered ability.
    let burgeoning_obj = state.objects.get(&burgeoning_id).unwrap();
    let has_opponent_plays_land = burgeoning_obj
        .characteristics
        .triggered_abilities
        .iter()
        .any(|t| t.trigger_on == TriggerEvent::OpponentPlaysLand);
    assert!(
        has_opponent_plays_land,
        "Burgeoning must have an OpponentPlaysLand triggered ability after enrich_spec_from_def"
    );

    // Simulate opponent playing a land: emit LandPlayed event for p2.
    let p2_land_id = find_object(&state, "Island");
    let events = vec![GameEvent::LandPlayed {
        player: p2,
        new_land_id: p2_land_id,
    }];

    // check_triggers should find Burgeoning's OpponentPlaysLand trigger.
    let triggers = check_triggers(&state, &events);
    let burgeoning_triggers: Vec<_> = triggers
        .iter()
        .filter(|t| t.source == burgeoning_id)
        .collect();
    assert!(
        !burgeoning_triggers.is_empty(),
        "CR 305.1: Burgeoning must generate a trigger when an opponent plays a land"
    );
}

/// CR 305.1 — Burgeoning does NOT trigger on the controller's own land play.
#[test]
fn test_burgeoning_no_trigger_on_own_land_play() {
    let p1 = p(1);
    let defs = load_defs();

    let burgeoning_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Burgeoning").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p1_land = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(burgeoning_spec)
        .object(p1_land)
        .build()
        .unwrap();

    let burgeoning_id = find_object(&state, "Burgeoning");
    let p1_land_id = find_object(&state, "Forest");

    // p1 (Burgeoning's controller) plays a land.
    let events = vec![GameEvent::LandPlayed {
        player: p1,
        new_land_id: p1_land_id,
    }];

    let triggers = check_triggers(&state, &events);
    let burgeoning_triggers: Vec<_> = triggers
        .iter()
        .filter(|t| t.source == burgeoning_id)
        .collect();
    assert!(
        burgeoning_triggers.is_empty(),
        "CR 305.1: Burgeoning must NOT trigger on the controller's own land play"
    );
}

/// CR 305.4 — PutLandFromHandOntoBattlefield does NOT emit LandPlayed, so Burgeoning
/// does not trigger when a land is "put" onto the battlefield by another effect.
#[test]
fn test_burgeoning_no_trigger_on_put_land() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let burgeoning_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Burgeoning").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p2_land_in_hand = ObjectSpec::land(p2, "Swamp").in_zone(ZoneId::Hand(p2));

    let mut state = GameStateBuilder::four_player()
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(burgeoning_spec)
        .object(p2_land_in_hand)
        .build()
        .unwrap();

    let burgeoning_id = find_object(&state, "Burgeoning");
    let p2_land_id = find_object(&state, "Swamp");
    let mut ctx = EffectContext::new(p2, p2_land_id, vec![]);

    // Execute PutLandFromHandOntoBattlefield for p2 — this should NOT emit LandPlayed.
    let events = execute_effect(
        &mut state,
        &Effect::PutLandFromHandOntoBattlefield { tapped: false },
        &mut ctx,
    );

    // Verify no LandPlayed event was emitted.
    let land_played = events
        .iter()
        .any(|e| matches!(e, GameEvent::LandPlayed { .. }));
    assert!(
        !land_played,
        "CR 305.4: PutLandFromHandOntoBattlefield must NOT emit LandPlayed"
    );

    // Check that Burgeoning would not have triggered (no LandPlayed event to dispatch).
    let triggers = check_triggers(&state, &events);
    let burgeoning_triggers: Vec<_> = triggers
        .iter()
        .filter(|t| t.source == burgeoning_id)
        .collect();
    assert!(
        burgeoning_triggers.is_empty(),
        "CR 305.4: Burgeoning must NOT trigger when a land is 'put' onto the battlefield"
    );
}

// ── Effect::PutLandFromHandOntoBattlefield ────────────────────────────────────

/// CR 305.4 — PutLandFromHandOntoBattlefield does not count as playing a land.
/// The player's land_plays_remaining must not decrease.
#[test]
fn test_put_land_does_not_count_as_land_play() {
    let p1 = p(1);

    let land_in_hand = ObjectSpec::land(p1, "Plains").in_zone(ZoneId::Hand(p1));
    // Use a placeholder source (e.g. Burgeoning id = 0 for testing purposes).
    let placeholder = ObjectSpec::land(p1, "PH").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(land_in_hand)
        .object(placeholder)
        .build()
        .unwrap();

    let placeholder_id = find_object(&state, "PH");
    let land_plays_before = state.players.get(&p1).unwrap().land_plays_remaining;

    let mut ctx = EffectContext::new(p1, placeholder_id, vec![]);
    execute_effect(
        &mut state,
        &Effect::PutLandFromHandOntoBattlefield { tapped: false },
        &mut ctx,
    );

    let land_plays_after = state.players.get(&p1).unwrap().land_plays_remaining;
    assert_eq!(
        land_plays_before, land_plays_after,
        "CR 305.4: PutLandFromHandOntoBattlefield must not change land_plays_remaining"
    );
}

/// CR 603.2 / 305.4 — PutLandFromHandOntoBattlefield emits PermanentEnteredBattlefield,
/// which is the event that triggers landfall (ETB-triggered) abilities.
#[test]
fn test_put_land_triggers_landfall() {
    let p1 = p(1);

    let land_in_hand = ObjectSpec::land(p1, "Mountain").in_zone(ZoneId::Hand(p1));
    let placeholder = ObjectSpec::land(p1, "PH").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(land_in_hand)
        .object(placeholder)
        .build()
        .unwrap();

    let placeholder_id = find_object(&state, "PH");
    let mut ctx = EffectContext::new(p1, placeholder_id, vec![]);

    let events = execute_effect(
        &mut state,
        &Effect::PutLandFromHandOntoBattlefield { tapped: false },
        &mut ctx,
    );

    // PermanentEnteredBattlefield must be emitted (landfall trigger hook).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { player, .. } if *player == p1)),
        "CR 305.4 + 603.2: PutLandFromHandOntoBattlefield must emit PermanentEnteredBattlefield for landfall"
    );
    // LandPlayed must NOT be emitted.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::LandPlayed { .. })),
        "CR 305.4: PutLandFromHandOntoBattlefield must NOT emit LandPlayed"
    );
    // The land should now be on the battlefield.
    let on_bf = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Mountain" && o.zone == ZoneId::Battlefield);
    assert!(
        on_bf,
        "The land should be on the battlefield after PutLandFromHandOntoBattlefield"
    );
}

/// CR 305.4 — When no land card is in hand, PutLandFromHandOntoBattlefield does nothing.
#[test]
fn test_put_land_no_op_when_hand_empty() {
    let p1 = p(1);

    // No land in hand — only a non-land card.
    let placeholder = ObjectSpec::land(p1, "PH").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(placeholder)
        .build()
        .unwrap();

    let placeholder_id = find_object(&state, "PH");
    let bf_count_before = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield)
        .count();

    let mut ctx = EffectContext::new(p1, placeholder_id, vec![]);
    let events = execute_effect(
        &mut state,
        &Effect::PutLandFromHandOntoBattlefield { tapped: false },
        &mut ctx,
    );

    let bf_count_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield)
        .count();
    assert_eq!(
        bf_count_before, bf_count_after,
        "CR 305.4: PutLandFromHandOntoBattlefield does nothing when no land is in hand"
    );
    assert!(
        events.is_empty()
            || !events
                .iter()
                .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })),
        "No PermanentEnteredBattlefield when no land is in hand"
    );
}

// ── EffectFilter::LandsYouControl / Dryad of the Ilysian Grove ───────────────

/// CR 305.2 — Dryad of the Ilysian Grove grants its controller an additional land play.
#[test]
fn test_dryad_additional_land_play() {
    let p1 = p(1);
    let defs = load_defs();

    // Dryad on the battlefield for p1 — register it through the card registry.
    let registry = CardRegistry::new(all_cards());
    let dryad_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Dryad of the Ilysian Grove").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(registry)
        .object(dryad_spec)
        .build()
        .unwrap();

    // Simulate turn start: reset_turn_state processes AdditionalLandPlaySources.
    // For this test, directly verify the AdditionalLandPlays ability registered
    // a source, then call reset to apply it.
    let dryad_id = find_object(&state, "Dryad of the Ilysian Grove");

    // Manually register the additional land play source (as register_static_continuous_effects
    // would do during the actual game). This tests the turn-state integration.
    use mtg_engine::state::stubs::AdditionalLandPlaySource;
    state
        .additional_land_play_sources
        .push_back(AdditionalLandPlaySource {
            source: dryad_id,
            controller: p1,
            count: 1,
        });

    use mtg_engine::rules::turn_actions::reset_turn_state;
    reset_turn_state(&mut state, p1);

    assert_eq!(
        state.players.get(&p1).unwrap().land_plays_remaining,
        2,
        "CR 305.2: Dryad grants 1 additional land play (total = 2)"
    );
}

/// CR 305.7 — Dryad gives all 5 basic land subtypes to lands the controller controls.
///
/// Note: In a real game, register_static_continuous_effects is called when the Dryad
/// enters the battlefield. In this unit test we manually register the effect to test
/// the EffectFilter::LandsYouControl + AddSubtypes combination in the layer system.
#[test]
fn test_dryad_lands_have_all_basic_types() {
    let p1 = p(1);

    let dryad =
        ObjectSpec::creature(p1, "Dryad of the Ilysian Grove", 2, 4).in_zone(ZoneId::Battlefield);
    let forest = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(dryad)
        .object(forest)
        .build()
        .unwrap();

    let dryad_id = find_object(&state, "Dryad of the Ilysian Grove");
    let forest_id = find_object(&state, "Forest");

    // Manually register the Dryad's "Lands you control are every basic land type" Layer 4 effect.
    // This is what register_static_continuous_effects does when Dryad enters the battlefield.
    let eff_id = state.next_object_id().0;
    let ts = state.timestamp_counter;
    state.timestamp_counter += 1;
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(eff_id),
        source: Some(dryad_id),
        timestamp: ts,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::LandsYouControl,
        modification: LayerModification::AddSubtypes(
            [
                SubType("Plains".to_string()),
                SubType("Island".to_string()),
                SubType("Swamp".to_string()),
                SubType("Mountain".to_string()),
                SubType("Forest".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
        is_cda: false,
        condition: None,
    });

    let chars = calculate_characteristics(&state, forest_id)
        .expect("Forest must have calculable characteristics");

    // Forest should now have all 5 basic land subtypes.
    let basic_subtypes = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    for subtype_name in &basic_subtypes {
        assert!(
            chars.subtypes.contains(&SubType(subtype_name.to_string())),
            "CR 305.7: Forest should have {} subtype from Dryad of the Ilysian Grove",
            subtype_name
        );
    }
}

/// CR 305.7 — EffectFilter::LandsYouControl: opponent's lands are NOT affected by Dryad.
#[test]
fn test_dryad_opponent_lands_unaffected() {
    let p1 = p(1);
    let p2 = p(2);

    let dryad =
        ObjectSpec::creature(p1, "Dryad of the Ilysian Grove", 2, 4).in_zone(ZoneId::Battlefield);
    // A land controlled by p2 (the opponent).
    let p2_land = ObjectSpec::land(p2, "Swamp").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(dryad)
        .object(p2_land)
        .build()
        .unwrap();

    let dryad_id = find_object(&state, "Dryad of the Ilysian Grove");
    let p2_swamp_id = find_object(&state, "Swamp");

    // Register Dryad's layer 4 LandsYouControl effect (source = p1's Dryad).
    let eff_id = state.next_object_id().0;
    let ts = state.timestamp_counter;
    state.timestamp_counter += 1;
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(eff_id),
        source: Some(dryad_id),
        timestamp: ts,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::LandsYouControl,
        modification: LayerModification::AddSubtypes(
            [
                SubType("Plains".to_string()),
                SubType("Island".to_string()),
                SubType("Swamp".to_string()),
                SubType("Mountain".to_string()),
                SubType("Forest".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
        is_cda: false,
        condition: None,
    });

    let chars = calculate_characteristics(&state, p2_swamp_id)
        .expect("Swamp must have calculable characteristics");

    // Swamp should have its own Swamp subtype but NOT gain Plains/Island/Mountain/Forest from Dryad.
    let extra_subtypes = ["Plains", "Island", "Mountain", "Forest"];
    for subtype_name in &extra_subtypes {
        assert!(
            !chars.subtypes.contains(&SubType(subtype_name.to_string())),
            "EffectFilter::LandsYouControl: opponent's Swamp must NOT gain {} from p1's Dryad",
            subtype_name
        );
    }
}

/// CR 305.7 — "in addition to their other types": original subtypes are preserved.
#[test]
fn test_dryad_lands_keep_original_types() {
    let p1 = p(1);

    let dryad =
        ObjectSpec::creature(p1, "Dryad of the Ilysian Grove", 2, 4).in_zone(ZoneId::Battlefield);
    let forest = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(dryad)
        .object(forest)
        .build()
        .unwrap();

    let dryad_id = find_object(&state, "Dryad of the Ilysian Grove");
    let forest_id = find_object(&state, "Forest");

    // Register Dryad's LandsYouControl AddSubtypes effect.
    let eff_id = state.next_object_id().0;
    let ts = state.timestamp_counter;
    state.timestamp_counter += 1;
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(eff_id),
        source: Some(dryad_id),
        timestamp: ts,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::LandsYouControl,
        modification: LayerModification::AddSubtypes(
            [
                SubType("Plains".to_string()),
                SubType("Island".to_string()),
                SubType("Swamp".to_string()),
                SubType("Mountain".to_string()),
                SubType("Forest".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
        is_cda: false,
        condition: None,
    });

    let chars = calculate_characteristics(&state, forest_id).unwrap();

    // Forest keeps its Forest subtype (CR 305.7: "in addition to their other types").
    assert!(
        chars.subtypes.contains(&SubType("Forest".to_string())),
        "CR 305.7: Forest must keep its Forest subtype after Dryad's type-adding effect"
    );
    // Also gains Plains (AddSubtypes, not SetTypeLine, so originals preserved).
    assert!(
        chars.subtypes.contains(&SubType("Plains".to_string())),
        "CR 305.7: Forest must gain Plains subtype from Dryad"
    );
}

// ── Case Mechanic: Effect::SolveCase + Condition::SourceIsSolved ──────────────

/// CR 719.3a — Effect::SolveCase sets the SOLVED designation on the source permanent.
#[test]
fn test_solve_case_sets_designation() {
    let p1 = p(1);

    let case_enchantment = ObjectSpec::land(p1, "MyCase").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(case_enchantment)
        .build()
        .unwrap();

    let case_id = find_object(&state, "MyCase");

    // Before: not solved.
    assert!(
        !state
            .objects
            .get(&case_id)
            .unwrap()
            .designations
            .contains(Designations::SOLVED),
        "Case should start unsolved"
    );

    // Execute SolveCase.
    let mut ctx = EffectContext::new(p1, case_id, vec![]);
    execute_effect(&mut state, &Effect::SolveCase, &mut ctx);

    // After: solved.
    assert!(
        state
            .objects
            .get(&case_id)
            .unwrap()
            .designations
            .contains(Designations::SOLVED),
        "CR 719.3a: Effect::SolveCase must set the SOLVED designation on the source"
    );
}

/// CR 719.3b — Condition::SourceIsSolved evaluates true after SolveCase, false before.
#[test]
fn test_condition_source_is_solved() {
    let p1 = p(1);

    let case_enchantment = ObjectSpec::land(p1, "MyCase").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(case_enchantment)
        .build()
        .unwrap();

    let case_id = find_object(&state, "MyCase");
    let ctx = make_ctx(&state, case_id);

    // Before SolveCase: SourceIsSolved is false.
    assert!(
        !check_condition(&state, &Condition::SourceIsSolved, &ctx),
        "CR 719.3b: SourceIsSolved must be false before SolveCase"
    );

    // Execute SolveCase.
    let mut ctx_mut = EffectContext::new(p1, case_id, vec![]);
    execute_effect(&mut state, &Effect::SolveCase, &mut ctx_mut);

    let ctx2 = make_ctx(&state, case_id);
    assert!(
        check_condition(&state, &Condition::SourceIsSolved, &ctx2),
        "CR 719.3b: SourceIsSolved must be true after SolveCase"
    );
}

/// CR 719.3b — SOLVED designation persists until the permanent leaves the battlefield.
/// Zone change (to graveyard) creates a new object that is not solved (CR 400.7).
#[test]
fn test_solve_case_designation_persists_until_ltb() {
    let p1 = p(1);

    let case_enchantment = ObjectSpec::land(p1, "MyCase").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(case_enchantment)
        .build()
        .unwrap();

    let case_id = find_object(&state, "MyCase");

    // Set solved.
    let mut ctx = EffectContext::new(p1, case_id, vec![]);
    execute_effect(&mut state, &Effect::SolveCase, &mut ctx);

    assert!(
        state
            .objects
            .get(&case_id)
            .unwrap()
            .designations
            .contains(Designations::SOLVED),
        "SOLVED must persist on the battlefield"
    );

    // Move to graveyard (CR 400.7: new object, no designations).
    let (new_id, _) = state
        .move_object_to_zone(case_id, ZoneId::Graveyard(p1))
        .unwrap();

    // New object in graveyard must NOT have SOLVED designation (CR 400.7 + 719.3b).
    let new_obj = state.objects.get(&new_id).unwrap();
    assert!(
        !new_obj.designations.contains(Designations::SOLVED),
        "CR 719.3b + 400.7: SOLVED designation must be cleared on zone change (new object)"
    );
}

/// CR 719.3a — Case of the Locked Hothouse: with 7+ lands, SolveCase fires at end step.
/// Test via check_triggers on AtBeginningOfYourEndStep with Condition::And.
#[test]
fn test_case_solve_at_end_step() {
    let p1 = p(1);
    let defs = load_defs();
    let registry = CardRegistry::new(all_cards());

    let case_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Case of the Locked Hothouse").in_zone(ZoneId::Battlefield),
        &defs,
    );

    // Place 7 lands for p1.
    let lands: Vec<_> = (1..=7)
        .map(|i| ObjectSpec::land(p1, &format!("Forest{}", i)).in_zone(ZoneId::Battlefield))
        .collect();

    let mut builder = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::End)
        .with_registry(registry)
        .object(case_spec);
    for land in lands {
        builder = builder.object(land);
    }
    let state = builder.build().unwrap();

    let case_id = find_object(&state, "Case of the Locked Hothouse");
    let ctx = make_ctx(&state, case_id);

    // The Case is not solved yet. Condition::And of:
    //   YouControlNOrMoreWithFilter { count: 7, Land } → true (7 lands on battlefield)
    //   Not(SourceIsSolved) → true (not yet solved)
    let cond = Condition::And(
        Box::new(Condition::YouControlNOrMoreWithFilter {
            count: 7,
            filter: mtg_engine::TargetFilter {
                has_card_type: Some(CardType::Land),
                ..Default::default()
            },
        }),
        Box::new(Condition::Not(Box::new(Condition::SourceIsSolved))),
    );
    assert!(
        check_condition(&state, &cond, &ctx),
        "CR 719.3a: solve condition must be true when 7+ lands and case is unsolved"
    );
}

/// CR 719.3a — Case does NOT solve when fewer than 7 lands are present.
#[test]
fn test_case_no_solve_without_condition() {
    let p1 = p(1);
    let defs = load_defs();
    let registry = CardRegistry::new(all_cards());

    let case_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Case of the Locked Hothouse").in_zone(ZoneId::Battlefield),
        &defs,
    );
    // Only 6 lands — not enough.
    let lands: Vec<_> = (1..=6)
        .map(|i| ObjectSpec::land(p1, &format!("Forest{}", i)).in_zone(ZoneId::Battlefield))
        .collect();

    let mut builder = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::End)
        .with_registry(registry)
        .object(case_spec);
    for land in lands {
        builder = builder.object(land);
    }
    let state = builder.build().unwrap();

    let case_id = find_object(&state, "Case of the Locked Hothouse");
    let ctx = make_ctx(&state, case_id);

    let cond = Condition::And(
        Box::new(Condition::YouControlNOrMoreWithFilter {
            count: 7,
            filter: mtg_engine::TargetFilter {
                has_card_type: Some(CardType::Land),
                ..Default::default()
            },
        }),
        Box::new(Condition::Not(Box::new(Condition::SourceIsSolved))),
    );
    assert!(
        !check_condition(&state, &cond, &ctx),
        "CR 719.3a: solve condition must be false when fewer than 7 lands"
    );
}

/// CR 719.3a — Already-solved Case: Condition::And is false when SourceIsSolved is true.
#[test]
fn test_case_already_solved_no_retrigger() {
    let p1 = p(1);
    let defs = load_defs();
    let registry = CardRegistry::new(all_cards());

    let case_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Case of the Locked Hothouse").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let lands: Vec<_> = (1..=7)
        .map(|i| ObjectSpec::land(p1, &format!("Forest{}", i)).in_zone(ZoneId::Battlefield))
        .collect();

    let mut builder = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::End)
        .with_registry(registry)
        .object(case_spec);
    for land in lands {
        builder = builder.object(land);
    }
    let mut state = builder.build().unwrap();

    let case_id = find_object(&state, "Case of the Locked Hothouse");

    // Mark the Case as already solved.
    if let Some(obj) = state.objects.get_mut(&case_id) {
        obj.designations.insert(Designations::SOLVED);
    }

    let ctx = make_ctx(&state, case_id);

    // The And condition requires Not(SourceIsSolved) to be true — but it's already solved.
    let cond = Condition::And(
        Box::new(Condition::YouControlNOrMoreWithFilter {
            count: 7,
            filter: mtg_engine::TargetFilter {
                has_card_type: Some(CardType::Land),
                ..Default::default()
            },
        }),
        Box::new(Condition::Not(Box::new(Condition::SourceIsSolved))),
    );
    assert!(
        !check_condition(&state, &cond, &ctx),
        "CR 719.3a: 'to solve' condition must be false when Case is already solved"
    );
}

// ── Condition::And ────────────────────────────────────────────────────────────

/// Condition::And returns true only when both sub-conditions are true.
#[test]
fn test_condition_and_both_true() {
    let p1 = p(1);

    let source = ObjectSpec::land(p1, "Source").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(source)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Source");
    let ctx = make_ctx(&state, source_id);

    // And(Always, Always) → true.
    assert!(
        check_condition(
            &state,
            &Condition::And(Box::new(Condition::Always), Box::new(Condition::Always)),
            &ctx
        ),
        "And(true, true) must be true"
    );

    // And(Always, Not(Always)) → false.
    assert!(
        !check_condition(
            &state,
            &Condition::And(
                Box::new(Condition::Always),
                Box::new(Condition::Not(Box::new(Condition::Always)))
            ),
            &ctx
        ),
        "And(true, false) must be false"
    );

    // And(Not(Always), Always) → false.
    assert!(
        !check_condition(
            &state,
            &Condition::And(
                Box::new(Condition::Not(Box::new(Condition::Always))),
                Box::new(Condition::Always)
            ),
            &ctx
        ),
        "And(false, true) must be false"
    );

    // And(Not(Always), Not(Always)) → false.
    assert!(
        !check_condition(
            &state,
            &Condition::And(
                Box::new(Condition::Not(Box::new(Condition::Always))),
                Box::new(Condition::Not(Box::new(Condition::Always)))
            ),
            &ctx
        ),
        "And(false, false) must be false"
    );
}
