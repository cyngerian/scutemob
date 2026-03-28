/// Tests for activation conditions on activated abilities (CR 602.5b).
///
/// "Activate only if [condition]" restrictions prevent activation when the
/// condition is not met. The condition is checked at activation time, not
/// resolution time.
use mtg_engine::rules::{process_command, Command};
use mtg_engine::state::game_object::ActivatedAbility;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::state::{ActivationCost, GameStateBuilder, PlayerId};
use mtg_engine::{
    CardType, Condition, Effect, EffectAmount, ObjectSpec, PlayerTarget, TargetFilter,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn activate(
    state: mtg_engine::GameState,
    player: PlayerId,
    source: mtg_engine::state::game_object::ObjectId,
) -> Result<
    (mtg_engine::GameState, Vec<mtg_engine::GameEvent>),
    mtg_engine::state::error::GameStateError,
> {
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
}

fn conditioned_artifact() -> ObjectSpec {
    ObjectSpec::card(p(1), "Conditioned Artifact")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Battlefield)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                ..Default::default()
            },
            description: "Gain 1 life (only if you control a creature)".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            targets: vec![],
            activation_condition: Some(Condition::YouControlPermanent(TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            })),

            activation_zone: None,
        })
}

/// CR 602.5b — activation condition met: ability activates successfully.
#[test]
fn test_activation_condition_met_allows_activation() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(conditioned_artifact())
        // A creature to satisfy the condition
        .object(ObjectSpec::creature(p(1), "Helper Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let artifact_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Conditioned Artifact")
        .map(|(id, _)| *id)
        .unwrap();

    // Activate — should succeed because P1 controls a creature
    let result = activate(state, p(1), artifact_id);
    assert!(
        result.is_ok(),
        "activation should succeed when condition is met"
    );

    let (state, _) = result.unwrap();
    assert_eq!(
        state.stack_objects.len(),
        1,
        "ability should be on the stack"
    );
}

/// CR 602.5b — activation condition NOT met: ability activation is rejected.
#[test]
fn test_activation_condition_not_met_rejects_activation() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(conditioned_artifact())
        // NO creature on the battlefield
        .build()
        .unwrap();

    let artifact_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Conditioned Artifact")
        .map(|(id, _)| *id)
        .unwrap();

    // Activate — should fail because P1 controls no creatures
    let result = activate(state, p(1), artifact_id);
    assert!(
        result.is_err(),
        "activation should be rejected when condition is not met"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("activation condition not met"),
        "error should mention activation condition, got: {}",
        err_msg
    );
}

/// CR 602.5b — condition is re-evaluated dynamically: a creature entering
/// later allows activation that was previously blocked.
#[test]
fn test_activation_condition_changes_dynamically() {
    // Use non-tap cost so we can attempt activation twice on same permanent
    let artifact_spec = ObjectSpec::card(p(1), "Conditioned Artifact 2")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Battlefield)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::default(), // no tap
            description: "Gain 1 life (only if you control a creature)".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            targets: vec![],
            activation_condition: Some(Condition::YouControlPermanent(TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            })),

            activation_zone: None,
        });

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(artifact_spec)
        .build()
        .unwrap();

    let artifact_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Conditioned Artifact 2")
        .map(|(id, _)| *id)
        .unwrap();

    // No creature: should fail
    let result = activate(state.clone(), p(1), artifact_id);
    assert!(result.is_err(), "should fail with no creature");

    // Now build a new state with both the artifact and a creature
    let state2 = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(
            ObjectSpec::card(p(1), "Conditioned Artifact 2")
                .with_types(vec![CardType::Artifact])
                .in_zone(ZoneId::Battlefield)
                .with_activated_ability(ActivatedAbility {
                    cost: ActivationCost::default(),
                    description: "Gain 1 life (only if you control a creature)".to_string(),
                    effect: Some(Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    }),
                    sorcery_speed: false,
                    targets: vec![],
                    activation_condition: Some(Condition::YouControlPermanent(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    })),

                    activation_zone: None,
                }),
        )
        .object(ObjectSpec::creature(p(1), "New Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let artifact_id2 = state2
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Conditioned Artifact 2")
        .map(|(id, _)| *id)
        .unwrap();

    // Should succeed with a creature present
    let result = activate(state2, p(1), artifact_id2);
    assert!(
        result.is_ok(),
        "should succeed once a creature is on the battlefield"
    );
}
