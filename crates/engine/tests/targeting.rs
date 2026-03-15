//! Tests for target legality, fizzle rule, and mana cost payment (CR 601.2, 608.2b).
//!
//! M3-D: Target validation at cast time, fizzle (all targets illegal → spell countered
//! without effect), partial fizzle (some targets illegal → spell resolves normally),
//! and mana cost payment with validation.

use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::game_object::ManaCost;
use mtg_engine::state::turn::Step;
use mtg_engine::state::{
    CardType, Color, GameStateBuilder, ManaPool, ObjectSpec, PlayerId, Target, ZoneId,
};
use mtg_engine::{all_cards, CardRegistry, ObjectId};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Helper: pass priority for all four players in order.
fn pass_all_four(
    state: mtg_engine::GameState,
    turn_order: [PlayerId; 4],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut s = state;
    let mut all_events = Vec::new();
    for player in &turn_order {
        let (ns, evs) = process_command(s, Command::PassPriority { player: *player }).unwrap();
        all_events.extend(evs);
        s = ns;
    }
    (s, all_events)
}

// ---------------------------------------------------------------------------
// CR 601.2c: Target validation at cast time
// ---------------------------------------------------------------------------

#[test]
/// CR 601.2c — targeting an active player is valid at cast time.
fn test_601_2c_targeting_active_player_is_valid() {
    let p1 = p(1);
    let p2 = p(2);
    let instant = ObjectSpec::card(p1, "Lightning Bolt")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Cast targeting p2 (active player p1 has priority at Upkeep).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![Target::Player(p2)],
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
    assert!(result.is_ok(), "targeting an active player should succeed");

    let (new_state, _) = result.unwrap();
    // Target is recorded on the StackObject.
    assert_eq!(new_state.stack_objects.len(), 1);
    assert_eq!(new_state.stack_objects[0].targets.len(), 1);
}

#[test]
/// CR 601.2c — targeting an object (creature on battlefield) is valid at cast time.
fn test_601_2c_targeting_object_is_valid() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p(2), "Target Creature", 2, 2);
    let instant = ObjectSpec::card(p1, "Terror")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(creature)
        .object(instant)
        .build()
        .unwrap();

    let creature_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
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
            targets: vec![Target::Object(creature_id)],
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

    let (new_state, _) = result.unwrap();
    let target = &new_state.stack_objects[0].targets[0];
    // Zone snapshot recorded as Battlefield.
    assert_eq!(target.zone_at_cast, Some(ZoneId::Battlefield));
}

#[test]
/// CR 601.2c — targeting a non-existent object fails.
fn test_601_2c_targeting_nonexistent_object_fails() {
    use mtg_engine::ObjectId;
    let p1 = p(1);
    let instant = ObjectSpec::card(p1, "Terror")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();
    let bogus_id = ObjectId(9999);

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![Target::Object(bogus_id)],
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
        result.is_err(),
        "targeting a non-existent object should fail"
    );
}

#[test]
/// CR 601.2c — targeting an eliminated player fails.
fn test_601_2c_targeting_eliminated_player_fails() {
    let p1 = p(1);
    let p2 = p(2);
    let instant = ObjectSpec::card(p1, "Lightning Bolt")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build()
        .unwrap();

    // Eliminate p2 by having them concede.
    let (state, _) = process_command(state, Command::Concede { player: p2 }).unwrap();

    // Now p1 gets priority back (concede granted priority to next player, but
    // let's manually check — in the four-player game after p2 concedes,
    // p1 should still hold priority or get it).
    // Re-check who has priority and get the card.
    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Ensure p1 has priority before trying to cast.
    let state = if state.turn.priority_holder == Some(p1) {
        state
    } else {
        // Pass from whoever has priority until p1 gets it.
        let holder = state.turn.priority_holder.unwrap();
        process_command(state, Command::PassPriority { player: holder })
            .unwrap()
            .0
    };

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![Target::Player(p2)], // p2 is eliminated
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
        result.is_err(),
        "targeting an eliminated player should fail"
    );
}

// ---------------------------------------------------------------------------
// CR 608.2b: Fizzle rule — all targets illegal at resolution
// ---------------------------------------------------------------------------

#[test]
/// CR 608.2b — spell fizzles when its only target (a player) concedes between
/// cast and resolution. Card goes to graveyard without effect.
fn test_608_2b_fizzle_player_target_concedes() {
    let p1 = p(1);
    let p2 = p(2);
    let instant = ObjectSpec::card(p1, "Mind Rot")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // p1 casts targeting p2.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![Target::Player(p2)],
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
    assert_eq!(state.stack_objects.len(), 1);

    // p2 concedes between cast and resolution — target becomes illegal.
    let (state, _) = process_command(state, Command::Concede { player: p2 }).unwrap();

    // Pass for all remaining active players (p1, p3, p4) until the stack resolves.
    let (mut state, mut all_events) = (state, Vec::new());
    for _ in 0..6 {
        // safety: max 6 passes for 3 active players × 2 rounds
        if state.stack_objects.is_empty() {
            break;
        }
        let holder = match state.turn.priority_holder {
            Some(h) => h,
            None => break,
        };
        let (ns, evs) = process_command(state, Command::PassPriority { player: holder }).unwrap();
        all_events.extend(evs);
        state = ns;
    }

    // Spell should have fizzled — stack is empty and SpellFizzled was emitted.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after fizzle"
    );
    assert!(
        all_events
            .iter()
            .any(|e| matches!(e, mtg_engine::rules::events::GameEvent::SpellFizzled { .. })),
        "expected SpellFizzled event"
    );
}

// Better fizzle test that handles 3 active players properly:
#[test]
/// CR 608.2b — fizzle: all targets illegal → SpellFizzled event, card to graveyard.
fn test_608_2b_fizzle_all_targets_illegal() {
    let p1 = p(1);
    let p2 = p(2);
    let instant = ObjectSpec::card(p1, "Thoughtseize")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Cast targeting p2.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![Target::Player(p2)],
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

    // p2 concedes — target becomes illegal.
    let (state, _) = process_command(state, Command::Concede { player: p2 }).unwrap();

    // After p2's concession, priority passes. We need to pass for all active players
    // (p1, p3, p4 remain) until all pass and the stack resolves.
    // Find who has priority and pass for all remaining active players.
    let (mut state, mut all_events) = (state, Vec::new());

    for _ in 0..6 {
        // safety: max 6 passes for 3 active players × 2 rounds
        if state.stack_objects.is_empty() {
            break;
        }
        let holder = match state.turn.priority_holder {
            Some(h) => h,
            None => break,
        };
        let (ns, evs) = process_command(state, Command::PassPriority { player: holder }).unwrap();
        all_events.extend(evs);
        state = ns;
    }

    // Spell should have fizzled.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after fizzle"
    );
    assert!(state.zones.get(&ZoneId::Stack).unwrap().is_empty());

    // Card is in p1's graveyard (not on battlefield).
    assert_eq!(state.zones.get(&ZoneId::Graveyard(p1)).unwrap().len(), 1);
    assert!(state.zones.get(&ZoneId::Battlefield).unwrap().is_empty());

    // SpellFizzled event emitted, NOT SpellResolved.
    assert!(
        all_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { player, .. } if *player == p1)),
        "SpellFizzled event expected"
    );
    assert!(
        !all_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellResolved { .. })),
        "SpellResolved should NOT be emitted on fizzle"
    );
}

// ---------------------------------------------------------------------------
// CR 608.2b: Partial fizzle — some targets illegal, spell still resolves
// ---------------------------------------------------------------------------

#[test]
/// CR 608.2b — partial fizzle: with two player targets and one concedes,
/// the spell still resolves (SpellResolved event, not SpellFizzled).
fn test_608_2b_partial_fizzle_spell_resolves() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let instant = ObjectSpec::card(p1, "Coercion") // targeting two players
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Cast targeting both p2 and p3.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![Target::Player(p2), Target::Player(p3)],
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

    // p2 concedes — one target becomes illegal, but p3 is still legal.
    let (state, _) = process_command(state, Command::Concede { player: p2 }).unwrap();

    // Pass priority for all remaining active players until spell resolves.
    let (mut state, mut all_events) = (state, Vec::new());
    for _ in 0..6 {
        if state.stack_objects.is_empty() {
            break;
        }
        let holder = match state.turn.priority_holder {
            Some(h) => h,
            None => break,
        };
        let (ns, evs) = process_command(state, Command::PassPriority { player: holder }).unwrap();
        all_events.extend(evs);
        state = ns;
    }

    // Spell resolved (NOT fizzled) because p3 is still a legal target.
    assert!(state.stack_objects.is_empty());
    assert!(
        all_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellResolved { player, .. } if *player == p1)),
        "SpellResolved expected for partial fizzle"
    );
    assert!(
        !all_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "SpellFizzled should NOT be emitted in partial fizzle"
    );

    // Card in graveyard (instant resolved normally, even if effect was partial).
    assert_eq!(state.zones.get(&ZoneId::Graveyard(p1)).unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// Mana cost payment (CR 601.2f-h)
// ---------------------------------------------------------------------------

#[test]
/// CR 601.2f-h — casting a spell deducts its mana cost from the player's pool.
fn test_601_mana_cost_deducted_on_cast() {
    let p1 = p(1);
    let sorcery = ObjectSpec::card(p1, "Mind Rot")
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            black: 1,
            generic: 2,
            ..ManaCost::default()
        }) // {2}{B}
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .player_mana(
            p1,
            ManaPool {
                black: 1,
                colorless: 2,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
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

    // Mana pool should be empty after paying {2}{B}.
    let pool = &new_state.players[&p1].mana_pool;
    assert_eq!(pool.black, 0);
    assert_eq!(pool.colorless, 0);
    assert_eq!(pool.total(), 0);

    // ManaCostPaid event emitted.
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::ManaCostPaid { player, .. } if *player == p1)));
}

#[test]
/// CR 601.2f-h — colored mana pays colored requirement; remaining pays generic.
fn test_601_mana_cost_colored_and_generic() {
    let p1 = p(1);
    let sorcery = ObjectSpec::card(p1, "Counterspell")
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            blue: 2,
            ..ManaCost::default()
        }) // {U}{U}
        .in_zone(ZoneId::Hand(p1));

    // Player has exactly {U}{U}.
    let state = GameStateBuilder::four_player()
        .player_mana(
            p1,
            ManaPool {
                blue: 2,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
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

    let (new_state, _) = process_command(
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

    assert_eq!(new_state.players[&p1].mana_pool.blue, 0);
    assert_eq!(new_state.players[&p1].mana_pool.total(), 0);
}

#[test]
/// CR 601.2f-h — insufficient mana causes cast to fail.
fn test_601_insufficient_mana_fails() {
    let p1 = p(1);
    let sorcery = ObjectSpec::card(p1, "Wrath of God")
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            white: 2,
            generic: 2,
            ..ManaCost::default()
        }) // {2}{W}{W}
        .in_zone(ZoneId::Hand(p1));

    // Player has only {W} — not enough.
    let state = GameStateBuilder::four_player()
        .player_mana(
            p1,
            ManaPool {
                white: 1,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
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
    assert!(result.is_err(), "casting without enough mana should fail");
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InsufficientMana
    ));
}

#[test]
/// CR 601.2f-h — generic mana can be paid with any color remaining after
/// colored requirements are satisfied.
fn test_601_generic_paid_from_any_color() {
    let p1 = p(1);
    let spell = ObjectSpec::card(p1, "Divination")
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            blue: 1,
            generic: 2,
            ..ManaCost::default()
        }) // {2}{U}
        .in_zone(ZoneId::Hand(p1));

    // Pay {U} + {R} + {G} (red and green satisfy generic {2}).
    let state = GameStateBuilder::four_player()
        .player_mana(
            p1,
            ManaPool {
                blue: 1,
                red: 1,
                green: 1,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, _) = process_command(
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

    // Pool should be empty — all mana spent.
    assert_eq!(new_state.players[&p1].mana_pool.total(), 0);
}

#[test]
/// CR 106.1 — colorless mana ({C}) must be paid with colorless, not with
/// colored mana. Insufficient colorless → fail even with colored available.
fn test_601_colorless_requirement_must_use_colorless() {
    let p1 = p(1);
    let spell = ObjectSpec::card(p1, "Eldrazi Temple Effect")
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            colorless: 2,
            ..ManaCost::default()
        }) // {C}{C}
        .in_zone(ZoneId::Hand(p1));

    // Player has {R}{G} but no colorless — cannot pay {C}{C}.
    let state = GameStateBuilder::four_player()
        .player_mana(
            p1,
            ManaPool {
                red: 1,
                green: 1,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
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
        result.is_err(),
        "colored mana cannot pay colorless {{C}} cost"
    );
}

#[test]
/// CR 601.2f-h — spells with no mana cost (None) cast for free without mana
/// validation; no ManaCostPaid event is emitted.
fn test_601_no_mana_cost_casts_free() {
    let p1 = p(1);
    let instant = ObjectSpec::card(p1, "Pact of Negation")
        .with_types(vec![CardType::Instant])
        // no mana_cost set — resolves to None
        .in_zone(ZoneId::Hand(p1));

    // Player has empty mana pool — casting a no-cost spell should still succeed.
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(instant)
        .build()
        .unwrap();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (_, events) = process_command(
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

    // No ManaCostPaid event for a free spell.
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::ManaCostPaid { .. })));
}

// ---------------------------------------------------------------------------
// CR 601.2c: TargetRequirement validation (MR-M7-02, MR-M7-03)
// ---------------------------------------------------------------------------

/// Helper: build a two-player state with the Doom Blade registry wired in,
/// p1 holding a Doom Blade, and the two provided creature specs on the battlefield.
fn doom_blade_state(
    black_creature: ObjectSpec,
    other_creature: ObjectSpec,
) -> (mtg_engine::GameState, ObjectId) {
    let p1 = p(1);
    let doom_blade_def = all_cards()
        .into_iter()
        .find(|d| d.name == "Doom Blade")
        .expect("Doom Blade must be in all_cards()");
    let doom_blade_id = doom_blade_def.card_id.clone();
    let registry = CardRegistry::new(vec![doom_blade_def]);

    let doom_blade = ObjectSpec::card(p1, "Doom Blade")
        .with_card_id(doom_blade_id)
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            black: 1,
            generic: 1,
            ..ManaCost::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player_with(p1, |b| {
            b.mana(ManaPool {
                black: 1,
                colorless: 1,
                ..ManaPool::default()
            })
        })
        .add_player(p(2))
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(doom_blade)
        .object(black_creature)
        .object(other_creature)
        .with_registry(registry)
        .build()
        .unwrap();

    let doom_card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    (state, doom_card_id)
}

#[test]
/// CR 601.2c, MR-M7-02, MR-M7-03 — Doom Blade cannot target a black creature.
/// The exclude_colors filter must cause the cast to fail with InvalidTarget.
fn test_601_2c_doom_blade_cannot_target_black_creature() {
    let p2 = p(2);

    let black_creature = ObjectSpec::creature(p2, "Black Knight", 2, 2)
        .with_colors(vec![Color::Black])
        .in_zone(ZoneId::Battlefield);
    let red_creature = ObjectSpec::creature(p2, "Goblin Guide", 2, 2)
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Battlefield);

    let (state, doom_card_id) = doom_blade_state(black_creature, red_creature);

    // Find the black creature ID on the battlefield.
    let black_id = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Black Knight")
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::CastSpell {
            player: p(1),
            card: doom_card_id,
            targets: vec![Target::Object(black_id)],
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
        result.is_err(),
        "Doom Blade should be rejected when targeting a black creature"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

#[test]
/// CR 601.2c, MR-M7-02, MR-M7-03 — Doom Blade can target a non-black creature.
/// The exclude_colors filter passes for creatures with no black color.
fn test_601_2c_doom_blade_can_target_non_black_creature() {
    let p2 = p(2);

    let black_creature = ObjectSpec::creature(p2, "Black Knight", 2, 2)
        .with_colors(vec![Color::Black])
        .in_zone(ZoneId::Battlefield);
    let red_creature = ObjectSpec::creature(p2, "Goblin Guide", 2, 2)
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Battlefield);

    let (state, doom_card_id) = doom_blade_state(black_creature, red_creature);

    // Find the red (non-black) creature ID.
    let red_id = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Goblin Guide")
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::CastSpell {
            player: p(1),
            card: doom_card_id,
            targets: vec![Target::Object(red_id)],
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
        "Doom Blade should be accepted when targeting a non-black creature"
    );
    let (_, events) = result.unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "SpellCast event should be emitted"
    );
}

#[test]
/// CR 601.2c, MR-M7-02 — TargetCreature requirement rejects non-creature objects.
/// Casting a spell with TargetCreature against a non-creature permanent fails.
fn test_601_2c_target_creature_rejects_non_creature() {
    let p1 = p(1);
    let p2 = p(2);

    // Use a hand-built Doom Blade-like spell but with TargetCreature only.
    // The easiest way is to use Doom Blade against an enchantment.
    // Doom Blade uses TargetCreatureWithFilter — so it checks is_creature too.
    let doom_blade_def = all_cards()
        .into_iter()
        .find(|d| d.name == "Doom Blade")
        .expect("Doom Blade must be in all_cards()");
    let doom_blade_card_id = doom_blade_def.card_id.clone();
    let registry = CardRegistry::new(vec![doom_blade_def]);

    let doom_blade = ObjectSpec::card(p1, "Doom Blade")
        .with_card_id(doom_blade_card_id)
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            black: 1,
            generic: 1,
            ..ManaCost::default()
        })
        .in_zone(ZoneId::Hand(p1));

    // A non-creature permanent (enchantment, no colors).
    let enchantment = ObjectSpec::card(p2, "Some Enchantment")
        .with_types(vec![CardType::Enchantment])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player_with(p1, |b| {
            b.mana(ManaPool {
                black: 1,
                colorless: 1,
                ..ManaPool::default()
            })
        })
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(doom_blade)
        .object(enchantment)
        .with_registry(registry)
        .build()
        .unwrap();

    let doom_card = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let enchantment_id = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: doom_card,
            targets: vec![Target::Object(enchantment_id)],
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
        result.is_err(),
        "Doom Blade (TargetCreatureWithFilter) should be rejected against an enchantment"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}
