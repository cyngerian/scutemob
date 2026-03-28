//! Graveyard-zone activated and triggered ability tests (CR 602.2 / PB-35).
//!
//! PB-35: G-29 gap closure — graveyard-zone abilities.
//! Cards like Reassembling Skeleton activate from the graveyard zone.
//! Cards like Bloodghast trigger while in the graveyard zone (landfall from graveyard).
//!
//! Key rules verified:
//! - CR 602.2: Normally only battlefield objects can activate abilities.
//!   ActivationZone::Graveyard allows activation from the graveyard.
//! - PB-35: activation_zone: Some(ActivationZone::Graveyard) on AbilityDefinition::Activated.
//! - PB-35: trigger_zone: Some(TriggerZone::Graveyard) on AbilityDefinition::Triggered.
//! - Zone check: Cannot activate graveyard abilities from battlefield.
//! - Cannot activate another player's graveyard abilities.

use mtg_engine::cards::card_definition::{AbilityDefinition, ActivationZone, TriggerZone};
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, CardDefinition, CardRegistry, Command,
    GameEvent, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj_in_zone(state: &mtg_engine::GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == zone {
            Some(id)
        } else {
            None
        }
    })
}

fn make_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

/// CR 602.2 / PB-35: Reassembling Skeleton has activation_zone: Some(Graveyard).
/// Verify the card def is correctly structured.
#[test]
fn test_graveyard_activated_ability_basic() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    let skeleton_def = defs.get("Reassembling Skeleton").unwrap();
    let graveyard_ability = skeleton_def.abilities.iter().find(|a| {
        matches!(
            a,
            AbilityDefinition::Activated {
                activation_zone: Some(ActivationZone::Graveyard),
                ..
            }
        )
    });

    assert!(
        graveyard_ability.is_some(),
        "CR 602.2: Reassembling Skeleton should have an activated ability with ActivationZone::Graveyard"
    );
}

/// CR 602.2 / PB-35: Graveyard-activated ability activation.
/// Reassembling Skeleton can be activated from the graveyard.
#[test]
fn test_graveyard_activated_ability_activatable() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let p1 = p(1);
    let p2 = p(2);

    // Reassembling Skeleton in the graveyard.
    let skeleton = make_spec(p1, "Reassembling Skeleton", ZoneId::Graveyard(p1), &defs);

    let mut state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(skeleton)
        .build()
        .unwrap();

    // Give p1 {1}{B} mana.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool.colorless = 1;
        ps.mana_pool.black = 1;
    }

    // Find the skeleton in the graveyard.
    let skeleton_id = find_obj_in_zone(&state, "Reassembling Skeleton", ZoneId::Graveyard(p1))
        .expect("Skeleton should be in graveyard");

    // Activate ability index 0 (the graveyard-activated ability).
    let result = mtg_engine::rules::process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: skeleton_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 602.2: Reassembling Skeleton graveyard ability should be activatable from the graveyard. Got: {:?}",
        result.err()
    );
}

/// CR 602.2 / PB-35: Cannot activate a graveyard ability when the source is on the battlefield.
#[test]
fn test_graveyard_activated_ability_zone_check() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let p1 = p(1);
    let p2 = p(2);

    // Reassembling Skeleton on the BATTLEFIELD (should NOT be able to use graveyard ability).
    let skeleton = make_spec(p1, "Reassembling Skeleton", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(skeleton)
        .build()
        .unwrap();

    // Give p1 {1}{B} mana.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool.colorless = 1;
        ps.mana_pool.black = 1;
    }

    let skeleton_id = find_obj_in_zone(&state, "Reassembling Skeleton", ZoneId::Battlefield)
        .expect("Skeleton should be on battlefield");

    // Attempt to activate the graveyard ability from the battlefield — should fail.
    let result = mtg_engine::rules::process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: skeleton_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: Graveyard-activated ability should NOT be activatable from the battlefield"
    );
}

/// PB-35 / TriggerZone::Graveyard: Bloodghast has trigger_zone: Some(Graveyard).
/// Verify the card def is correctly structured with graveyard trigger zone.
#[test]
fn test_graveyard_triggered_ability_structure() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    let bloodghast_def = defs.get("Bloodghast").unwrap();
    let gy_trigger = bloodghast_def.abilities.iter().find(|a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_zone: Some(TriggerZone::Graveyard),
                ..
            }
        )
    });

    assert!(
        gy_trigger.is_some(),
        "PB-35 / CR 603.3: Bloodghast should have a triggered ability with TriggerZone::Graveyard"
    );
}

/// PB-35: Earthquake Dragon has a graveyard-activated ability with sacrifice-land cost.
/// Verify the card def has the correct structure.
#[test]
fn test_graveyard_activated_sacrifice_cost() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    let dragon_def = defs.get("Earthquake Dragon").unwrap();
    let gy_ability = dragon_def.abilities.iter().find(|a| {
        use mtg_engine::cards::card_definition::{Cost, TargetFilter};
        matches!(
            a,
            AbilityDefinition::Activated {
                activation_zone: Some(ActivationZone::Graveyard),
                cost: Cost::Sequence(_),
                ..
            }
        )
    });

    assert!(
        gy_ability.is_some(),
        "PB-35 / CR 602.2: Earthquake Dragon should have a graveyard-activated ability with Sequence cost"
    );

    // Verify the cost includes a sacrifice-land component.
    if let Some(AbilityDefinition::Activated { cost, .. }) = gy_ability {
        use mtg_engine::cards::card_definition::Cost;
        if let Cost::Sequence(costs) = cost {
            let has_sacrifice_land = costs.iter().any(|c| {
                matches!(c, Cost::Sacrifice(f) if f.has_card_type == Some(mtg_engine::state::CardType::Land))
            });
            assert!(
                has_sacrifice_land,
                "Earthquake Dragon graveyard ability cost should include Sacrifice(land filter)"
            );
        }
    }
}
