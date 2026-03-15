//! Tests for spell casting (CR 601).
//!
//! M3-B: CastSpell command — casting windows, stack placement, priority reset.
//! Cost payment and target validation are deferred to M3-D.

use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::turn::Step;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::state::{
    error::GameStateError, CardType, GameStateBuilder, KeywordAbility, ObjectSpec, PlayerId,
    StackObjectKind,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

// ---------------------------------------------------------------------------
// Happy-path: sorcery-speed casting
// ---------------------------------------------------------------------------

#[test]
/// CR 307.1, CR 601.2 — active player casts a sorcery in main phase, empty stack
fn test_cast_spell_sorcery_speed_happy_path() {
    let p1 = p(1);
    let sorcery_card = ObjectSpec::card(p1, "Divination")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(sorcery_card)
        .build()
        .unwrap();

    // Find the card's ObjectId in hand.
    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // Card moved from hand to Stack zone.
    assert!(new_state.zones.get(&ZoneId::Hand(p1)).unwrap().is_empty());
    assert_eq!(new_state.zones.get(&ZoneId::Stack).unwrap().len(), 1);

    // One StackObject pushed.
    assert_eq!(new_state.stack_objects.len(), 1);
    let stack_obj = &new_state.stack_objects[0];
    assert_eq!(stack_obj.controller, p1);
    assert!(matches!(stack_obj.kind, StackObjectKind::Spell { .. }));

    // Events: SpellCast then PriorityGiven.
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)));
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::PriorityGiven { player } if *player == p1)));

    // Active player retains priority.
    assert_eq!(new_state.turn.priority_holder, Some(p1));
    // players_passed reset.
    assert!(new_state.turn.players_passed.is_empty());
}

#[test]
/// CR 601.2 — sorcery can also be cast in postcombat main phase
fn test_cast_spell_sorcery_postcombat_main_ok() {
    let p1 = p(1);
    let sorcery_card = ObjectSpec::card(p1, "Recover")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .object(sorcery_card)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Happy-path: instant-speed casting
// ---------------------------------------------------------------------------

#[test]
/// CR 601.3, CR 116 — instant can be cast any time the player has priority
fn test_cast_spell_instant_during_opponents_upkeep() {
    let p1 = p(1);
    let p2 = p(2);
    let instant_card = ObjectSpec::card(p2, "Counterspell")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    // p1's turn (active player), but p2 has priority (e.g., p1 passed).
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant_card)
        .build()
        .unwrap();

    // Manually give priority to p2 (simulate p1 passing).
    let mut state = state;
    state.turn.priority_holder = Some(p2);

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p2))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    assert_eq!(new_state.stack_objects.len(), 1);
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p2)));
    // CR 601.2i: active player (p1) gets priority.
    assert_eq!(new_state.turn.priority_holder, Some(p1));
}

#[test]
/// CR 702.36 (Flash) — Flash spells have instant speed
fn test_cast_spell_flash_at_instant_speed() {
    let p1 = p(1);
    let flash_creature = ObjectSpec::card(p1, "Briarbridge Patrol")
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Flash)
        .in_zone(ZoneId::Hand(p1));

    // Upkeep — not a main phase; only works because of Flash.
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(flash_creature)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_ok(),
        "Flash creature should be castable at instant speed"
    );
}

// ---------------------------------------------------------------------------
// LIFO stack order
// ---------------------------------------------------------------------------

#[test]
/// CR 405.1 — stack is LIFO; second spell cast is on top
fn test_cast_spell_lifo_stack_order() {
    let p1 = p(1);
    let spell1 = ObjectSpec::card(p1, "Shock")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));
    let spell2 = ObjectSpec::card(p1, "Bolt")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell1)
        .object(spell2)
        .build()
        .unwrap();

    // Cast first spell.
    let hand_ids: Vec<_> = state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .into_iter()
        .collect();
    let first_card = hand_ids[0];
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: first_card,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // Cast second spell — stack has something on it but instants are ok.
    let hand_ids: Vec<_> = state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .into_iter()
        .collect();
    let second_card = hand_ids[0];
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: second_card,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    assert_eq!(state.stack_objects.len(), 2);
    // Second spell is at the back (top of stack, LIFO).
    let top = &state.stack_objects[state.stack_objects.len() - 1];
    assert_eq!(top.controller, p1);
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
/// CR 116 — casting without priority is illegal
fn test_cast_spell_not_priority_holder_fails() {
    let p1 = p(1);
    let p2 = p(2);
    let sorcery = ObjectSpec::card(p2, "Divination")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(sorcery)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p2))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // p2 does not have priority (p1 does).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(matches!(
        result,
        Err(GameStateError::NotPriorityHolder { .. })
    ));
}

#[test]
/// CR 307.1 — sorcery-speed spell cannot be cast during opponent's turn
fn test_cast_spell_sorcery_during_opponents_turn_fails() {
    let p1 = p(1);
    let p2 = p(2);
    let sorcery = ObjectSpec::card(p2, "Divination")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p2));

    // p1 is active player, but p2 has priority (e.g., p1 passed).
    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(sorcery)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p2);

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p2))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(matches!(result, Err(GameStateError::InvalidCommand(_))));
}

#[test]
/// CR 307.1 — sorcery-speed spell cannot be cast outside a main phase
fn test_cast_spell_sorcery_in_upkeep_fails() {
    let p1 = p(1);
    let sorcery = ObjectSpec::card(p1, "Divination")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(sorcery)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(matches!(result, Err(GameStateError::NotMainPhase)));
}

#[test]
/// CR 307.1 — sorcery-speed spell cannot be cast when the stack is not empty
fn test_cast_spell_sorcery_with_nonempty_stack_fails() {
    let p1 = p(1);
    let spell1 = ObjectSpec::card(p1, "Shock")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));
    let spell2 = ObjectSpec::card(p1, "Divination")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell1)
        .object(spell2)
        .build()
        .unwrap();

    // Cast the instant first (puts something on the stack).
    let hand_ids: Vec<_> = state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .into_iter()
        .collect();
    // Find the instant by card_type.
    let instant_id = hand_ids
        .iter()
        .find(|&&id| {
            state
                .objects
                .get(&id)
                .map(|o| o.characteristics.card_types.contains(&CardType::Instant))
                .unwrap_or(false)
        })
        .copied()
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: instant_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // Now try to cast the sorcery — stack is not empty.
    let sorcery_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: sorcery_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(matches!(result, Err(GameStateError::StackNotEmpty)));
}

#[test]
/// CR 305.1 — lands cannot be cast; use PlayLand instead
fn test_cast_spell_land_fails() {
    let p1 = p(1);
    let land = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(land)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(matches!(result, Err(GameStateError::InvalidCommand(_))));
}

#[test]
/// CR 601.2 — card must be in caster's hand
fn test_cast_spell_card_not_in_hand_fails() {
    let p1 = p(1);
    // A sorcery on the battlefield (wrong zone).
    let sorcery = ObjectSpec::card(p1, "Divination")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(sorcery)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(matches!(result, Err(GameStateError::InvalidCommand(_))));
}

#[test]
/// CR 601.2i — after casting, players_passed resets and active player gets priority
fn test_cast_spell_priority_resets_to_active_player() {
    let p1 = p(1);
    let p2 = p(2);
    let instant = ObjectSpec::card(p2, "Counterspell")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    // p2 has priority on p1's turn.
    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p2);
    // Simulate p1 already having passed.
    state.turn.players_passed.insert(p1);

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p2))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: card_id,
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
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // Active player (p1) gets priority; passed set is empty.
    assert_eq!(new_state.turn.priority_holder, Some(p1));
    assert!(new_state.turn.players_passed.is_empty());
}
