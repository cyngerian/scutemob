//! PB-DX2 — gate the resolution-time commands nothing gates.
//!
//! `memory/primitives/pb-plan-DX2.md` is authoritative for what each test pins.
//!
//! Before this batch, `Command::ChooseDredge` had NO pending-state gate:
//! `card: None` drew a free card for any player at any time (bypassing the
//! pre-PB-DX2 decline path, which validated nothing beyond
//! has_lost/has_conceded), and `card: Some(x)` dredged at will regardless of
//! whether a draw was ever outstanding. `Command::KeepHand` also validated
//! only the COUNT of `cards_to_bottom`, not that the named objects were
//! actually in the sender's hand — a malformed or hostile command could
//! bottom a permanent from the battlefield, a card from a graveyard, or a
//! card from ANOTHER PLAYER'S HAND. This batch:
//!   1. Records an outstanding draw (`PendingDraw`) at the dredge-offer site
//!      and requires-and-consumes it in `handle_choose_dredge` (CR 702.52a).
//!   2. Stops a multi-draw sequence at a dredge offer and resumes the
//!      remaining draws after the answer (CR 614.11a / 121.2).
//!   3. Adds a per-entry hand-zone guard to `handle_keep_hand` (CR 103.5).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, GameStateError, KeywordAbility,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, TypeLine, ZoneId,
};

// ── Helpers (copied from `tests/mechanics_a_d/dredge.rs`, SR-9a — do not
// `mod`-import across integration-test targets) ────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn object_in_zone(state: &GameState, name: &str, zone: ZoneId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == zone)
}

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state.objects().values().filter(|o| o.zone == zone).count()
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

/// Build a state ready for the draw step draw to fire (mirrors
/// `dredge.rs::build_upkeep_state`).
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
    state.turn_mut().is_first_turn_of_game = false;
    state.turn_mut().priority_holder = Some(p1);
    state
}

/// A state with a dredge card in `p1`'s graveyard and enough library to
/// dredge, sitting at `PreCombatMain` with NO draw having been attempted
/// (mirrors `dredge.rs` test 9's "no offer" fixture).
fn build_no_offer_state(p1: PlayerId, p2: PlayerId, dredge_n: u32) -> (GameState, ObjectId) {
    let registry = CardRegistry::new(vec![dredge_card_def(
        "dredge-dx2",
        "Dredge DX2 Test Card",
        dredge_n,
    )]);
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Dredge DX2 Test Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2".to_string()))
                .with_keyword(KeywordAbility::Dredge(dredge_n)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    // Library needs >= dredge_n cards (CR 702.52b).
    for i in 0..(dredge_n as usize + 2) {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Library Filler {}", i)).in_zone(ZoneId::Library(p1)));
    }
    let mut state = builder.build().unwrap();
    state.turn_mut().priority_holder = Some(p1);
    state.turn_mut().is_first_turn_of_game = false;
    let dredge_card_id = find_object(&state, "Dredge DX2 Test Card");
    (state, dredge_card_id)
}

// ── T1 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a — `Command::ChooseDredge { card: None }` with NO draw outstanding
/// must be rejected. Before PB-DX2 this was accepted unconditionally (the
/// pre-PB-DX2 decline path validated only has_lost/has_conceded) and drew a
/// free card for any player at any time (plan §1 P1).
fn test_dx2_choose_dredge_none_without_offer_is_a_free_card_today() {
    let p1 = p(1);
    let p2 = p(2);
    let (state, _dredge_id) = build_no_offer_state(p1, p2, 3);
    let hand_before = count_in_zone(&state, ZoneId::Hand(p1));

    let result = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    );

    match result {
        Err(GameStateError::InvalidCommand(_)) => {}
        Err(other) => panic!(
            "expected GameStateError::InvalidCommand (CR 702.52a: no draw outstanding), got {:?}",
            other
        ),
        Ok((state, events)) => {
            let hand_after = count_in_zone(&state, ZoneId::Hand(p1));
            panic!(
                "CR 702.52a: ChooseDredge{{None}} with no draw outstanding must be \
                 rejected, but it succeeded and p1's hand went from {} to {} \
                 cards (a free card). Events: {:?}",
                hand_before, hand_after, events
            );
        }
    }
}

// ── T2 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a / 702.52b — `Command::ChooseDredge { card: Some(x) }` with NO draw
/// outstanding must be rejected. Before PB-DX2 this dredged at will — milled N
/// cards and returned the dredge card to hand — regardless of whether a draw
/// had ever been offered (plan §1 P1).
fn test_dx2_choose_dredge_some_without_offer_dredges_at_will_today() {
    let p1 = p(1);
    let p2 = p(2);
    let (state, dredge_id) = build_no_offer_state(p1, p2, 3);
    let hand_before = count_in_zone(&state, ZoneId::Hand(p1));
    let grave_before = count_in_zone(&state, ZoneId::Graveyard(p1));
    let lib_before = count_in_zone(&state, ZoneId::Library(p1));

    let result = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge_id),
        },
    );

    match result {
        Err(GameStateError::InvalidCommand(_)) => {}
        Err(other) => panic!(
            "expected GameStateError::InvalidCommand (CR 702.52a: no draw outstanding), got {:?}",
            other
        ),
        Ok((state, events)) => {
            let hand_after = count_in_zone(&state, ZoneId::Hand(p1));
            let grave_after = count_in_zone(&state, ZoneId::Graveyard(p1));
            let lib_after = count_in_zone(&state, ZoneId::Library(p1));
            panic!(
                "CR 702.52a: ChooseDredge{{Some}} with no draw outstanding must be \
                 rejected, but it succeeded. hand {}->{}, graveyard {}->{}, \
                 library {}->{}. Events: {:?}",
                hand_before, hand_after, grave_before, grave_after, lib_before, lib_after, events
            );
        }
    }
}

// ── T5 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 614.11a / 121.2 — a multi-draw sequence (`Effect::DrawCards { count: 3 }`)
/// for a player with a dredge card in their graveyard must stop at the FIRST
/// dredge offer (exactly ONE `DredgeChoiceRequired`, ZERO `CardDrawn`), record
/// a `PendingDraw` with `remaining == 2`, and resume the rest of the sequence
/// once answered. Before PB-DX2, `draw_cards_for_player`'s break set did not
/// include `DredgeOffered`, so the loop iterated again, re-offered dredge for
/// the SAME card, and destroyed the other draws (plan §1 P3).
fn test_dx2_multi_draw_sequence_stops_at_the_dredge_offer() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dredge_card_def("dredge-dx2", "Dredge DX2 Card", 3)]);
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Dredge DX2 Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
    // 10 library cards -- enough to dredge (>=3) and enough for 3 draws.
    for i in 0..10 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }
    let mut state = builder.build().unwrap();

    let effect = Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(3),
    };
    let mut ctx = EffectContext::new(p1, ObjectId(999), vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    let dredge_offer_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1))
        .count();
    let card_drawn_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();

    assert_eq!(
        dredge_offer_count, 1,
        "CR 614.11a: the sequence must stop at the FIRST dredge offer -- exactly \
         one DredgeChoiceRequired, not one-per-remaining-draw. Events: {:?}",
        events
    );
    assert_eq!(
        card_drawn_count, 0,
        "no card should be drawn yet -- the offer replaces the first draw. \
         Events: {:?}",
        events
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "a PendingDraw entry should be recorded for the outstanding dredge offer"
    );
    assert_eq!(
        state.pending_draws()[0].remaining,
        2,
        "two further draws remain in the sequence"
    );

    // Decline: should complete all 3 draws.
    let (state, decline_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();
    let total_drawn = decline_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        total_drawn, 3,
        "CR 614.11a: declining the offer must complete the full sequence -- 3 \
         cards drawn, not 1 (the other 2 must not be destroyed). Events: {:?}",
        decline_events
    );
    assert!(
        state.pending_draws().is_empty(),
        "the pending draw obligation should be fully discharged"
    );
}

// ── T10 ─────────────────────────────────────────────────────────────────────

#[test]
/// CR 103.5 — "then puts a number of THOSE CARDS ... on the bottom of their
/// library" -- "those cards" are the cards of the hand just drawn via
/// mulligan, not any object in the game. Before PB-DX2, `handle_keep_hand`
/// validated only the COUNT of `cards_to_bottom`, so a card in ANOTHER
/// PLAYER'S HAND could be named and moved to p1's library bottom.
fn test_dx2_keep_hand_rejects_a_card_in_another_players_hand() {
    let p1 = p(1);
    let p2 = p(2);
    let mut builder = GameStateBuilder::four_player().active_player(p1);
    for i in 0..20 {
        builder =
            builder.object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Library(p1)));
    }
    builder = builder.object(ObjectSpec::card(p2, "P2 Secret Card").in_zone(ZoneId::Hand(p2)));
    let state = builder.build().unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    assert_eq!(state.players().get(&p1).unwrap().mulligan_count, 2);

    let p2_card = find_object(&state, "P2 Secret Card");

    let result = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![p2_card],
        },
    );

    match result {
        Err(GameStateError::InvalidCommand(_)) => {}
        Err(other) => panic!(
            "expected GameStateError::InvalidCommand (CR 103.5: not p1's hand card), got {:?}",
            other
        ),
        Ok((state, _events)) => {
            assert!(
                object_in_zone(&state, "P2 Secret Card", ZoneId::Hand(p2)),
                "CR 103.5: KeepHand must not be able to bottom a card from \
                 another player's hand, but it succeeded and moved it"
            );
            panic!(
                "CR 103.5: KeepHand naming a card in p2's hand must be rejected, \
                 but process_command returned Ok"
            );
        }
    }
}

// ── T4 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a / 614.11a — the draw-step dredge offer records a `PendingDraw`
/// entry for the outstanding draw. Before PB-DX2, nothing was recorded at all.
fn test_dx2_dredge_offer_records_a_pending_draw() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-dx2-t4", "Dredge T4 Card", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge T4 Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2-t4".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    let (state, events) = pass_all(state, &[p1, p2]);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1)),
        "expected DredgeChoiceRequired. Events: {:?}",
        events
    );

    assert_eq!(
        state.pending_draws().len(),
        1,
        "a PendingDraw entry should be recorded for the outstanding dredge offer"
    );
    assert_eq!(state.pending_draws()[0].player, p1);
    assert_eq!(
        state.pending_draws()[0].remaining,
        0,
        "the draw step draws exactly one card -- no further draws in the sequence"
    );
    assert!(
        state.pending_draws()[0].already_applied.is_empty(),
        "no other WouldDraw replacement applied to this draw"
    );
}

// ── T7 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 614.11a + 104.4b — a second draw for a player who already has an
/// outstanding dredge offer FOLDS into the existing entry (`remaining` grows)
/// rather than pushing a second entry (plan §4.2's fold guard). Declining the
/// single offer must then discharge the WHOLE conserved obligation.
fn test_dx2_dredge_offers_do_not_stack_entries() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-dx2-t7", "Dredge T7 Card", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge T7 Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2-t7".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        for i in 0..8 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    // Draw-step offer -- unanswered.
    let (mut state, _events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.pending_draws().len(), 1);
    assert_eq!(state.pending_draws()[0].remaining, 0);

    // A second, unrelated draw sequence for the SAME player while the offer
    // stands (e.g. `Effect::DrawCards { count: 2 }` from a spell).
    let effect = Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(2),
    };
    let mut ctx = EffectContext::new(p1, ObjectId(998), vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    // CR 616.1e: each draw is separately replaceable, so the SECOND sequence's
    // first draw is offered dredge again (its own `DredgeChoiceRequired` event
    // fires) -- what must NOT happen is a second `PendingDraw` entry (plan
    // §4.2's fold guard is about the STORED entry, not the emitted event).
    let dredge_offer_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DredgeChoiceRequired { .. }))
        .count();
    assert_eq!(
        dredge_offer_count, 1,
        "the second sequence's first draw is offered dredge again (CR 616.1e); \
         it must fold into the EXISTING PendingDraw entry, not push a second \
         one. Events: {:?}",
        events
    );

    assert_eq!(
        state.pending_draws().len(),
        1,
        "still exactly one entry -- the fold guard must not create a second"
    );
    assert_eq!(
        state.pending_draws()[0].remaining,
        2,
        "the folded entry must conserve BOTH the original draw-step draw and the \
         2 further draws from the effect (1 + 2 = 3 total; 1 already accounted \
         for by the offer itself, so 2 remain)"
    );

    // Declining the single offer must complete the WHOLE conserved obligation:
    // draw-step draw (1) + effect draws (2) = 3 cards total.
    let (state, decline_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();
    let total_drawn = decline_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        total_drawn, 3,
        "CR 614.11a + 104.4b: declining must discharge the FULL conserved \
         obligation -- 3 cards, not 1. Events: {:?}",
        decline_events
    );
    assert!(state.pending_draws().is_empty());
}

// ── T11 ─────────────────────────────────────────────────────────────────────

#[test]
/// CR 103.5 — as T10, but naming a permanent p1 CONTROLS on the battlefield
/// rather than a card in another player's hand. Before PB-DX2 this too would
/// have been "moved" to the bottom of p1's library.
fn test_dx2_keep_hand_rejects_a_battlefield_permanent() {
    let p1 = p(1);
    let p2 = p(2);
    let mut builder = GameStateBuilder::four_player().active_player(p1);
    for i in 0..20 {
        builder =
            builder.object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Library(p1)));
    }
    builder = builder.object(
        ObjectSpec::creature(p1, "P1 Battlefield Bear", 2, 2).in_zone(ZoneId::Battlefield),
    );
    let state = builder.build().unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    assert_eq!(state.players().get(&p1).unwrap().mulligan_count, 2);

    let bear = find_object(&state, "P1 Battlefield Bear");

    let result = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![bear],
        },
    );

    match result {
        Err(GameStateError::InvalidCommand(_)) => {}
        Err(other) => panic!(
            "expected GameStateError::InvalidCommand (CR 103.5: not a hand card), got {:?}",
            other
        ),
        Ok((state, _events)) => {
            assert!(
                object_in_zone(&state, "P1 Battlefield Bear", ZoneId::Battlefield),
                "CR 103.5: KeepHand must not be able to bottom a battlefield \
                 permanent, but it succeeded and moved it"
            );
            panic!(
                "CR 103.5: KeepHand naming a battlefield permanent must be \
                 rejected, but process_command returned Ok"
            );
        }
    }
    let _ = p2;
}

// ── T12 ─────────────────────────────────────────────────────────────────────

#[test]
/// CR 103.5 / CR 400.7 — `cards_to_bottom: [a, a]` (the same object named
/// twice) must be rejected, and the error MESSAGE must name the duplicate
/// (not merely `is_err()` -- see plan §8.1: today `[a, a]` already errors via
/// `ObjectNotFound` because the first move mints a new ObjectId (CR 400.7) so
/// the second lookup misses. An `is_err()`-only assertion would therefore be
/// VACUOUS -- it passes both before and after this fix for the wrong reason.
/// This test asserts on the message to distinguish "rejected as a duplicate
/// intent" (post-fix) from "rejected because the object already moved"
/// (pre-fix, an accident of implementation order, not CR 103.5 -- and it
/// would additionally have already bottomed ONE copy of the card by the time
/// it errors, which the pre-fix code never validated against).
fn test_dx2_keep_hand_rejects_duplicate_ids() {
    let p1 = p(1);
    let mut builder = GameStateBuilder::four_player().active_player(p1);
    for i in 0..20 {
        builder =
            builder.object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Library(p1)));
    }
    let state = builder.build().unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    assert_eq!(
        state.players().get(&p1).unwrap().mulligan_count,
        3,
        "3 mulligans -> required_bottom == 2"
    );

    let hand_ids = state.zone(&ZoneId::Hand(p1)).unwrap().object_ids();
    let a = hand_ids[0];

    let result = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![a, a],
        },
    );

    match result {
        Err(GameStateError::InvalidCommand(msg)) => {
            assert!(
                msg.contains("twice"),
                "CR 103.5: the error message must name the duplicate as a \
                 duplicate, not accidentally succeed via ObjectNotFound on the \
                 second move. Message: {:?}",
                msg
            );
        }
        Err(other) => panic!(
            "expected GameStateError::InvalidCommand naming the duplicate, got {:?}",
            other
        ),
        Ok(_) => panic!("KeepHand with a duplicate id must be rejected, got Ok"),
    }
}

// ── T13 ─────────────────────────────────────────────────────────────────────

#[test]
/// Non-regression: CR 103.5 / 103.5c — `KeepHand` naming the player's OWN hand
/// cards must still succeed after the guard is added.
fn test_dx2_keep_hand_still_accepts_the_players_own_hand_cards() {
    let p1 = p(1);
    let mut builder = GameStateBuilder::four_player().active_player(p1);
    for i in 0..20 {
        builder =
            builder.object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Library(p1)));
    }
    let state = builder.build().unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    assert_eq!(state.players().get(&p1).unwrap().mulligan_count, 2);

    let card_to_bottom = state.zone(&ZoneId::Hand(p1)).unwrap().object_ids()[0];
    let card_name = state.object(card_to_bottom).unwrap().characteristics.name.clone();

    let (state, events) = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![card_to_bottom],
        },
    )
    .unwrap();

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::MulliganKept { player, .. } if *player == p1)),
        "MulliganKept should be emitted for a legitimate KeepHand"
    );
    // CR 400.7: the move mints a NEW ObjectId, so look up by name+zone rather
    // than the pre-move id.
    assert!(
        object_in_zone(&state, &card_name, ZoneId::Library(p1)),
        "the named card should be in p1's library after KeepHand bottoms it"
    );
}
