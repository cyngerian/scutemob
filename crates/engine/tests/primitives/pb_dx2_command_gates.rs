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
    Effect, EffectAmount, EffectDuration, GameEvent, GameState, GameStateBuilder, GameStateError,
    KeywordAbility, ObjectId, ObjectSpec, PlayerFilter, PlayerId, PlayerTarget, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger, Step, TypeLine, ZoneId,
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
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Filler {}", i)).in_zone(ZoneId::Library(p1)),
        );
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
///
/// **Decline section REWRITTEN by PB-DX23 §3 Q3 (closing `OOS-DX2-2`).** The
/// dredge card is never actually dredged away in this fixture — it is only
/// ever DECLINED — so it stays eligible in the graveyard for every remaining
/// draw of the sequence. Before PB-DX23, `perform_remaining_draws` hard-coded
/// `offer_dredge: false` for the whole tail, so one decline drained all 3
/// draws in a single `CardDrawn`-only burst. CR 121.2 makes "draw three"
/// three SEPARATE draws and CR 614.11a/121.6b say the replacement's actions
/// complete and THEN the sequence resumes — so each resumed draw is its own
/// fresh "would draw" event and is independently dredge-offerable. With a
/// dredge card that is never removed from the graveyard, that means each of
/// the 3 draws is offered and declined in turn: this test now drives that to
/// completion with one `ChooseDredge { None }` per remaining draw rather than
/// asserting the whole sequence completes off a single decline.
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

    // PB-DX23 §3 Q3 / OOS-DX2-2: decline once per remaining draw. The dredge
    // card is never actually dredged in this fixture, so it stays eligible
    // and each resumed draw (a DIFFERENT draw event, CR 121.2) is offered
    // dredge again -- decline drains the sequence one draw at a time rather
    // than all at once.
    let mut total_drawn = 0usize;
    let mut rounds = 0usize;
    while !state.pending_draws().is_empty() {
        rounds += 1;
        assert!(
            rounds <= 3,
            "the 3-draw sequence must fully discharge within 3 decline rounds \
             -- exceeding that means a draw is being lost or an offer is \
             looping instead of terminating"
        );
        let (next_state, decline_events) = process_command(
            state,
            Command::ChooseDredge {
                player: p1,
                card: None,
            },
        )
        .unwrap();
        state = next_state;
        total_drawn += decline_events
            .iter()
            .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
            .count();
    }
    assert_eq!(
        total_drawn, 3,
        "CR 614.11a / 121.2: across the whole decline chain, the full \
         sequence must complete -- 3 cards drawn total, none destroyed, none \
         double-counted."
    );
    assert_eq!(
        rounds, 3,
        "CR 121.2: with a dredge card that is never removed from the \
         graveyard, each of the 3 draws is independently offered and \
         declined -- exactly 3 decline rounds, not 1 (that would be the \
         pre-PB-DX23 OOS-DX2-2 tail-immunity defect)."
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
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Library(p1)));
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
/// Fix-cycle rewrite (Finding 1, `pb-review-DX2.md`). The original T7 pinned a
/// FOLD design (`entry.remaining += 1 + remaining_after`) that the fix cycle
/// replaced: a fold let the obligation accumulate WITHOUT BOUND across turns
/// and be cashed in a single command at an arbitrary later moment (the
/// review's scenario: seven cards drawn during another player's
/// declare-blockers step). The new behaviour DISCHARGES the earlier entry
/// (as though declined) the instant a second draw event arrives for the same
/// player, THEN records a fresh entry for the new offer -- so the total draw
/// count is still conserved (CR 614.11a), just split across two moments
/// instead of banked into one, and `pending_draws` never holds more than the
/// most recently offered draw's own remainder.
///
/// **Decline section REWRITTEN by PB-DX23 §3 Q3 (closing `OOS-DX2-2`), same
/// reason as `test_dx2_multi_draw_sequence_stops_at_the_dredge_offer`**: the
/// dredge card is never dredged away in this fixture, so it stays eligible
/// for every remaining draw of the second sequence and each resumed draw
/// (CR 121.2) is independently offered dredge again. A single decline no
/// longer drains the whole remainder; this test now drives the second
/// sequence to completion with one `ChooseDredge { None }` per remaining
/// draw. The end-to-end conservation invariant (3 total cards drawn, none
/// destroyed, none banked past its own offer's window) is unchanged and is
/// re-asserted across the WHOLE decline chain rather than a single call.
fn test_dx2_second_dredge_offer_discharges_the_first_and_conserves_draws() {
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

    // The FIRST (draw-step) offer is auto-discharged as a decline the moment
    // this second draw is processed -- CR 702.52a: declining is always legal,
    // "you may instead" supplies the default -- which draws it normally
    // BEFORE the second sequence's own offer is even evaluated.
    let discharge_drawn = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        discharge_drawn, 1,
        "the stale draw-step entry must be discharged (drawn), not destroyed \
         and not silently folded away. Events: {:?}",
        events
    );

    // The second sequence's own first draw is ALSO offered dredge (CR 616.1e:
    // each draw is separately replaceable) -- exactly one NEW offer.
    let dredge_offer_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DredgeChoiceRequired { .. }))
        .count();
    assert_eq!(
        dredge_offer_count, 1,
        "the second sequence's first draw is offered dredge again (CR 616.1e). \
         Events: {:?}",
        events
    );

    assert_eq!(
        state.pending_draws().len(),
        1,
        "still exactly one entry -- discharge-then-push never leaves two"
    );
    assert_eq!(
        state.pending_draws()[0].remaining,
        1,
        "the NEW entry carries only the SECOND sequence's own remainder (draw \
         2 of 2 -- one further draw), NOT a sum with the discharged draw-step \
         entry. This is the accumulation bound (fix-cycle Finding 1): the \
         entry can never grow past a single offer's own remainder."
    );

    // PB-DX23 §3 Q3 / OOS-DX2-2: decline once per remaining draw of the
    // second sequence -- the dredge card is never dredged away, so each
    // resumed draw (a DIFFERENT draw event, CR 121.2) is offered again.
    let mut decline_drawn = 0usize;
    let mut decline_rounds = 0usize;
    while !state.pending_draws().is_empty() {
        decline_rounds += 1;
        assert!(
            decline_rounds <= 2,
            "the second sequence's 2-draw remainder must fully discharge \
             within 2 decline rounds -- exceeding that means a draw is being \
             lost or an offer is looping instead of terminating"
        );
        let (next_state, decline_events) = process_command(
            state,
            Command::ChooseDredge {
                player: p1,
                card: None,
            },
        )
        .unwrap();
        state = next_state;
        decline_drawn += decline_events
            .iter()
            .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
            .count();
    }
    assert_eq!(
        decline_drawn, 2,
        "CR 614.11a: across the whole decline chain, declining must complete \
         the SECOND sequence's remaining two draws."
    );
    assert_eq!(
        decline_rounds, 2,
        "CR 121.2: with a dredge card that is never removed from the \
         graveyard, each of the second sequence's 2 remaining draws is \
         independently offered and declined -- exactly 2 decline rounds, not \
         1 (that would be the pre-PB-DX23 OOS-DX2-2 tail-immunity defect)."
    );
    assert!(state.pending_draws().is_empty());

    // CONSERVATION, end to end: draw-step draw (1, auto-discharged) + the
    // second sequence's two draws (2, via decline) = 3 total. No draw was
    // ever destroyed and none was ever banked past its own offer's window.
    assert_eq!(
        discharge_drawn + decline_drawn,
        3,
        "total cards drawn across the whole scenario must be exactly 3"
    );
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
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Library(p1)));
    }
    builder = builder
        .object(ObjectSpec::creature(p1, "P1 Battlefield Bear", 2, 2).in_zone(ZoneId::Battlefield));
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
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Library(p1)));
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
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Card {}", i)).in_zone(ZoneId::Library(p1)));
    }
    let state = builder.build().unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    assert_eq!(state.players().get(&p1).unwrap().mulligan_count, 2);

    let card_to_bottom = state.zone(&ZoneId::Hand(p1)).unwrap().object_ids()[0];
    let card_name = state
        .object(card_to_bottom)
        .unwrap()
        .characteristics
        .name
        .clone();

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

// ── T3 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a — the CONSUME half, distinct from T1's REQUIRE half: reach the
/// draw-step offer, answer it once (`ChooseDredge { None }` -> Ok), then send
/// the identical command AGAIN -> must now be rejected because the entry was
/// already consumed. Before PB-DX2 the second answer drew a SECOND card (no
/// gate at all, so nothing was ever consumed).
fn test_dx2_choose_dredge_is_consumed_by_its_answer() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-dx2-t3", "Dredge T3 Card", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge T3 Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2-t3".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    let (state, _events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.pending_draws().len(), 1);

    // First answer: consumes the entry.
    let (state, first_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();
    assert!(first_events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)));
    assert!(state.pending_draws().is_empty());

    // Second, identical answer: the entry is gone -- must be rejected.
    let result = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    );
    match result {
        Err(GameStateError::InvalidCommand(_)) => {}
        Err(other) => panic!("expected InvalidCommand on the re-send, got {:?}", other),
        Ok((state, events)) => {
            let hand_count = count_in_zone(&state, ZoneId::Hand(p1));
            panic!(
                "CR 702.52a: a second ChooseDredge with the entry already \
                 consumed must be rejected, but it succeeded (hand count now \
                 {}). Events: {:?}",
                hand_count, events
            );
        }
    }
}

// ── T6 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a + 614.11a — as T5, but choosing to DREDGE (`Some`) rather than
/// decline: one `Dredged` event plus the two remaining draws of the sequence
/// complete. Before PB-DX2 there was one `Dredged` and zero further draws.
fn test_dx2_dredge_then_remaining_draws_complete() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dredge_card_def("dredge-dx2-t6", "Dredge T6 Card", 3)]);
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Dredge T6 Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2-t6".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
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
    let mut ctx = EffectContext::new(p1, ObjectId(997), vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    let (dredge_id, _n) = events
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
        .expect("DredgeChoiceRequired expected");
    assert_eq!(state.pending_draws()[0].remaining, 2);

    let (state, dredge_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge_id),
        },
    )
    .unwrap();

    let dredged_count = dredge_events
        .iter()
        .filter(|e| matches!(e, GameEvent::Dredged { player, .. } if *player == p1))
        .count();
    let drawn_count = dredge_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        dredged_count, 1,
        "exactly one Dredged event. Events: {:?}",
        dredge_events
    );
    assert_eq!(
        drawn_count, 2,
        "CR 614.11a: the two remaining draws of the sequence must complete \
         after dredging. Events: {:?}",
        dredge_events
    );
    assert!(state.pending_draws().is_empty());
}

// ── T8 ──────────────────────────────────────────────────────────────────────

#[test]
/// Hard constraint (plan §4.5 / risk 1): an outstanding dredge `PendingDraw`
/// entry does NOT gate priority, SBAs or step advancement. Both players pass
/// priority repeatedly with the entry outstanding and unanswered, never
/// reaching p1's own next draw step: no error, no hang, no
/// `BlockingDecision`, and the entry survives identically. **Corrected
/// (R7, `pb-review-DX2.md`)**: this is NOT a claim that nothing ever
/// resolves the entry for the player in general -- the player's own next
/// draw DOES discharge it (PB-DX2's stale-entry discharge); this test's
/// fixture simply never reaches that draw.
fn test_dx2_unanswered_dredge_offer_does_not_deadlock() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-dx2-t8", "Dredge T8 Card", 3)]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge T8 Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2-t8".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    let (mut state, _events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.pending_draws().len(), 1);
    assert!(state.blocking_decision().is_none());
    // R7 re-review: capture the entry itself (not just the count) so the
    // final assertion can prove IDENTITY, not merely count. `len() == 1`
    // alone cannot distinguish "the same entry survived untouched" from "it
    // was discharged (by the player's own next draw) and replaced by a
    // fresh one" -- and after PB-DX2's fix cycle, a same-player draw DOES
    // resolve it (`perform_one_draw`'s stale-entry discharge). This loop
    // does not reach p1's next draw step (six `pass_all` rounds from
    // Upkeep), so identity is expected to hold here, but the message must
    // not overclaim a guarantee ("nothing else resolves it") the engine no
    // longer makes in general.
    let entry_before = state.pending_draws()[0].clone();

    // Pass priority repeatedly through several steps WITHOUT ever answering
    // and WITHOUT ever reaching p1's next draw step (which would discharge
    // this entry via PB-DX2's stale-entry discharge -- see
    // `test_dx2_second_dredge_offer_discharges_the_first_and_conserves_draws`
    // for that mechanism exercised directly).
    for _ in 0..6 {
        let (s, _ev) = pass_all(state, &[p1, p2]);
        state = s;
        assert!(
            state.blocking_decision().is_none(),
            "an outstanding dredge offer must never become a BlockingDecision"
        );
    }

    assert_eq!(
        state.pending_draws().len(),
        1,
        "exactly one entry must remain outstanding after passing priority \
         through several steps with no draw step reached for p1"
    );
    assert_eq!(
        state.pending_draws()[0],
        entry_before,
        "the SAME entry must survive byte-for-byte across steps that pass \
         priority without a draw for p1 -- proving identity, not merely a \
         stable count, distinguishes 'untouched' from 'discharged and \
         replaced' (R7, `pb-review-DX2.md`)"
    );
}

// ── T9 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a (§4.4 step 0) — a dead player's outstanding dredge entry is
/// discharged, not left to sit in the hash forever. 3 players so the game is
/// not over when p1 concedes.
fn test_dx2_dead_players_dredge_entry_is_discharged() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = PlayerId(3);

    let registry = CardRegistry::new(vec![dredge_card_def("dredge-dx2-t9", "Dredge T9 Card", 3)]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(
            ObjectSpec::card(p1, "Dredge T9 Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2-t9".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
    for i in 0..5 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }
    let mut state = builder.build().unwrap();
    state.turn_mut().is_first_turn_of_game = false;
    state.turn_mut().priority_holder = Some(p1);

    let (state, _events) = pass_all(state, &[p1, p2, p3]);
    assert_eq!(state.pending_draws().len(), 1);

    let (state, _) = process_command(state, Command::Concede { player: p1 }).unwrap();
    assert!(state.players().get(&p1).unwrap().has_conceded);

    let (state, events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();
    assert!(
        events.is_empty(),
        "a dead player's ChooseDredge should be a pure no-op: {:?}",
        events
    );
    assert!(
        state.pending_draws().is_empty(),
        "the dead player's entry must be discharged, not left outstanding"
    );
}

// ── T17 (fix cycle, Finding 4/case 1) ─────────────────────────────────────

#[test]
/// Trust boundary (Finding 4/case 1, `pb-review-DX2.md`): `handle_choose_dredge`'s
/// gate is `position(|pd| pd.player == player)` -- a player can only consume
/// THEIR OWN entry. This is the trust-boundary property of a trust-boundary
/// batch, and its sibling on `OrderReplacements` IS pinned
/// (`pb_dp5_pending_draw_choice.rs::test_dp5_order_replacements_rejects_non_affected_player`)
/// but this one was not, before the fix cycle.
fn test_dx2_choose_dredge_cannot_consume_another_players_entry() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def(
        "dredge-dx2-t17",
        "Dredge T17 Card",
        3,
    )]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge T17 Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2-t17".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        for i in 0..5 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    let (state, _events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.pending_draws().len(), 1);
    assert_eq!(state.pending_draws()[0].player, p1);
    let p2_hand_before = count_in_zone(&state, ZoneId::Hand(p2));

    // p2 attempts to consume p1's entry.
    let result = process_command(
        state,
        Command::ChooseDredge {
            player: p2,
            card: None,
        },
    );

    match result {
        Err(GameStateError::InvalidCommand(_)) => {}
        Err(other) => panic!(
            "expected InvalidCommand (CR 702.52a: no draw outstanding for p2), \
             got {:?}",
            other
        ),
        Ok((state, events)) => {
            let p2_hand_after = count_in_zone(&state, ZoneId::Hand(p2));
            panic!(
                "p2 must not be able to consume p1's entry, but succeeded: p2 \
                 hand {} -> {}. Events: {:?}",
                p2_hand_before, p2_hand_after, events
            );
        }
    }
}

// ── T18 (fix cycle, Finding 4/case 2, plan §3.3 row 2) ────────────────────

#[test]
/// CR 616.1e / plan §3.3 row 2 (Finding 4/case 2, `pb-review-DX2.md`):
/// `Command::OrderReplacements` landing on a DREDGE-originated `PendingDraw`
/// entry is a legal CR 616.1e choice, not a hole -- dredge is itself one of
/// the applicable `WouldDraw` replacements for the draw, and the player may
/// choose a DIFFERENT applicable replacement instead of dredging. Zero
/// coverage before the fix cycle.
fn test_dx2_order_replacements_can_answer_a_dredge_originated_entry() {
    let p1 = p(1);
    let p2 = p(2);
    let skip = ReplacementEffect {
        id: ReplacementId(950),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Dredge Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_keyword(KeywordAbility::Dredge(3)),
        )
        .object(ObjectSpec::card(p1, "Library Card 0").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(skip)
        .build()
        .unwrap();

    // Dredge is checked FIRST when offer_dredge: true, so the entry that
    // results here is dredge-originated, not NeedsChoice-originated.
    let events = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1)),
        "expected a dredge-originated entry. Events: {:?}",
        events
    );
    assert_eq!(state.pending_draws().len(), 1);

    let result = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(950)],
        },
    );
    assert!(
        result.is_ok(),
        "CR 616.1e: OrderReplacements naming a genuinely applicable \
         replacement must be accepted even though the entry is \
         dredge-originated, got {:?}",
        result.err()
    );
    let (state, order_events) = result.unwrap();
    assert!(
        order_events.iter().any(
            |e| matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(950))
        ),
        "the chosen SkipDraw replacement should be applied. Events: {:?}",
        order_events
    );
    assert!(
        state.pending_draws().is_empty(),
        "the SkipDraw chain terminates -- no further deferral"
    );
    assert!(
        object_in_zone(&state, "Dredge Card", ZoneId::Graveyard(p1)),
        "the dredge card was never dredged -- the player chose the OTHER \
         applicable replacement instead (CR 616.1e)"
    );
}

// ── T19 (fix cycle, Finding 4/case 3, plan §3.3 row 4; Finding 10) ────────

#[test]
/// CR 616.1e / plan §3.3 row 4 (Finding 4/case 3 and Finding 10,
/// `pb-review-DX2.md`): `Command::ChooseDredge { Some }` landing on a
/// NeedsChoice-originated entry (reached via a decline that re-defers because
/// other WouldDraw replacements are still applicable) is a legal CR 616.1e
/// choice. The decline is NOT sticky: the `Some` arm validates only that the
/// named card is dredge-eligible against the player's OWN graveyard/library
/// -- the same test dredge law itself uses -- regardless of which mechanism
/// raised the entry. Zero coverage before the fix cycle.
fn test_dx2_choose_dredge_some_can_answer_a_needschoice_originated_entry() {
    let p1 = p(1);
    let p2 = p(2);
    let skip_a = ReplacementEffect {
        id: ReplacementId(960),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let skip_b = ReplacementEffect {
        id: ReplacementId(961),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Dredge Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_keyword(KeywordAbility::Dredge(3)),
        )
        .object(ObjectSpec::card(p1, "Library Card 0").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 3").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(skip_a)
        .with_replacement_effect(skip_b)
        .build()
        .unwrap();
    let dredge_id = find_object(&state, "Dredge Card");

    let events = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1)),
        "dredge should be offered first. Events: {:?}",
        events
    );

    // Decline dredge -- resume hits NeedsChoice (2 SkipDraw replacements
    // still apply), pushing a FRESH, NeedsChoice-originated entry.
    let (state, decline_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();
    assert!(
        decline_events.iter().any(
            |e| matches!(e, GameEvent::ReplacementChoiceRequired { player, .. } if *player == p1)
        ),
        "declining dredge should re-check WouldDraw replacements and defer \
         via ReplacementChoiceRequired. Events: {:?}",
        decline_events
    );
    assert_eq!(state.pending_draws().len(), 1);

    // The player may STILL dredge -- CR 616.1e, the decline is not sticky.
    let (state, dredge_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: Some(dredge_id),
        },
    )
    .unwrap();
    assert!(
        dredge_events
            .iter()
            .any(|e| matches!(e, GameEvent::Dredged { player, .. } if *player == p1)),
        "CR 616.1e: the player must still be able to dredge on the \
         re-deferred entry. Events: {:?}",
        dredge_events
    );
    assert!(state.pending_draws().is_empty());
    assert!(object_in_zone(&state, "Dredge Card", ZoneId::Hand(p1)));
}

// ── T20 (re-review Finding R1) ────────────────────────────────────────────

/// CR 616.1f / 614.11a — pins the TRUE per-player invariant after re-review
/// Finding R1 (`pb-review-DX2.md`): `pending_draws` is NOT bounded to one
/// entry per player. A `NeedsChoice`-origin stale entry re-defers INSIDE
/// `perform_one_draw`'s own stale-entry discharge (`resolve_declined_pending_draw`
/// re-enters `perform_one_draw`, which independently re-checks
/// `check_would_draw_replacement` and can push a fresh entry of its own), so
/// a second, unrelated draw for the same player can leave TWO outstanding
/// entries — this is `OOS-DX2-3`, REOPENED, not the "structurally
/// impossible" state the fix cycle originally (and wrongly) closed it as.
///
/// Extends T19's fixture: decline a `NeedsChoice`-originated dredge offer
/// (the decline itself re-defers, since both `SkipDraw` replacements remain
/// applicable — CR 616.1f excludes only what was *applied*), then issue one
/// MORE independent draw for the same player. This test fails
/// (`pending_draws().len()` reverts to 1, or panics) if a future change
/// re-clears a player's entries before each `push_back` — which the review
/// explicitly warned against, since that would silently destroy the
/// re-deferred draw rather than merely record its existence honestly.
#[test]
fn test_dx2_needschoice_redefer_grows_the_queue() {
    let p1 = p(1);
    let p2 = p(2);
    let skip_a = ReplacementEffect {
        id: ReplacementId(960),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let skip_b = ReplacementEffect {
        id: ReplacementId(961),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Dredge Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_keyword(KeywordAbility::Dredge(3)),
        )
        .object(ObjectSpec::card(p1, "Library Card 0").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 3").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(skip_a)
        .with_replacement_effect(skip_b)
        .build()
        .unwrap();

    mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    let (mut state, _decline_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();
    assert_eq!(
        state.pending_draws().len(),
        1,
        "sanity: the decline re-defers to a fresh NeedsChoice entry"
    );

    mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    assert_eq!(
        state.pending_draws().len(),
        2,
        "R1: a second independent draw for the same player must NOT clobber \
         or merge with the entry the decline re-raised -- both are legitimate \
         CR 616.1 obligations and neither may be silently destroyed. If this \
         reads 1, someone 'fixed' the growth by clearing entries early, which \
         is the exact anti-fix the re-review warned against."
    );
    assert!(
        state.pending_draws().iter().all(|pd| pd.player == p1),
        "both entries belong to p1"
    );
}

// ── T21 (re-review Finding R3) ─────────────────────────────────────────────

/// CR 614.11a — pins the restructured `perform_one_draw::Proceed` control
/// flow (re-review Finding R3, `pb-review-DX2.md`). The fix cycle converted
/// two early `return`s in the `Proceed` arm into nested `match` tail
/// expressions specifically because a bare `return` there would skip
/// `events.extend(draw_events)` and silently drop the discharge's own
/// events -- caught during implementation by the runner reading the code,
/// not by any test. This test reaches the ONLY shape in which that defect
/// is observable: the discharge produces events (a real draw, via
/// `Proceed`) AND the current draw ALSO takes the `Proceed` path in the
/// same `perform_one_draw` call (T7's fixture never exercises this, because
/// the dredge card stays in the graveyard there and every draw is offered
/// -- neither call ever reaches `Proceed`).
#[test]
fn test_dx2_discharge_then_proceed_both_produce_events_in_one_call() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dredge_card_def(
        "dredge-dx2-t21",
        "Dredge T21 Card",
        3,
    )]);

    let state = build_upkeep_state(p1, p2, registry, |mut b| {
        b = b.object(
            ObjectSpec::card(p1, "Dredge T21 Card")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("dredge-dx2-t21".to_string()))
                .with_keyword(KeywordAbility::Dredge(3)),
        );
        // Exactly at the CR 702.52b threshold (library == 3): eligible for
        // the FIRST offer, then milled below threshold before the second
        // draw so dredge is no longer offered on either the discharge's
        // resume OR the new draw's own check.
        for i in 0..3 {
            b = b.object(
                ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
            );
        }
        b
    });

    // Draw-step offer -- unanswered. Library untouched by the offer itself
    // (dredge REPLACES the draw; nothing is milled/drawn yet).
    let (mut state, _events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.pending_draws().len(), 1, "sanity: one stale entry");
    assert_eq!(
        count_in_zone(&state, ZoneId::Library(p1)),
        3,
        "sanity: library still at the CR 702.52b threshold"
    );

    // Mill the library below the Dredge(3) threshold -- CR 702.52b: dredge
    // requires library >= n. After this, dredge cannot be offered again for
    // EITHER the discharge's resume or the fresh draw's own check.
    let mill = Effect::MillCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
    };
    let mut ctx = EffectContext::new(p1, ObjectId(998), vec![]);
    let _mill_events = execute_effect(&mut state, &mill, &mut ctx);
    assert_eq!(
        count_in_zone(&state, ZoneId::Library(p1)),
        2,
        "sanity: library now below the Dredge(3) threshold"
    );

    // A single fresh draw for the SAME player: this must discharge the
    // stale entry (a REAL draw via `Proceed`, since dredge is no longer
    // eligible and no other WouldDraw replacement is registered) AND take
    // `Proceed` itself for the draw it was asked to perform.
    let events = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();

    let drawn = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        drawn, 2,
        "the discharge's own Proceed draw AND the new draw's Proceed both \
         produce a CardDrawn event from this single call -- reverting the \
         fix-cycle's match-tail restructuring to a bare `return` in the \
         `Proceed` arm would drop the discharge's event and read 1 here. \
         Events: {:?}",
        events
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { .. })),
        "dredge must not be re-offered on either side -- library is below \
         threshold. Events: {:?}",
        events
    );
    assert!(
        state.pending_draws().is_empty(),
        "no entry should remain outstanding -- the discharge consumed the \
         stale one and the new draw completed rather than deferring"
    );
    // Discharge-first ordering: the discharge's CardDrawn precedes the
    // outer draw's own (perform_one_draw's discharge step runs BEFORE the
    // outer `check_would_draw_replacement`/`Proceed` match).
    let first_drawn_index = events
        .iter()
        .position(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .unwrap();
    let second_drawn_index = events
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .nth(1)
        .map(|(i, _)| i)
        .unwrap();
    assert!(
        first_drawn_index < second_drawn_index,
        "discharge-first ordering: the discharge's CardDrawn must precede \
         the outer draw's own. Events: {:?}",
        events
    );
}

// ── T16 ─────────────────────────────────────────────────────────────────────

#[test]
/// Wire-neutrality pin (plan §7.1 / AC 5873). PB-DX2 takes design (b) --
/// reuse the existing `pending_draws` queue, no new type, no new `GameState`
/// field, no new `Command`/`GameEvent` variant -- so `PROTOCOL_VERSION` and
/// `HASH_SCHEMA_VERSION` must stay exactly where PB-DX1 left them.
fn test_dx2_wire_version_sentinels() {
    assert_eq!(
        mtg_engine::HASH_SCHEMA_VERSION,
        79u8,
        "HASH_SCHEMA_VERSION live sentinel -- moved 70->71 by ENG-1 (effect-driven \
         discard, unrelated to this batch), 71->72 by ENG-2 (an announcement-time \
         target event, also unrelated to this batch), 72->73 by PB-DX21 \
         (CombatState gains attackers_declared, also unrelated to this batch), and \
         74->75 by the PB-DX27 rider (LayerModification::SetLandTypes, also \
         unrelated to this batch); this sentinel pins the LIVE version like every \
         other scattered sentinel in the suite, not PB-DX2's own contribution"
    );
    assert_eq!(
        mtg_engine::PROTOCOL_VERSION,
        40u32,
        "PROTOCOL_VERSION live sentinel -- moved 33->34 by ENG-1 (effect-driven \
         discard), 34->35 by ENG-2 (an announcement-time target event), and 35->36 \
         by the PB-DX27 rider (LayerModification is reachable via \
         ContinuousEffectDef in the wire closure), all unrelated to this batch; \
         this sentinel pins the LIVE version like every other scattered sentinel \
         in the suite, not PB-DX2's own contribution -- PB-DX2 itself left it \
         unmoved"
    );
}
