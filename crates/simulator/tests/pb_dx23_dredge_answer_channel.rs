//! PB-DX23 — dredge has no answer channel for anyone (`OOS-DX2-5`).
//!
//! This is the batch's mandatory probe (plan §3 Q5, §5 T1.1), and its Stage-0
//! deliverable is the BEFORE picture: run against unmodified production source it is
//! **expected to FAIL**. That failure is Stage 0's proof that the discriminator is
//! real rather than a tautology — see `memory/primitives/pb-DX23-execution-notes.md`
//! for the literal pre-fix numbers this file produced.
//!
//! CR 702.52a — "Dredge is a static ability that functions only while the card with
//! dredge is in a player's graveyard. 'Dredge N' means '... if you would draw a
//! card, you may instead mill N cards and return this card from your graveyard to
//! your hand.'"
//! CR 121.1 — a card is drawn as a turn-based action during each player's draw step.
//! CR 103.8a — in a two-player game, the player who plays first skips the draw on
//! their own first turn. Both players here are bots, `p1` is active_player and
//! therefore plays first, so "three of `p1`'s own turns" (turns 1, 3, 5, since the
//! fixture halts once the turn counter passes 6) means only **two** real draw
//! steps — turn 1's draw is skipped, turns 3 and 5's are not.
//!
//! **"No state pokes", defined** (plan §3 Q5):
//! - PERMITTED: anything expressible on `GameStateBuilder` *before*
//!   `LocalGame::start` — the registry, players, zone contents (including the real
//!   `golgari_grave_troll` def in `p1`'s graveyard via a `CardRegistry`, enriched
//!   with `enrich_spec_from_def` so `characteristics.keywords` actually carries
//!   `Dredge(6)` — the standing `ObjectSpec::card` gotcha), library stocking, the
//!   starting step (left to `LocalGame::start`'s own `start_game` call, which sets
//!   `is_first_turn_of_game` and resets to `Step::Untap` on its own).
//! - FORBIDDEN: any mutation of `GameState` after `start` — in particular
//!   `state.pending_draws` — and any direct call from the test body to
//!   `perform_one_draw`, `turn_actions::draw_card`, `replacement::*`, or
//!   `process_command`. Every command in this test comes from `advance()`'s own bot
//!   path (`StubProvider` + `HeuristicBot`, both seats — `human_seats` is empty).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition, Command,
    EffectDuration, GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec,
    PlayerFilter, PlayerId, ReplacementEffect, ReplacementId, ReplacementModification,
    ReplacementTrigger, Step, ZoneId,
};
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, HaltReason, HeuristicBot, LegalAction,
    LegalActionProvider, LocalGame, LocalGameLimits, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Every `Complete` card definition, keyed by name — the shape `enrich_spec_from_def`
/// wants, mirroring `crates/engine/tests/mechanics_e_l/golgari_grave_troll.rs`'s own
/// `build_defs_and_registry` helper.
fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// Seed used for both the `LocalGame` RNG stream and the two `HeuristicBot`s. Not a
/// recorded fuzz seed (this fixture never goes through `StubProvider`'s fuzz-shaped
/// deck pool), so nothing in `memory/primitives/pb-plan-DX23.md` §6 R1's ratchet
/// table is affected by this constant's value.
const SEED: u64 = 70252;

/// Library filler per player — no `card_id`, so Architecture Invariant 9's
/// completeness gate (`start_game`'s `check_all_defs_complete`) never sees them.
/// 40 is the plan's own floor (§3 Q5): enough to survive the run and to keep
/// `library_count >= 12` so `HeuristicBot`'s CR 702.52b 2x-margin dredge policy
/// (once it exists, Stage 4) has room to say yes.
const LIBRARY_SIZE: usize = 40;

/// The turn cap. `LocalGame::advance()` halts once `turn_number > max_turns`
/// (`local_game.rs:713-718`), so `max_turns: 6` runs turns 1-6 to completion (three
/// of `p1`'s own turns: 1, 3, 5) and halts before turn 7 begins.
const MAX_TURNS: u32 = 6;

/// **T1.1** — CR 702.52a, CR 121.1, CR 103.8a. `p1` carries a real
/// `golgari_grave_troll` in their graveyard through an entire real, both-bot-seat
/// game (`StubProvider` + `HeuristicBot`) and nothing ever answers its dredge
/// offers, because at HEAD `LegalAction::ChooseDredge` does not exist
/// (`grep -rn "ChooseDredge" crates/simulator/src/ tools/` returns 0 hits — Stage 0
/// step 5). The draw cadence corruption this causes (plan §1.1) is asserted three
/// ways: A1 is the primary, bot-policy-robust check; A2 is the exact arithmetic; A3
/// is a non-vacuity floor without which A1/A2 could pass on a fixture that never
/// actually reaches a dredge offer.
///
/// **Revert to watch red** (post-fix): delete the `ChooseDredge` push block from
/// `StubProvider::legal_actions`. A1 and A2 must both redden, with the rebuild
/// confirmed (`Compiling mtg-simulator` observed in the captured output).
#[test]
fn test_dx23_real_game_with_a_grave_troll_keeps_its_draw_cadence() {
    let p1 = p(1);
    let p2 = p(2);

    let defs = card_defs_by_name();
    let registry = build_registry();

    let troll_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Golgari Grave-Troll")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(card_name_to_id("Golgari Grave-Troll")),
        &defs,
    );

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(troll_spec);

    for player in [p1, p2] {
        for i in 0..LIBRARY_SIZE {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Library Filler {i}"))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }

    let state = builder.build().expect("PB-DX23 T1.1 fixture must build");

    // Non-vacuity precondition (plan §3 Q5): if enrichment silently failed to carry
    // the keyword, the whole probe is meaningless — fail loudly here rather than
    // downstream in a confusing A1/A2/A3 mismatch.
    let (troll_id, troll_obj) = state
        .objects()
        .iter()
        .find(|(_, o)| {
            o.characteristics.name == "Golgari Grave-Troll" && o.zone == ZoneId::Graveyard(p1)
        })
        .expect("Golgari Grave-Troll must be in p1's graveyard before LocalGame::start");
    assert!(
        troll_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Dredge(6)),
        "precondition failed: enrich_spec_from_def did not carry Dredge(6) onto object {:?} \
         (name={}) -- got keywords {:?}. Without this the whole probe is vacuous.",
        troll_id,
        troll_obj.characteristics.name,
        troll_obj.characteristics.keywords,
    );

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        p1,
        Box::new(HeuristicBot::new(SEED, "p1-heuristic".to_string())),
    );
    bots.insert(
        p2,
        Box::new(HeuristicBot::new(SEED + 1, "p2-heuristic".to_string())),
    );

    let limits = LocalGameLimits {
        max_turns: MAX_TURNS,
        max_commands: MAX_TURNS * 200,
        max_consecutive_passes: 500,
        record_journal: true,
    };

    let (mut game, _start_events) = LocalGame::start(
        state,
        SEED,
        StubProvider,
        bots,
        BTreeSet::new(), // human_seats empty -- both seats are bots (§3 Q5)
        limits,
        true, // check_invariants
    )
    .expect("PB-DX23 T1.1 LocalGame::start must succeed");

    // With no human seats, a single advance() call runs the whole game internally
    // and only returns at GameOver or Halted (mirrors pb_dx32_fuzz_output.rs's own
    // `play_fuzz_shaped` helper and its comment).
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(_) => unreachable!("no human seats in this fixture"),
        AdvanceOutcome::GameOver(result) => panic!(
            "PB-DX23 T1.1 fixture concluded before the turn-6 cap was reached \
             (winner={:?}) -- the fixture must be widened (bigger libraries?) so the \
             probe actually observes three of p1's own turns. GameResult: {:?}",
            result.winner, result
        ),
        AdvanceOutcome::Halted(HaltReason::MaxTurns { max_turns, turn }) => {
            assert_eq!(
                max_turns, MAX_TURNS,
                "sanity: the configured cap round-trips"
            );
            assert_eq!(
                turn,
                MAX_TURNS + 1,
                "MaxTurns halts once turn_number > max_turns, i.e. at the first tick \
                 of turn {}, meaning turns 1..={} ran to completion",
                MAX_TURNS + 1,
                MAX_TURNS
            );
        }
        AdvanceOutcome::Halted(other) => panic!(
            "PB-DX23 T1.1 fixture halted for an unexpected reason before the turn-6 \
             cap: {other:?}"
        ),
    }

    // ---- A1 (primary, robust to bot policy): no PendingDraw survives to the halt.
    let pending_at_halt = game.state().pending_draws().len();

    // ---- A2 (the arithmetic) + A3 (non-vacuity floor). All three counted from the
    // real journal of applied bot commands -- never from a hand-computed prediction.
    let mut card_drawn_p1 = 0usize;
    let mut dredged_p1 = 0usize;
    let mut dredge_choice_required_p1 = 0usize;
    let mut turn_started_p1_with_draw_eligible = 0usize;
    for record in game.journal() {
        for event in &record.events {
            match event {
                GameEvent::CardDrawn { player, .. } if *player == p1 => card_drawn_p1 += 1,
                GameEvent::Dredged { player, .. } if *player == p1 => dredged_p1 += 1,
                GameEvent::DredgeChoiceRequired { player, .. } if *player == p1 => {
                    dredge_choice_required_p1 += 1
                }
                // CR 103.8a: only turn_number == 1 is exempt from the draw in a
                // <=2-player game, and turn 1's TurnStarted is emitted by
                // start_game() itself -- BEFORE the journal begins -- so every
                // TurnStarted{player: p1} that DOES appear in the journal is
                // already past the CR 103.8a exemption.
                //
                // MEASURED DIVERGENCE FROM THE NAIVE PREDICTION (recorded in
                // memory/primitives/pb-DX23-execution-notes.md): a first draft of
                // this filter counted every non-turn-1 TurnStarted{player: p1} and
                // got turn_number in {3, 5, 7} -- THREE, not two. Turn 7's
                // TurnStarted fires because `advance_turn()` runs (and journals its
                // event) as part of the LAST command applied inside turn 6 itself
                // (the transition that increments turn_number 6->7); `advance()`'s
                // own turn cap only checks turn_number > max_turns at the TOP of its
                // NEXT loop iteration (local_game.rs:713-718), so turn 7 is reported
                // as started but its draw step never runs -- the halt fires before
                // any command inside turn 7 is applied. So "started" is not the same
                // predicate as "reached a draw step"; the correct filter additionally
                // requires turn_number <= MAX_TURNS, which is what actually bounds
                // which turns got to run their draw step at all.
                GameEvent::TurnStarted {
                    player,
                    turn_number,
                } if *player == p1 => {
                    assert_ne!(
                        *turn_number, 1,
                        "turn 1's TurnStarted must never appear in the journal -- \
                         it is emitted by start_game() before LocalGame::start() \
                         returns, not by any journalled command"
                    );
                    if *turn_number <= MAX_TURNS {
                        turn_started_p1_with_draw_eligible += 1;
                    }
                }
                _ => {}
            }
        }
    }

    let a2_lhs = card_drawn_p1 + dredged_p1;
    let a2_rhs = turn_started_p1_with_draw_eligible;

    eprintln!(
        "PB-DX23 T1.1 pre-fix measurement: pending_draws_at_halt={pending_at_halt}, \
         card_drawn_p1={card_drawn_p1}, dredged_p1={dredged_p1}, a2_lhs={a2_lhs}, \
         a2_rhs(p1 draw-eligible turns)={a2_rhs}, \
         dredge_choice_required_p1={dredge_choice_required_p1}"
    );

    // A3 first: a fixture that never reaches an offer makes A1/A2 vacuous.
    assert!(
        dredge_choice_required_p1 >= 1,
        "A3 non-vacuity floor: p1 must have been offered at least one dredge \
         choice (CR 702.52a) across the run, or A1/A2 prove nothing. Observed 0. \
         Diagnose the fixture (enrichment? library floor? draw path?) before \
         concluding anything about the engine."
    );

    // A1 -- the primary, bot-policy-robust assertion.
    assert_eq!(
        pending_at_halt,
        0,
        "CR 702.52a/121.1: no PendingDraw should survive to the halt of a real \
         game once dredge offers are answerable. pending_draws() at halt: {:?}",
        game.state().pending_draws()
    );

    // A2 -- the exact arithmetic: every draw-eligible p1 turn must have produced
    // exactly one CardDrawn-or-Dredged event for p1.
    assert_eq!(
        a2_lhs, a2_rhs,
        "CR 121.1/702.52a: count(CardDrawn{{player: p1}}) + count(Dredged{{player: \
         p1}}) ({a2_lhs}) must equal the number of p1 draw-eligible turns that \
         occurred ({a2_rhs}). card_drawn_p1={card_drawn_p1} dredged_p1={dredged_p1}"
    );
}

// ── T4 -- provider and bot (plan §5 T4, acceptance criterion 1, bot half) ──────

/// Pass priority for `players`, in order, once each -- mirrors
/// `crates/engine/tests/primitives/pb_dx2_command_gates.rs::pass_all` (SR-9a:
/// a fresh copy, not a shared import across integration-test targets).
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

/// Build a state where `p1` has exactly one dredge-eligible card (a real,
/// enriched `Golgari Grave-Troll`, Dredge 6) in their graveyard and a
/// `PendingDraw` is outstanding for them, raised by a REAL draw-step draw
/// (mirrors `pb_dx2_command_gates.rs::build_upkeep_state` + `pass_all`, not a
/// hand-poked `pending_draws` field). `library_count` controls `p1`'s library
/// size at the moment of the offer -- callers pick it to land above or below
/// the heuristic bot's 2x-margin threshold (T4.5).
fn build_single_dredge_offer_state(
    p1: PlayerId,
    p2: PlayerId,
    library_count: usize,
) -> (GameState, ObjectId) {
    let defs = card_defs_by_name();
    let registry = build_registry();

    let troll_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Golgari Grave-Troll")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(card_name_to_id("Golgari Grave-Troll")),
        &defs,
    );

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(troll_spec);

    for i in 0..library_count {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("P1 Library Filler {i}")).in_zone(ZoneId::Library(p1)),
        );
    }
    // p2 needs at least one library card so their own upkeep/draw-step passes
    // do not themselves trip an unrelated deck-out SBA before this fixture's
    // assertions run.
    builder =
        builder.object(ObjectSpec::card(p2, "P2 Library Filler 0").in_zone(ZoneId::Library(p2)));

    let mut state = builder.build().expect("PB-DX23 T4 fixture must build");
    state.turn_mut().is_first_turn_of_game = false;
    state.turn_mut().priority_holder = Some(p1);

    let troll_id = state
        .objects()
        .iter()
        .find(|(_, o)| {
            o.characteristics.name == "Golgari Grave-Troll" && o.zone == ZoneId::Graveyard(p1)
        })
        .map(|(id, _)| *id)
        .expect("troll must be in p1's graveyard before the draw step runs");

    let (state, events) = pass_all(state, &[p1, p2]);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DredgeChoiceRequired { player, .. } if *player == p1)),
        "fixture precondition: dredge must be offered at the draw step. Events: {:?}",
        events
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "fixture precondition: exactly one PendingDraw must be outstanding for p1"
    );

    (state, troll_id)
}

/// **T4.1** -- CR 702.52a/b (plan §5 T4.1). With one dredge-eligible card the
/// provider offers the always-legal decline PLUS exactly one `Some` entry for
/// that card, and nothing more.
///
/// **Revert to watch red**: drop the `None` push in `StubProvider::legal_actions`'s
/// dredge block -- the decline assertion fails and the count drops to 1.
#[test]
fn test_dx23_provider_offers_decline_plus_one_per_eligible_card() {
    let p1 = PlayerId(4101);
    let p2 = PlayerId(4102);
    let (state, troll_id) = build_single_dredge_offer_state(p1, p2, 12);

    let actions = StubProvider.legal_actions(&state, p1);
    let dredge_actions: Vec<&LegalAction> = actions
        .iter()
        .filter(|a| matches!(a, LegalAction::ChooseDredge { .. }))
        .collect();

    assert!(
        dredge_actions.iter().any(|a| matches!(
            a,
            LegalAction::ChooseDredge {
                card: None,
                mill: 0
            }
        )),
        "CR 702.52a: the always-legal decline must be offered. dredge actions: {:?}",
        dredge_actions
    );
    assert!(
        dredge_actions.iter().any(
            |a| matches!(a, LegalAction::ChooseDredge { card: Some(id), mill: 6 } if *id == troll_id)
        ),
        "CR 702.52a/b: the eligible troll must be offered with mill == its Dredge N (6). \
         dredge actions: {:?}",
        dredge_actions
    );
    assert_eq!(
        dredge_actions.len(),
        2,
        "exactly one decline PLUS one per eligible card -- 1 eligible card here, so 2 \
         total. dredge actions: {:?}",
        dredge_actions
    );
}

/// **T4.2** -- CR 702.52a, 616.1e (plan §5 T4.2). A `NeedsChoice`-origin
/// `PendingDraw` (CR 616.1e: two ambiguous `WouldDraw` replacements, zero
/// dredge cards) must be offered NOTHING through this channel -- not even the
/// bare decline. Without the suppression, `ChooseDredge { None }` would
/// re-defer this exact entry rather than discharge it (plan §3 Q2, pinned from
/// the other side by `pb_dx2_command_gates.rs::
/// test_dx2_choose_dredge_some_can_answer_a_needschoice_originated_entry`).
///
/// **Revert to watch red**: remove the `dredge_options(..).is_empty()` guard
/// in `StubProvider::legal_actions` -- a bare decline appears for this
/// zero-eligible-card entry.
#[test]
fn test_dx23_provider_offers_nothing_when_no_dredge_card_is_eligible() {
    let p1 = PlayerId(4201);
    let p2 = PlayerId(4202);

    let skip_a = ReplacementEffect {
        id: ReplacementId(942_010),
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
        id: ReplacementId(942_011),
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
        .with_replacement_effect(skip_a)
        .with_replacement_effect(skip_b)
        .build()
        .expect("PB-DX23 T4.2 fixture must build");

    // Mirrors pb_dx2_command_gates.rs T19's own fixture-construction path: call
    // the draw directly (no dredge card exists in this fixture at all, so
    // there is no dredge-offer stage to pass through first).
    let events = mtg_engine::rules::turn_actions::draw_card(&mut state, p1)
        .unwrap_or_else(|e| panic!("draw_card failed: {:?}", e));
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::ReplacementChoiceRequired { player, .. } if *player == p1)
        ),
        "fixture precondition: two ambiguous WouldDraw replacements must raise \
         NeedsChoice. Events: {:?}",
        events
    );
    assert_eq!(
        state.pending_draws().len(),
        1,
        "fixture precondition: a NeedsChoice-origin PendingDraw must be recorded"
    );
    assert!(
        mtg_engine::rules::queries::dredge_options(&state, p1).is_empty(),
        "fixture precondition (zero corpus reach, plan §1.4): no dredge card exists in \
         this fixture, so dredge_options must be empty"
    );

    state.turn_mut().priority_holder = Some(p1);
    let actions = StubProvider.legal_actions(&state, p1);
    let dredge_actions: Vec<&LegalAction> = actions
        .iter()
        .filter(|a| matches!(a, LegalAction::ChooseDredge { .. }))
        .collect();
    assert!(
        dredge_actions.is_empty(),
        "CR 702.52a/616.1e (plan §3 Q2): a NeedsChoice-origin PendingDraw with zero \
         dredge-eligible cards must not be offered even the bare decline -- offering it \
         would re-defer the entry instead of discharging it. Offered: {:?}",
        dredge_actions
    );
}

/// Drive `state` forward, answering every priority window with `PassPriority`
/// from whoever currently holds it, until `state.blocking_decision()` is
/// `Some`. A dredge `PendingDraw` left unanswered along the way does not
/// block this (plan §1.1: "priority, SBAs and step advancement all
/// continue"). Panics if no blocking decision appears within `max_commands`.
fn drive_to_a_blocking_decision(mut state: GameState, max_commands: usize) -> GameState {
    for _ in 0..max_commands {
        if state.blocking_decision().is_some() {
            return state;
        }
        let holder = state.turn().priority_holder.unwrap_or_else(|| {
            panic!(
                "driver stuck: no priority holder and no blocking decision. turn={:?}",
                state.turn()
            )
        });
        let (next_state, _events) =
            process_command(state, Command::PassPriority { player: holder })
                .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = next_state;
    }
    panic!("did not reach a blocking decision within {max_commands} commands");
}

/// **T4.3** -- CR 514.1 (admission gate, `engine.rs:304-314`; plan §5 T4.3).
/// While a `BlockingDecision` stands (here, CR 514.1's cleanup discard), the
/// provider offers NOTHING but the answer to that decision -- in particular
/// no `ChooseDredge`, even though a dredge `PendingDraw` from earlier in the
/// same turn is still outstanding and unanswered.
///
/// **Revert to watch red**: move the dredge emission block ABOVE the
/// `blocking_decision()` early return in `StubProvider::legal_actions` -- the
/// offer appears alongside the cleanup discard, i.e. an action the engine's
/// admission gate would reject (`BlockedByPendingDecision`, SR-38).
#[test]
fn test_dx23_provider_is_silent_while_a_blocking_decision_stands() {
    let p1 = PlayerId(4301);
    let p2 = PlayerId(4302);

    let defs = card_defs_by_name();
    let registry = build_registry();
    let troll_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Golgari Grave-Troll")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(card_name_to_id("Golgari Grave-Troll")),
        &defs,
    );

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(troll_spec);
    // 8 hand cards > the default max hand size of 7 (mirrors
    // `local_game.rs::state_with_oversized_hand_for_p1`), so cleanup pauses.
    for i in 0..8u32 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Hand Filler {i}")).in_zone(ZoneId::Hand(p1)));
    }
    for i in 0..12u32 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("P1 Library Filler {i}")).in_zone(ZoneId::Library(p1)),
        );
    }
    builder =
        builder.object(ObjectSpec::card(p2, "P2 Library Filler 0").in_zone(ZoneId::Library(p2)));

    let mut state = builder.build().expect("PB-DX23 T4.3 fixture must build");
    // CR 103.8a: p1 must NOT skip their turn-1 draw, or the dredge offer this
    // test exists to observe alongside the cleanup block would never fire.
    state.turn_mut().is_first_turn_of_game = false;
    state.turn_mut().priority_holder = Some(p1);

    let state = drive_to_a_blocking_decision(state, 200);

    match state.blocking_decision() {
        Some(mtg_engine::rules::engine::BlockingDecision::CleanupDiscard { player, .. })
            if player == p1 => {}
        other => panic!(
            "fixture precondition: must halt on p1's cleanup discard. got {:?}",
            other
        ),
    }
    assert_eq!(
        state.pending_draws().len(),
        1,
        "fixture precondition: the draw-step dredge PendingDraw must still be \
         outstanding, unanswered, when cleanup blocks"
    );

    let actions = StubProvider.legal_actions(&state, p1);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, LegalAction::ChooseDredge { .. })),
        "CR 514.1/SR-38: while a blocking decision stands the provider must offer \
         nothing else -- a ChooseDredge alongside the cleanup discard would be an \
         action the engine's admission gate rejects. Offered: {:?}",
        actions
    );
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, LegalAction::DiscardToHandSize { .. })),
        "the cleanup discard itself must still be offered. Offered: {:?}",
        actions
    );
}

/// **T4.4** -- SR-38 (plan §5 T4.4). Every `ChooseDredge` action this provider
/// actually offers is one the engine accepts.
///
/// **Hazard, per the dispatching brief**: `process_command`'s `Err` arm
/// carries no `GameState`, so this test never inspects state through it --
/// only the `Ok`/`Err` discriminant.
///
/// **Revert to watch red**: temporarily make the provider's dredge block
/// offer a `Some(id)` for a graveyard object that has no `Dredge` keyword
/// (bypassing `dredge_options`'s own filter) -- `process_command` returns
/// `Err` for that action, and the loop below panics on the first non-`Ok`
/// result.
#[test]
fn test_dx23_every_offered_action_is_engine_accepted() {
    let p1 = PlayerId(4401);
    let p2 = PlayerId(4402);
    let (state, _troll_id) = build_single_dredge_offer_state(p1, p2, 12);

    let actions = StubProvider.legal_actions(&state, p1);
    let dredge_actions: Vec<&LegalAction> = actions
        .iter()
        .filter(|a| matches!(a, LegalAction::ChooseDredge { .. }))
        .collect();
    assert!(
        !dredge_actions.is_empty(),
        "non-vacuity: the fixture must actually offer at least one ChooseDredge"
    );

    for action in dredge_actions {
        let LegalAction::ChooseDredge { card, .. } = action else {
            unreachable!()
        };
        let result = process_command(
            state.clone(),
            Command::ChooseDredge {
                player: p1,
                card: *card,
            },
        );
        assert!(
            result.is_ok(),
            "SR-38: every action StubProvider offers must be accepted by the engine. \
             card={:?} error={:?}",
            card,
            result.err()
        );
    }
}

/// **T4.5** -- CR 702.52b, 104.3c (plan §5 T4.5 / §3 Q4). Below the heuristic
/// bot's 2x-library-headroom margin (`library_count >= 2 * mill`), the bot
/// must decline rather than mill itself toward CR 104.3c, even though the
/// engine and the offer both still consider the dredge itself LEGAL (library
/// == 7 >= mill == 6, so `Some` is genuinely offered here -- this is a
/// SURVIVAL policy, not a legality gate).
///
/// **Revert to watch red**: drop the `2 * mill` margin in
/// `HeuristicBot::score_action`'s `ChooseDredge { Some, .. }` arm (score it 3
/// unconditionally, matching the offer's own legality floor) -- the bot mills
/// itself at library == 7, well under 2x headroom.
#[test]
fn test_dx23_heuristic_bot_declines_rather_than_milling_itself_out() {
    let p1 = PlayerId(4501);
    let p2 = PlayerId(4502);
    // library_count == 7: >= mill (6), so CR 702.52b legality is satisfied and
    // `Some` is genuinely offered -- but < 2 * mill (12), so the bot's own
    // survival margin (plan §3 Q4) must refuse it.
    let (state, troll_id) = build_single_dredge_offer_state(p1, p2, 7);

    let actions = StubProvider.legal_actions(&state, p1);
    assert!(
        actions.iter().any(
            |a| matches!(a, LegalAction::ChooseDredge { card: Some(id), mill: 6 } if *id == troll_id)
        ),
        "non-vacuity: the offer must still contain Some(troll) at library == 7 (>= mill \
         == 6) -- this test is about bot POLICY, not engine legality. Offered: {:?}",
        actions
    );

    let mut bot = HeuristicBot::new(SEED, "p1-heuristic".to_string());
    match bot.choose_action(&state, p1, &actions) {
        Command::ChooseDredge { card: None, .. } => {}
        other => panic!(
            "CR 702.52b/104.3c (plan §3 Q4): below the 2x library margin the bot must \
             decline rather than mill itself out. Chose: {:?}",
            other
        ),
    }
}

/// **S1** (PB-DX23 review fix cycle, MEDIUM) -- CR 616.1e/616.1f, 702.52a.
///
/// The Q2 suppression guard (`dredge_options(state, player).is_empty()`) is a
/// property of the GRAVEYARD. The entry `handle_choose_dredge`'s `None` arm
/// actually answers is `state.pending_draws.iter().position(|p| p.player ==
/// player)` -- FIFO, oldest first, with no discriminator between a
/// dredge-origin and a `NeedsChoice`-origin entry. So the two-conjunct guard
/// passes (graveyard eligible) even when the FIFO entry it will actually
/// discharge is `NeedsChoice`-origin, and declining THAT entry re-defers it
/// rather than completing it -- the exact loop the guard exists to prevent.
///
/// This reproduces `pb_dx2_command_gates.rs::
/// test_dx2_needschoice_redefer_grows_the_queue`'s exact fixture recipe (same
/// `Dredge(3)` card, same two `SkipDraw` `WouldDraw` replacements watching
/// `p1`, same 4-card library) via the SAME direct-engine calls that test
/// uses (`turn_actions::draw_card` + `Command::ChooseDredge`, never a hand-poke
/// of `state.pending_draws`), to reach the two-entry queue state
/// `[NeedsChoice-origin, dredge-origin]` -- FIFO order, `NeedsChoice` first --
/// and then asserts the PROVIDER's behaviour against it, which
/// `pb_dx2_command_gates.rs` (an engine-level test with no provider) never
/// does.
///
/// **Revert to watch RED**: drop the third guard conjunct
/// (`fifo_decline_would_redefer`) from `StubProvider::legal_actions`'s dredge
/// block, restoring the two-conjunct (graveyard-only) guard this batch
/// originally shipped -- the provider re-offers `ChooseDredge` against the
/// queue's FIFO `NeedsChoice`-origin entry.
#[test]
fn test_dx23_provider_withholds_when_declining_would_re_defer() {
    let p1 = PlayerId(4601);
    let p2 = PlayerId(4602);
    let skip_a = ReplacementEffect {
        id: ReplacementId(9601),
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
        id: ReplacementId(9602),
        ..skip_a.clone()
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

    // Step 1: a real draw offers dredge first (dredge is checked before the
    // WouldDraw replacements) -- one dredge-origin entry, `[D1]`.
    mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    // Step 2: declining it re-checks the two SkipDraw replacements, which are
    // 2+ applicable -- CR 616.1 NeedsChoice -- and re-defers to a fresh
    // NeedsChoice-origin entry, `[N1]`.
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
        "sanity (matches pb_dx2_command_gates.rs's own fixture): the decline \
         re-defers to a fresh NeedsChoice entry"
    );

    // Step 3: a second, independent draw discharges the stale N1 entry first
    // (its own resume re-defers to N1'), THEN raises its own dredge offer for
    // THIS draw (the card is still in the graveyard) -- D2. Queue ends at
    // `[N1', D2]`, NeedsChoice FIRST -- the exact `OOS-DX2-3` two-entry trace,
    // reproduced live.
    mtg_engine::rules::turn_actions::draw_card(&mut state, p1).unwrap();
    assert_eq!(
        state.pending_draws().len(),
        2,
        "sanity (matches pb_dx2_command_gates.rs's own fixture): a second \
         independent draw must not clobber or merge with the re-raised entry"
    );

    // The FIFO-oldest entry (index 0) is the one `handle_choose_dredge`'s
    // `None` arm will actually answer. Confirm it is NOT the dredge-origin
    // one -- if this is ever wrong, the whole premise of S1 does not apply to
    // this fixture and the test below would be checking nothing.
    let fifo = &state.pending_draws()[0];
    assert!(
        fifo.already_applied.is_empty(),
        "S1 premise check: the FIFO-oldest entry must be the re-raised \
         NeedsChoice-origin one (empty already_applied, no replacement chosen \
         yet), not the dredge-origin D2. pending_draws: {:?}",
        state.pending_draws()
    );

    // The graveyard is still eligible (the Dredge Card was never actually
    // dredged away -- only declined). Under the two-conjunct guard this
    // batch originally shipped, that alone is enough to offer ChooseDredge.
    // Under the corrected three-conjunct guard, the provider must also see
    // that declining the FIFO entry (a NeedsChoice-origin one) would
    // re-defer rather than discharge -- and withhold.
    let actions = StubProvider.legal_actions(&state, p1);
    let dredge_actions: Vec<&LegalAction> = actions
        .iter()
        .filter(|a| matches!(a, LegalAction::ChooseDredge { .. }))
        .collect();
    assert!(
        dredge_actions.is_empty(),
        "S1: the provider must withhold ChooseDredge entirely while the \
         FIFO-oldest PendingDraw is NeedsChoice-origin, even though the \
         graveyard itself still has an eligible dredge card -- offering it \
         would answer the wrong entry and re-defer it forever. Offered: {:?}",
        dredge_actions
    );
}
