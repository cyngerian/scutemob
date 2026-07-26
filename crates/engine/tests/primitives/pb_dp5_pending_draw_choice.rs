//! PB-DP5 — DP-5: the `WouldDraw` multi-replacement prompt is unanswerable.
//!
//! `memory/primitives/pb-plan-DP5.md` §7 is authoritative for what each test pins.
//!
//! Before this batch, `check_would_draw_replacement` emitted
//! `GameEvent::ReplacementChoiceRequired` on a CR 616.1e multi-replacement draw but
//! recorded no pending state anywhere on `GameState`. `Command::OrderReplacements`
//! hard-required a matching `pending_zone_changes` entry and rejected any answer to
//! the draw prompt, so the draw was silently destroyed (worse on the effect-draw
//! path, which kept looping after the deferral and emitted one unanswerable prompt
//! per remaining draw). This batch adds `GameState.pending_draws` and a resume path
//! (`resolve_pending_draw`, reached through the existing `Command::OrderReplacements`)
//! so the draw completes through the player's chosen order (CR 616.1 / 616.1f /
//! 614.11).

use mtg_engine::{
    process_command, Command, Effect, EffectAmount, EffectDuration, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ObjectSpec, PlayerFilter, PlayerId, PlayerTarget,
    ReplacementEffect, ReplacementId, ReplacementModification, ReplacementTrigger, ZoneId,
    ZoneType,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Two non-self `SkipDraw` `WouldDraw(Specific(p1))` replacements (ids 600, 601) and
/// one library card for `p1`. CR 616.1e: no self-replacement present, so 2+
/// applicable replacements force `NeedsChoice`.
fn build_two_skipdraw_state() -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let skip_a = ReplacementEffect {
        id: ReplacementId(600),
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
        id: ReplacementId(601),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p1, "Island").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(skip_a)
        .with_replacement_effect(skip_b)
        .build()
        .unwrap();
    (state, p1, p2)
}

// ── T1 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 616.1 / 616.1e — with 2+ `WouldDraw` replacements applicable to one draw,
/// the drawing player's `Command::OrderReplacements` answer is ACCEPTED. Pre-PB-DP5,
/// `handle_order_replacements` hard-required a `pending_zone_changes` entry and always
/// rejected this with "player PlayerId(1) is not the affected player of any pending
/// replacement choice" — the draw could never complete.
fn test_dp5_order_replacements_after_deferred_draw_is_accepted() {
    let (mut state, p1, _p2) = build_two_skipdraw_state();

    let events = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::ReplacementChoiceRequired { player, .. } if *player == p1)
        ),
        "CR 616.1e: expected the draw to defer with ReplacementChoiceRequired. Events: {:?}",
        events
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "a PendingDraw entry should be recorded for the deferred draw"
    );

    let result = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(601), ReplacementId(600)],
        },
    );
    assert!(
        result.is_ok(),
        "CR 616.1: OrderReplacements answering a deferred draw must be accepted, got {:?}",
        result.err()
    );
}

// ── T2 / T3 ─────────────────────────────────────────────────────────────────

#[test]
/// CR 616.1 — the FIRST id in the submitted order is the one credited via
/// `ReplacementEffectApplied`, not registration order or list order. 601 is the
/// SECOND-registered id, so this rules out `choices.first()`, registration order,
/// and `applicable[0]` all at once. Paired with the mirrored test below.
fn test_dp5_chosen_replacement_is_the_one_applied() {
    let (mut state, p1, _p2) = build_two_skipdraw_state();
    let _ = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();

    let (_new_state, events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(601), ReplacementId(600)],
        },
    )
    .unwrap();

    let first_applied = events.iter().find_map(|e| match e {
        GameEvent::ReplacementEffectApplied { effect_id, .. } => Some(*effect_id),
        _ => None,
    });
    assert_eq!(
        first_applied,
        Some(ReplacementId(601)),
        "the first-submitted id (601) should be credited first. Events: {:?}",
        events
    );
}

#[test]
/// CR 616.1 — mirrored order of the test above: submitting [600, 601] credits 600
/// first. T2 + T3 differ ONLY in submission order and assert DIFFERENT effect_ids,
/// making criterion 5532's order-discrimination non-vacuous.
fn test_dp5_chosen_replacement_is_the_one_applied_mirrored() {
    let (mut state, p1, _p2) = build_two_skipdraw_state();
    let _ = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();

    let (_new_state, events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(600), ReplacementId(601)],
        },
    )
    .unwrap();

    let first_applied = events.iter().find_map(|e| match e {
        GameEvent::ReplacementEffectApplied { effect_id, .. } => Some(*effect_id),
        _ => None,
    });
    assert_eq!(
        first_applied,
        Some(ReplacementId(600)),
        "the first-submitted id (600) should be credited first. Events: {:?}",
        events
    );
}

// ── T4 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 616.1 / 616.1f — criterion 5532: with two non-`SkipDraw` `WouldDraw`
/// replacements, after the player answers, the draw genuinely COMPLETES (card in
/// hand, library decremented, cards_drawn_this_turn incremented, CardDrawn emitted)
/// — not merely defers. Neither modification the draw path honours besides
/// `SkipDraw` changes the draw's outcome (`RedirectToZone`/`DoubleTokens` are both
/// draw no-ops today), so after applying the chosen one the CR 616.1f re-check finds
/// the other still applicable, silently no-ops it too (OOS-DP5-8), and the draw
/// proceeds.
fn test_dp5_draw_completes_through_chosen_order() {
    let p1 = p(1);
    let p2 = p(2);
    let redirect = ReplacementEffect {
        id: ReplacementId(700),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    let double = ReplacementEffect {
        id: ReplacementId(701),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::DoubleTokens,
    };
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p1, "Island").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(redirect)
        .with_replacement_effect(double)
        .build()
        .unwrap();

    let _ = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    assert_eq!(state.pending_draws().len(), 1);

    let (new_state, events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(700), ReplacementId(701)],
        },
    )
    .unwrap();

    assert_eq!(
        new_state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        1,
        "the card should be in hand — the draw completed"
    );
    assert_eq!(
        new_state.zone(&ZoneId::Library(p1)).unwrap().len(),
        0,
        "the library should be empty"
    );
    assert_eq!(
        new_state.players().get(&p1).unwrap().cards_drawn_this_turn,
        1,
        "CR 121.1: cards_drawn_this_turn should increment on a completed draw"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CardDrawn should be emitted. Events: {:?}",
        events
    );
    assert!(
        new_state.pending_draws().is_empty(),
        "the pending draw should be cleared once the draw completes"
    );
}

// ── T5 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 616.1 / 614.11 — criterion 5531's "both emit sites": the effect-draw path
/// (`Effect::DrawCards`, formerly `draw_one_card`) records pending state exactly like
/// the turn-draw path (`turn_actions::draw_card`, covered by T1-T4).
fn test_dp5_effect_draw_path_records_pending_state() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let (mut state, p1, _p2) = build_two_skipdraw_state();
    let effect = Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
    };
    let mut ctx = EffectContext::new(p1, mtg_engine::ObjectId(999), vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::ReplacementChoiceRequired { player, .. } if *player == p1)
        ),
        "the effect-draw path should also defer with ReplacementChoiceRequired. Events: {:?}",
        events
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "the effect-draw path should also record a PendingDraw entry"
    );

    // And it must be answerable, exactly like the turn-draw path.
    let result = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(601), ReplacementId(600)],
        },
    );
    assert!(
        result.is_ok(),
        "the effect-draw path's deferred draw must also be answerable, got {:?}",
        result.err()
    );
}

// ── T6 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 614.11a / 121.2 — a draw SEQUENCE (`Effect::DrawCards { count: 3 }`) stops at
/// the first deferred draw (exactly ONE `ReplacementChoiceRequired`, not three) and
/// resumes through the rest of the sequence once each deferred draw is answered.
/// Pre-PB-DP5 the `for _ in 0..n` loop kept iterating after a deferral, so this
/// scenario emitted THREE unanswerable prompts and drew ZERO cards.
fn test_dp5_draw_sequence_stops_and_resumes() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let p2 = p(2);
    let redirect = ReplacementEffect {
        id: ReplacementId(800),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    let double = ReplacementEffect {
        id: ReplacementId(801),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::DoubleTokens,
    };
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p1, "Card A").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Card B").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Card C").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(redirect)
        .with_replacement_effect(double)
        .build()
        .unwrap();

    let effect = Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(3),
    };
    let mut ctx = EffectContext::new(p1, mtg_engine::ObjectId(999), vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    let choice_count = events
        .iter()
        .filter(
            |e| matches!(e, GameEvent::ReplacementChoiceRequired { player, .. } if *player == p1),
        )
        .count();
    assert_eq!(
        choice_count, 1,
        "the sequence must stop at the FIRST deferred draw — exactly one \
         ReplacementChoiceRequired, not three. Events: {:?}",
        events
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "exactly one PendingDraw should be recorded"
    );
    assert_eq!(
        state.pending_draws()[0].remaining,
        2,
        "two further draws remain in the sequence"
    );
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        0,
        "no card should be in hand yet"
    );

    // Answer #1 — completes 1 card, resumes into a fresh deferral (remaining: 1).
    let (state, events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(800), ReplacementId(801)],
        },
    )
    .unwrap();
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        1,
        "one card should be in hand after the first answer"
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "resuming the sequence should defer again (both effects still apply)"
    );
    assert_eq!(state.pending_draws()[0].remaining, 1);
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)));

    // Answer #2 — completes 2nd card, resumes into a fresh deferral (remaining: 0).
    let (state, _events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(800), ReplacementId(801)],
        },
    )
    .unwrap();
    assert_eq!(state.zone(&ZoneId::Hand(p1)).unwrap().len(), 2);
    assert_eq!(state.pending_draws().len(), 1);
    assert_eq!(state.pending_draws()[0].remaining, 0);

    // Answer #3 — completes the 3rd and final card; no further resume needed.
    let (state, _events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(800), ReplacementId(801)],
        },
    )
    .unwrap();
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        3,
        "all three cards should be in hand once the sequence fully resumes"
    );
    assert!(
        state.pending_draws().is_empty(),
        "no pending draw should remain once the sequence is complete"
    );
}

// ── T7 ──────────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a / 616.1 — the THIRD emit site: `draw_card_skipping_dredge`
/// (reached via `Command::ChooseDredge { card: None }`, declining dredge). The
/// original DP-5 audit named only two emit sites; this one is reachable and was
/// equally unanswerable pre-PB-DP5.
fn test_dp5_dredge_decline_path_records_pending_state() {
    let p1 = p(1);
    let p2 = p(2);
    let skip_a = ReplacementEffect {
        id: ReplacementId(900),
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
        id: ReplacementId(901),
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

    // Draw offers dredge first (library has 4 >= 3 cards).
    let events = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1)),
        "dredge should be offered first. Events: {:?}",
        events
    );

    // Decline dredge — this reaches draw_card_skipping_dredge, the third emit site.
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
        "declining dredge should re-check WouldDraw replacements and defer via \
         ReplacementChoiceRequired. Events: {:?}",
        decline_events
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "the dredge-decline path should also record a PendingDraw entry"
    );

    // And it must be answerable.
    let result = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(901), ReplacementId(900)],
        },
    );
    assert!(
        result.is_ok(),
        "the dredge-decline path's deferred draw must be answerable, got {:?}",
        result.err()
    );
}

// ── T8 / T9 ─────────────────────────────────────────────────────────────────

#[test]
/// SR-29 trust boundary — a player who is NOT the affected chooser of any pending
/// replacement choice (zone change OR draw) cannot order one.
fn test_dp5_order_replacements_rejects_non_affected_player() {
    let (mut state, p1, p2) = build_two_skipdraw_state();
    let _ = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();

    let err = process_command(
        state,
        Command::OrderReplacements {
            player: p2,
            ids: vec![ReplacementId(600)],
        },
    )
    .unwrap_err();
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("not the affected player"),
        "error should name the missing pending event for p2, got: {}",
        msg
    );
}

#[test]
/// SR-29 trust boundary — the affected player CAN send `OrderReplacements`, but an id
/// that is not applicable to their pending draw (registered for a different player)
/// is rejected with an applicability error, not silently substituted.
fn test_dp5_order_replacements_rejects_inapplicable_id() {
    let p1 = p(1);
    let p2 = p(2);
    // A WouldDraw replacement scoped to p2, not p1 -- never applicable to p1's draw.
    let scoped_to_p2 = ReplacementEffect {
        id: ReplacementId(950),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p2),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let (mut state, p1_check, _p2) = build_two_skipdraw_state();
    assert_eq!(p1, p1_check);
    state
        .replacement_effects_mut()
        .push_back(scoped_to_p2.clone());
    let _ = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();

    let err = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(950)],
        },
    )
    .unwrap_err();
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("applicable"),
        "error should be an applicability failure, not a missing-pending-event error, got: {}",
        msg
    );
}

// ── T10 ─────────────────────────────────────────────────────────────────────

#[test]
/// CR 616.1 precedence — a player with BOTH a pending zone change and a pending draw
/// at the same time can answer either independently: submitting the zone-change ids
/// resolves the zone change and leaves `pending_draws()` untouched; submitting the
/// draw ids afterward resolves the draw. Routing is by applicability (provably
/// disjoint trigger sets), not by which pending kind was recorded first.
fn test_dp5_precedence_zone_change_and_draw_coexist() {
    use mtg_engine::state::replacement_effect::PendingZoneChange;

    let p1 = p(1);
    let p2 = p(2);

    // Two competing zone-change replacements on a dying creature p1 controls.
    let zc1 = ReplacementEffect {
        id: ReplacementId(1000),
        source: None,
        controller: p1,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: mtg_engine::ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };
    let zc2 = ReplacementEffect {
        id: ReplacementId(1001),
        source: None,
        controller: p1,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: mtg_engine::ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Command),
    };
    // Two competing WouldDraw replacements.
    let dr1 = ReplacementEffect {
        id: ReplacementId(1002),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };
    let dr2 = ReplacementEffect {
        id: ReplacementId(1003),
        source: None,
        controller: p2,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(p1),
        },
        modification: ReplacementModification::SkipDraw,
    };

    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Creature", 2, 2))
        .object(ObjectSpec::card(p1, "Island").in_zone(ZoneId::Library(p1)))
        .with_replacement_effect(zc1)
        .with_replacement_effect(zc2)
        .with_replacement_effect(dr1)
        .with_replacement_effect(dr2)
        .build()
        .unwrap();

    let creature_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;
    state
        .pending_zone_changes_mut()
        .push_back(PendingZoneChange {
            object_id: creature_id,
            original_from: ZoneType::Battlefield,
            original_destination: ZoneType::Graveyard,
            affected_player: p1,
            already_applied: Vec::new(),
        });

    let _ = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    assert_eq!(state.pending_zone_changes().len(), 1);
    assert_eq!(state.pending_draws().len(), 1);

    // Submit the zone-change ids first.
    let (state, zc_events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(1001), ReplacementId(1000)],
        },
    )
    .unwrap();
    assert!(
        zc_events.iter().any(
            |e| matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(1001))
        ),
        "the zone-change answer should resolve the zone change. Events: {:?}",
        zc_events
    );
    assert!(
        state.pending_zone_changes().is_empty(),
        "the zone change should be resolved"
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "the pending draw must be UNTOUCHED by resolving the zone change"
    );

    // Now submit the draw ids.
    let (state, dr_events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(1003), ReplacementId(1002)],
        },
    )
    .unwrap();
    assert!(
        dr_events.iter().any(
            |e| matches!(e, GameEvent::ReplacementEffectApplied { effect_id, .. } if *effect_id == ReplacementId(1003))
        ),
        "the draw answer should resolve the draw. Events: {:?}",
        dr_events
    );
    assert!(state.pending_draws().is_empty());
}

// ── T11 ─────────────────────────────────────────────────────────────────────

#[test]
/// CR 616.1f / 614.5 / 614.10 — the re-check loop applies at most once per effect and
/// terminates: with `{SkipDraw(A), RedirectToZone(B)}`, choosing B first applies B
/// (no game-state effect), the re-check finds A still applicable and auto-applies it
/// (SkipDraw — no CardDrawn, card stays in library). Choosing A first stops the chain
/// immediately (no CardDrawn either) because SkipDraw is terminal.
fn test_dp5_616_1f_recheck_stops_at_skip_draw() {
    let make_state = || {
        let p1 = p(1);
        let p2 = p(2);
        let skip_a = ReplacementEffect {
            id: ReplacementId(1100),
            source: None,
            controller: p2,
            duration: EffectDuration::Indefinite,
            is_self_replacement: false,
            trigger: ReplacementTrigger::WouldDraw {
                player_filter: PlayerFilter::Specific(p1),
            },
            modification: ReplacementModification::SkipDraw,
        };
        let redirect_b = ReplacementEffect {
            id: ReplacementId(1101),
            source: None,
            controller: p2,
            duration: EffectDuration::Indefinite,
            is_self_replacement: false,
            trigger: ReplacementTrigger::WouldDraw {
                player_filter: PlayerFilter::Specific(p1),
            },
            modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
        };
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(ObjectSpec::card(p1, "Island").in_zone(ZoneId::Library(p1)))
            .with_replacement_effect(skip_a)
            .with_replacement_effect(redirect_b)
            .build()
            .unwrap();
        (state, p1)
    };

    // Choose B (1101) first.
    let (mut state, p1) = make_state();
    let _ = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    let (state, events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(1101), ReplacementId(1100)],
        },
    )
    .unwrap();
    let applied_ids: Vec<ReplacementId> = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::ReplacementEffectApplied { effect_id, .. } => Some(*effect_id),
            _ => None,
        })
        .collect();
    assert_eq!(
        applied_ids,
        vec![ReplacementId(1101), ReplacementId(1100)],
        "B should be credited first (chosen), then A auto-applied by the re-check. Events: {:?}",
        events
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "no CardDrawn: the re-check auto-applied SkipDraw. Events: {:?}",
        events
    );
    assert_eq!(state.zone(&ZoneId::Library(p1)).unwrap().len(), 1);
    assert!(state.pending_draws().is_empty());

    // Choose A (1100, SkipDraw) first.
    let (mut state, p1) = make_state();
    let _ = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    let (state, events) = process_command(
        state,
        Command::OrderReplacements {
            player: p1,
            ids: vec![ReplacementId(1100), ReplacementId(1101)],
        },
    )
    .unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "no CardDrawn: SkipDraw stops the chain immediately. Events: {:?}",
        events
    );
    assert_eq!(state.zone(&ZoneId::Library(p1)).unwrap().len(), 1);
    assert!(state.pending_draws().is_empty());
}

// ── T12 ─────────────────────────────────────────────────────────────────────

#[test]
/// Hard constraint 3 — an unanswered `PendingDraw` does not deadlock anything. This is
/// a regression guard: if a future change ever gates progress (priority, SBAs, step
/// advancement) on `pending_draws`, this test starts failing. Passing priority through
/// with a deferred draw outstanding must not error and must not hang.
fn test_dp5_unanswered_pending_draw_does_not_deadlock() {
    let (mut state, p1, p2) = build_two_skipdraw_state();
    let _ = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    assert_eq!(state.pending_draws().len(), 1);

    // Pass priority for both players without ever answering the OrderReplacements
    // prompt. Nothing should error, hang, or silently resolve the pending draw.
    let (state, _events) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _events) = process_command(state, Command::PassPriority { player: p2 }).unwrap();

    assert_eq!(
        state.pending_draws().len(),
        1,
        "the unanswered pending draw should still be present -- it is a recorded, \
         non-blocking obligation (PB-DP5 plan §5.1), not something anything else \
         resolves for the player"
    );
}

// ── T13 ─────────────────────────────────────────────────────────────────────

#[test]
/// PB-DP5 §6 — wire-version sentinel. HASH_SCHEMA_VERSION bumped 63 -> 64 for the new
/// `GameState.pending_draws` field; PROTOCOL_VERSION unchanged at 27 (`PendingDraw` is
/// reachable only from `GameState`, never `Command`/`GameEvent`/`ReplayLog`).
fn test_dp5_wire_version_sentinels() {
    assert_eq!(
        mtg_engine::HASH_SCHEMA_VERSION,
        64u8,
        "HASH_SCHEMA_VERSION should be 64 after PB-DP5's pending_draws field"
    );
    assert_eq!(
        mtg_engine::PROTOCOL_VERSION,
        27,
        "PROTOCOL_VERSION must be unchanged -- pending_draws never touches the wire closure"
    );
}
