//! Cycling keyword ability tests (CR 702.29).
//!
//! Cycling is an activated ability that functions only while the card with cycling
//! is in a player's hand. "Cycling [cost]" means "[cost], Discard this card: Draw a card."
//! (CR 702.29a). The keyword exists in all zones (CR 702.29b).
//!
//! Key rules verified:
//! - Cycling from hand: discard (cost) immediately, draw goes on stack (CR 702.29a).
//! - Cycling can only be activated from hand (CR 702.29a).
//! - Cycling cost must be paid (CR 602.2b).
//! - Cycling requires priority (CR 602.2).
//! - Cycling card requires the Cycling keyword (CR 702.29a).
//! - Cycling is instant-speed: no sorcery-speed restriction (CR 702.29a).
//! - Keyword visible in all zones, including battlefield (CR 702.29b).
//! - Zero-cost cycling works (e.g., Street Wraith pattern).
//! - Discard (cost) happens before draw (effect) resolves.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command, GameEvent,
    GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec,
    PlayerId, Step, TypeLine, ZoneId,
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

/// Pass priority for all listed players once.
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

/// Windcaller Aven (simplified): Plain card with Cycling {1} (1 generic mana).
/// Represents any card with a generic cycling cost.
fn cycling_one_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("cycling-card".to_string()),
        name: "Cycling Card".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [mtg_engine::CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cycling {1}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Card with Cycling {U} (1 blue mana).
fn cycling_blue_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("cycling-blue-card".to_string()),
        name: "Cycling Blue Card".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [mtg_engine::CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cycling {U}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    blue: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Card with Cycling {2} (2 generic mana).
fn cycling_two_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("cycling-two-card".to_string()),
        name: "Cycling Two Card".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [mtg_engine::CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cycling {2}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Card with Cycling {0} (zero cost, like Street Wraith).
fn cycling_zero_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("cycling-zero-card".to_string()),
        name: "Cycling Zero Card".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [mtg_engine::CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cycling {0}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Card with NO cycling keyword.
fn no_cycling_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("no-cycling-card".to_string()),
        name: "No Cycling Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
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

// ── Test 1: Basic cycling — discard and draw ──────────────────────────────────

#[test]
/// CR 702.29a — Cycling costs mana and discards self. The draw effect goes on the
/// stack. After all players pass priority, the draw resolves and player draws a card.
fn test_cycling_basic_discards_and_draws() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_one_def()]);

    let cycling_card = ObjectSpec::card(p1, "Cycling Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    // Add a card to p1's library so the draw resolves without loss.
    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {1} mana (cycling cost).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Card");

    // p1 cycles the card.
    let (state, cycle_events) = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // CardDiscarded event emitted (discard is cost — CR 702.29a).
    assert!(
        cycle_events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { player, .. } if *player == p1)),
        "CR 702.29a: CardDiscarded event expected when cycling"
    );

    // CardCycled event emitted (distinct from generic discard — CR 702.29a).
    assert!(
        cycle_events
            .iter()
            .any(|e| matches!(e, GameEvent::CardCycled { player, .. } if *player == p1)),
        "CR 702.29a: CardCycled event expected when cycling"
    );

    // AbilityActivated event emitted.
    assert!(
        cycle_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 702.29a: AbilityActivated event expected when cycling"
    );

    // Cycling Card is now in p1's graveyard (discard happened immediately as cost).
    let in_grave = state.objects.values().any(|o| {
        o.characteristics.name == "Cycling Card" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_grave,
        "CR 702.29a: Cycling Card should be in graveyard after cycling (discard as cost)"
    );

    // The cycling ability (draw) is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.29a: cycling ability (draw) should be on the stack"
    );

    // ManaCostPaid event emitted for cycling cost {1}.
    assert!(
        cycle_events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { player, .. } if *player == p1)),
        "CR 702.29a: ManaCostPaid event expected for cycling cost {{1}}"
    );

    // Mana pool is now empty.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.colorless + pool.blue + pool.red + pool.green + pool.black + pool.white,
        0,
        "CR 702.29a: cycling cost {{1}} should be deducted from mana pool"
    );

    // Both players pass priority — draw ability resolves.
    let (state, _resolve_events) = pass_all(state, &[p1, p2]);

    // p1's hand now contains the drawn card (Library Card → Hand).
    let p1_hand_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        p1_hand_count, 1,
        "CR 702.29a: player should have drawn 1 card after cycling ability resolves"
    );
}

// ── Test 2: Cycling requires card in hand ─────────────────────────────────────

#[test]
/// CR 702.29a — Cycling can only be activated while the card is in a player's hand.
/// Attempting to cycle a card that is on the battlefield should be rejected.
fn test_cycling_card_not_in_hand_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_one_def()]);

    // Card is on the battlefield, NOT in hand.
    let cycling_card = ObjectSpec::card(p1, "Cycling Card")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("cycling-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Card");

    // Attempt to cycle — should fail (card not in hand).
    let result = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.29a: CycleCard should be rejected when card is not in player's hand"
    );
}

// ── Test 3: Cycling requires sufficient mana ──────────────────────────────────

#[test]
/// CR 702.29a, CR 602.2b — The cycling cost must be paid. With insufficient mana,
/// CycleCard should return an InsufficientMana error.
fn test_cycling_insufficient_mana_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_two_def()]);

    let cycling_card = ObjectSpec::card(p1, "Cycling Two Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-two-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 only 1 colorless — NOT enough for Cycling {2}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Two Card");

    let result = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.29a / CR 602.2b: CycleCard should be rejected with insufficient mana"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("InsufficientMana"),
        "CR 602.2b: error should be InsufficientMana, got: {err_msg}"
    );
}

// ── Test 4: Cycling requires the Cycling keyword ──────────────────────────────

#[test]
/// CR 702.29a — Only cards with the Cycling keyword can be cycled.
/// Attempting to cycle a card without the keyword should be rejected.
fn test_cycling_card_without_cycling_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![no_cycling_def()]);

    let plain_card = ObjectSpec::card(p1, "No Cycling Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("no-cycling-card".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(plain_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "No Cycling Card");

    let result = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.29a: CycleCard should be rejected for cards without the Cycling keyword"
    );
}

// ── Test 5: Cycling is instant-speed ─────────────────────────────────────────

#[test]
/// CR 702.29a — Cycling has NO timing restriction. It can be activated any time the
/// player has priority, including during an opponent's turn or with spells on the stack.
/// It is NOT limited to sorcery speed.
fn test_cycling_instant_speed_valid() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_one_def()]);

    let cycling_card = ObjectSpec::card(p1, "Cycling Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    // Library card so draw can resolve.
    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .object(library_card)
        .active_player(p2) // p2 is active player — p1 is NOT the active player
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    // Give priority to p1 (non-active player) — simulating p2 passing to p1.
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Card");

    // p1 cycles during p2's turn — should succeed (instant speed).
    let result = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.29a: cycling should succeed at instant speed (non-active player during opponent's turn): {:?}",
        result.err()
    );
}

// ── Test 6: Discard happens before draw (cost precedes effect) ────────────────

#[test]
/// CR 702.29a — The discard is a cost paid immediately at activation time.
/// The card is in the graveyard BEFORE the draw ability goes on the stack.
/// Inspecting state after CycleCard (before resolution) confirms this ordering.
fn test_cycling_card_goes_to_graveyard_before_draw() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_one_def()]);

    let cycling_card = ObjectSpec::card(p1, "Cycling Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Card");

    // Cycle the card.
    let (state, _events) = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // At this point, BEFORE resolution, the card should be in the graveyard.
    let in_grave = state.objects.values().any(|o| {
        o.characteristics.name == "Cycling Card" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_grave,
        "CR 702.29a: card should be in graveyard immediately after CycleCard (discard is cost, not effect)"
    );

    // The card is NOT in the hand any more.
    let in_hand = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Cycling Card" && o.zone == ZoneId::Hand(p1));
    assert!(
        !in_hand,
        "CR 702.29a: card should NOT be in hand after cycling"
    );

    // The draw ability is still on the stack (not yet resolved).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.29a: draw ability should be on stack, not yet resolved"
    );

    // The player's hand is still EMPTY (draw has not happened yet).
    let hand_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_count, 0,
        "CR 702.29a: player's hand should be empty before draw resolves"
    );
}

// ── Test 7: Cycling draw is on the stack ─────────────────────────────────────

#[test]
/// CR 702.29a — The "draw a card" effect is placed on the stack after the discard cost.
/// The stack object is an ActivatedAbility with an embedded DrawCards effect.
fn test_cycling_draw_is_on_stack() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_one_def()]);

    let cycling_card = ObjectSpec::card(p1, "Cycling Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Card");

    let (state, _events) = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Stack has exactly one object — the cycling draw ability.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.29a: draw ability should be on stack"
    );

    // The stack object is an ActivatedAbility controlled by p1.
    let stack_obj = state.stack_objects.back().unwrap();
    assert_eq!(
        stack_obj.controller, p1,
        "CR 702.29a: cycling ability should be controlled by the cycling player"
    );
    assert!(
        matches!(
            stack_obj.kind,
            mtg_engine::StackObjectKind::ActivatedAbility { .. }
        ),
        "CR 702.29a: cycling ability on stack should be ActivatedAbility kind"
    );
}

// ── Test 8: Zero-cost cycling ─────────────────────────────────────────────────

#[test]
/// CR 702.29a — Cycling with cost {0} (like Street Wraith's "{0}: Cycling") requires
/// no mana payment. The card is discarded and the draw goes on the stack for free.
fn test_cycling_zero_cost_cycling() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_zero_def()]);

    let cycling_card = ObjectSpec::card(p1, "Cycling Zero Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-zero-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // NO mana given — {0} cycling costs nothing.
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Zero Card");

    // Cycling with no mana in pool should succeed.
    let (state, cycle_events) = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // No ManaCostPaid event (cost is {0}, nothing to pay).
    assert!(
        !cycle_events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { .. })),
        "CR 702.29a: no ManaCostPaid event expected for {{0}} cycling cost"
    );

    // Card is in graveyard.
    let in_grave = state.objects.values().any(|o| {
        o.characteristics.name == "Cycling Zero Card" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_grave,
        "CR 702.29a: zero-cost cycling should discard the card immediately"
    );

    // Draw ability on stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.29a: draw ability should be on stack for zero-cost cycling"
    );

    // Resolve — player draws.
    let (state, _) = pass_all(state, &[p1, p2]);
    let hand_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_count, 1,
        "CR 702.29a: player should draw 1 card after zero-cost cycling resolves"
    );
}

// ── Test 9: Cycling keyword visible on battlefield ────────────────────────────

#[test]
/// CR 702.29b — The cycling keyword exists in all zones. A permanent on the battlefield
/// with cycling should have the keyword in its characteristics, even though cycling
/// cannot be activated from the battlefield.
fn test_cycling_keyword_on_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_one_def()]);

    // Place the card on the battlefield (not in hand).
    let cycling_card = ObjectSpec::card(p1, "Cycling Card")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("cycling-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Verify the permanent has the Cycling keyword.
    let permanent = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Cycling Card")
        .expect("Cycling Card should exist");

    assert_eq!(
        permanent.zone,
        ZoneId::Battlefield,
        "CR 702.29b: card is on the battlefield"
    );
    assert!(
        permanent
            .characteristics
            .keywords
            .contains(&KeywordAbility::Cycling),
        "CR 702.29b: cycling keyword should be present on the battlefield permanent"
    );
}

// ── Test 10: Cycling requires priority ───────────────────────────────────────

#[test]
/// CR 602.2 — Activating an ability requires priority. CycleCard should fail if
/// the cycling player does not currently hold priority.
fn test_cycling_requires_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_one_def()]);

    let cycling_card = ObjectSpec::card(p1, "Cycling Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    // Give priority to p2 — p1 does NOT have priority.
    state.turn.priority_holder = Some(p2);

    let card_id = find_object(&state, "Cycling Card");

    let result = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: CycleCard should be rejected when the player does not have priority"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("NotPriorityHolder"),
        "CR 602.2: error should be NotPriorityHolder, got: {err_msg}"
    );
}

// ── Test 11: Cycling with blue mana cost ─────────────────────────────────────

#[test]
/// CR 702.29a — Cycling can have colored mana costs. "Cycling {U}" requires exactly
/// one blue mana. Colorless mana should not be able to pay a colored cycling cost.
fn test_cycling_colored_mana_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_blue_def()]);

    let cycling_card = ObjectSpec::card(p1, "Cycling Blue Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-blue-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {U} (1 blue) — correct mana for cycling {U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Blue Card");

    // Should succeed with correct mana.
    let result = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.29a: cycling {{U}} should succeed with 1 blue mana: {:?}",
        result.err()
    );
}

// ── Test 12: Cycling colored cost — wrong mana rejected ──────────────────────

#[test]
/// CR 702.29a / CR 106.1 — Colored cycling costs require the specified color.
/// Cycling {U} cannot be paid with colorless mana.
fn test_cycling_colored_mana_wrong_color_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![cycling_blue_def()]);

    let cycling_card = ObjectSpec::card(p1, "Cycling Blue Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-blue-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cycling_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {C} (colorless) — NOT a valid payment for {U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Blue Card");

    let result = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 106.1: cycling {{U}} should be rejected when player only has colorless mana"
    );
}
