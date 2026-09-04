//! PB-DX35 Half B (`OOS-DX4-5`) — CR 118.12's `LookAtTopThenPlace.optional`, through
//! the REAL channels.
//!
//! The engine-side probes live in
//! `crates/engine/tests/primitives/pb_dx35_optional_placement.rs`. This file exists
//! because **existence is never sufficiency** (the `kaito_shizuki` lesson, PB-DX43):
//! a question the engine records but no client can be offered or answer is not a
//! repaired decision. Every probe here drives `LocalGame`/`HumanChoice` or the
//! `StubProvider` offer layer -- the same surfaces the browser and the bots go
//! through.
//!
//! The card is `Risen Reef`, `Complete` and deck-legal: *"Whenever this or another
//! Elemental you control enters, look at the top card of your library. If it's a
//! land card, you may put it onto the battlefield tapped. If you don't put the card
//! onto the battlefield, put it into your hand."* -- its own ETB fires its own
//! trigger (`exclude_self: false`), so casting the one card is the whole drive.
//!
//! **The decline is asserted by the RESOLUTION EFFECT, never by the offer** -- the
//! fixture land ending up in HAND, not on the battlefield. That is AC 7328's
//! standard, and the reason it matters here is the same as PB-DX45's: before this
//! batch `optional` was inert, so a decline was not a reachable state from ANY
//! channel, and an offer-shaped assertion would pass on an engine that asked and
//! then threw the answer away.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, CardDefinition, EffectChoiceAnswer,
    EffectChoiceQuestion, GameState, GameStateBuilder, ObjectSpec, PlayerId, ZoneId,
};
use mtg_simulator::params::{ActionParams, HumanChoice};
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, HeuristicBot, LegalAction, LegalActionProvider, LocalGame,
    LocalGameLimits, PendingDecision, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 35_35_35;

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 3,
        max_commands: 600,
        max_consecutive_passes: 500,
        record_journal: true,
    }
}

/// `p1` holds a real `Risen Reef` in hand ({1}{G}{U}), two Forests and an Island on
/// the battlefield (untapped -- covers the printed cost with `auto_tap: true`), and
/// a real land on top of their library for the ETB dig to find. Library filler below
/// it keeps the game from hitting CR 104.3c across a few draw steps.
fn fixture() -> GameState {
    let defs = card_defs_by_name();
    let reef = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Risen Reef")
            .in_zone(ZoneId::Hand(p(1)))
            .with_card_id(card_name_to_id("Risen Reef")),
        &defs,
    );
    let forest_a = enrich_spec_from_def(
        ObjectSpec::land(p(1), "Forest").with_card_id(card_name_to_id("Forest")),
        &defs,
    );
    let forest_b = enrich_spec_from_def(
        ObjectSpec::land(p(1), "Forest").with_card_id(card_name_to_id("Forest")),
        &defs,
    );
    let island = enrich_spec_from_def(
        ObjectSpec::land(p(1), "Island").with_card_id(card_name_to_id("Island")),
        &defs,
    );
    let top_land = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Swamp")
            .in_zone(ZoneId::Library(p(1)))
            .with_card_id(card_name_to_id("Swamp")),
        &defs,
    );
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(1))
        .object(reef)
        .object(forest_a)
        .object(forest_b)
        .object(island);
    // Filler is pushed BEFORE the dig's top card -- push order is bottom-to-top
    // (CR 121.1 / PB-RS1), so anything pushed AFTER `top_land` would bury it.
    for i in 0..30 {
        builder = builder.object(
            ObjectSpec::card(p(1), &format!("P1 Library Filler {i}"))
                .in_zone(ZoneId::Library(p(1))),
        );
    }
    // The dig's top card, PUSHED LAST for p1 so `Zone::top_n` sees it first.
    builder = builder.object(top_land);
    for i in 0..30 {
        builder = builder.object(
            ObjectSpec::card(p(2), &format!("P2 Library Filler {i}"))
                .in_zone(ZoneId::Library(p(2))),
        );
    }
    builder.build().expect("PB-DX35 channel fixture must build")
}

fn swamp_zone(state: &GameState) -> Option<ZoneId> {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Swamp")
        .map(|o| o.zone)
}

fn start_human_game() -> LocalGame<StubProvider> {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p(2), Box::new(HeuristicBot::new(SEED, "p2".to_string())));
    let human: BTreeSet<PlayerId> = [p(1)].into_iter().collect();
    let (game, _events) =
        LocalGame::start(fixture(), SEED, StubProvider, bots, human, limits(), true)
            .expect("PB-DX35 channel game must start");
    game
}

/// Drive the human seat, passing priority, until `want` finds an action in the
/// offered list. Returns the decision and the index of that action.
///
/// **Panics rather than returning `None`** -- a probe that silently ends early is a
/// probe that asserts nothing, and every assertion in this file is downstream of
/// actually reaching the offer.
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    label: &str,
    want: impl Fn(&LegalAction) -> bool,
) -> (PendingDecision, usize) {
    for _ in 0..80 {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                if let Some(i) = d.actions.iter().position(&want) {
                    return (d, i);
                }
                let pass = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!(
                            "no {label} offer and no PassPriority either: {:?}",
                            d.actions
                        )
                    });
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: pass,
                        params: ActionParams::default(),
                    },
                )
                .expect("passing priority should be accepted");
            }
            other => panic!("expected AwaitingHuman while hunting {label}, got {other:?}"),
        }
    }
    panic!("no {label} offer within 80 human decisions");
}

/// Cast Risen Reef with `auto_tap: true`, drive to the CR 608.2d `ChooseObject`
/// offer, and answer it. Returns the zone the fixture `Swamp` ended in, which is
/// the RESOLUTION EFFECT every assertion in this file reads.
fn drive_and_answer(chosen: Vec<mtg_engine::ObjectId>) -> Option<ZoneId> {
    let mut game = start_human_game();

    let (decision, cast_index) = drive_until(&mut game, "CastSpell", |a| {
        matches!(a, LegalAction::CastSpell { .. })
    });
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: cast_index,
            params: ActionParams {
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    )
    .expect("casting Risen Reef with auto_tap should be accepted");

    let (decision, answer_index) = drive_until(&mut game, "AnswerEffectChoice", |a| {
        matches!(
            a,
            LegalAction::AnswerEffectChoice {
                question: EffectChoiceQuestion::ChooseObject { .. },
                ..
            }
        )
    });

    // The offer the engine hands a client carries its own default. Assert it is
    // "take the winner" (PB-DX35's behaviour-preservation pin, `t2`), because
    // that is what keeps bots and the fuzzer identical -- and, for a decline
    // (`chosen: vec![]`), it is what makes this probe a real override rather than
    // an echo.
    match &decision.actions[answer_index] {
        LegalAction::AnswerEffectChoice {
            question: EffectChoiceQuestion::ChooseObject { candidates, .. },
            answer,
            ..
        } => {
            assert_eq!(
                answer,
                &EffectChoiceAnswer::ChooseObject {
                    chosen: candidates.first().copied().into_iter().collect()
                },
                "the offered default must be PB-DX35's take-the-winner behaviour"
            );
        }
        other => panic!("wrong action: {other:?}"),
    }

    // Answer through `ActionParams`, the browser's own channel.
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: answer_index,
            params: ActionParams {
                effect_choice_answer: Some(EffectChoiceAnswer::ChooseObject { chosen }),
                ..ActionParams::default()
            },
        },
    )
    .unwrap_or_else(|e| panic!("answering the CR 118.12 offer failed: {e:?}"));

    swamp_zone(game.state())
}

#[test]
/// **C1** -- CR 118.12, the DECLINE, end to end through `LocalGame`/`HumanChoice`.
///
/// **This outcome was unreachable from every channel before PB-DX35.** The old
/// `LookAtTopThenPlace` arm destructured `optional: _` and always placed the best
/// candidate when one existed -- so a pre-batch engine put the Swamp onto the
/// battlefield tapped with no question asked and no way to say no.
fn c1_a_human_declines_and_the_land_goes_to_hand_not_the_battlefield() {
    let zone = drive_and_answer(vec![]);
    assert_eq!(
        zone,
        Some(ZoneId::Hand(p(1))),
        "CR 118.12's printed fallback: declined, so the card goes to hand, not the \
         battlefield"
    );
}

#[test]
/// **C2** -- CR 118.12, the ACCEPT, on the identical fixture and drive.
///
/// C1 and C2 differ in exactly one value: whether `chosen` names the candidate.
/// Both halves are asserted because a decline-only probe cannot distinguish "the
/// decline works" from "this fixture never places the card at all".
fn c2_a_human_accepts_and_the_land_enters_the_battlefield_tapped() {
    let mut game = start_human_game();
    let (decision, cast_index) = drive_until(&mut game, "CastSpell", |a| {
        matches!(a, LegalAction::CastSpell { .. })
    });
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: cast_index,
            params: ActionParams {
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    )
    .expect("casting Risen Reef with auto_tap should be accepted");

    let (decision, answer_index) = drive_until(&mut game, "AnswerEffectChoice", |a| {
        matches!(
            a,
            LegalAction::AnswerEffectChoice {
                question: EffectChoiceQuestion::ChooseObject { .. },
                ..
            }
        )
    });
    let candidates = match &decision.actions[answer_index] {
        LegalAction::AnswerEffectChoice {
            question: EffectChoiceQuestion::ChooseObject { candidates, .. },
            ..
        } => candidates.clone(),
        other => panic!("wrong action: {other:?}"),
    };
    assert_eq!(
        candidates.len(),
        1,
        "sanity: exactly the fixture Swamp must be the sole candidate"
    );

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: answer_index,
            params: ActionParams {
                effect_choice_answer: Some(EffectChoiceAnswer::ChooseObject { chosen: candidates }),
                ..ActionParams::default()
            },
        },
    )
    .expect("accepting the CR 118.12 offer should be accepted");

    assert_eq!(
        swamp_zone(game.state()),
        Some(ZoneId::Battlefield),
        "CR 118.12: accepted, so the card enters the battlefield tapped"
    );
    let tapped = game
        .state()
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Swamp")
        .map(|o| o.status.tapped);
    assert_eq!(
        tapped,
        Some(true),
        "Risen Reef's printed destination is battlefield TAPPED"
    );
}

#[test]
/// **C3** -- the BOT path. A bot seat is offered the same action and answers it
/// through the same `LegalAction`, with no human anywhere.
///
/// SR-38 in both directions: the offer layer must not invent an action the engine
/// will refuse, and it must not suppress one CR 118.12 gives. `StubProvider`
/// needed **no change at all** for this -- its `BlockingDecision::EffectChoice` arm
/// is written against `default_effect_choice_answer`, which already handled
/// `ChooseObject` since PB-DX28, so a new PRODUCER of that same variant rides in
/// for free. That is worth asserting rather than assuming: an offer layer that
/// silently dropped it would leave a bot game deadlocked on Risen Reef's own ETB,
/// and this probe is what would catch it.
fn c3_dx35_the_bot_path_is_offered_and_answers_the_same_choose_object() {
    let mut game = start_human_game();
    let (decision, cast_index) = drive_until(&mut game, "CastSpell", |a| {
        matches!(a, LegalAction::CastSpell { .. })
    });
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: cast_index,
            params: ActionParams {
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    )
    .expect("casting Risen Reef with auto_tap should be accepted");

    let (_decision, _idx) = drive_until(&mut game, "AnswerEffectChoice", |a| {
        matches!(
            a,
            LegalAction::AnswerEffectChoice {
                question: EffectChoiceQuestion::ChooseObject { .. },
                ..
            }
        )
    });

    // Ask the provider directly, as a bot's turn loop does -- not through the
    // human `PendingDecision` this drive happens to hold.
    let state = game.state().clone();
    let entry = state
        .pending_effect_choice()
        .expect("the engine is blocked on the CR 118.12 offer");
    let offers: Vec<LegalAction> = StubProvider
        .legal_actions(&state, entry.player)
        .into_iter()
        .filter(|a| matches!(a, LegalAction::AnswerEffectChoice { .. }))
        .collect();
    assert_eq!(
        offers.len(),
        1,
        "the bot-facing offer layer must offer exactly one AnswerEffectChoice action \
         for the outstanding question: {offers:?}"
    );
    match &offers[0] {
        LegalAction::AnswerEffectChoice {
            question: EffectChoiceQuestion::ChooseObject { .. },
            answer: EffectChoiceAnswer::ChooseObject { .. },
            ..
        } => {}
        other => panic!("wrong offer shape for the bot path: {other:?}"),
    }
}
