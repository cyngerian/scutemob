//! W3-LOW PB-S-L03/L04 regression tests: animated-creature sacrifice cost emits CreatureDied.
//!
//! CR 613.1f: Layer-resolved characteristics are authoritative when determining a permanent's
//! types. A permanent animated into a creature via a Layer 4 type-change effect IS a creature
//! for purposes of all rules, including the choice of CreatureDied vs PermanentDestroyed when
//! it enters a graveyard from the battlefield.
//!
//! CR 603.10a: "Whenever a creature dies" triggered abilities trigger on CreatureDied events.
//! If the wrong event (PermanentDestroyed) is emitted, those triggers fail to fire.
//!
//! CR 613.1e: Layer 4 (type-changing effects) are applied before Layer 5 (color) and
//! Layer 6 (ability-adding), so the permanent's type at the point of death must reflect
//! all applied Layer 4 effects.

use im::ordset;
use mtg_engine::{
    calculate_characteristics, process_command, CardEffectTarget, CardType, Command,
    ContinuousEffect, Effect, EffectAmount, EffectDuration, EffectFilter, EffectId, EffectLayer,
    GameEvent, GameStateBuilder, LayerModification, ObjectSpec, PlayerId, Step, TriggeredAbilityDef,
    TriggerEvent,
};
use mtg_engine::state::{ActivatedAbility, ActivationCost, SacrificeFilter};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

fn find_obj(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn life_total(state: &mtg_engine::GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(&player)
        .map(|p| p.life_total)
        .unwrap_or_default()
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
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

/// Build a Layer 4 type-change effect that adds Creature type to all permanents.
fn animate_all_permanents(eff_id: u64, ts: u64) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(eff_id),
        source: None,
        timestamp: ts,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllPermanents,
        modification: LayerModification::AddCardTypes(ordset![CardType::Creature]),
        is_cda: false,
        condition: None,
    }
}

/// Build a "whenever a creature dies, deal 1 damage to each opponent" triggered ability
/// (Blood Artist-style witness trigger). Used to verify the trigger fires when an animated
/// artifact dies.
fn creature_dies_witness_trigger() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureDies,
        intervening_if: None,
        description: "Whenever a creature dies, deal 1 damage to each opponent".to_string(),
        effect: Some(Effect::DealDamage {
            target: CardEffectTarget::EachOpponent,
            amount: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
    }
}

// ---------------------------------------------------------------------------
// Test L03 — animated artifact dying via sacrifice_self cost emits CreatureDied
//
// CR 613.1f + CR 603.10a + CR 613.1e
// Pre-fix: obj.characteristics.card_types read returns base type (Artifact, not Creature)
// → PermanentDestroyed emitted; CreatureDied absent; witness trigger fails to fire.
// Post-fix: calculate_characteristics returns layer-resolved types (Artifact + Creature)
// → CreatureDied emitted; witness trigger fires.
// ---------------------------------------------------------------------------

/// CR 613.1f + CR 603.10a + CR 613.1e: An artifact animated into a creature by a Layer 4
/// type-change effect and then sacrificed as a cost for its OWN activated ability must emit
/// GameEvent::CreatureDied (not just GameEvent::PermanentDestroyed). "Whenever a creature
/// dies" triggers on other permanents must fire as a result.
#[test]
fn test_animated_artifact_sacrifice_self_emits_creature_died() {
    // Build a Layer 4 "animate all permanents" effect.
    let animate = animate_all_permanents(1, 10);

    // Artifact with an activated ability: "Sacrifice this: deal 1 damage to each opponent."
    // sacrifice_self=true triggers the L03 code path.
    let sacrificing_artifact = ObjectSpec::artifact(p1(), "Sacrificial Rock")
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                sacrifice_self: true,
                ..Default::default()
            },
            description: "Sacrifice this: deal 1 damage to each opponent".to_string(),
            effect: Some(Effect::DealDamage {
                target: CardEffectTarget::EachOpponent,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        });

    // Witness: another permanent with "whenever a creature dies" trigger.
    // This exercises the WheneverCreatureDies trigger path — if CreatureDied fires,
    // the trigger queues and eventually resolves.
    let witness = ObjectSpec::creature(p1(), "Death Witness", 1, 1)
        .with_triggered_ability(creature_dies_witness_trigger());

    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .add_continuous_effect(animate)
        .object(sacrificing_artifact)
        .object(witness)
        .active_player(p1())
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1());

    let artifact_id = find_obj(&state, "Sacrificial Rock");

    // Sanity check: the artifact should appear as a creature via layer resolution.
    let chars = calculate_characteristics(&state, artifact_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "animated artifact should be a creature via Layer 4 before activation"
    );

    // Activate the sacrifice ability.
    let (state_after, activation_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1(),
            source: artifact_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("ActivateAbility with sacrifice_self should succeed");

    // CR 613.1f: The sacrifice happens at activation time. CreatureDied must be in
    // the events from the activation command (before resolution of the ability itself).
    let has_creature_died = activation_events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        has_creature_died,
        "animated artifact sacrificed via sacrifice_self should emit CreatureDied; events: {:?}",
        activation_events
    );

    // Also verify that PermanentSacrificed was emitted (CR 701.21a).
    let has_permanent_sacrificed = activation_events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentSacrificed { .. }));
    assert!(
        has_permanent_sacrificed,
        "sacrifice should also emit PermanentSacrificed"
    );

    // Resolve the stack: ability resolves (DealDamage to each opponent),
    // then witness trigger resolves (DealDamage to each opponent again).
    // Both players pass twice to drain the 2-item stack.
    let (final_state, _) = pass_all(state_after, &[p1(), p2(), p1(), p2(), p1(), p2()]);

    // p2's life total should be reduced: once from ability effect, once from witness trigger.
    // 40 - 1 (ability) - 1 (witness trigger) = 38.
    let p2_life = life_total(&final_state, p2());
    assert_eq!(
        p2_life, 38,
        "p2 should take 2 damage total (ability + witness trigger); life={p2_life}"
    );
}

// ---------------------------------------------------------------------------
// Test L04 — animated artifact dying via sacrifice_filter cost emits CreatureDied
//
// CR 613.1f + CR 603.10a + CR 613.1e
// Pre-fix: obj.characteristics.card_types read returns base type (Artifact, not Creature)
// → PermanentDestroyed emitted; CreatureDied absent; witness trigger fails to fire.
// Post-fix: calculate_characteristics returns layer-resolved types (Artifact + Creature)
// → CreatureDied emitted; witness trigger fires.
// ---------------------------------------------------------------------------

/// CR 613.1f + CR 603.10a + CR 613.1e: A second artifact animated into a creature by a
/// Layer 4 type-change effect and then sacrificed as a cost via sacrifice_filter on ANOTHER
/// permanent's activated ability must emit GameEvent::CreatureDied. "Whenever a creature
/// dies" triggers must fire.
#[test]
fn test_animated_artifact_sacrifice_filter_emits_creature_died() {
    // Build a Layer 4 "animate all permanents" effect.
    let animate = animate_all_permanents(1, 10);

    // Activating permanent: an artifact with "Sacrifice a creature: deal 1 damage to each opponent."
    // sacrifice_filter=Some(SacrificeFilter::Creature) triggers the L04 code path.
    let activating_artifact = ObjectSpec::artifact(p1(), "Carnage Altar")
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                sacrifice_filter: Some(SacrificeFilter::Creature),
                ..Default::default()
            },
            description: "Sacrifice a creature: deal 1 damage to each opponent".to_string(),
            effect: Some(Effect::DealDamage {
                target: CardEffectTarget::EachOpponent,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        });

    // The sacrificed permanent: a plain artifact animated into a creature by the Layer 4 effect.
    let sacrificed_artifact = ObjectSpec::artifact(p1(), "Fodder Artifact");

    // Witness: another permanent with "whenever a creature dies" trigger.
    let witness = ObjectSpec::creature(p1(), "Death Witness", 1, 1)
        .with_triggered_ability(creature_dies_witness_trigger());

    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .add_continuous_effect(animate)
        .object(activating_artifact)
        .object(sacrificed_artifact)
        .object(witness)
        .active_player(p1())
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1());

    let altar_id = find_obj(&state, "Carnage Altar");
    let fodder_id = find_obj(&state, "Fodder Artifact");

    // Sanity check: the sacrificed artifact is a creature via layer resolution.
    let chars = calculate_characteristics(&state, fodder_id).unwrap();
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "animated artifact (Fodder) should be a creature via Layer 4 before activation"
    );

    // Activate the sacrificing ability, providing the animated artifact as the sacrifice target.
    let (state_after, activation_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1(),
            source: altar_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(fodder_id),
            x_value: None,
        },
    )
    .expect("ActivateAbility with sacrifice_filter should succeed");

    // CR 613.1f: The sacrifice cost is paid at activation time. CreatureDied must be in
    // the events from the activation command.
    let has_creature_died = activation_events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        has_creature_died,
        "animated artifact sacrificed via sacrifice_filter should emit CreatureDied; events: {:?}",
        activation_events
    );

    // Also verify PermanentSacrificed (CR 701.21a).
    let has_permanent_sacrificed = activation_events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentSacrificed { .. }));
    assert!(
        has_permanent_sacrificed,
        "sacrifice_filter should also emit PermanentSacrificed"
    );

    // Resolve the stack: ability resolves (DealDamage), then witness trigger resolves.
    let (final_state, _) = pass_all(state_after, &[p1(), p2(), p1(), p2(), p1(), p2()]);

    // p2 takes 2 damage: once from the resolved ability, once from the witness trigger.
    let p2_life = life_total(&final_state, p2());
    assert_eq!(
        p2_life, 38,
        "p2 should take 2 damage total (ability + witness trigger); life={p2_life}"
    );
}
