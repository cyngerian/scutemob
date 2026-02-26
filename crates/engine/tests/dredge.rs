//! Dredge keyword ability tests (CR 702.52).
//!
//! Dredge is a static ability that functions only while the card with dredge is in
//! a player's graveyard. "Dredge N" means "As long as you have at least N cards in
//! your library, if you would draw a card, you may instead mill N cards and return
//! this card from your graveyard to your hand." (CR 702.52a)
//!
//! Key rules verified:
//! - Dredge replaces draws from any source — not just the draw step (CR 702.52a ruling).
//! - Player choice: "you may instead" — the player can decline and draw normally.
//! - Must have >= N cards in library (CR 702.52b).
//! - Functions only while in the graveyard (CR 702.52a).
//! - Dredge does NOT count as drawing (cards_drawn_this_turn not incremented).
//! - Multiple dredge cards in graveyard: all eligible options are offered.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step,
    TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn object_in_zone(state: &GameState, name: &str, zone: ZoneId) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == zone)
}

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state.objects.values().filter(|o| o.zone == zone).count()
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

/// Create a Dredge N card definition (sorcery, no mana cost, dredge N).
fn dredge_card_def(card_id: &str, name: &str, dredge_n: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: format!("Dredge {}", dredge_n),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Dredge(dredge_n))],
        ..Default::default()
    }
}

/// Build a state ready for the draw step draw to fire.
///
/// Starts at Step::Upkeep with is_first_turn_of_game = false so the draw
/// won't be skipped. When both players pass priority in Upkeep, the engine
/// advances to Draw step and fires the draw turn-based action.
fn build_upkeep_state(
    p1: PlayerId,
    p2: PlayerId,
    registry: std::sync::Arc<mtg_engine::CardRegistry>,
    extra_objects: impl FnOnce(GameStateBuilder) -> GameStateBuilder,
) -> GameState {
    let builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Upkeep);

    let builder = extra_objects(builder);

    let mut state = builder.build().unwrap();
    // CR 103.8: mark as NOT first turn so draw step draw is not skipped.
    state.turn.is_first_turn_of_game = false;
    state.turn.priority_holder = Some(p1);
    state
}

// ── Test 1: Draw step emits DredgeChoiceRequired when dredge is available ────

#[test]
/// CR 702.52a — During the draw step, if the player has a dredge card in graveyard
/// with >= N cards in library, the engine emits DredgeChoiceRequired and pauses.
fn test_dredge_draw_step_emits_choice_required() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-3", "Dredge Three", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        // Dredge card in graveyard.
        b = b.object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        // 5 cards in library — enough to dredge (need >= 3).
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    // Pass priority in Upkeep for both players — engine advances to Draw step
    // and fires the draw turn-based action (which checks dredge first).
    let (_, events) = pass_all(state, &[p1, p2]);

    // DredgeChoiceRequired must have been emitted.
    let choice_event = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::DredgeChoiceRequired { player, .. } if *player == p1
        )
    });
    assert!(
        choice_event.is_some(),
        "CR 702.52a: DredgeChoiceRequired expected when dredge card in graveyard \
         with enough library cards. Events: {:?}",
        events
    );

    // No CardDrawn event (draw is paused waiting for ChooseDredge).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 702.52a: CardDrawn must NOT be emitted before player chooses whether to dredge"
    );
}

// ── Test 2: Dredge replaces draw — mills N cards, returns card to hand ─────

#[test]
/// CR 702.52a — When the player chooses to dredge, N cards are milled from their
/// library and the dredge card moves from graveyard to hand (not a draw).
fn test_dredge_mills_and_returns_card_to_hand() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-3", "Dredge Three", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    // Advance to the point where DredgeChoiceRequired is emitted.
    let (state, events) = pass_all(state, &[p1, p2]);

    // Find the DredgeChoiceRequired event to get the dredge card ID.
    let (dredge_card_id, _n) = events
        .iter()
        .find_map(|e| {
            if let GameEvent::DredgeChoiceRequired { player, options } = e {
                if *player == p1 {
                    options.first().copied()
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("DredgeChoiceRequired event expected");

    let lib_before = count_in_zone(&state, ZoneId::Library(p1));
    let grave_before = count_in_zone(&state, ZoneId::Graveyard(p1));
    let hand_before = count_in_zone(&state, ZoneId::Hand(p1));

    // Player chooses to dredge "Dredge Three".
    let (state, dredge_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge_card_id),
        },
    )
    .unwrap();

    // Dredged event must be emitted.
    let dredged = dredge_events
        .iter()
        .find(|e| matches!(e, GameEvent::Dredged { player, milled: 3, .. } if *player == p1));
    assert!(
        dredged.is_some(),
        "CR 702.52a: Dredged event expected with milled=3. Events: {:?}",
        dredge_events
    );

    // Exactly 3 CardMilled events.
    let mill_count = dredge_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardMilled { player, .. } if *player == p1))
        .count();
    assert_eq!(
        mill_count, 3,
        "CR 702.52a: exactly 3 cards should be milled when dredge N=3"
    );

    // Library decreased by 3.
    let lib_after = count_in_zone(&state, ZoneId::Library(p1));
    assert_eq!(
        lib_after,
        lib_before - 3,
        "CR 702.52a: library should have 3 fewer cards after dredge 3"
    );

    // Graveyard: +3 milled cards, -1 (dredge card left), so net +2.
    let grave_after = count_in_zone(&state, ZoneId::Graveyard(p1));
    assert_eq!(
        grave_after,
        grave_before + 3 - 1,
        "CR 702.52a: graveyard should have 3 new milled cards and the dredge card gone"
    );

    // Hand: +1 (dredge card returned).
    let hand_after = count_in_zone(&state, ZoneId::Hand(p1));
    assert_eq!(
        hand_after,
        hand_before + 1,
        "CR 702.52a: dredge card should be in hand after dredging"
    );

    // "Dredge Three" is in hand.
    assert!(
        object_in_zone(&state, "Dredge Three", ZoneId::Hand(p1)),
        "CR 702.52a: Dredge Three should be in hand after dredging"
    );

    // No CardDrawn event (dredge is not drawing).
    assert!(
        !dredge_events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 702.52a: CardDrawn must NOT be emitted when dredging — dredge is not a draw"
    );
}

// ── Test 3: Player declines dredge — draws normally ────────────────────────

#[test]
/// CR 702.52a — "you may instead" — the player can decline dredge and draw normally.
/// Declining emits CardDrawn and increments cards_drawn_this_turn.
fn test_dredge_decline_draws_normally() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-3", "Dredge Three", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        // Add filler cards first (bottom of library), then "Top of Library" last (actual top).
        // zone.insert() uses push_back; zone.top() returns the last element.
        for i in 1..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b = b.object(ObjectSpec::card(p1, "Top of Library").in_zone(ZoneId::Library(p1)));
        b
    });

    let cards_drawn_before = state.players[&p1].cards_drawn_this_turn;

    // Advance to DredgeChoiceRequired.
    let (state, _events) = pass_all(state, &[p1, p2]);

    // Player declines dredge (card: None).
    let (state, decline_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();

    // CardDrawn event must be emitted.
    let drawn = decline_events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
    assert!(
        drawn,
        "CR 702.52a: declining dredge should result in normal draw (CardDrawn event). \
         Events: {:?}",
        decline_events
    );

    // cards_drawn_this_turn incremented.
    let cards_drawn_after = state.players[&p1].cards_drawn_this_turn;
    assert_eq!(
        cards_drawn_after,
        cards_drawn_before + 1,
        "CR 702.52a: declining dredge and drawing normally should increment cards_drawn_this_turn"
    );

    // "Top of Library" moved to hand.
    assert!(
        object_in_zone(&state, "Top of Library", ZoneId::Hand(p1)),
        "CR 702.52a: declining dredge should draw the top card of library"
    );
}

// ── Test 4: Dredge not offered when library has fewer cards than N ──────────

#[test]
/// CR 702.52b — "A player with fewer cards in their library than the number required
/// by a dredge ability can't mill any of them this way."
/// Dredge is not offered when library.len() < N.
fn test_dredge_insufficient_library_not_offered() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-5", "Dredge Five", 5)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        // Dredge 5 card but only 3 cards in library.
        b = b.object(
            ObjectSpec::card(p1, "Dredge Five")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-5".to_string()))
                .with_keyword(KeywordAbility::Dredge(5)),
        );
        for i in 0..3 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    // Advance through Upkeep → Draw step.
    let (_, events) = pass_all(state, &[p1, p2]);

    // DredgeChoiceRequired must NOT be emitted (library has 3, need 5).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { .. })),
        "CR 702.52b: DredgeChoiceRequired must NOT be emitted when library.len() < dredge N. \
         Events: {:?}",
        events
    );

    // Normal draw should proceed instead.
    let drawn = events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
    assert!(
        drawn,
        "CR 702.52b: when dredge is not eligible, normal draw should proceed. \
         Events: {:?}",
        events
    );
}

// ── Test 5: Dredge card on battlefield — not offered ────────────────────────

#[test]
/// CR 702.52a — "functions only while the card with dredge is in a player's graveyard."
/// A dredge card on the battlefield does NOT offer dredge during draws.
fn test_dredge_card_on_battlefield_not_offered() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-3", "Dredge Three", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        // Dredge card on the battlefield (not in graveyard).
        b = b.object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    let (_, events) = pass_all(state, &[p1, p2]);

    // DredgeChoiceRequired must NOT be emitted (card not in graveyard).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { .. })),
        "CR 702.52a: DredgeChoiceRequired must NOT be emitted when dredge card is not \
         in the graveyard. Events: {:?}",
        events
    );

    // Normal draw should proceed.
    let drawn = events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
    assert!(
        drawn,
        "CR 702.52a: when dredge card is on battlefield (not graveyard), draw normally"
    );
}

// ── Test 6: Multiple dredge cards in graveyard — all offered ───────────────

#[test]
/// CR 702.52a — Multiple dredge cards in graveyard: all eligible options appear
/// in the DredgeChoiceRequired event.
fn test_dredge_multiple_options_both_offered() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        dredge_card_def("dredge-3", "Dredge Three", 3),
        dredge_card_def("dredge-5", "Dredge Five", 5),
    ]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        // Both dredge cards in graveyard; library has 5 cards.
        b = b.object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        b = b.object(
            ObjectSpec::card(p1, "Dredge Five")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-5".to_string()))
                .with_keyword(KeywordAbility::Dredge(5)),
        );
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    let (_, events) = pass_all(state, &[p1, p2]);

    // DredgeChoiceRequired with both options.
    let options = events.iter().find_map(|e| {
        if let GameEvent::DredgeChoiceRequired { player, options } = e {
            if *player == p1 {
                Some(options.clone())
            } else {
                None
            }
        } else {
            None
        }
    });

    let options = options
        .expect("DredgeChoiceRequired must be emitted when multiple dredge cards are in graveyard");

    assert_eq!(
        options.len(),
        2,
        "CR 702.52a: both dredge cards (Dredge 3 and Dredge 5) should be in options list. \
         Got: {:?}",
        options
    );
}

// ── Test 7: Dredge does not increment cards_drawn_this_turn ─────────────────

#[test]
/// CR 702.52a + CC#33 — Dredge is a replacement effect; the card returned to hand
/// was NOT drawn. `cards_drawn_this_turn` must NOT be incremented.
fn test_dredge_does_not_increment_cards_drawn_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-3", "Dredge Three", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    let cards_drawn_before = state.players[&p1].cards_drawn_this_turn;

    // Advance to DredgeChoiceRequired.
    let (state, events) = pass_all(state, &[p1, p2]);

    let (dredge_card_id, _n) = events
        .iter()
        .find_map(|e| {
            if let GameEvent::DredgeChoiceRequired { player, options } = e {
                if *player == p1 {
                    options.first().copied()
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("DredgeChoiceRequired event expected");

    // Dredge the card.
    let (state, _) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge_card_id),
        },
    )
    .unwrap();

    let cards_drawn_after = state.players[&p1].cards_drawn_this_turn;

    assert_eq!(
        cards_drawn_after, cards_drawn_before,
        "CR 702.52a + CC#33: dredge must NOT increment cards_drawn_this_turn \
         (dredge is a replacement effect, not drawing)"
    );
}

// ── Test 8: Dredge exact library count — offered when library.len() == N ───

#[test]
/// CR 702.52b — "As long as you have at least N cards in your library."
/// Dredge IS offered when library.len() == N (exactly enough).
/// After dredging, library is empty but no PlayerLost (didn't try to draw).
fn test_dredge_exact_library_count_is_eligible() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-3", "Dredge Three", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        // Exactly 3 cards in library.
        for i in 0..3 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    let (state, events) = pass_all(state, &[p1, p2]);

    // Dredge MUST be offered when library has exactly N cards.
    let choice_event = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::DredgeChoiceRequired { player, options }
            if *player == p1 && options.iter().any(|(_, n)| *n == 3)
        )
    });
    assert!(
        choice_event.is_some(),
        "CR 702.52b: dredge should be offered when library.len() == N (exactly 3). \
         Events: {:?}",
        events
    );

    // Find dredge card option.
    let (dredge_card_id, _) = events
        .iter()
        .find_map(|e| {
            if let GameEvent::DredgeChoiceRequired { player, options } = e {
                if *player == p1 {
                    options.first().copied()
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("DredgeChoiceRequired event expected");

    // Player dredges — mills 3 cards (library becomes empty).
    let (state, dredge_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge_card_id),
        },
    )
    .unwrap();

    // Dredged event emitted.
    assert!(
        dredge_events
            .iter()
            .any(|e| matches!(e, GameEvent::Dredged { player, milled: 3, .. } if *player == p1)),
        "CR 702.52b: Dredged event expected with milled=3 when library has exactly 3 cards"
    );

    // No PlayerLost event (we dredged, didn't draw from empty library).
    assert!(
        !dredge_events
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerLost { player, .. } if *player == p1)),
        "CR 702.52b: dredging from a library that becomes empty should NOT cause PlayerLost \
         (player didn't draw, they dredged)"
    );

    // Library is empty.
    let lib_count = count_in_zone(&state, ZoneId::Library(p1));
    assert_eq!(
        lib_count, 0,
        "CR 702.52b: library should be empty after dredging 3 from 3-card library"
    );
}

// ── Test 9: Invalid ChooseDredge — card not in graveyard ────────────────────

#[test]
/// Error handling: ChooseDredge with a card that is not in the player's graveyard
/// should return an error.
fn test_dredge_invalid_command_card_not_in_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-3", "Dredge Three", 3)]);

    // Dredge card is in the HAND (not graveyard) — used only to get the ObjectId.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    state.turn.is_first_turn_of_game = false;

    let dredge_card_id = find_object(&state, "Dredge Three");

    // Attempt to dredge a card that is not in the graveyard.
    let result = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge_card_id),
        },
    );

    assert!(
        result.is_err(),
        "ChooseDredge with card not in graveyard should return an error"
    );
}

// ── Test 10: Declining dredge does not re-offer dredge ───────────────────────

#[test]
/// CR 702.52a — "you may instead" — after the player declines dredge once, the
/// normal draw proceeds without offering dredge again. `draw_card_skipping_dredge`
/// bypasses the dredge check to avoid an infinite loop of choices.
fn test_dredge_decline_does_not_reoffer() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-3", "Dredge Three", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    // Advance to DredgeChoiceRequired.
    let (state, _events) = pass_all(state, &[p1, p2]);

    // Player declines dredge (card: None).
    let (state, decline_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();

    // CardDrawn must be emitted (normal draw proceeded).
    let drawn = decline_events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
    assert!(
        drawn,
        "CR 702.52a: after declining dredge, CardDrawn should be emitted. \
         Events: {:?}",
        decline_events
    );

    // DredgeChoiceRequired must NOT be emitted again after decline.
    assert!(
        !decline_events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { .. })),
        "CR 702.52a: declining dredge must NOT re-offer dredge — that would create \
         an infinite loop of choices. Events: {:?}",
        decline_events
    );

    // Exactly one card in hand (the drawn card, not the dredge card).
    let hand_count = count_in_zone(&state, ZoneId::Hand(p1));
    assert_eq!(
        hand_count, 1,
        "after declining dredge and drawing normally, exactly one card should be in hand"
    );

    // The dredge card is still in the graveyard.
    assert!(
        object_in_zone(&state, "Dredge Three", ZoneId::Graveyard(p1)),
        "CR 702.52a: after declining dredge, the dredge card should still be in the graveyard"
    );
}

// ── Test 12: Dredge offered during effect-based draw (not just draw step) ──

#[test]
/// CR 702.52a — "if you would draw a card" includes draws from any source, not
/// only the draw step. This test exercises the `effects/mod.rs::draw_one_card`
/// code path (DrawCards effect) to ensure DredgeChoiceRequired fires there too.
/// Source: Finding 1 — plan test #6.
fn test_dredge_during_effect_draw_not_just_draw_step() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{Effect, EffectAmount, PlayerTarget};

    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-2", "Dredge Two", 2)]);

    // Build a state in Main phase (not draw step) so we exercise the effect path.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        // Dredge card in graveyard.
        .object(
            ObjectSpec::card(p1, "Dredge Two")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-2".to_string()))
                .with_keyword(KeywordAbility::Dredge(2)),
        )
        // 4 library cards — enough to dredge (need >= 2).
        .object(ObjectSpec::card(p1, "Library Card 0").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 3").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    state.turn.is_first_turn_of_game = false;
    state.turn.priority_holder = Some(p1);

    // Get the dredge card's ObjectId before executing the effect.
    let dredge_card_id = find_object(&state, "Dredge Two");

    // Execute a DrawCards(1) effect — this goes through effects/mod.rs::draw_one_card.
    let draw_effect = Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
    };
    let mut ctx = EffectContext::new(p1, ObjectId(999), vec![]);

    let events = execute_effect(&mut state, &draw_effect, &mut ctx);

    // CR 702.52a: DredgeChoiceRequired must be emitted via the effect-based draw path.
    let choice_event = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::DredgeChoiceRequired { player, .. } if *player == p1
        )
    });
    assert!(
        choice_event.is_some(),
        "CR 702.52a: DredgeChoiceRequired must be emitted via effect-based draw (draw_one_card). \
         This tests the effects/mod.rs path, not the draw step path. Events: {:?}",
        events
    );

    // No CardDrawn yet — the draw is paused for the player's choice.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 702.52a: CardDrawn must NOT be emitted before player chooses whether to dredge \
         (effect-based draw path)"
    );

    let lib_before = count_in_zone(&state, ZoneId::Library(p1));
    let hand_before = count_in_zone(&state, ZoneId::Hand(p1));

    // Player chooses to dredge — send ChooseDredge via process_command.
    let (state, dredge_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge_card_id),
        },
    )
    .unwrap();

    // Dredged event emitted (2 cards milled, dredge card returned to hand).
    assert!(
        dredge_events
            .iter()
            .any(|e| matches!(e, GameEvent::Dredged { player, milled: 2, .. } if *player == p1)),
        "CR 702.52a: Dredged event expected with milled=2 after effect-based draw dredge. \
         Events: {:?}",
        dredge_events
    );

    // Library decreased by 2 (the mill).
    let lib_after = count_in_zone(&state, ZoneId::Library(p1));
    assert_eq!(
        lib_after,
        lib_before - 2,
        "CR 702.52a: library should have 2 fewer cards after Dredge 2 (effect-based draw path)"
    );

    // Hand gained the dredge card.
    let hand_after = count_in_zone(&state, ZoneId::Hand(p1));
    assert_eq!(
        hand_after,
        hand_before + 1,
        "CR 702.52a: hand should have the dredge card after dredging (effect-based draw path)"
    );

    assert!(
        object_in_zone(&state, "Dredge Two", ZoneId::Hand(p1)),
        "CR 702.52a: Dredge Two should be in hand after dredging via effect-based draw"
    );
}

// ── Test 13: Freshly-milled dredge card available for next draw ─────────────

#[test]
/// CR 614.11a — "if an effect replaces a draw within a sequence of card draws,
/// all actions required by the replacement are completed, if possible, before
/// resuming the sequence." A dredge card milled by the first dredge can be used
/// to replace the second draw in a two-draw sequence.
/// Source: Finding 2 — plan test #7. Ruling: "another card with a dredge ability
/// (including one that was milled by the first dredge ability) may be used to
/// replace the second draw." (2024-01-12, multiple dredge cards)
fn test_dredge_milled_card_available_for_second_draw() {
    use mtg_engine::rules::turn_actions::draw_card;

    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        dredge_card_def("dredge-3", "Dredge Three", 3),
        dredge_card_def("dredge-2", "Dredge Two", 2),
    ]);

    // Setup:
    // - "Dredge Three" (Dredge 3) is in p1's graveyard.
    // - Library (top to bottom): [Library Card 0, Library Card 1, Dredge Two, Library Card 3, Library Card 4]
    //   (push_back order; top() returns the LAST element, so we push in reverse of desired order)
    //   In GameStateBuilder, objects are added via push_back. Zone::top() returns last element.
    //   So to get Dredge Two at position 3 from top (index 2 of a 0-indexed from-top view):
    //   push order: Library Card 4, Library Card 3, Dredge Two, Library Card 1, Library Card 0
    //   After push: [Card4, Card3, DredgeTwo, Card1, Card0] — top() = Card0, second = Card1, third = DredgeTwo
    //   Dredging 3 from this library mills Card0, Card1, DredgeTwo → DredgeTwo lands in graveyard!
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        // Dredge Three in graveyard (used for first dredge).
        .object(
            ObjectSpec::card(p1, "Dredge Three")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        )
        // Library bottom-to-top order (push_back; top = last added).
        // Want top-to-bottom: Card0, Card1, DredgeTwo, Card3, Card4.
        // Push order (bottom first): Card4, Card3, DredgeTwo, Card1, Card0.
        .object(ObjectSpec::card(p1, "Library Card 4").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 3").in_zone(ZoneId::Library(p1)))
        .object(
            ObjectSpec::card(p1, "Dredge Two")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("dredge-2".to_string()))
                .with_keyword(KeywordAbility::Dredge(2)),
        )
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 0").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    state.turn.is_first_turn_of_game = false;
    state.turn.priority_holder = Some(p1);

    // Verify initial setup: Dredge Three in graveyard, 5 cards in library.
    assert!(
        object_in_zone(&state, "Dredge Three", ZoneId::Graveyard(p1)),
        "setup: Dredge Three should be in graveyard"
    );
    assert_eq!(
        count_in_zone(&state, ZoneId::Library(p1)),
        5,
        "setup: library should have 5 cards"
    );
    assert!(
        object_in_zone(&state, "Dredge Two", ZoneId::Library(p1)),
        "setup: Dredge Two should be in library (not graveyard yet)"
    );

    // ── First draw: DredgeChoiceRequired fires for Dredge Three. ──
    let events1 = draw_card(&mut state, p1).unwrap();

    let (dredge3_id, _) = events1
        .iter()
        .find_map(|e| {
            if let GameEvent::DredgeChoiceRequired { player, options } = e {
                if *player == p1 {
                    options.first().copied()
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("CR 702.52a: first draw should emit DredgeChoiceRequired for Dredge Three");

    // Player uses Dredge Three: mills top 3 cards (Card0, Card1, Dredge Two → graveyard).
    let (mut state, dredge3_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge3_id),
        },
    )
    .unwrap();

    // Verify Dredge Two is now in the graveyard (milled by the dredge).
    assert!(
        object_in_zone(&state, "Dredge Two", ZoneId::Graveyard(p1)),
        "CR 614.11a: Dredge Two should be in graveyard after being milled by the Dredge Three \
         replacement. Events: {:?}",
        dredge3_events
    );

    // Verify Dredge Three is in hand.
    assert!(
        object_in_zone(&state, "Dredge Three", ZoneId::Hand(p1)),
        "Dredge Three should be in hand after dredging"
    );

    // ── Second draw: DredgeChoiceRequired must include the newly-milled Dredge Two. ──
    let events2 = draw_card(&mut state, p1).unwrap();

    let options2 = events2.iter().find_map(|e| {
        if let GameEvent::DredgeChoiceRequired { player, options } = e {
            if *player == p1 {
                Some(options.clone())
            } else {
                None
            }
        } else {
            None
        }
    });

    assert!(
        options2.is_some(),
        "CR 614.11a: second draw should also emit DredgeChoiceRequired (Dredge Two is now \
         in graveyard after being milled by the first dredge). Events: {:?}",
        events2
    );

    let options2 = options2.unwrap();

    // The newly-milled Dredge Two should be among the options.
    // Library has 2 cards left (Card3, Card4) — Dredge Two needs >= 2, so it's eligible.
    let dredge_two_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Dredge Two" && o.zone == ZoneId::Graveyard(p1));
    let dredge_two_id = dredge_two_obj
        .map(|o| o.id)
        .expect("Dredge Two should be in graveyard for second draw check");

    assert!(
        options2
            .iter()
            .any(|(id, n)| *id == dredge_two_id && *n == 2),
        "CR 614.11a: second draw's DredgeChoiceRequired must include the freshly-milled \
         Dredge Two (dredge N=2). Options: {:?}",
        options2
    );
}

// ── Test 11: Dredge not offered when graveyard is empty ────────────────────

#[test]
/// CR 702.52a — Dredge functions only while in the graveyard. No graveyard cards
/// with Dredge means normal draw proceeds without any choice.
fn test_dredge_no_graveyard_no_choice() {
    let p1 = p(1);
    let p2 = p(2);

    // No dredge cards at all.
    let registry = CardRegistry::new(vec![]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    let (_, events) = pass_all(state, &[p1, p2]);

    // No dredge choice.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { .. })),
        "CR 702.52a: no dredge choice should be offered when graveyard has no dredge cards"
    );

    // Normal draw proceeds.
    let drawn = events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
    assert!(
        drawn,
        "normal draw should proceed when no dredge is available"
    );
}
