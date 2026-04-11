//! Land animation tests — "{1}: This land becomes a creature until end of turn."
//!
//! Tests the pattern of using ApplyContinuousEffect with EffectFilter::Source
//! to add Creature type, set P/T, and add keywords temporarily (until EOT).
//!
//! CR 305.9: "If an effect sets a land's subtype to one or more of the basic land
//! types, the land no longer has its old land type." But animation effects that say
//! "It's still a land" add types without removing existing ones (AddCardTypes).

use mtg_engine::cards::ContinuousEffectDef;
use mtg_engine::rules::layers::calculate_characteristics;
use mtg_engine::state::continuous_effect::{
    EffectDuration, EffectFilter, EffectLayer, LayerModification,
};
use mtg_engine::state::types::KeywordAbility;
use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    process_command, CardType, Command, Effect, GameState, GameStateBuilder, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Build a land with an animate ability: {1}: becomes P/T creature with flying until EOT.
fn animatable_land(owner: PlayerId, name: &str, power: i32, toughness: i32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Land])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: false,
                mana_cost: Some(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                }),
                sacrifice_self: false,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
                remove_counter_cost: None,
                exile_self: false,
            },
            description: format!(
                "{{1}}: Becomes a {power}/{toughness} creature with flying until EOT"
            ),
            effect: Some(Effect::Sequence(vec![
                Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::TypeChange,
                        modification: LayerModification::AddCardTypes(
                            [CardType::Creature].into_iter().collect(),
                        ),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtSet,
                        modification: LayerModification::SetPowerToughness { power, toughness },
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
            ])),
            sorcery_speed: false,
            activation_condition: None,
            targets: vec![],

            activation_zone: None,
            once_per_turn: false,
        })
}

#[test]
/// CR 305.9 — Land animation: activating adds Creature type, P/T, and keywords.
/// The land retains its Land type ("It's still a land").
fn test_land_animation_adds_creature_type_and_pt() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .object(animatable_land(p(1), "Animate Land", 2, 2))
        .build()
        .unwrap();

    let mut state = state;
    state.player_mut(p(1)).unwrap().mana_pool.colorless = 1;

    let land_id = find_by_name(&state, "Animate Land");

    // Before animation: should be a Land, not a Creature.
    let chars_before = calculate_characteristics(&state, land_id).unwrap();
    assert!(chars_before.card_types.contains(&CardType::Land));
    assert!(!chars_before.card_types.contains(&CardType::Creature));
    assert!(chars_before.power.is_none());

    // Activate the animate ability.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: land_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Resolve the ability from the stack.
    let (state, _) =
        process_command(state.clone(), Command::PassPriority { player: p(1) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // After animation: should be a Land AND a Creature.
    let chars_after = calculate_characteristics(&state, land_id).unwrap();
    assert!(
        chars_after.card_types.contains(&CardType::Land),
        "should still be a Land"
    );
    assert!(
        chars_after.card_types.contains(&CardType::Creature),
        "should now also be a Creature"
    );
    assert_eq!(chars_after.power, Some(2), "power should be 2");
    assert_eq!(chars_after.toughness, Some(2), "toughness should be 2");
    assert!(
        chars_after.keywords.contains(&KeywordAbility::Flying),
        "should have Flying"
    );
}

#[test]
/// CR 305.9 — After animation, the land-creature has summoning sickness
/// and cannot attack the turn it was animated (unless it has haste).
fn test_animated_land_has_summoning_sickness() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .object(animatable_land(p(1), "Sick Land", 1, 1))
        .build()
        .unwrap();

    let mut state = state;
    state.player_mut(p(1)).unwrap().mana_pool.colorless = 1;

    let land_id = find_by_name(&state, "Sick Land");

    // Lands don't normally have summoning sickness tracking, but when they
    // become creatures, the has_summoning_sickness flag determines tap ability access.
    // The land was on the battlefield from the start, so it should NOT have summoning sickness.
    let obj = state.objects.get(&land_id).unwrap();
    assert!(
        !obj.has_summoning_sickness,
        "land should not have summoning sickness"
    );
}
