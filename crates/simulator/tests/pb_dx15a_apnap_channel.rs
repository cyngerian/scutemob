//! PB-DX15a — CR 608.2e APNAP order is **reachable**, not merely computed
//! (`scutemob-216`, closes the reachability half of `OOS-DP9-8`).
//!
//! # Why this file exists and `pb_dp9_effect_choice.rs` is not enough
//!
//! The engine-side probe
//! (`crates/engine/tests/primitives/pb_dp9_effect_choice.rs::test_dx15a_each_player_search_asks_in_apnap_order`)
//! drives `process_command` directly and reads `state.pending_effect_choice()`. That
//! proves the engine *computes* the order. It is the `kaito_shizuki` lesson's exact
//! shape to stop there: **existence is necessary and never sufficient** — an order no
//! offer layer surfaces, no bot consumes and no human is asked in is the same shape of
//! nothing.
//!
//! So every probe here goes through a **real** channel — `StubProvider`'s offer layer,
//! `LocalGame::advance`, `LocalGame::submit` + `params.rs::HumanChoice`, and the bots —
//! and asserts a **real** consequence: the sequence of seats the game actually stopped
//! and asked, the sequence of `Command::AnswerEffectChoice`s a bot game actually
//! applied, the order of the resolution's own `GameEvent`s, and where each card ended
//! up.
//!
//! | probe | channel | evidence |
//! |---|---|---|
//! | C1 | human — `LocalGame` + `HumanChoice` | the SEQUENCE of `PendingDecision::player`, a NON-DEFAULT answer per seat, and the resulting zones |
//! | C2 | bot — `LocalGame::advance` | the SEQUENCE of `Command::AnswerEffectChoice { player }` in the applied-command journal |
//! | C3 | human, real corpus card (`burglar_rat`) | `EachOpponent`: the SEQUENCE of asked seats AND of the `CardDiscarded` events |
//! | C4 | human, real corpus card (`fleshbag_marauder`) | the RESOLUTION's own `GameEvent::PermanentSacrificed` order — an effect with no question at all |
//! | C5 | none — a structural guard | this file's fixture can *express* the deviation; an edit that makes it vacuous reddens here |
//!
//! # THE fixture rule, and why every probe obeys it
//!
//! **A fixture whose active player is the LOWEST `PlayerId` cannot tell CR 608.2e APNAP
//! order from ascending `PlayerId` order.** `GameStateBuilder` seeds `turn_order` in
//! `add_player` call order, which is ascending everywhere in this repository, so
//! "rotate turn order to start at the active player" is the identity when the active
//! player is first. That is precisely why `OOS-DP9-8` survived from PB-DP9
//! (`scutemob-157`) to here behind a test whose doc said it pinned the deviation.
//!
//! Every probe below therefore uses **three seats with `p(2)` active**: APNAP is
//! `[p2, p3, p1]` and ascending is `[p1, p2, p3]`, which differ in **every** position.
//! C5 asserts that property of the fixture itself rather than leaving it to prose.
//!
//! # CR citations
//!
//! - **CR 608.2e** — "Some spells and abilities have multiple steps or actions … that
//!   involve multiple players. In these cases, the choices for the first action are
//!   made in APNAP order, and then the first action is processed simultaneously."
//! - **CR 101.4** — "If multiple players would make choices … at the same time, the
//!   active player … makes any choices required, then each other player in turn order
//!   does the same."
//! - **CR 701.23i** — "If multiple players search at once, each of those players looks
//!   at the appropriate cards at the same time, then those players decide in APNAP
//!   order which card to find."
//! - **CR 608.2d** — the resolution-time announcement itself (PB-DP9), which is what
//!   makes the order *observable* as a sequence of questions.
//! - **CR 307.1** — sorcery timing: the caster of C1/C2's spell is the active player,
//!   which is why `p(2)` both is active and casts.
//! - **CR 701.9b** — discard (C3). **CR 701.21a** — sacrifice (C4).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::cards::card_definition::{PlayerTarget, TargetFilter, ZoneTarget};
use mtg_engine::state::turn::Step;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, Effect, EffectChoiceAnswer, EffectChoiceQuestion, GameEvent,
    GameState, GameStateBuilder, ManaCost, ObjectId, ObjectSpec, PlayerId, TypeLine, ZoneId,
};
use mtg_simulator::{
    build_registry, ActionParams, AdvanceOutcome, Bot, DecisionKind, HeuristicBot, HumanChoice,
    LegalAction, LocalGame, LocalGameLimits, PendingDecision, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// CR 608.2e / CR 101.4 on this file's fixture: active `p(2)`, turn order
/// `[p1, p2, p3]`.
const APNAP: [PlayerId; 3] = [PlayerId(2), PlayerId(3), PlayerId(1)];
/// What `state.players.keys()` (an `imbl::OrdMap`) gave before PB-DX15a. Named so the
/// assertions below can say what they are NOT asserting.
const ASCENDING: [PlayerId; 3] = [PlayerId(1), PlayerId(2), PlayerId(3)];

fn limits(max_turns: u32) -> LocalGameLimits {
    LocalGameLimits {
        max_turns,
        max_commands: max_turns * 400,
        max_consecutive_passes: 200,
        record_journal: true,
    }
}

// ── Fixture construction ─────────────────────────────────────────────────────

/// A zero-cost sorcery carrying `effect`.
///
/// Zero cost on purpose: CR 500.4 empties a mana pool at every step boundary, and
/// `LocalGame::start` runs the game from `Step::Untap`, so a pool poked in before
/// `start` is gone before the first human decision. A free spell removes mana from the
/// picture entirely for C1/C2, whose subject is the ORDER of the questions and not the
/// payment path — C3/C4 pay for real, off real `Swamp`s, through `params.auto_tap`.
fn free_sorcery(name: &str, card_id: &str, effect: Effect) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(card_id.to_string()),
        mana_cost: Some(ManaCost::default()),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// "Each player searches their library for a creature card and puts it into their
/// hand."
///
/// A **stated-quality** search (CR 701.23b), so the answer space is genuinely wider
/// than one — which is what makes C1's non-default answer expressible.
fn everyone_tutors() -> CardDefinition {
    free_sorcery(
        "Everyone Tutors",
        "dx15a-everyone-tutors",
        Effect::SearchLibrary {
            player: PlayerTarget::EachPlayer,
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            reveal: false,
            destination: ZoneTarget::Hand {
                owner: PlayerTarget::Controller,
            },
            shuffle_before_placing: false,
            also_search_graveyard: false,
        },
    )
}

fn library_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).in_zone(ZoneId::Library(owner))
}

/// A non-creature library card. Added AFTER the creatures so it sits on TOP
/// (`Zone::Ordered` keeps the top at the last index), which is what keeps the draw step
/// from eating a search candidate and making the active seat's candidate count differ
/// from everyone else's.
fn library_filler(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name).in_zone(ZoneId::Library(owner))
}

/// Three seats with **`p(2)` active** — see this module's fixture rule. `hand_card`, if
/// given, goes in `p(2)`'s hand, because CR 307.1 makes the active player the only
/// legal caster of a sorcery.
fn fixture(
    registry: std::sync::Arc<CardRegistry>,
    hand_card: Option<&CardDefinition>,
    extra: Vec<ObjectSpec>,
) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(registry)
        .active_player(p(2))
        .at_step(Step::PreCombatMain);
    if let Some(def) = hand_card {
        builder = builder.object(
            ObjectSpec::card(p(2), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(def.types.card_types.iter().cloned().collect::<Vec<_>>())
                .with_mana_cost(def.mana_cost.clone().unwrap_or_default())
                .in_zone(ZoneId::Hand(p(2))),
        );
    }
    for spec in extra {
        builder = builder.object(spec);
    }
    builder.build().unwrap()
}

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// A real card object in a real zone, enriched from its real `CardDefinition`.
/// `ObjectSpec::card()` creates naked objects (the standing `memory/gotchas-infra.md`
/// gotcha), so a hand-built "Swamp-like" spec would tap for nothing and a hand-built
/// "Fleshbag-like" one would carry no ETB trigger at all.
fn real_card(
    defs: &HashMap<String, CardDefinition>,
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

fn name_of(state: &GameState, id: ObjectId) -> String {
    state
        .objects()
        .get(&id)
        .map(|o| o.characteristics.name.clone())
        .unwrap_or_else(|| format!("<retired {id:?}>"))
}

fn zone_of(state: &GameState, name: &str) -> Option<ZoneId> {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name)
        .map(|o| o.zone)
}

/// Every `GameEvent` the game has actually applied, in application order, read off
/// `LocalGame::journal()` — the same feed `tools/play-server` ships to the browser.
fn applied_events(game: &LocalGame<StubProvider>) -> Vec<GameEvent> {
    game.journal()
        .iter()
        .flat_map(|r| r.events.iter().cloned())
        .collect()
}

fn all_human() -> BTreeSet<PlayerId> {
    [p(1), p(2), p(3)].into_iter().collect()
}

// ── The shared human drive ───────────────────────────────────────────────────

/// What one CR 608.2d stop recorded.
struct Asked {
    player: PlayerId,
    /// The name of the card this seat's answer named, when the question was a search.
    chosen: Option<String>,
}

/// Drive an all-human `LocalGame` through the real `advance`/`submit` loop.
///
/// Policy, in order: answer a CR 608.2d question (recording the seat), else cast
/// `cast_name` once if it is offered, else pass priority. Every submission goes through
/// `LocalGame::submit`, i.e. through `params.rs::action_to_command_with_params` — the
/// same function the browser's `POST /api/game/action` reaches.
///
/// `answer` is handed the offered `LegalAction::AnswerEffectChoice`'s question **and**
/// the engine's own default answer, and returns the answer to submit. Being able to
/// return something other than the default is what makes C1 a probe of the human's
/// choice rather than of the engine's fallback.
fn drive_all_human(
    game: &mut LocalGame<StubProvider>,
    cast_name: &str,
    want_answers: usize,
    mut answer: impl FnMut(&EffectChoiceQuestion, &EffectChoiceAnswer, &GameState) -> EffectChoiceAnswer,
) -> Vec<Asked> {
    let mut asked: Vec<Asked> = Vec::new();
    let mut cast_done = false;
    for step in 0..400 {
        if asked.len() == want_answers {
            return asked;
        }
        let decision: PendingDecision = match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => d,
            other => panic!(
                "step {step}: the drive ended before {want_answers} questions were asked \
                 ({} so far): {other:?}",
                asked.len()
            ),
        };

        if decision.kind == DecisionKind::EffectChoice {
            let idx = decision
                .actions
                .iter()
                .position(|a| matches!(a, LegalAction::AnswerEffectChoice { .. }))
                .expect("SR-38: the provider must offer an answer to its own blocking decision");
            let LegalAction::AnswerEffectChoice {
                question,
                answer: default_answer,
                ..
            } = &decision.actions[idx]
            else {
                unreachable!("just matched")
            };
            let submitted = answer(question, default_answer, game.state());
            let chosen = match &submitted {
                EffectChoiceAnswer::SearchLibrary { found } => {
                    found.map(|id| name_of(game.state(), id))
                }
                _ => None,
            };
            asked.push(Asked {
                player: decision.player,
                chosen,
            });
            game.submit(
                decision.seq,
                HumanChoice {
                    action_index: idx,
                    params: ActionParams {
                        effect_choice_answer: Some(submitted),
                        ..Default::default()
                    },
                },
            )
            .unwrap_or_else(|e| panic!("step {step}: the answer was refused: {e:?}"));
            continue;
        }

        if !cast_done {
            let cast_idx = decision.actions.iter().position(|a| {
                matches!(a, LegalAction::CastSpell { card, .. }
                         if name_of(game.state(), *card) == cast_name)
            });
            if let Some(i) = cast_idx {
                cast_done = true;
                game.submit(
                    decision.seq,
                    HumanChoice {
                        action_index: i,
                        // CR 601.2g: pay for it off real permanents where there is a
                        // cost (C3). A free spell simply prepends no taps.
                        params: ActionParams {
                            auto_tap: true,
                            ..Default::default()
                        },
                    },
                )
                .unwrap_or_else(|e| panic!("step {step}: casting {cast_name} was refused: {e:?}"));
                continue;
            }
        }

        let pass = decision
            .actions
            .iter()
            .position(|a| matches!(a, LegalAction::PassPriority))
            .unwrap_or_else(|| {
                panic!(
                    "step {step}: no PassPriority offered to {:?}: {:?}",
                    decision.player, decision.actions
                )
            });
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: pass,
                params: ActionParams::default(),
            },
        )
        .unwrap_or_else(|e| panic!("step {step}: PassPriority was refused: {e:?}"));
    }
    panic!("the drive did not reach {want_answers} questions within 400 decisions");
}

// ── C1 — the human channel: the sequence of seats the game stops and asks ────

/// **C1** — CR 608.2e / CR 101.4 / CR 701.23i. "Each player searches their library",
/// driven through `LocalGame` + `HumanChoice` on three human seats.
///
/// Three separate claims, each of which fails differently:
///
/// 1. **The ORDER of the questions.** `PendingDecision::player`, in the order
///    `advance()` handed them out, must be `[p2, p3, p1]`. Ascending `PlayerId` — what
///    `resolve_player_target_list` did before PB-DX15a — is `[p1, p2, p3]`, which
///    differs in every position.
/// 2. **The human's own answer is what happens.** Each seat is answered with a card
///    that is *not* the engine's default, asserted against the offered
///    `LegalAction::AnswerEffectChoice`'s own `answer` field rather than against a
///    transcription of what that default is believed to be; then the resolved state is
///    checked — the named card is in a hand, and the default pick is still in a library.
/// 3. **The answers are applied to the seat that gave them.** An implementation that
///    asked in APNAP order but applied answers by ascending index would satisfy (1);
///    the per-seat name check is what separates them.
#[test]
fn c1_each_player_search_asks_the_human_seats_in_apnap_order() {
    let def = everyone_tutors();
    let state = fixture(
        CardRegistry::new(vec![def.clone()]),
        Some(&def),
        vec![
            library_creature(p(1), "P1 Alpha"),
            library_creature(p(1), "P1 Beta"),
            library_creature(p(1), "P1 Gamma"),
            library_filler(p(1), "P1 Filler"),
            library_creature(p(2), "P2 Alpha"),
            library_creature(p(2), "P2 Beta"),
            library_creature(p(2), "P2 Gamma"),
            library_filler(p(2), "P2 Filler"),
            library_creature(p(3), "P3 Alpha"),
            library_creature(p(3), "P3 Beta"),
            library_creature(p(3), "P3 Gamma"),
            library_filler(p(3), "P3 Filler"),
        ],
    );
    let (mut game, _) = LocalGame::start(
        state,
        15_15,
        StubProvider,
        HashMap::new(),
        all_human(),
        limits(2),
        true,
    )
    .expect("the fixture game starts");

    let mut non_default_proved = 0usize;
    let asked = drive_all_human(
        &mut game,
        "Everyone Tutors",
        3,
        |question, default, state| {
            let EffectChoiceQuestion::SearchLibrary { candidates, .. } = question else {
                panic!("expected a CR 701.23a search question, got {question:?}");
            };
            assert_eq!(
                candidates.len(),
                3,
                "each seat is offered only its OWN library's three creature cards: {:?}",
                candidates
                    .iter()
                    .map(|id| name_of(state, *id))
                    .collect::<Vec<_>>()
            );
            // The LAST candidate, which the engine's default (`candidates.first()`)
            // never is.
            let pick = *candidates.last().expect("three candidates");
            assert_eq!(
                default,
                &EffectChoiceAnswer::SearchLibrary {
                    found: candidates.first().copied()
                },
                "the offered default is the FIRST candidate; the assertions below are \
                 only meaningful because this probe submits a different one"
            );
            non_default_proved += 1;
            EffectChoiceAnswer::SearchLibrary { found: Some(pick) }
        },
    );

    let seats: Vec<PlayerId> = asked.iter().map(|a| a.player).collect();
    assert_eq!(
        seats,
        APNAP.to_vec(),
        "CR 608.2e / CR 101.4 / CR 701.23i: the HUMAN channel must stop and ask the \
         active player first, then the remaining players in turn order. Ascending \
         PlayerId -- what the engine did before PB-DX15a -- is {ASCENDING:?}."
    );
    assert_eq!(
        non_default_proved, 3,
        "every seat's answer was checked against the offer's own default"
    );

    let chosen: Vec<String> = asked
        .iter()
        .map(|a| a.chosen.clone().expect("a search answer names a card"))
        .collect();
    assert_eq!(
        chosen,
        vec![
            "P2 Gamma".to_string(),
            "P3 Gamma".to_string(),
            "P1 Gamma".to_string()
        ],
        "each seat's OWN announcement is the one applied, in the order it was asked -- \
         an implementation that asked in APNAP order and applied answers by ascending \
         index would pass the order assertion above and fail here"
    );

    for name in ["P1 Gamma", "P2 Gamma", "P3 Gamma"] {
        assert!(
            matches!(zone_of(game.state(), name), Some(ZoneId::Hand(_))),
            "{name} -- the human's non-default choice -- must actually have been found"
        );
    }
    for name in ["P1 Alpha", "P2 Alpha", "P3 Alpha"] {
        assert!(
            matches!(zone_of(game.state(), name), Some(ZoneId::Library(_))),
            "{name} -- the ENGINE's default pick -- must still be in the library, or \
             this probe is measuring the default and not the human"
        );
    }
}

// ── C2 — the bot channel ─────────────────────────────────────────────────────

/// **C2** — the same effect in a **bot-driven** game. `advance()` runs every seat
/// autonomously, so the questions never surface as `PendingDecision`s at all; the
/// evidence is the order of the `Command::AnswerEffectChoice`s the game actually
/// applied, read off `LocalGame::journal()`.
///
/// A separate channel from C1, not a restatement: the human path goes through
/// `submit` → `action_to_command_with_params`, the bot path through
/// `Bot::choose_action` → `apply_sequence`. `OOS-SIM6-3` is the standing reminder that
/// the two are separately reachable and can disagree.
///
/// `Command::AnswerEffectChoice { player }` names the seat the ENGINE asked, so this
/// reads the order out of the engine's own accepted commands rather than out of any
/// bot's preference — `HeuristicBot` scores every `AnswerEffectChoice` identically, so
/// nothing here depends on bot policy beyond "it casts the free sorcery".
#[test]
fn c2_each_player_search_answers_are_applied_in_apnap_order_in_a_bot_game() {
    let def = everyone_tutors();
    let state = fixture(
        CardRegistry::new(vec![def.clone()]),
        Some(&def),
        vec![
            library_creature(p(1), "P1 Alpha"),
            library_creature(p(1), "P1 Beta"),
            library_filler(p(1), "P1 Filler"),
            library_creature(p(2), "P2 Alpha"),
            library_creature(p(2), "P2 Beta"),
            library_filler(p(2), "P2 Filler"),
            library_creature(p(3), "P3 Alpha"),
            library_creature(p(3), "P3 Beta"),
            library_filler(p(3), "P3 Filler"),
        ],
    );
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for n in 1..=3u64 {
        bots.insert(
            p(n),
            Box::new(HeuristicBot::new(1500 + n, format!("Bot-{n}"))),
        );
    }
    let (mut game, _) = LocalGame::start(
        state,
        15_15,
        StubProvider,
        bots,
        BTreeSet::new(),
        limits(2),
        true,
    )
    .expect("the fixture game starts");

    // A bot-only game runs to a halt or a conclusion without ever awaiting a human.
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => panic!("a bot-only game must never await: {d:?}"),
        AdvanceOutcome::Halted(_) | AdvanceOutcome::GameOver { .. } => {}
    }

    let answered: Vec<PlayerId> = game
        .journal()
        .iter()
        .filter_map(|r| match &r.command {
            Command::AnswerEffectChoice { player, .. } => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(
        answered.len(),
        3,
        "the bot game must actually have resolved the spell and answered all three \
         seats' questions; got {answered:?}"
    );
    assert_eq!(
        answered,
        APNAP.to_vec(),
        "CR 608.2e: the BOT channel asks the same order the human channel does -- \
         active player first, then turn order. Ascending PlayerId is {ASCENDING:?}."
    );
}

// ── C3 — EachOpponent, on a real corpus card ────────────────────────────────

/// **C3** — `PlayerTarget::EachOpponent` is a **separate arm** of
/// `resolve_player_target_list` and was rewired separately; a probe that only covered
/// `EachPlayer` would leave half the change unmeasured.
///
/// The card is `burglar_rat` — a real, deck-legal `Complete` corpus def, not a
/// synthesised one: `{1}{B}`, "When this creature enters, each opponent discards a
/// card." Its ETB is an `Effect::ForEach { over: ForEachTarget::EachOpponent, .. }`
/// wrapping `Effect::DiscardCards`, and `ForEach`'s player arm resolves through the
/// same `resolve_player_target_list` this batch rewired — so this probe also covers the
/// `ForEach` route, which is the one every corpus card in this family actually uses.
///
/// The controller is the active player `p(2)`, so the opponents are `[p3, p1]` in APNAP
/// order and `[p1, p3]` ascending — the two are exact reversals, so an **ordered**
/// assertion discriminates and a set assertion would not.
///
/// Both halves are asserted, and they are different claims: the order the seats were
/// **asked** (CR 608.2d — the offer layer stopping the game) and the order the
/// `CardDiscarded` events were **emitted** (CR 701.9b — what the resolution did).
///
/// This one pays real mana: two real `Swamp`s through `ActionParams::auto_tap`, i.e.
/// through `LocalGame::auto_tap_commands_for` and `mana_solver`.
#[test]
fn c3_each_opponent_discard_asks_and_discards_in_apnap_order() {
    let defs = card_defs_by_name();
    let rat = defs
        .get("Burglar Rat")
        .expect("burglar_rat is a real corpus def")
        .clone();
    assert!(
        rat.completeness.is_complete(),
        "C3's premise is that this is a deck-legal Complete def: {:?}",
        rat.completeness
    );

    let mut extra = vec![
        real_card(&defs, p(2), "Burglar Rat", ZoneId::Hand(p(2))),
        real_card(&defs, p(2), "Swamp", ZoneId::Battlefield),
        real_card(&defs, p(2), "Swamp", ZoneId::Battlefield),
    ];
    // CR 701.9b: the engine only ASKS when the hand is larger than the discard count
    // (a one-card hand is a determined answer and is short-circuited), so each
    // opponent needs at least two cards in hand.
    for seat in [p(1), p(3)] {
        for tag in ["A", "B"] {
            extra.push(
                ObjectSpec::card(seat, &format!("P{} Hand {tag}", seat.0))
                    .in_zone(ZoneId::Hand(seat)),
            );
        }
    }
    for seat in [p(1), p(2), p(3)] {
        for i in 0..4 {
            extra.push(library_filler(seat, &format!("P{} Lib {i}", seat.0)));
        }
    }

    let state = fixture(build_registry(), None, extra);
    let (mut game, _) = LocalGame::start(
        state,
        15_15,
        StubProvider,
        HashMap::new(),
        all_human(),
        limits(2),
        true,
    )
    .expect("the fixture game starts");

    let asked = drive_all_human(&mut game, "Burglar Rat", 2, |question, _default, _state| {
        let EffectChoiceQuestion::Discard { hand, count } = question else {
            panic!("expected a CR 701.9b discard question, got {question:?}");
        };
        assert_eq!(*count, 1, "Burglar Rat discards exactly one");
        assert_eq!(
            hand.len(),
            2,
            "each opponent is asked about its OWN two-card hand"
        );
        EffectChoiceAnswer::Discard {
            chosen: vec![hand[1]],
        }
    });

    let seats: Vec<PlayerId> = asked.iter().map(|a| a.player).collect();
    assert_eq!(
        seats,
        vec![p(3), p(1)],
        "CR 608.2e / CR 101.4: with p2 active and casting, its opponents answer in \
         APNAP order [p3, p1]. Ascending PlayerId is the exact reversal, [p1, p3] -- \
         which is what the engine did before PB-DX15a."
    );

    let discarded: Vec<PlayerId> = applied_events(&game)
        .into_iter()
        .filter_map(|e| match e {
            GameEvent::CardDiscarded { player, .. } => Some(player),
            _ => None,
        })
        .collect();
    assert_eq!(
        discarded,
        vec![p(3), p(1)],
        "the RESOLUTION's own events must be in the same order as the questions -- \
         asking in APNAP order and then applying in ascending order would satisfy the \
         assertion above and fail here"
    );
}

// ── C4 — the resolution's own event order, with no question at all ──────────

/// **C4** — `fleshbag_marauder`: `{2}{B}`, "When this enters, each player sacrifices a
/// creature." A real, deck-legal `Complete` corpus def, and the family (Fleshbag, Grave
/// Pact and relatives) the seed's framing is usually about.
///
/// # This family has NO per-player question, and this probe does not pretend otherwise
///
/// `effects::sacrifice_permanents_for_player` selects deterministically — the eligible
/// permanents are sorted by `ObjectId` and the first `n` are taken — and calls nothing
/// in the `ask_or_consume_effect_choice` family. So `Effect::SacrificePermanents` with
/// `PlayerTarget::EachPlayer` raises **no CR 608.2d announcement for any seat**, and
/// this batch does not give it one. Each player's *choice* of which creature to
/// sacrifice is still the engine's, which is a separate, **pre-existing agency gap**
/// that PB-DX15a neither closes nor claims to. The `assert_ne!` inside the drive below
/// pins that: if this family ever gains a question, this test reddens and its doc must
/// be rewritten rather than the assertion deleted.
///
/// What this batch does fix here is the half that IS observable: the ORDER in which the
/// players sacrifice. `Effect::SacrificePermanents` iterates
/// `resolve_player_target_list`, so before PB-DX15a the sacrifices happened in ascending
/// `PlayerId` order rather than CR 608.2e APNAP order — and the order is real game
/// state, because each sacrifice's dies-triggers and SBAs are processed around it, so a
/// seat that sacrifices later can be affected by an earlier one.
///
/// The evidence is therefore the RESOLUTION's own `GameEvent::PermanentSacrificed`
/// sequence, read off the applied-command journal — no question anywhere in it.
#[test]
fn c4_each_player_sacrifice_resolves_in_apnap_order_though_it_asks_nobody() {
    let defs = card_defs_by_name();
    let fleshbag = defs
        .get("Fleshbag Marauder")
        .expect("fleshbag_marauder is a real corpus def")
        .clone();
    assert!(
        fleshbag.completeness.is_complete(),
        "C4's premise is that this is a deck-legal Complete def: {:?}",
        fleshbag.completeness
    );

    let mut extra = vec![
        real_card(&defs, p(2), "Fleshbag Marauder", ZoneId::Hand(p(2))),
        real_card(&defs, p(2), "Swamp", ZoneId::Battlefield),
        real_card(&defs, p(2), "Swamp", ZoneId::Battlefield),
        real_card(&defs, p(2), "Swamp", ZoneId::Battlefield),
    ];
    // One sacrificeable creature per seat. Naked specs on purpose: they carry no
    // `card_id`, so Architecture Invariant 9's `check_all_defs_complete` does not apply
    // to them, and nothing about this probe depends on which creature they are.
    for seat in [p(1), p(2), p(3)] {
        extra.push(
            ObjectSpec::creature(seat, &format!("P{} Victim", seat.0), 2, 2)
                .in_zone(ZoneId::Battlefield),
        );
        for i in 0..4 {
            extra.push(library_filler(seat, &format!("P{} Lib {i}", seat.0)));
        }
    }

    let state = fixture(build_registry(), None, extra);
    let (mut game, _) = LocalGame::start(
        state,
        15_15,
        StubProvider,
        HashMap::new(),
        all_human(),
        limits(2),
        true,
    )
    .expect("the fixture game starts");

    let mut sacrificed: Vec<PlayerId> = Vec::new();
    let mut cast_done = false;
    for step in 0..400 {
        sacrificed = applied_events(&game)
            .into_iter()
            .filter_map(|e| match e {
                GameEvent::PermanentSacrificed { player, .. } => Some(player),
                _ => None,
            })
            .collect();
        if sacrificed.len() == 3 {
            break;
        }
        let decision = match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => d,
            other => panic!("step {step}: the drive ended early: {other:?}"),
        };
        assert_ne!(
            decision.kind,
            DecisionKind::EffectChoice,
            "CR 701.21a: `Effect::SacrificePermanents` asks nobody -- see this test's \
             doc. If this reddens, the family gained an agency the doc says it does \
             not have."
        );
        let cast_idx = if cast_done {
            None
        } else {
            decision.actions.iter().position(|a| {
                matches!(a, LegalAction::CastSpell { card, .. }
                         if name_of(game.state(), *card) == "Fleshbag Marauder")
            })
        };
        let (index, params) = match cast_idx {
            Some(i) => {
                cast_done = true;
                (
                    i,
                    ActionParams {
                        auto_tap: true,
                        ..Default::default()
                    },
                )
            }
            None => (
                decision
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!("step {step}: no PassPriority: {:?}", decision.actions)
                    }),
                ActionParams::default(),
            ),
        };
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: index,
                params,
            },
        )
        .unwrap_or_else(|e| panic!("step {step}: submission refused: {e:?}"));
    }

    assert_eq!(
        sacrificed,
        APNAP.to_vec(),
        "CR 608.2e / CR 101.4 / CR 701.21a: each player sacrifices in APNAP order -- \
         active player p2 first, then p3 and p1 in turn order. Ascending PlayerId, \
         which is what the engine did before PB-DX15a, is {ASCENDING:?}."
    );
}

// ── C5 — the fixture can express the deviation ──────────────────────────────

/// **C5** — the structural guard, and the reason it is not left to prose.
///
/// Every assertion above is worth exactly as much as the claim that its fixture can
/// tell the two orders apart. `OOS-DP9-8` survived a whole batch cycle behind a test
/// that asserted a per-player order on a fixture where APNAP and ascending `PlayerId`
/// are the same list, and whose doc comment said it was pinning the deviation.
///
/// So this asserts the property of the fixture directly: on the exact state the probes
/// above build, `apnap_order_all_players` is `[p2, p3, p1]` and ascending is
/// `[p1, p2, p3]`, and the two differ **in every position**. If a future edit moves the
/// fixture's active player to `p(1)`, or drops it to two seats, this reddens here
/// rather than leaving four green tests quietly measuring nothing.
#[test]
fn c5_this_files_fixture_can_tell_apnap_from_ascending_player_id() {
    let def = everyone_tutors();
    let state = fixture(CardRegistry::new(vec![def.clone()]), Some(&def), vec![]);

    let apnap = mtg_engine::rules::abilities::apnap_order_all_players(&state);
    let mut ascending: Vec<PlayerId> = state.players().keys().copied().collect();
    ascending.sort();

    assert_eq!(apnap, APNAP.to_vec());
    assert_eq!(ascending, ASCENDING.to_vec());
    assert_eq!(
        apnap.len(),
        ascending.len(),
        "the two candidate orders name the same seats"
    );
    for (i, (a, b)) in apnap.iter().zip(ascending.iter()).enumerate() {
        assert_ne!(
            a, b,
            "position {i}: the two orders must differ HERE too -- a fixture where they \
             agree in any position weakens the corresponding assertion above by exactly \
             that much"
        );
    }
    assert_eq!(
        state.turn().active_player,
        p(2),
        "the active player must not be the lowest PlayerId, or APNAP and ascending \
         collapse into the same list (see this module's fixture rule)"
    );
}
