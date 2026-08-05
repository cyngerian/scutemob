//! PB-DX23 — Stage 1 (`rules::queries::dredge_options`, the shared CR 702.52a/b
//! eligibility derivation) and Stage 2 (the `OOS-DX2-2` tail flip:
//! `perform_remaining_draws` / `resolve_declined_pending_draw` now take a
//! caller-supplied `offer_dredge` / `tail_offers_dredge` flag instead of a
//! hard-coded `false`).
//!
//! `memory/primitives/pb-plan-DX23.md` §3 Q2/Q3 and §5 T2/T3 are authoritative
//! for what each test pins. The companion simulator-side probe (Stage 3, not
//! this file) is `crates/simulator/tests/pb_dx23_dredge_answer_channel.rs`.
//!
//! These tests do not answer a dredge offer through any `LegalAction` —
//! `crates/simulator` gains no channel until Stage 3. Every offer here is
//! answered directly with `Command::ChooseDredge`, exactly as the pre-PB-DX23
//! engine-only tests (`mechanics_a_d/dredge.rs`, `pb_dx2_command_gates.rs`) do.

use std::collections::HashSet;

use mtg_engine::rules::queries::dredge_options;
use mtg_engine::rules::replacement::{check_would_draw_replacement, DrawAction};
use mtg_engine::{
    process_command, Command, GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId,
    ObjectSpec, PlayerId, ZoneId,
};

// ── Helpers (copied from `tests/mechanics_a_d/dredge.rs` /
// `pb_dx2_command_gates.rs`, SR-9a — do not `mod`-import across integration-
// test targets) ─────────────────────────────────────────────────────────────

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

// ═══════════════════════════════════════════════════════════════════════════
// T2 — the shared query (Stage 1, §3 Q2)
// ═══════════════════════════════════════════════════════════════════════════

// ── T2.1 ─────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a — `dredge_options` only considers cards in `player`'s
/// GRAVEYARD. Dredge "is a static ability that functions only while the card
/// with dredge is in a player's graveyard" — a card carrying the identical
/// `KeywordAbility::Dredge` marker on the BATTLEFIELD (or any other zone)
/// must never appear, even though nothing about its characteristics differs
/// from the graveyard card.
///
/// Revert to watch red: drop the `.filter(|obj| obj.zone == graveyard_zone)`
/// line from `dredge_options` — the battlefield card appears alongside the
/// graveyard one.
fn test_dx23_dredge_options_matches_cr_702_52a_eligibility() {
    let p1 = p(1);
    let p2 = p(2);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Graveyard Dredger")
                .in_zone(ZoneId::Graveyard(p1))
                .with_keyword(KeywordAbility::Dredge(3)),
        )
        .object(
            ObjectSpec::card(p1, "Battlefield Dredger")
                .in_zone(ZoneId::Battlefield)
                .with_keyword(KeywordAbility::Dredge(3)),
        )
        .object(ObjectSpec::card(p1, "Library Card 0").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let graveyard_id = find_object(&state, "Graveyard Dredger");
    // Sanity: the battlefield card really is in the pool `dredge_options`
    // scans, so its absence below is a filter decision, not an accident of
    // the fixture.
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Battlefield Dredger"
                && o.zone == ZoneId::Battlefield),
        "sanity: the battlefield dredger must actually be on the battlefield"
    );

    let options = dredge_options(&state, p1);

    assert_eq!(
        options,
        vec![(graveyard_id, 3)],
        "CR 702.52a: only the GRAVEYARD dredge card is eligible -- the \
         battlefield one must not appear even though it carries the \
         identical keyword. options: {:?}",
        options
    );
}

// ── T2.2 ─────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52b — "A player with fewer cards in their library than the number
/// required by a dredge ability can't mill any of them this way." The floor
/// is `library_count >= n`, i.e. an EXACT-count library still qualifies
/// (mirrors `mechanics_a_d/dredge.rs::test_dredge_exact_library_count_is_eligible`,
/// applied directly to the query rather than through a draw-step offer).
///
/// Revert to watch red: change `<=` to `<` in `dredge_options`'s library
/// comparison — the exact-count card (N == library_count) disappears.
fn test_dx23_dredge_options_respects_the_library_floor() {
    let p1 = p(1);
    let p2 = p(2);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            // Eligible: library has EXACTLY 3, dredge needs >= 3.
            ObjectSpec::card(p1, "Exact Floor Dredger")
                .in_zone(ZoneId::Graveyard(p1))
                .with_keyword(KeywordAbility::Dredge(3)),
        )
        .object(
            // Ineligible: dredge needs >= 4, library has only 3.
            ObjectSpec::card(p1, "Over Floor Dredger")
                .in_zone(ZoneId::Graveyard(p1))
                .with_keyword(KeywordAbility::Dredge(4)),
        )
        .object(ObjectSpec::card(p1, "Library Card 0").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let exact_id = find_object(&state, "Exact Floor Dredger");
    let options = dredge_options(&state, p1);

    assert_eq!(
        options,
        vec![(exact_id, 3)],
        "CR 702.52b: the exact-count card (N == library_count == 3) must be \
         eligible and the over-floor card (N == 4 > library_count == 3) must \
         not. options: {:?}",
        options
    );
}

// ── T2.3 ─────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a — the offer (`check_would_draw_replacement`'s
/// `DredgeChoiceRequired.options`) and the shared query
/// (`rules::queries::dredge_options`) must return the SAME list, since
/// `check_would_draw_replacement` now calls `dredge_options` directly rather
/// than keeping a second copy of the scan (PB-DX23 §3 Q2, the PB-DX20 shape).
///
/// **This is a CONSISTENCY probe, not a CORRECTNESS one — that is exactly why
/// T2.1 and T2.2 exist separately** (PB-DX20's durable lesson: "a
/// differential probe between two consumers of one function proves
/// consistency, not correctness"). A bug shared by both call sites -- e.g. a
/// wrong library-floor comparison inside `dredge_options` itself -- would
/// pass this test while failing T2.2.
///
/// Revert to watch red: this batch's whole point is that there is only ONE
/// derivation, so the discriminating revert is to re-introduce a SECOND,
/// independently-drifted one inside `check_would_draw_replacement` -- e.g.
/// temporarily return an empty `Vec` from its `dredge_options` call instead
/// of delegating. That makes `check_would_draw_replacement` return
/// `DrawAction::Proceed` (no eligible cards found) while `dredge_options`
/// itself still reports the eligible card, so the two disagree.
fn test_dx23_offer_and_engine_scan_are_one_derivation() {
    let p1 = p(1);
    let p2 = p(2);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "First Dredger")
                .in_zone(ZoneId::Graveyard(p1))
                .with_keyword(KeywordAbility::Dredge(2)),
        )
        .object(
            ObjectSpec::card(p1, "Second Dredger")
                .in_zone(ZoneId::Graveyard(p1))
                .with_keyword(KeywordAbility::Dredge(3)),
        )
        .object(ObjectSpec::card(p1, "Library Card 0").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 3").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let query_options = dredge_options(&state, p1);
    assert_eq!(
        query_options.len(),
        2,
        "sanity: both dredgers must be eligible so a divergence has \
         something to disagree about. options: {:?}",
        query_options
    );

    let offer_options = match check_would_draw_replacement(&state, p1, &HashSet::new(), true) {
        DrawAction::DredgeAvailable(GameEvent::DredgeChoiceRequired { options, .. }) => options,
        other => panic!(
            "expected DredgeAvailable(DredgeChoiceRequired), got {:?}",
            other
        ),
    };

    assert_eq!(
        offer_options, query_options,
        "PB-DX23 §3 Q2: the offer's options and the shared query's return \
         value must be the SAME list -- check_would_draw_replacement calls \
         dredge_options rather than keeping its own copy."
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T3 — the tail flip (Stage 2, §3 Q3, closing `OOS-DX2-2`)
// ═══════════════════════════════════════════════════════════════════════════

/// Shared fixture for T3.x: `player`'s graveyard holds one Dredge(3) card,
/// library has `library_cards` filler objects (never registered with a
/// `CardId`, so Architecture Invariant 9 never sees them).
fn dredge_fixture(p1: PlayerId, p2: PlayerId, dredge_n: u32, library_cards: u32) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Tail Dredger")
                .in_zone(ZoneId::Graveyard(p1))
                .with_keyword(KeywordAbility::Dredge(dredge_n)),
        );
    for i in 0..library_cards {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }
    builder.build().unwrap()
}

fn draw_cards_effect(
    state: &mut GameState,
    player: PlayerId,
    count: i32,
) -> Vec<mtg_engine::GameEvent> {
    use mtg_engine::effects::{execute_effect, EffectContext};
    let effect = mtg_engine::Effect::DrawCards {
        player: mtg_engine::PlayerTarget::Controller,
        count: mtg_engine::EffectAmount::Fixed(count),
    };
    let mut ctx = EffectContext::new(player, ObjectId(9990), vec![]);
    execute_effect(state, &effect, &mut ctx)
}

// ── T3.1 ─────────────────────────────────────────────────────────────────

#[test]
/// CR 121.2 / 614.11a / 121.6b — closing `OOS-DX2-2`: the TAIL of a
/// multi-draw sequence is independently dredge-offerable, because each draw
/// of "draw N" is a separate draw event (CR 121.2) and CR 614.11a/121.6b
/// resume the sequence only after the replacement's own actions complete —
/// so each resumed draw is a fresh "would draw" event under CR 702.52a.
///
/// With a dredge card that is never actually dredged away (only declined),
/// declining the FIRST offer of a 3-draw sequence must surface a SECOND
/// `DredgeChoiceRequired` for the tail's first draw, not silently draw
/// through it.
///
/// Revert to watch red: restore the hard-coded `false` at
/// `perform_remaining_draws`'s `perform_one_draw` call (undo the
/// `offer_dredge` parameter) — the second `DredgeChoiceRequired` vanishes and
/// the tail completes in one silent burst.
fn test_dx23_tail_of_an_answered_multi_draw_offers_dredge_again() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = dredge_fixture(p1, p2, 3, 10);

    let events = draw_cards_effect(&mut state, p1, 3);
    assert_eq!(
        events
            .iter()
            .filter(
                |e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1)
            )
            .count(),
        1,
        "sanity: the sequence must stop at exactly one offer for draw 1"
    );
    assert_eq!(state.pending_draws()[0].remaining, 2);

    // Explicit decline of the FIRST offer (draw 1).
    let (state, decline_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();

    let card_drawn = decline_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    let second_offer = decline_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1))
        .count();

    assert_eq!(
        card_drawn, 1,
        "draw 1 itself completes on decline (offer_dredge: false for THIS \
         draw, unconditionally -- PB-DP5 §3.3 still covers it). Events: {:?}",
        decline_events
    );
    assert_eq!(
        second_offer, 1,
        "CR 121.2/614.11a: the tail's first draw (draw 2 of 3) is a \
         DIFFERENT draw event and must be independently offered dredge \
         again, since the dredge card is still in the graveyard and still \
         eligible. Events: {:?}",
        decline_events
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "exactly one NEW entry for the tail's own offer"
    );
    assert_eq!(
        state.pending_draws()[0].remaining,
        1,
        "one further draw (draw 3) still owed after the tail's own offer"
    );
}

// ── T3.2 ─────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a / 616.1f — declining an offer does NOT re-offer dredge for
/// THAT SAME draw event (PB-DP5 §3.3's argument, which the tail flip does
/// NOT disturb): a SINGLE draw (no tail, `remaining == 0`) must complete on
/// decline with no further `DredgeChoiceRequired`. This is the boundary
/// `mechanics_a_d/dredge.rs::test_dredge_decline_does_not_reoffer` guards
/// from the draw-step path; this test guards the same boundary directly
/// against `resolve_declined_pending_draw`'s unconditional `false` for THIS
/// draw, independent of `tail_offers_dredge`.
///
/// Revert to watch red: pass `true` (instead of the unconditional `false`)
/// at `resolve_declined_pending_draw`'s own `perform_one_draw` call
/// (`replacement.rs`, the call inside the function, NOT the tail's
/// `perform_remaining_draws` call) — a re-offer appears on the SAME draw.
fn test_dx23_declining_does_not_reoffer_for_the_same_draw() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = dredge_fixture(p1, p2, 3, 10);

    // A single draw (remaining == 0, no tail).
    let events = draw_cards_effect(&mut state, p1, 1);
    assert_eq!(
        events
            .iter()
            .filter(
                |e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1)
            )
            .count(),
        1
    );
    assert_eq!(state.pending_draws()[0].remaining, 0);

    let (state, decline_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();

    assert!(
        decline_events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "the single draw must complete on decline. Events: {:?}",
        decline_events
    );
    assert!(
        !decline_events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { .. })),
        "CR 702.52a/616.1f: declining must NOT re-offer dredge for THIS SAME \
         draw -- there is no tail here (remaining == 0), so any re-offer \
         would be an infinite loop of choices on one draw event. \
         Events: {:?}",
        decline_events
    );
    assert!(state.pending_draws().is_empty());
}

// ── T3.3 ─────────────────────────────────────────────────────────────────

#[test]
/// CR 702.52a / 614.11a — the `OOS-DX2-3` guard (REOPENED, not closed; see
/// `perform_one_draw`'s "Per-player invariant" doc and PB-DX23 §3 Q3's
/// five-step trace). `perform_one_draw`'s IMPLICIT stale-entry discharge
/// (forced when a second, unrelated draw arrives before the player answers a
/// standing offer) must NOT let the tail it resumes mint a fresh
/// dredge-originated `PendingDraw` -- if it did, the OUTER call (whose own
/// draw triggered the discharge) would push a SECOND dredge-originated entry
/// on top of it, growing the per-player invariant `perform_one_draw`'s doc
/// states: "at most one dredge-originated entry per player."
///
/// This is THE reason `tail_offers_dredge` exists as a parameter rather than
/// `perform_remaining_draws` always being called with `true` once the tail
/// was made offerable at all.
///
/// Revert to watch red: pass `true` (instead of `false`) from
/// `perform_one_draw`'s implicit stale-entry discharge call to
/// `resolve_declined_pending_draw` -- `pending_draws().len()` becomes 2.
fn test_dx23_implicit_discharge_does_not_mint_a_second_dredge_entry() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = dredge_fixture(p1, p2, 3, 10);

    // Turn N: a 3-draw sequence offers dredge on draw 1, entry E{remaining:2}
    // pushed, UNANSWERED.
    let events = draw_cards_effect(&mut state, p1, 3);
    assert_eq!(
        events
            .iter()
            .filter(
                |e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1)
            )
            .count(),
        1,
        "sanity: the first sequence's own offer"
    );
    assert_eq!(state.pending_draws().len(), 1);
    assert_eq!(state.pending_draws()[0].remaining, 2);

    // Turn N+2: a second, UNRELATED draw arrives for the same player while E
    // still stands -- `turn_actions::draw_card` -> `perform_one_draw`'s
    // implicit stale-entry discharge fires FIRST (draws draw 1 of the first
    // sequence, then resumes ITS tail with `tail_offers_dredge: false`),
    // THEN the outer call examines its own draw and offers dredge again
    // (still eligible, never dredged).
    let outer_events = mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();

    // THE GUARD (headline assertion, moved above the two supporting ones per
    // review finding T3 -- this is the `OOS-DX2-3` discriminator this test
    // exists to pin, and it must be the FIRST assertion this test reaches so
    // a revert reddens on IT, not on `outer_offer_count` below, which fires
    // first if left in its original position and would leave the reader
    // thinking that assertion is the guard when it is not): exactly ONE
    // outstanding entry -- the outer call's own new offer -- not two.
    assert_eq!(
        state.pending_draws().len(),
        1,
        "OOS-DX2-3 guard: at most one dredge-originated PendingDraw entry \
         may exist per player. If the discharged sequence's tail also \
         pushed a dredge-originated entry, this would be 2."
    );

    let outer_offer_count = outer_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1))
        .count();
    assert_eq!(
        outer_offer_count, 1,
        "the OUTER call's own draw is offered dredge (its own, independent \
         'would draw' event, CR 702.52a). Events: {:?}",
        outer_events
    );

    // The FIRST sequence's discharge must complete ALL of its own draws
    // silently (offer_dredge: false threaded through its own tail) rather
    // than pausing partway and leaving a dredge-originated entry behind.
    let discharge_card_drawn = outer_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        discharge_card_drawn, 3,
        "the discharged first sequence's own 3 draws (draw 1's resume + its \
         2-draw tail) must all complete silently, with no dredge offers of \
         their own, before the outer call's offer is even evaluated. \
         Events: {:?}",
        outer_events
    );
}

// ── T3.4 ─────────────────────────────────────────────────────────────────

#[test]
/// CR 614.11a / 121.2 — `remaining` bookkeeping survives a tail deferral.
/// `perform_remaining_draws` computes `remaining_after = remaining - 1 - i`
/// for each tail iteration and hands it to `perform_one_draw`, which stores
/// it verbatim on any entry it pushes. When the tail's OWN first draw is
/// itself deferred by a fresh dredge offer, the pushed entry must carry the
/// correct count of draws STILL owed after it (draw 3 of a 3-draw sequence,
/// after declining the offer on draw 1 and being re-offered on draw 2) —
/// not zero, and not the original `remaining`.
///
/// Revert to watch red: hard-code `remaining_after: 0` in
/// `perform_remaining_draws` (instead of computing `remaining - 1 - i`) — the
/// resumed tail's own deferred entry claims zero further draws owed and the
/// sequence's last draw (draw 3) is silently lost when this entry is
/// eventually answered.
fn test_dx23_remaining_bookkeeping_survives_a_tail_deferral() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = dredge_fixture(p1, p2, 3, 10);

    // Draw 1 offers dredge, entry{remaining: 2} pushed.
    draw_cards_effect(&mut state, p1, 3);
    assert_eq!(state.pending_draws()[0].remaining, 2);

    // Decline draw 1 -- draw 1 completes, tail's draw 2 is offered dredge
    // again (the dredge card is still in the graveyard) and its own entry is
    // pushed.
    let (state, decline_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();
    assert!(
        decline_events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1)),
        "sanity: draw 2 must be re-offered for this bookkeeping check to be \
         meaningful. Events: {:?}",
        decline_events
    );

    assert_eq!(
        state.pending_draws().len(),
        1,
        "exactly one entry for the tail's own (draw 2) offer"
    );
    assert_eq!(
        state.pending_draws()[0].remaining,
        1,
        "CR 614.11a/121.2: after draw 1 completes and draw 2 is offered and \
         deferred, exactly ONE further draw (draw 3) is still owed -- not \
         zero (which would silently drop draw 3 forever) and not 2 (which \
         would double-count draw 2)."
    );

    // Drain the rest to confirm draw 3 is not, in fact, lost: decline draw 2,
    // then decline draw 3.
    let (state, decline2_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();
    assert!(
        decline2_events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "draw 2 completes. Events: {:?}",
        decline2_events
    );
    assert_eq!(state.pending_draws().len(), 1, "draw 3 is offered in turn");
    assert_eq!(state.pending_draws()[0].remaining, 0);

    let (state, decline3_events) = process_command(
        state,
        Command::ChooseDredge {
            player: p1,
            card: None,
        },
    )
    .unwrap();
    assert!(
        decline3_events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "draw 3 completes -- it was not lost. Events: {:?}",
        decline3_events
    );
    assert!(state.pending_draws().is_empty());
}
