//! PB-DX45 (`OOS-DX24-9` ≡ `OOS-DX27-5`): CR 118.12's optional cost is the
//! PLAYER's decision.
//!
//! CR 118.12 — *"Some costs are optional. … If a player chooses to pay an
//! optional cost, they do so as part of the process of applying the effect."*
//! Before this batch `effects/mod.rs` called `try_pay_optional_cost`
//! unconditionally at **both** of its call sites — the `Effect::MayPayThenEffect`
//! arm and `Effect::LookAtTopThenPlace`'s `place_cost` — so the engine paid
//! whenever it could and **the decline was not merely hard to reach: it did not
//! exist**. That is why every probe below that asserts a decline asserts it by
//! the RESOLUTION EFFECT (the card still in the graveyard, the mana still in the
//! pool, nothing sacrificed) rather than by the offer: an offer-shaped assertion
//! would pass on a fixture where the question is asked and the answer thrown
//! away.
//!
//! # The vacuity trap this file walks into deliberately, once
//!
//! `p1` calls `execute_effect` DIRECTLY and asserts a suspension. Every other
//! direct-execute probe here banks its answer first
//! (`test_util::bank_effect_choice_answer`), because a bare `execute_effect` on
//! an asking effect measures **nothing** — the arm records the question, returns,
//! and applies nothing, and only `resolve_top_of_stack`'s abort-and-replay
//! wrapper turns that into a real question. PB-DX15a hit exactly this shape and
//! recorded it; naming it here is cheaper than rediscovering it.

use mtg_engine::cards::card_definition::PlayerTarget;
use mtg_engine::effects::{default_effect_choice_answer, execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::rules::engine::BlockingDecision;
use mtg_engine::state::test_util;
use mtg_engine::state::turn::Step;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Cost, Effect, EffectAmount, EffectChoiceAnswer, EffectChoiceQuestion, GameEvent, GameState,
    GameStateBuilder, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, TypeLine, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn one_black() -> ManaCost {
    ManaCost {
        black: 1,
        ..Default::default()
    }
}

fn black_pool(n: u32) -> ManaPool {
    ManaPool {
        black: n,
        ..Default::default()
    }
}

/// The question this batch's cost shape produces, for banking and for equality
/// assertions. Built from the SAME `Cost` value the effect carries — if the two
/// ever diverge, `ask_or_consume_effect_choice`'s structural equality check
/// re-suspends and the probe fails loudly rather than passing on a coincidence.
fn pay_question(cost: Cost) -> EffectChoiceQuestion {
    EffectChoiceQuestion::PayOptionalCost {
        cost: Box::new(cost),
    }
}

fn life_of(state: &GameState, pl: PlayerId) -> i32 {
    state.player(pl).expect("player exists").life_total
}

fn black_in_pool(state: &GameState, pl: PlayerId) -> u32 {
    state.player(pl).expect("player exists").mana_pool.black
}

/// A two-seat state with `p(1)` holding `pool` black mana and `life` life.
fn bare_state(pool: u32, life: i32) -> GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .player_life(p(1), life)
        .player_mana(p(1), black_pool(pool))
        .build()
        .expect("fixture builds")
}

/// `Effect::MayPayThenEffect { cost, payer: Controller, then }`, executed
/// directly against `state` for `p(1)`.
fn run_may_pay(state: &mut GameState, cost: Cost, then: Effect) -> Vec<GameEvent> {
    let effect = Effect::MayPayThenEffect {
        cost,
        payer: PlayerTarget::Controller,
        then: Box::new(then),
    };
    let mut ctx = EffectContext::new(p(1), ObjectId(0), vec![]);
    execute_effect(state, &effect, &mut ctx)
}

fn gain_two_life() -> Effect {
    Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::Fixed(2),
    }
}

// ── P1-P4: the primitive, at the effect level ────────────────────────────────

#[test]
/// **P1** — CR 118.12 / CR 608.2d. With nothing banked, the arm SUSPENDS: it
/// records the question and applies **nothing**.
///
/// This is the batch's headline stated at its smallest. Pre-PB-DX45 this same
/// call paid the `{B}` and gained the 2 life with no question anywhere.
fn p1_may_pay_suspends_and_applies_nothing() {
    let mut state = bare_state(1, 20);
    let events = run_may_pay(&mut state, Cost::Mana(one_black()), gain_two_life());

    assert!(
        state.pending_effect_choice().is_some(),
        "CR 118.12: the payer must be asked, not paid for"
    );
    assert_eq!(
        state.pending_effect_choice().unwrap().question,
        pay_question(Cost::Mana(one_black())),
        "the recorded question must carry the effect's own Cost verbatim"
    );
    assert_eq!(
        black_in_pool(&state, p(1)),
        1,
        "CR 118.12: nothing may be paid before the announcement"
    );
    assert_eq!(
        life_of(&state, p(1)),
        20,
        "the `then` arm must not run before the answer"
    );
    assert!(
        events.is_empty(),
        "a suspended pass must leak no events; got {events:?}"
    );
}

#[test]
/// **P2** — CR 118.12. Answer PAY: the cost is charged and `then` runs.
fn p2_answering_pay_charges_the_cost_and_runs_then() {
    let mut state = bare_state(1, 20);
    test_util::bank_effect_choice_answer(
        &mut state,
        pay_question(Cost::Mana(one_black())),
        EffectChoiceAnswer::PayOptionalCost { pay: true },
    );
    run_may_pay(&mut state, Cost::Mana(one_black()), gain_two_life());

    assert!(
        state.pending_effect_choice().is_none(),
        "the banked answer must be consumed, not re-asked"
    );
    assert_eq!(black_in_pool(&state, p(1)), 0, "CR 118.8: the {{B}} is spent");
    assert_eq!(life_of(&state, p(1)), 22, "`then` runs only if the cost was paid");
}

#[test]
/// **P3** — CR 118.12. Answer DECLINE: **nothing** is paid and `then` does not
/// run.
///
/// **This is a state the pre-PB-DX45 engine could not produce.** With the mana
/// available, the old arm paid unconditionally; there was no answer, no
/// suspension and no branch that reached this outcome. The assertion is on the
/// two observable effects (pool unchanged, life unchanged), not on the offer.
fn p3_answering_decline_pays_nothing_and_skips_then() {
    let mut state = bare_state(1, 20);
    test_util::bank_effect_choice_answer(
        &mut state,
        pay_question(Cost::Mana(one_black())),
        EffectChoiceAnswer::PayOptionalCost { pay: false },
    );
    run_may_pay(&mut state, Cost::Mana(one_black()), gain_two_life());

    assert!(state.pending_effect_choice().is_none());
    assert_eq!(
        black_in_pool(&state, p(1)),
        1,
        "CR 118.12: a declined cost is not paid -- the mana stays for something else, \
         which is the whole reason declining is real play"
    );
    assert_eq!(
        life_of(&state, p(1)),
        20,
        "CR 118.12: `then` runs ONLY IF the cost was paid"
    );
}

#[test]
/// **P4** — CR 118.12. An UNPAYABLE cost asks nothing (the DETERMINED
/// short-circuit) and behaves exactly as it did before this batch.
///
/// Non-vacuity matters here: an assertion that "no question was asked" passes
/// trivially on a broken engine that never asks at all, so this probe also
/// asserts the payable sibling DOES ask, in the same body.
fn p4_unpayable_cost_asks_nothing() {
    let mut state = bare_state(0, 20);
    run_may_pay(&mut state, Cost::Mana(one_black()), gain_two_life());
    assert!(
        state.pending_effect_choice().is_none(),
        "CR 118.12: a cost you cannot pay is not a choice you have"
    );
    assert_eq!(life_of(&state, p(1)), 20, "`then` must not run");

    // The same effect on a state that CAN pay does ask -- so the assertion above
    // is about payability, not about a mechanism that never fires.
    let mut payable = bare_state(1, 20);
    run_may_pay(&mut payable, Cost::Mana(one_black()), gain_two_life());
    assert!(
        payable.pending_effect_choice().is_some(),
        "non-vacuity: the identical effect asks when the cost IS payable"
    );
}

#[test]
/// **P5** — the DEFAULT answer is `pay: true`, and that is load-bearing.
///
/// `default_effect_choice_answer` is what every bot, the fuzzer and every
/// pre-PB-DX45 golden script submits. `true` is the exact recovery of the
/// pre-batch auto-pay, which is what keeps this batch behaviourally neutral for
/// them — only the command trace grows. Pinned so a future "safer default" edit
/// is a deliberate act with a suite-wide cost, not a one-line drive-by.
fn p5_default_answer_is_pay_which_recovers_the_pre_batch_autopay() {
    assert_eq!(
        default_effect_choice_answer(&pay_question(Cost::PayLife(3))),
        EffectChoiceAnswer::PayOptionalCost { pay: true }
    );
}

// ── P6-P8: through a real resolution, via `process_command` ──────────────────

/// A sorcery whose only effect is `MayPayThenEffect { Cost::PayLife(3), gain 2 }`.
///
/// `PayLife` rather than `Mana`, deliberately: a mana pool empties between steps
/// (CR 500.4) and `can_pay_optional_cost` reads only what is floating
/// (`OOS-DX45-7`), so a mana cost would make the fixture's payability depend on
/// step timing rather than on the answer. Life does not evaporate.
fn optional_life_sorcery() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx45-optional-life".to_string()),
        name: "DX45 Optional Life".to_string(),
        mana_cost: Some(ManaCost::default()),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "You may pay 3 life. If you do, you gain 2 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::MayPayThenEffect {
                cost: Cost::PayLife(3),
                payer: PlayerTarget::Controller,
                then: Box::new(gain_two_life()),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn sorcery_fixture(def: CardDefinition) -> GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .player_life(p(1), 20)
        .object(
            ObjectSpec::card(p(1), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Hand(p(1))),
        )
        .build()
        .expect("fixture builds")
}

fn cast_and_resolve(state: GameState, name: &str) -> (GameState, Vec<GameEvent>) {
    let spell_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name && o.zone == ZoneId::Hand(p(1)))
        .map(|(id, _)| *id)
        .expect("spell in hand");
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p(1),
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .expect("cast succeeds");
    let mut all = Vec::new();
    let mut cur = state;
    for pl in [p(1), p(2)] {
        let (s, ev) = process_command(cur, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        cur = s;
        all.extend(ev);
    }
    (cur, all)
}

fn answer_with(state: GameState, pay: bool) -> GameState {
    let entry = state
        .pending_effect_choice()
        .expect("a CR 608.2d choice must be outstanding")
        .clone();
    process_command(
        state,
        Command::AnswerEffectChoice {
            player: entry.player,
            choice_id: entry.choice_id,
            answer: EffectChoiceAnswer::PayOptionalCost { pay },
        },
    )
    .expect("the answer must be accepted")
    .0
}

#[test]
/// **P6** — CR 608.1 / CR 608.2d. A real resolution BLOCKS on the offer, rolls
/// back completely, and publishes the question as a `GameEvent`.
fn p6_real_resolution_blocks_and_rolls_back() {
    let state = sorcery_fixture(optional_life_sorcery());
    let (state, events) = cast_and_resolve(state, "DX45 Optional Life");

    assert!(
        matches!(
            state.blocking_decision(),
            Some(BlockingDecision::EffectChoice { .. })
        ),
        "the CR 608.2d entry must gate the engine"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 608.1: the roll-back puts the resolving spell back on the stack"
    );
    assert_eq!(life_of(&state, p(1)), 20, "nothing may be paid before the answer");
    let asked: Vec<&GameEvent> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::EffectChoiceRequired { .. }))
        .collect();
    assert_eq!(asked.len(), 1, "exactly one question, got {asked:?}");
    match asked[0] {
        GameEvent::EffectChoiceRequired {
            player, question, ..
        } => {
            assert_eq!(*player, p(1), "CR 118.12 asks the payer");
            assert_eq!(*question, pay_question(Cost::PayLife(3)));
        }
        other => panic!("wrong event: {other:?}"),
    }
}

#[test]
/// **P7** — CR 118.12, the DECLINE end to end through `process_command`, asserted
/// by resolution effect.
///
/// Life is 20 before and 20 after: the 3 was not paid and the 2 was not gained.
/// Both halves matter — a fixture that only checked "life != 19" would pass on an
/// engine that paid and gained (20 - 3 + 2 = 19, so that one happens to
/// discriminate; a `gain 3` variant would not, which is why the assertion is on
/// the exact value).
fn p7_declining_a_real_resolution_pays_nothing_and_skips_then() {
    let state = sorcery_fixture(optional_life_sorcery());
    let (state, _) = cast_and_resolve(state, "DX45 Optional Life");
    let state = answer_with(state, false);

    assert!(state.pending_effect_choice().is_none(), "the block must clear");
    assert!(
        state.stack_objects().is_empty(),
        "CR 608.2m: the spell finishes resolving after the answer"
    );
    assert_eq!(
        life_of(&state, p(1)),
        20,
        "CR 118.12: declined -- neither the 3 life paid nor the 2 gained"
    );
}

#[test]
/// **P8** — CR 118.12, the ACCEPT end to end, same fixture and same channel.
fn p8_accepting_a_real_resolution_charges_and_runs() {
    let state = sorcery_fixture(optional_life_sorcery());
    let (state, _) = cast_and_resolve(state, "DX45 Optional Life");
    let state = answer_with(state, true);

    assert!(state.pending_effect_choice().is_none());
    assert!(state.stack_objects().is_empty());
    assert_eq!(
        life_of(&state, p(1)),
        19,
        "CR 118.12: 20 - 3 paid + 2 gained. The two answers land on DIFFERENT life \
         totals, which is what makes P7 and P8 a discriminating pair rather than two \
         assertions about the same number."
    );
}

// ── P9-P10: the trust boundary ───────────────────────────────────────────────

#[test]
/// **P9** — SR-29 / CR 608.2d. A FOREIGN seat cannot answer someone else's
/// optional cost, and a wrong-variant answer is refused.
fn p9_the_answer_is_validated_against_the_recorded_question() {
    let state = sorcery_fixture(optional_life_sorcery());
    let (state, _) = cast_and_resolve(state, "DX45 Optional Life");
    let entry = state.pending_effect_choice().expect("blocked").clone();

    let foreign = process_command(
        state.clone(),
        Command::AnswerEffectChoice {
            player: p(2),
            choice_id: entry.choice_id,
            answer: EffectChoiceAnswer::PayOptionalCost { pay: true },
        },
    );
    assert!(foreign.is_err(), "only the named payer may answer");

    let wrong_variant = process_command(
        state.clone(),
        Command::AnswerEffectChoice {
            player: entry.player,
            choice_id: entry.choice_id,
            answer: EffectChoiceAnswer::SearchLibrary { found: None },
        },
    );
    assert!(
        wrong_variant.is_err(),
        "a SearchLibrary answer does not answer a PayOptionalCost question"
    );

    let stale = process_command(
        state,
        Command::AnswerEffectChoice {
            player: entry.player,
            choice_id: entry.choice_id.wrapping_add(1),
            answer: EffectChoiceAnswer::PayOptionalCost { pay: true },
        },
    );
    assert!(stale.is_err(), "the moment guard must reject a stale choice id");
}

#[test]
/// **P10** — CR 608.2e / CR 101.4. `PlayerTarget::EachPlayer` asks EACH payer,
/// one at a time, and the two answers are independent.
///
/// The two questions are structurally IDENTICAL (same `Cost`), so the answer bank
/// is consumed positionally — this probe is what proves that positional
/// consumption lines up with the payer iteration order rather than merely
/// happening not to crash.
fn p10_each_player_is_asked_separately_and_answers_independently() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .player_life(p(1), 20)
        .player_life(p(2), 20)
        .build()
        .expect("fixture builds");
    // p1 (asked first: active player, CR 101.4) pays; p2 declines.
    test_util::bank_effect_choice_answer(
        &mut state,
        pay_question(Cost::PayLife(3)),
        EffectChoiceAnswer::PayOptionalCost { pay: true },
    );
    test_util::bank_effect_choice_answer(
        &mut state,
        pay_question(Cost::PayLife(3)),
        EffectChoiceAnswer::PayOptionalCost { pay: false },
    );

    let effect = Effect::MayPayThenEffect {
        cost: Cost::PayLife(3),
        payer: PlayerTarget::EachPlayer,
        then: Box::new(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(2),
        }),
    };
    let mut ctx = EffectContext::new(p(1), ObjectId(0), vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert!(state.pending_effect_choice().is_none(), "both answers consumed");
    assert_eq!(
        life_of(&state, p(1)),
        19,
        "p1 paid 3 and gained 2 (the `then` arm rebinds ctx.controller to the payer)"
    );
    assert_eq!(
        life_of(&state, p(2)),
        20,
        "p2 declined: nothing paid, nothing gained -- the answers are independent"
    );
}

// ── P11-P12: the SECOND `try_pay_optional_cost` call site ────────────────────

/// `Effect::LookAtTopThenPlace` with an interposed `place_cost`, executed
/// directly for `p(1)`.
///
/// **This site is not named by any document in PB-DX45's chain** — `OOS-DX24-9`,
/// `OOS-DX27-5`, the v4 memo's row and the task brief all say
/// `Effect::MayPayThenEffect`. `effects/mod.rs` has TWO callers of
/// `try_pay_optional_cost`, and this is the other one, live on one deck-legal
/// `Complete` def (`birthing_ritual`). Population pinned by
/// `core::pb_dx45_may_pay_roster::r4_second_pay_site_population_is_pinned`.
fn run_look_place(state: &mut GameState, place_cost: Cost) -> Vec<GameEvent> {
    use mtg_engine::cards::card_definition::ZoneTarget;
    use mtg_engine::TargetFilter;
    let effect = Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(2),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        place_cost: Some(Box::new(place_cost)),
        destination: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        rest_to: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: mtg_engine::cards::card_definition::LibraryPosition::Bottom,
        },
        optional: false,
    };
    let mut ctx = EffectContext::new(p(1), ObjectId(0), vec![]);
    execute_effect(state, &effect, &mut ctx)
}

/// Two creature cards in `p(1)`'s library, `life` life, so
/// `LookAtTopThenPlace { count: 2 }` has something to find.
fn library_fixture(life: i32) -> GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .player_life(p(1), life)
        .object(
            ObjectSpec::card(p(1), "DX45 Library Bear")
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Library(p(1))),
        )
        .object(
            ObjectSpec::card(p(1), "DX45 Library Ox")
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Library(p(1))),
        )
        .build()
        .expect("fixture builds")
}

fn hand_size(state: &GameState, pl: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(pl))
        .count()
}

#[test]
/// **P11** — CR 118.12. The second site SUSPENDS too, and applies nothing.
fn p11_look_at_top_then_place_cost_suspends() {
    let mut state = library_fixture(20);
    run_look_place(&mut state, Cost::PayLife(3));

    assert!(
        state.pending_effect_choice().is_some(),
        "CR 118.12: the interposed place_cost is the payer's decision at THIS site too"
    );
    assert_eq!(
        state.pending_effect_choice().unwrap().question,
        pay_question(Cost::PayLife(3))
    );
    assert_eq!(life_of(&state, p(1)), 20, "nothing paid before the answer");
    assert_eq!(hand_size(&state, p(1)), 0, "nothing placed before the answer");
}

#[test]
/// **P12** — CR 118.12 at the second site: DECLINE places nothing, ACCEPT places
/// one and charges.
///
/// Asserted as a PAIR in one body, because the two outcomes differ only in the
/// answer: a single-outcome probe here would not distinguish "the decline works"
/// from "this effect never places anything on this fixture".
fn p12_second_site_decline_and_accept_differ_by_the_answer_alone() {
    let mut declined = library_fixture(20);
    test_util::bank_effect_choice_answer(
        &mut declined,
        pay_question(Cost::PayLife(3)),
        EffectChoiceAnswer::PayOptionalCost { pay: false },
    );
    run_look_place(&mut declined, Cost::PayLife(3));
    assert!(declined.pending_effect_choice().is_none());
    assert_eq!(life_of(&declined, p(1)), 20, "declined: no life paid");
    assert_eq!(
        hand_size(&declined, p(1)),
        0,
        "CR 118.12: placement happens only if the interposed cost was paid"
    );

    let mut accepted = library_fixture(20);
    test_util::bank_effect_choice_answer(
        &mut accepted,
        pay_question(Cost::PayLife(3)),
        EffectChoiceAnswer::PayOptionalCost { pay: true },
    );
    run_look_place(&mut accepted, Cost::PayLife(3));
    assert!(accepted.pending_effect_choice().is_none());
    assert_eq!(life_of(&accepted, p(1)), 17, "accepted: CR 119.4, 3 life paid");
    assert_eq!(
        hand_size(&accepted, p(1)),
        1,
        "one matching card placed -- the same fixture, the same effect, the OTHER answer"
    );
}
