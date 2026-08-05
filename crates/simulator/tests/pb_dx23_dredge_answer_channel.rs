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
    all_cards, card_name_to_id, enrich_spec_from_def, CardDefinition, GameEvent, GameStateBuilder,
    KeywordAbility, ObjectSpec, PlayerId, ZoneId,
};
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, HaltReason, HeuristicBot, LocalGame, LocalGameLimits,
    StubProvider,
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
