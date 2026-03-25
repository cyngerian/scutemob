//! Channel ability tests (CR 702.34).
//!
//! Channel is an activated ability that can be activated from a player's hand
//! by discarding the card as part of the cost. The ability uses the stack and
//! resolves normally.
//!
//! Key rules verified:
//! - Channel abilities are activated from hand, not the battlefield (CR 702.34).
//! - Discarding the card is part of the activation cost (CR 602.2).
//! - The ability goes on the stack and must be responded to.
//! - After discarding, the card moves to the graveyard (CR 701.8).
//! - Mana cost is paid as part of the activation cost.
//! - Channel abilities cannot be activated from the battlefield.
//! - Only the owner can activate channel abilities on their cards.

use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    process_command, CardType, Command, Effect, EffectAmount, GameEvent, GameState,
    GameStateBuilder, ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, ZoneId,
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

fn find_by_name_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Build a card with a Channel ability (discard-self + mana cost → effect).
fn channel_card(owner: PlayerId, name: &str, mana: ManaCost, effect: Effect) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_types(vec![CardType::Land])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: false,
                mana_cost: Some(mana),
                sacrifice_self: false,
                discard_card: false,
                discard_self: true,
                forage: false,
                sacrifice_filter: None,
            },
            description: "Channel — discard, pay mana: effect".to_string(),
            effect: Some(effect),
            sorcery_speed: false,
            activation_condition: None,
            targets: vec![],
        })
}

/// Helper: advance all 4 players through priority.
fn pass_all_four(
    state: GameState,
    player_order: [PlayerId; 4],
) -> (GameState, Vec<Vec<GameEvent>>) {
    let mut s = state;
    let mut all_events = Vec::new();
    for &pid in &player_order {
        let (ns, evts) = process_command(s, Command::PassPriority { player: pid }).unwrap();
        s = ns;
        all_events.push(evts);
    }
    (s, all_events)
}

#[test]
/// CR 702.34 — Channel ability can be activated from hand by discarding the card.
/// The effect goes on the stack and resolves normally.
fn test_channel_activate_from_hand_basic() {
    let channel_effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::Fixed(3),
    };
    let mana_cost = ManaCost {
        generic: 1,
        green: 1,
        ..ManaCost::default()
    };

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .object(channel_card(
            p(1),
            "Channel Land",
            mana_cost.clone(),
            channel_effect,
        ))
        .build()
        .unwrap();

    // Give player 1 enough mana.
    let mut state = state;
    state.player_mut(p(1)).unwrap().mana_pool.green = 1;
    state.player_mut(p(1)).unwrap().mana_pool.colorless = 1;

    // The card should be in hand.
    let card_id = find_by_name(&state, "Channel Land");
    assert_eq!(
        state.objects.get(&card_id).unwrap().zone,
        ZoneId::Hand(p(1))
    );

    // Activate the channel ability (ability_index 0).
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: card_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Card should be discarded to graveyard.
    assert!(
        find_by_name_in_zone(&state, "Channel Land", ZoneId::Hand(p(1))).is_none(),
        "card should no longer be in hand"
    );
    assert!(
        state.objects.values().any(|o| {
            o.characteristics.name == "Channel Land" && matches!(o.zone, ZoneId::Graveyard(_))
        }),
        "card should be in graveyard after discard"
    );

    // Should have CardDiscarded event.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { .. })),
        "should emit CardDiscarded event"
    );

    // Ability should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "channel ability should be on the stack"
    );

    // Mana should be spent.
    assert_eq!(state.player(p(1)).unwrap().mana_pool.green, 0);
    assert_eq!(state.player(p(1)).unwrap().mana_pool.colorless, 0);

    // Resolve the ability.
    let life_before = state.player(p(1)).unwrap().life_total;
    let (state, _) = pass_all_four(state, [p(1), p(2), p(3), p(4)]);
    let life_after = state.player(p(1)).unwrap().life_total;
    assert_eq!(
        life_after,
        life_before + 3,
        "channel effect should gain 3 life"
    );
}

#[test]
/// CR 702.34 — Channel abilities cannot be activated from the battlefield.
fn test_channel_cannot_activate_from_battlefield() {
    let channel_effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::Fixed(3),
    };
    let mana_cost = ManaCost {
        generic: 1,
        ..ManaCost::default()
    };

    // Put the card on the battlefield instead of hand.
    let spec = ObjectSpec::card(p(1), "Channel Land BF")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Land])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: false,
                mana_cost: Some(mana_cost.clone()),
                sacrifice_self: false,
                discard_card: false,
                discard_self: true,
                forage: false,
                sacrifice_filter: None,
            },
            description: "Channel".to_string(),
            effect: Some(channel_effect),
            sorcery_speed: false,
            activation_condition: None,
            targets: vec![],
        });

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .object(spec)
        .build()
        .unwrap();
    let mut state = state;
    state.player_mut(p(1)).unwrap().mana_pool.colorless = 2;

    let card_id = find_by_name(&state, "Channel Land BF");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: card_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "channel ability should not be activatable from battlefield"
    );
}

#[test]
/// CR 702.34 — Only the card's owner can activate its channel ability.
fn test_channel_only_owner_can_activate() {
    let channel_effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::Fixed(3),
    };
    let mana_cost = ManaCost {
        generic: 1,
        ..ManaCost::default()
    };

    // Card owned by player 1, but player 2 tries to activate it.
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .object(channel_card(
            p(1),
            "P1 Channel",
            mana_cost.clone(),
            channel_effect,
        ))
        .build()
        .unwrap();

    let mut state = state;
    state.player_mut(p(2)).unwrap().mana_pool.colorless = 2;
    // Give p(2) priority.
    state.turn.priority_holder = Some(p(2));

    let card_id = find_by_name(&state, "P1 Channel");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(2),
            source: card_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "only the owner should be able to activate channel abilities"
    );
}

#[test]
/// CR 702.34 + CR 602.2 — Channel requires sufficient mana; activation fails if
/// the player cannot pay.
fn test_channel_insufficient_mana_fails() {
    let channel_effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::Fixed(3),
    };
    let mana_cost = ManaCost {
        generic: 2,
        green: 1,
        ..ManaCost::default()
    };

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .object(channel_card(
            p(1),
            "Expensive Channel",
            mana_cost,
            channel_effect,
        ))
        .build()
        .unwrap();

    // Give only 1 mana (not enough for {2}{G}).
    let mut state = state;
    state.player_mut(p(1)).unwrap().mana_pool.green = 1;

    let card_id = find_by_name(&state, "Expensive Channel");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: card_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(result.is_err(), "should fail with insufficient mana");
}

#[test]
/// CR 702.34 — Channel ability uses the stack and can be responded to.
/// After activation, the source is in the graveyard but the ability is on the stack.
fn test_channel_ability_uses_stack() {
    let channel_effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::Fixed(5),
    };
    let mana_cost = ManaCost {
        generic: 2,
        white: 1,
        ..ManaCost::default()
    };

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .object(channel_card(
            p(1),
            "Damage Channel",
            mana_cost,
            channel_effect,
        ))
        .build()
        .unwrap();

    let mut state = state;
    state.player_mut(p(1)).unwrap().mana_pool.white = 1;
    state.player_mut(p(1)).unwrap().mana_pool.colorless = 2;

    let card_id = find_by_name(&state, "Damage Channel");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: card_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Ability on stack, card in graveyard.
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.objects.values().any(|o| {
        o.characteristics.name == "Damage Channel" && matches!(o.zone, ZoneId::Graveyard(_))
    }),);
}
