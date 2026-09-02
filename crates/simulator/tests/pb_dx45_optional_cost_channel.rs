//! PB-DX45 — CR 118.12's optional cost, through the REAL channels
//! (`OOS-DX24-9` ≡ `OOS-DX27-5`).
//!
//! The engine-side probes live in
//! `crates/engine/tests/primitives/pb_dx45_optional_cost.rs`. This file exists
//! because **existence is never sufficiency** (the `kaito_shizuki` lesson,
//! PB-DX43): a question the engine records but no client can be offered or answer
//! is not a repaired decision. Every probe here drives
//! `LocalGame`/`HumanChoice`, the `StubProvider` offer layer, or
//! `params::action_to_command_with_params` — the same three surfaces the browser
//! and the bots go through.
//!
//! The card is `nether_traitor`, the `Complete`, deck-legal def `OOS-DX24-9` was
//! filed about: *"Whenever another creature you own dies, **you may pay {B}**. If
//! you do, return this card from your graveyard to the battlefield."*
//!
//! **The decline is asserted by the RESOLUTION EFFECT, never by the offer** — the
//! Traitor still in the graveyard and the `{B}` still floating. That is the
//! standard AC 7241 sets, and it is the right one here for a specific reason:
//! before PB-DX45 the engine paid unconditionally, so a decline was not a
//! reachable state at all, and an offer-shaped assertion would pass on an engine
//! that asked and then threw the answer away.
//!
//! # The Swamp is not decoration
//!
//! `can_pay_optional_cost`'s `Cost::Mana` arm reads the FLOATING pool only (CR
//! 118.8 / CR 500.4), and a pool set on `GameStateBuilder` does not survive
//! `LocalGame::start`'s reset to `Step::Untap`. So these probes hand the human a
//! real Swamp and make them tap it, in the priority window while the Traitor's
//! trigger is on the stack. That is both more honest and the only thing that
//! works — and the fact that it is REQUIRED is the symptom filed as
//! `OOS-DX45-7` (a player asked to pay an optional mana cost at resolution gets
//! no window to activate mana abilities, so the offer is decided by whatever
//! happens to be floating).

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

const SEED: u64 = 45_45_45;

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

/// `p1` owns a real `nether_traitor` in their graveyard, a real `Swamp` on the
/// battlefield, and a `0/0` creature that CR 704.5f destroys at the first
/// state-based check — which is what fires the Traitor's
/// `WheneverCreatureDies { owner: You, exclude_self: true }` trigger with no
/// player action at all.
///
/// The `0/0` carries no `card_id`, so Architecture Invariant 9's completeness
/// gate never sees it (the `Library Filler` convention from PB-DX23).
fn fixture() -> GameState {
    let defs = card_defs_by_name();
    let traitor = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Nether Traitor")
            .in_zone(ZoneId::Graveyard(p(1)))
            .with_card_id(card_name_to_id("Nether Traitor")),
        &defs,
    );
    let swamp = enrich_spec_from_def(
        ObjectSpec::land(p(1), "Swamp").with_card_id(card_name_to_id("Swamp")),
        &defs,
    );
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(1))
        .object(traitor)
        .object(swamp)
        .object(ObjectSpec::creature(p(1), "DX45 Doomed Sliver", 0, 0));
    for player in [p(1), p(2)] {
        for i in 0..30 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Library Filler {i}"))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder.build().expect("PB-DX45 channel fixture must build")
}

fn traitor_zone(state: &GameState) -> Option<ZoneId> {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Nether Traitor")
        .map(|o| o.zone)
}

fn black_floating(state: &GameState) -> u32 {
    state
        .players()
        .get(&p(1))
        .expect("p1 exists")
        .mana_pool
        .black
}

fn start_human_game() -> LocalGame<StubProvider> {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p(2), Box::new(HeuristicBot::new(SEED, "p2".to_string())));
    let human: BTreeSet<PlayerId> = [p(1)].into_iter().collect();
    let (game, _events) =
        LocalGame::start(fixture(), SEED, StubProvider, bots, human, limits(), true)
            .expect("PB-DX45 channel game must start");
    game
}

/// Drive the human seat, passing priority, until `want` finds an action in the
/// offered list. Returns the decision and the index of that action.
///
/// **Panics rather than returning `None`** — a probe that silently ends early is
/// a probe that asserts nothing, and every assertion in this file is downstream
/// of actually reaching the offer.
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

/// Run the whole real-path drive and answer the CR 118.12 offer with `pay`.
///
/// Returns `(zone the Traitor ended in, black mana still floating)`. Both are
/// RESOLUTION EFFECTS — nothing here reads the offer to decide the verdict.
fn drive_and_answer(pay: bool) -> (Option<ZoneId>, u32) {
    let mut game = start_human_game();

    // 1. Tap the Swamp. The offer layer is asked for it; nothing is poked.
    let (decision, tap_index) = drive_until(&mut game, "TapForMana", |a| {
        matches!(a, LegalAction::TapForMana { .. })
    });
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: tap_index,
            params: ActionParams::default(),
        },
    )
    .expect("tapping the Swamp should be accepted");
    assert_eq!(
        black_floating(game.state()),
        1,
        "precondition: the Swamp must actually have produced {{B}}, or the CR 118.12 \
         offer below is unreachable for a reason that has nothing to do with this batch"
    );

    // 2. Pass until the Traitor's trigger resolves and the engine asks.
    let (decision, answer_index) = drive_until(&mut game, "AnswerEffectChoice", |a| {
        matches!(
            a,
            LegalAction::AnswerEffectChoice {
                question: EffectChoiceQuestion::PayOptionalCost { .. },
                ..
            }
        )
    });

    // The offer the engine hands a client carries its own default. Assert it is
    // the pre-batch auto-pay, because that is what keeps bots behaviourally
    // identical -- and, for `pay == false`, it is what makes this probe a real
    // override rather than an echo.
    match &decision.actions[answer_index] {
        LegalAction::AnswerEffectChoice { answer, .. } => assert_eq!(
            answer,
            &EffectChoiceAnswer::PayOptionalCost { pay: true },
            "the offered default must be the pre-PB-DX45 auto-pay"
        ),
        other => panic!("wrong action: {other:?}"),
    }

    // 3. Answer through `ActionParams`, the browser's own channel.
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: answer_index,
            params: ActionParams {
                effect_choice_answer: Some(EffectChoiceAnswer::PayOptionalCost { pay }),
                ..ActionParams::default()
            },
        },
    )
    .unwrap_or_else(|e| panic!("answering the CR 118.12 offer with pay={pay} failed: {e:?}"));

    (traitor_zone(game.state()), black_floating(game.state()))
}

#[test]
/// **C1** — CR 118.12, the DECLINE, end to end through `LocalGame`/`HumanChoice`.
///
/// **This outcome was unreachable from every channel before PB-DX45.** The old
/// `MayPayThenEffect` arm paid whenever `can_pay_optional_cost` was true, and by
/// step 1 of the drive it is true — the Swamp is tapped and the `{B}` is
/// floating. So a pre-batch engine returned the Traitor to the battlefield and
/// spent the mana, with no question asked and no way to say no.
fn c1_a_human_declines_and_the_traitor_stays_in_the_graveyard() {
    let (zone, floating) = drive_and_answer(false);
    assert_eq!(
        zone,
        Some(ZoneId::Graveyard(p(1))),
        "CR 118.12: declined, so the CR 702.x return never happens"
    );
    assert_eq!(
        floating, 1,
        "CR 118.12: a declined cost is not paid -- the {{B}} is still there for \
         something else, which is the whole reason declining is real play"
    );
}

#[test]
/// **C2** — CR 118.12, the ACCEPT, on the identical fixture and drive.
///
/// C1 and C2 differ in exactly one value: the `pay` bool in `ActionParams`. Both
/// halves are asserted because a decline-only probe cannot distinguish "the
/// decline works" from "this fixture never returns the Traitor at all".
fn c2_a_human_accepts_and_the_traitor_returns_to_the_battlefield() {
    let (zone, floating) = drive_and_answer(true);
    assert_eq!(
        zone,
        Some(ZoneId::Battlefield),
        "CR 118.12: paid, so `then` runs and the Traitor returns"
    );
    assert_eq!(floating, 0, "CR 118.8: the {{B}} was spent to pay the cost");
}

#[test]
/// **C3** — the BOT path. A bot seat is offered the same action and answers it
/// through the same `LegalAction`, with no human anywhere.
///
/// SR-38 in both directions: the offer layer must not invent an action the engine
/// will refuse, and it must not suppress one CR 118.12 gives. `StubProvider`
/// needed **no change at all** for this — its `BlockingDecision::EffectChoice`
/// arm is written against `default_effect_choice_answer`, so a sixth question
/// variant rides in for free. That is worth asserting rather than assuming: an
/// offer layer that silently dropped the new variant would leave a bot game
/// deadlocked, and this probe is what would catch it.
fn c3_the_bot_path_is_offered_and_answers_the_same_action() {
    let mut game = start_human_game();
    let (decision, _) = drive_until(&mut game, "TapForMana", |a| {
        matches!(a, LegalAction::TapForMana { .. })
    });
    let tap = decision
        .actions
        .iter()
        .position(|a| matches!(a, LegalAction::TapForMana { .. }))
        .expect("found above");
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: tap,
            params: ActionParams::default(),
        },
    )
    .expect("tap accepted");
    let (decision, _) = drive_until(&mut game, "AnswerEffectChoice", |a| {
        matches!(
            a,
            LegalAction::AnswerEffectChoice {
                question: EffectChoiceQuestion::PayOptionalCost { .. },
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
        "exactly one answer action is offered to the blocked seat, got {offers:?}"
    );
    match &offers[0] {
        LegalAction::AnswerEffectChoice {
            question, answer, ..
        } => {
            assert!(
                matches!(question, EffectChoiceQuestion::PayOptionalCost { .. }),
                "the offered question is the CR 118.12 one, got {question:?}"
            );
            assert_eq!(
                answer,
                &EffectChoiceAnswer::PayOptionalCost { pay: true },
                "a bot submitting the offered default plays the identical game the \
                 pre-PB-DX45 engine played"
            );
        }
        other => panic!("wrong action: {other:?}"),
    }

    // And every OTHER seat is offered nothing while the block stands -- the
    // liveness/ownership filter, asserted rather than assumed.
    let foreign: Vec<LegalAction> = StubProvider.legal_actions(&state, p(2));
    assert!(
        !foreign
            .iter()
            .any(|a| matches!(a, LegalAction::AnswerEffectChoice { .. })),
        "only the named payer may be offered the answer; p2 got {foreign:?}"
    );

    // Submitting the bot's own default resolves it -- proving the offer is not a
    // dead action the engine would refuse (SR-38).
    let idx = decision
        .actions
        .iter()
        .position(|a| matches!(a, LegalAction::AnswerEffectChoice { .. }))
        .expect("found above");
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams::default(),
        },
    )
    .expect("the engine must accept the action it offered, unmodified (SR-38)");
    assert_eq!(
        traitor_zone(game.state()),
        Some(ZoneId::Battlefield),
        "the offered default pays, exactly as the pre-batch engine did"
    );
}
