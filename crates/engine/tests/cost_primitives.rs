//! Tests for PB-31: Cost::RemoveCounter and SpellAdditionalCost (CR 118.3, 118.8, 602.2).
//!
//! Covers:
//! - Activated ability remove-counter cost: validation, payment, error cases
//! - Spell additional sacrifice cost: card def configuration and casting validation
use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    enrich_spec_from_def, process_command, AdditionalCost, CardDefinition, CardId, CardRegistry,
    Command, CounterType, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, ManaColor,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, SpellAdditionalCost, Step, SubType, ZoneId,
};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Build an ActivationCost with only a remove_counter_cost.
fn remove_counter_cost(counter: CounterType, count: u32) -> ActivationCost {
    ActivationCost {
        requires_tap: false,
        mana_cost: None,
        sacrifice_self: false,
        discard_card: false,
        discard_self: false,
        forage: false,
        sacrifice_filter: None,
        remove_counter_cost: Some((counter, count)),
        exile_self: false,
    }
}

// ── Remove-Counter Cost Tests (G-16) ─────────────────────────────────────────

/// CR 602.2 / CR 118.3 — activate ability with remove-counter cost on a permanent
/// that has sufficient counters; verify counter is decremented and ability resolves.
#[test]
fn test_remove_counter_cost_basic() {
    let p1 = p(1);
    let source = ObjectSpec::artifact(p1, "Charge Stone")
        .in_zone(ZoneId::Battlefield)
        .with_counter(CounterType::Charge, 2)
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: remove_counter_cost(CounterType::Charge, 1),
            description: "Remove a charge counter: Gain 1 life.".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,

            activation_zone: None,
            once_per_turn: false,
        });
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();
    let src_id = find_by_name(&state, "Charge Stone");

    let (state2, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: src_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("activate with remove-counter cost should succeed (CR 602.2)");

    // CounterRemoved event should have fired.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterRemoved {
                counter: CounterType::Charge,
                count: 1,
                ..
            }
        )),
        "CounterRemoved event should be emitted (CR 602.2)"
    );
    // Permanent should now have 1 charge counter remaining.
    let obj = state2.objects.get(&src_id).unwrap();
    assert_eq!(
        obj.counters.get(&CounterType::Charge).copied().unwrap_or(0),
        1,
        "charge counter should decrease from 2 to 1"
    );
    // Ability should be on the stack.
    assert_eq!(
        state2.stack_objects.len(),
        1,
        "ability should be on the stack"
    );
}

/// CR 118.3 — try to activate with insufficient counters; expect InvalidCommand error.
#[test]
fn test_remove_counter_cost_insufficient() {
    use mtg_engine::state::error::GameStateError;
    let p1 = p(1);
    // Permanent has only 1 counter but ability removes 2.
    let source = ObjectSpec::artifact(p1, "Empty Stone")
        .in_zone(ZoneId::Battlefield)
        .with_counter(CounterType::Charge, 1)
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: remove_counter_cost(CounterType::Charge, 2),
            description: "Remove two charge counters: Gain 1 life.".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,

            activation_zone: None,
            once_per_turn: false,
        });
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();
    let src_id = find_by_name(&state, "Empty Stone");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: src_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(_))),
        "should fail with InvalidCommand when counters are insufficient (CR 118.3)"
    );
}

/// CR 118.3 — remove the last counter: counter entry should be removed, not set to 0.
#[test]
fn test_remove_counter_cost_exact_zero() {
    let p1 = p(1);
    let source = ObjectSpec::artifact(p1, "Last Counter")
        .in_zone(ZoneId::Battlefield)
        .with_counter(CounterType::Charge, 1)
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: remove_counter_cost(CounterType::Charge, 1),
            description: "Remove a charge counter: Gain 1 life.".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,

            activation_zone: None,
            once_per_turn: false,
        });
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();
    let src_id = find_by_name(&state, "Last Counter");

    let (state2, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: src_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("activate should succeed when removing last counter");

    let obj = state2.objects.get(&src_id).unwrap();
    // Counter entry should be completely removed, not left as 0.
    assert!(
        !obj.counters.contains_key(&CounterType::Charge),
        "counter entry should be removed when count reaches 0 (not left as 0) (CR 118.3)"
    );
}

/// CR 601.2h — ActivationCost with both requires_tap + remove_counter_cost:
/// both costs are paid simultaneously.
#[test]
fn test_remove_counter_cost_in_sequence() {
    let p1 = p(1);
    // Tap + remove a +1/+1 counter.
    let source = ObjectSpec::artifact(p1, "Tap Counter Artifact")
        .in_zone(ZoneId::Battlefield)
        .with_counter(CounterType::PlusOnePlusOne, 3)
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: None,
                sacrifice_self: false,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                exile_self: false,
            },
            description: "{T}, Remove a +1/+1 counter: Gain 1 life.".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,

            activation_zone: None,
            once_per_turn: false,
        });
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();
    let src_id = find_by_name(&state, "Tap Counter Artifact");

    let (state2, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: src_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("activate with tap + remove-counter cost should succeed (CR 601.2h)");

    // Both tap and counter-removal events should be present.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentTapped { .. })),
        "PermanentTapped event should be emitted (CR 601.2h)"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::CounterRemoved {
                counter: CounterType::PlusOnePlusOne,
                count: 1,
                ..
            }
        )),
        "CounterRemoved event should be emitted (CR 601.2h)"
    );
    let obj = state2.objects.get(&src_id).unwrap();
    assert_eq!(
        obj.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        2,
        "+1/+1 counter should decrease from 3 to 2"
    );
}

// ── Spell Additional Cost Tests (G-17) ───────────────────────────────────────

/// CR 118.8 — SpellAdditionalCost::SacrificeCreature declared on Village Rites.
#[test]
fn test_village_rites_has_sacrifice_creature_cost() {
    let def = mtg_engine::cards::defs::village_rites::card();
    assert_eq!(
        def.spell_additional_costs.len(),
        1,
        "Village Rites should have 1 spell additional cost"
    );
    assert!(
        matches!(
            def.spell_additional_costs[0],
            SpellAdditionalCost::SacrificeCreature
        ),
        "Village Rites spell_additional_costs[0] should be SacrificeCreature (CR 118.8)"
    );
}

/// CR 118.8 — SpellAdditionalCost::SacrificeLand declared on Crop Rotation.
#[test]
fn test_crop_rotation_has_sacrifice_land_cost() {
    let def = mtg_engine::cards::defs::crop_rotation::card();
    assert_eq!(
        def.spell_additional_costs.len(),
        1,
        "Crop Rotation should have 1 spell additional cost"
    );
    assert!(
        matches!(
            def.spell_additional_costs[0],
            SpellAdditionalCost::SacrificeLand
        ),
        "Crop Rotation spell_additional_costs[0] should be SacrificeLand (CR 118.8)"
    );
}

/// CR 118.8 — SpellAdditionalCost::SacrificeArtifactOrCreature declared on Deadly Dispute.
#[test]
fn test_deadly_dispute_has_sacrifice_artifact_or_creature_cost() {
    let def = mtg_engine::cards::defs::deadly_dispute::card();
    assert_eq!(
        def.spell_additional_costs.len(),
        1,
        "Deadly Dispute should have 1 spell additional cost"
    );
    assert!(
        matches!(
            def.spell_additional_costs[0],
            SpellAdditionalCost::SacrificeArtifactOrCreature
        ),
        "Deadly Dispute spell_additional_costs[0] should be SacrificeArtifactOrCreature (CR 118.8)"
    );
}

/// CR 118.8 — SpellAdditionalCost::SacrificeSubtype(Goblin) declared on Goblin Grenade.
#[test]
fn test_goblin_grenade_has_sacrifice_goblin_cost() {
    let def = mtg_engine::cards::defs::goblin_grenade::card();
    assert_eq!(
        def.spell_additional_costs.len(),
        1,
        "Goblin Grenade should have 1 spell additional cost"
    );
    assert!(
        matches!(
            &def.spell_additional_costs[0],
            SpellAdditionalCost::SacrificeSubtype(sub) if sub.0 == "Goblin"
        ),
        "Goblin Grenade spell_additional_costs[0] should be SacrificeSubtype(Goblin) (CR 118.8)"
    );
}

/// CR 118.8 — SpellAdditionalCost::SacrificeColorPermanent(Blue) declared on Abjure.
#[test]
fn test_abjure_has_sacrifice_blue_permanent_cost() {
    use mtg_engine::Color;
    let def = mtg_engine::cards::defs::abjure::card();
    assert_eq!(
        def.spell_additional_costs.len(),
        1,
        "Abjure should have 1 spell additional cost"
    );
    assert!(
        matches!(
            def.spell_additional_costs[0],
            SpellAdditionalCost::SacrificeColorPermanent(Color::Blue)
        ),
        "Abjure spell_additional_costs[0] should be SacrificeColorPermanent(Blue) (CR 118.8)"
    );
}

/// Helper: build a def map + Arc<CardRegistry> for a single card def.
fn single_def_registry(
    def: CardDefinition,
) -> (
    HashMap<String, CardDefinition>,
    std::sync::Arc<CardRegistry>,
) {
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((def.name.clone(), def.clone())).collect();
    let registry = CardRegistry::new(vec![def]);
    (defs, registry)
}

/// CR 118.8 — cast Village Rites with a creature sacrifice: creature goes to graveyard.
#[test]
fn test_spell_sacrifice_cost_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let vr_def = mtg_engine::cards::defs::village_rites::card();
    let (defs, registry) = single_def_registry(vr_def);
    let vr_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Village Rites")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(CardId("village-rites".to_string())),
        &defs,
    );
    let bear_spec = ObjectSpec::creature(p1, "Bear", 2, 2).in_zone(ZoneId::Battlefield);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(vr_spec)
        .object(bear_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);
    let vr_id = find_by_name(&state, "Village Rites");
    let bear_id = find_by_name(&state, "Bear");

    let (state2, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: vr_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![AdditionalCost::Sacrifice {
                ids: vec![bear_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("Village Rites with creature sacrifice should succeed (CR 118.8)");

    let bear_in_graveyard = state2
        .objects
        .values()
        .any(|obj| obj.zone == ZoneId::Graveyard(p1) && obj.characteristics.name == "Bear");
    assert!(
        bear_in_graveyard,
        "Bear should be sacrificed at cast time (CR 118.8)"
    );
    assert_eq!(
        state2.stack_objects.len(),
        1,
        "Village Rites should be on the stack"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentSacrificed { .. })),
        "PermanentSacrificed event should be emitted (CR 118.8)"
    );
}

/// CR 118.8 — cast Village Rites WITHOUT providing a sacrifice; expect error.
#[test]
fn test_spell_sacrifice_cost_missing() {
    use mtg_engine::state::error::GameStateError;
    let p1 = p(1);
    let p2 = p(2);
    let vr_def = mtg_engine::cards::defs::village_rites::card();
    let (defs, registry) = single_def_registry(vr_def);
    let vr_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Village Rites")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(CardId("village-rites".to_string())),
        &defs,
    );
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(vr_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);
    let vr_id = find_by_name(&state, "Village Rites");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: vr_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![], // no sacrifice
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(_))),
        "casting Village Rites without sacrifice should fail (CR 118.8)"
    );
}

/// CR 118.8 — try to sacrifice a non-creature for Village Rites; expect error.
#[test]
fn test_spell_sacrifice_cost_wrong_type() {
    use mtg_engine::state::error::GameStateError;
    let p1 = p(1);
    let p2 = p(2);
    let vr_def = mtg_engine::cards::defs::village_rites::card();
    let (defs, registry) = single_def_registry(vr_def);
    let vr_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Village Rites")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(CardId("village-rites".to_string())),
        &defs,
    );
    let artifact_spec = ObjectSpec::artifact(p1, "Mox Pearl").in_zone(ZoneId::Battlefield);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(vr_spec)
        .object(artifact_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);
    let vr_id = find_by_name(&state, "Village Rites");
    let artifact_id = find_by_name(&state, "Mox Pearl");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: vr_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![AdditionalCost::Sacrifice {
                ids: vec![artifact_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(_))),
        "sacrificing an artifact for Village Rites should fail — creature required (CR 118.8)"
    );
}
