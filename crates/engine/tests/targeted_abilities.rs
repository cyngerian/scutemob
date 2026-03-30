//! Tests for targeted activated and triggered abilities (CR 601.2c, CR 602, CR 603).
//!
//! PB-5: Validates that AbilityDefinition::Activated and AbilityDefinition::Triggered
//! with TargetRequirement fields properly validate targets at activation/trigger time.

use mtg_engine::rules::{process_command, Command};
use mtg_engine::state::turn::Step;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::state::{
    error::GameStateError, ActivatedAbility, ActivationCost, GameStateBuilder, ObjectSpec,
    PlayerId, TriggerEvent, TriggeredAbilityDef,
};
use mtg_engine::{Effect, EffectAmount, PlayerTarget, Target, TargetRequirement};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

// ── Targeted Activated Ability Tests ────────────────────────────────────────

/// CR 601.2c: Activated ability with TargetCreature validates the target is a creature.
#[test]
fn targeted_activated_ability_valid_creature_target() {
    let p1 = p(1);

    // A permanent with an activated ability: "{T}: Target creature gets +1/+1 until EOT"
    let source = ObjectSpec::creature(p1, "Pump Source", 1, 1)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: None,
                sacrifice_self: false,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
                remove_counter_cost: None,
            },
            description: "Target creature gets +1/+1 until EOT".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
            targets: vec![TargetRequirement::TargetCreature],

            activation_zone: None,
            once_per_turn: false,
        })
        .in_zone(ZoneId::Battlefield);

    let target_creature = ObjectSpec::creature(p1, "Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(target_creature)
        .build()
        .unwrap();

    let source_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Pump Source" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;
    let target_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Bear" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    // Activating with a valid creature target should succeed.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(target_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result.is_ok(),
        "targeted activated ability with valid creature target should succeed"
    );
}

/// CR 601.2c: Activated ability with TargetCreature rejects a non-creature target.
#[test]
fn targeted_activated_ability_rejects_non_creature() {
    let p1 = p(1);

    let source = ObjectSpec::creature(p1, "Pump Source", 1, 1)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: None,
                sacrifice_self: false,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
                remove_counter_cost: None,
            },
            description: "Target creature gets +1/+1 until EOT".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
            targets: vec![TargetRequirement::TargetCreature],

            activation_zone: None,
            once_per_turn: false,
        })
        .in_zone(ZoneId::Battlefield);

    // An artifact (not a creature) — should not be a valid target.
    let artifact = ObjectSpec::artifact(p1, "Mox").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(artifact)
        .build()
        .unwrap();

    let source_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Pump Source" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;
    let artifact_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Mox" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(artifact_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "targeting a non-creature with TargetCreature should fail with InvalidTarget, got: {:?}",
        result
    );
}

/// CR 601.2c: Activated ability with TargetPlayer accepts a player target.
#[test]
fn targeted_activated_ability_target_player() {
    let p1 = p(1);
    let p2 = p(2);

    let source = ObjectSpec::creature(p1, "Pinger", 1, 1)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: None,
                sacrifice_self: false,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
                remove_counter_cost: None,
            },
            description: "Target player loses 1 life".to_string(),
            effect: Some(Effect::LoseLife {
                player: PlayerTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
            targets: vec![TargetRequirement::TargetPlayer],

            activation_zone: None,
            once_per_turn: false,
        })
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();

    let source_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Pinger" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Player(p2)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result.is_ok(),
        "targeting a player with TargetPlayer should succeed"
    );
}

/// CR 601.2c: Activated ability with no TargetRequirements still works (backward compat).
#[test]
fn activated_ability_no_targets_backward_compatible() {
    let p1 = p(1);

    let source = ObjectSpec::creature(p1, "Self Buffer", 1, 1)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: None,
                sacrifice_self: false,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
                remove_counter_cost: None,
            },
            description: "Gain 1 life".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
            targets: vec![], // No target requirements

            activation_zone: None,
            once_per_turn: false,
        })
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();

    let source_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Self Buffer" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result.is_ok(),
        "ability with no target requirements should work with empty targets"
    );
}

// ── Targeted Triggered Ability Tests ──────────────────────────────────────

/// CR 603.3d: Triggered ability with TargetCreature stores target requirements on the runtime struct.
#[test]
fn triggered_ability_targets_propagate_to_runtime() {
    let p1 = p(1);

    let source = ObjectSpec::creature(p1, "ETB Trigger Source", 2, 2)
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfEntersBattlefield,
            intervening_if: None,
            description: "Target creature gets -1/-1 until EOT".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            targets: vec![TargetRequirement::TargetCreature],
        })
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();

    let obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "ETB Trigger Source" && o.zone == ZoneId::Battlefield)
        .unwrap();

    assert_eq!(obj.characteristics.triggered_abilities.len(), 1);
    assert_eq!(
        obj.characteristics.triggered_abilities[0].targets,
        vec![TargetRequirement::TargetCreature],
        "TargetRequirement should propagate to runtime TriggeredAbilityDef"
    );
}

/// CR 603.3d: Triggered ability with TargetPlayer stores the requirement correctly.
#[test]
fn triggered_ability_target_player_propagates() {
    let p1 = p(1);

    let source = ObjectSpec::creature(p1, "Damage Trigger", 1, 1)
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
            intervening_if: None,
            description: "Target player discards a card".to_string(),
            effect: None,
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            targets: vec![TargetRequirement::TargetPlayer],
        })
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();

    let obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Damage Trigger" && o.zone == ZoneId::Battlefield)
        .unwrap();

    assert_eq!(
        obj.characteristics.triggered_abilities[0].targets,
        vec![TargetRequirement::TargetPlayer]
    );
}

/// CR 603: Triggered ability with no targets (empty vec) works correctly — backward compat.
#[test]
fn triggered_ability_no_targets_backward_compatible() {
    let p1 = p(1);

    let source = ObjectSpec::creature(p1, "Untargeted Trigger", 3, 3)
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfEntersBattlefield,
            intervening_if: None,
            description: "Draw a card".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            targets: vec![],
        })
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();

    let obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Untargeted Trigger" && o.zone == ZoneId::Battlefield)
        .unwrap();

    assert!(
        obj.characteristics.triggered_abilities[0]
            .targets
            .is_empty(),
        "empty targets should be preserved on runtime struct"
    );
}

/// CR 601.2c: Activated ability with TargetCreature rejects a player target (wrong target type).
#[test]
fn targeted_activated_ability_rejects_player_for_creature_requirement() {
    let p1 = p(1);
    let p2 = p(2);

    let source = ObjectSpec::creature(p1, "Creature Pinger", 1, 1)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: None,
                sacrifice_self: false,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
                remove_counter_cost: None,
            },
            description: "Target creature gets +1/+1".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
            targets: vec![TargetRequirement::TargetCreature],

            activation_zone: None,
            once_per_turn: false,
        })
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();

    let source_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Creature Pinger" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    // Passing a Player target when TargetCreature is required should fail.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Player(p2)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "player target should be rejected when TargetCreature is required, got: {:?}",
        result
    );
}

/// CR 601.2c: Wrong number of targets is rejected.
#[test]
fn targeted_activated_ability_rejects_wrong_target_count() {
    let p1 = p(1);

    let source = ObjectSpec::creature(p1, "Single Target", 1, 1)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: None,
                sacrifice_self: false,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
                remove_counter_cost: None,
            },
            description: "Target creature gets +1/+1".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
            targets: vec![TargetRequirement::TargetCreature],

            activation_zone: None,
            once_per_turn: false,
        })
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();

    let source_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Single Target" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    // No targets when one is required.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "zero targets when one TargetCreature is required should fail, got: {:?}",
        result
    );
}
