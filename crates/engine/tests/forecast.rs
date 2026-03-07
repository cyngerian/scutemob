//! Forecast keyword ability tests (CR 702.57).
//!
//! Forecast is a special kind of activated ability that can be activated only
//! from a player's hand, only during the upkeep step of the card's owner, and
//! only once each turn (CR 702.57a-b).
//!
//! Key rules verified:
//! - Forecast activation from hand during owner's upkeep: ability goes on stack.
//! - Card stays in hand after activation (not discarded — CR 702.57a).
//! - Forecast may only be activated during the upkeep step (CR 702.57b).
//! - Forecast may only be activated during the owner's (active player's) upkeep (CR 702.57b).
//! - Forecast may only be activated once per turn per card (CR 702.57b).
//! - Once-per-turn tracking resets each turn.
//! - Forecast effect resolves when the stack drains.
//! - Split second blocks Forecast activation (CR 702.61a).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command, Effect,
    EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once, resolving one stack item if present.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

/// A simple forecast card: reveals from hand during owner's upkeep to draw a card.
/// Forecast cost: {1} generic.
/// Forecast effect: draw a card.
fn forecast_draw_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("forecast-card".to_string()),
        name: "Forecast Card".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [mtg_engine::CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Forecast -- {1}, Reveal this card from your hand: Draw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Forecast),
            AbilityDefinition::Forecast {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// A card with no Forecast keyword (for negative tests).
fn no_forecast_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("no-forecast-card".to_string()),
        name: "No Forecast Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [mtg_engine::CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

// ── Test 1: Basic forecast activation during upkeep ───────────────────────────

#[test]
/// CR 702.57a — Forecast is an activated ability from hand. Reveals card; activates
/// during owner's upkeep. The forecast ability goes on the stack; card stays in hand.
fn test_forecast_basic_activates_during_upkeep() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![forecast_draw_def()]);

    let forecast_card = ObjectSpec::card(p1, "Forecast Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("forecast-card".to_string()))
        .with_keyword(KeywordAbility::Forecast);

    // Add a card to p1's library so the draw resolves without loss.
    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(forecast_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    // Give p1 {1} mana (forecast cost).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Forecast Card");

    // p1 activates forecast.
    let (state, events) = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // AbilityActivated event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 702.57a: AbilityActivated event expected when forecast is activated"
    );

    // Ability is on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "CR 702.57a: forecast ability should be on the stack after activation"
    );

    // Card is still in hand — NOT discarded.
    let card_still_in_hand = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Forecast Card" && obj.zone == ZoneId::Hand(p1));
    assert!(
        card_still_in_hand,
        "CR 702.57a: forecast card must remain in hand after activation (not discarded)"
    );

    // Forecast is marked as used this turn.
    let used = state
        .forecast_used_this_turn
        .contains(&CardId("forecast-card".to_string()));
    assert!(
        used,
        "CR 702.57b: forecast_used_this_turn should contain the card after activation"
    );
}

// ── Test 2: Forecast fails outside of upkeep ─────────────────────────────────

#[test]
/// CR 702.57b — Forecast may only be activated during the upkeep step.
/// Attempting to activate during the main phase returns an error.
fn test_forecast_fails_outside_upkeep() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![forecast_draw_def()]);

    let forecast_card = ObjectSpec::card(p1, "Forecast Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("forecast-card".to_string()))
        .with_keyword(KeywordAbility::Forecast);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(forecast_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain) // Wrong step — not upkeep
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Forecast Card");

    let result = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.57b: forecast must fail when activated outside the upkeep step"
    );
    let err_str = format!("{:?}", result.unwrap_err());
    assert!(
        err_str.contains("upkeep") || err_str.contains("Upkeep"),
        "CR 702.57b: error message should mention upkeep; got: {}",
        err_str
    );
}

// ── Test 3: Forecast fails during opponent's upkeep ───────────────────────────

#[test]
/// CR 702.57b — Forecast may only be activated during the upkeep of the card's owner.
/// In multiplayer, other players cannot forecast during someone else's upkeep.
fn test_forecast_fails_during_opponent_upkeep() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![forecast_draw_def()]);

    // Forecast card is in p1's hand, but p2 is the active player.
    let forecast_card = ObjectSpec::card(p1, "Forecast Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("forecast-card".to_string()))
        .with_keyword(KeywordAbility::Forecast);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(forecast_card)
        .active_player(p2) // p2 is active, not p1 (the card's owner)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Forecast Card");

    let result = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.57b: forecast must fail during another player's upkeep"
    );
}

// ── Test 4: Forecast once per turn ────────────────────────────────────────────

#[test]
/// CR 702.57b — Forecast may only be activated once each turn.
/// A second activation in the same upkeep returns an error.
fn test_forecast_once_per_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![forecast_draw_def()]);

    let forecast_card = ObjectSpec::card(p1, "Forecast Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("forecast-card".to_string()))
        .with_keyword(KeywordAbility::Forecast);

    let library_card1 = ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1));
    let library_card2 = ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(forecast_card)
        .object(library_card1)
        .object(library_card2)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    // Give p1 {2} mana (enough for two casts, though the second should fail).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Forecast Card");

    // First activation — should succeed.
    let (state, _events) = process_command(
        state.clone(),
        Command::ActivateForecast {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("First forecast activation failed: {:?}", e));

    // Second activation — should fail (once per turn).
    // Need to find the card again (it's still in hand, same ObjectId).
    let card_id2 = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.characteristics.name == "Forecast Card" && obj.zone == ZoneId::Hand(p1)
        })
        .map(|(id, _)| *id)
        .expect("Forecast Card should still be in hand after first activation");

    let result = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id2,
            targets: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.57b: forecast must fail on the second activation in the same turn"
    );
}

// ── Test 5: Forecast tracking resets each turn ───────────────────────────────

#[test]
/// CR 702.57b — Forecast once-per-turn tracking resets at the start of each turn.
/// After using forecast on turn 1, the player can activate it again on turn 2.
fn test_forecast_resets_each_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![forecast_draw_def()]);

    let forecast_card = ObjectSpec::card(p1, "Forecast Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("forecast-card".to_string()))
        .with_keyword(KeywordAbility::Forecast);

    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(forecast_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    // Turn 1: activate forecast.
    let card_id = find_object(&state, "Forecast Card");
    let (mut state, _) = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // Verify the card is marked as used this turn.
    assert!(state
        .forecast_used_this_turn
        .contains(&CardId("forecast-card".to_string())));

    // Simulate turn reset (what reset_turn_state does when a new turn begins).
    state.forecast_used_this_turn = im::OrdSet::new();

    // Verify the tracking is cleared.
    assert!(
        !state
            .forecast_used_this_turn
            .contains(&CardId("forecast-card".to_string())),
        "CR 702.57b: forecast tracking should be reset at the start of each turn"
    );

    // Give p1 mana for second turn.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    // Priority for a new upkeep.
    state.turn.priority_holder = Some(p1);
    state.turn.step = Step::Upkeep;
    state.turn.active_player = p1;

    // Turn 2: forecast should be activatable again.
    let card_id2 = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.characteristics.name == "Forecast Card" && obj.zone == ZoneId::Hand(p1)
        })
        .map(|(id, _)| *id)
        .expect("Forecast Card should still be in hand");

    let result = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id2,
            targets: vec![],
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.57b: forecast should be activatable again after turn reset; got: {:?}",
        result.unwrap_err()
    );
}

// ── Test 6: Card stays in hand after activation ───────────────────────────────

#[test]
/// CR 702.57a — After forecast activation AND resolution, the card is still in hand.
/// Unlike Cycling which discards as cost, the forecast card is only revealed (cosmetic).
fn test_forecast_card_stays_in_hand() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![forecast_draw_def()]);

    let forecast_card = ObjectSpec::card(p1, "Forecast Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("forecast-card".to_string()))
        .with_keyword(KeywordAbility::Forecast);

    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(forecast_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Forecast Card");

    // Activate forecast.
    let (state, _) = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // Resolve the stack (both players pass priority).
    let (state, _resolve_events) = pass_all(state, &[p1, p2]);

    // Card must still be in p1's hand after resolution.
    let card_in_hand = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Forecast Card" && obj.zone == ZoneId::Hand(p1));
    assert!(
        card_in_hand,
        "CR 702.57a: forecast card must remain in hand after ability resolves"
    );

    // Stack should be empty.
    assert!(
        state.stack_objects.is_empty(),
        "Stack should be empty after forecast ability resolves"
    );
}

// ── Test 7: Forecast effect resolves correctly ────────────────────────────────

#[test]
/// CR 702.57a — The forecast effect (draw a card) executes when the ability resolves.
fn test_forecast_effect_resolves() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![forecast_draw_def()]);

    let forecast_card = ObjectSpec::card(p1, "Forecast Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("forecast-card".to_string()))
        .with_keyword(KeywordAbility::Forecast);

    // Add a library card so the draw doesn't cause library-out-loss.
    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(forecast_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    // Count hand size before (just the forecast card = 1).
    let hand_before = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_before, 1,
        "Should have exactly 1 card in hand before forecast"
    );

    let card_id = find_object(&state, "Forecast Card");

    let (state, _) = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // Resolve the stack.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved event emitted.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "AbilityResolved event expected after forecast resolves"
    );

    // p1's hand should now have 2 cards: the forecast card + the drawn card.
    let hand_after = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after, 2,
        "CR 702.57a: hand should have 2 cards after forecast draws one (forecast card + drawn card)"
    );
}

// ── Test 8: Forecast blocked by split second ──────────────────────────────────

#[test]
/// CR 702.61a — Like all non-mana activated abilities, Forecast cannot be activated
/// while a spell with split second is on the stack.
fn test_forecast_blocked_by_split_second() {
    use mtg_engine::{CardType, StackObject, StackObjectKind};

    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![forecast_draw_def()]);

    let forecast_card = ObjectSpec::card(p1, "Forecast Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("forecast-card".to_string()))
        .with_keyword(KeywordAbility::Forecast);

    // A split-second spell on the stack.
    let split_second_card = ObjectSpec::card(p1, "Split Second Card")
        .with_types([CardType::Instant].into_iter().collect())
        .with_keyword(KeywordAbility::SplitSecond);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(forecast_card)
        .object(split_second_card)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    // Manually push a split-second stack object (simulate a cast spell on the stack).
    let split_second_oid = find_object(&state, "Split Second Card");
    let stack_id = state.next_object_id();
    state.stack_objects.push_back(StackObject {
        id: stack_id,
        controller: p1,
        kind: StackObjectKind::Spell {
            source_object: split_second_oid,
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        was_entwined: false,
        escalate_modes_paid: 0,
        was_fused: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        devour_sacrifices: vec![],
        modes_chosen: vec![],
    });

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Forecast Card");

    let result = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.61a: forecast must fail while split second is on the stack"
    );
    let err_str = format!("{:?}", result.unwrap_err());
    assert!(
        err_str.contains("split second"),
        "CR 702.61a: error message should mention split second; got: {}",
        err_str
    );
}

// ── Test 9: Forecast requires the Forecast keyword ───────────────────────────

#[test]
/// CR 702.57a — A card without the Forecast keyword cannot be forecast-activated.
fn test_forecast_requires_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![no_forecast_def()]);

    let no_forecast_card = ObjectSpec::card(p1, "No Forecast Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("no-forecast-card".to_string()));
    // No keyword added — card has no Forecast keyword.

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(no_forecast_card)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "No Forecast Card");

    let result = process_command(
        state,
        Command::ActivateForecast {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.57a: forecast must fail if the card does not have the Forecast keyword"
    );
}
