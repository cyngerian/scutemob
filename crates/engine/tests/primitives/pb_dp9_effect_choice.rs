//! PB-DP9 (DP-7 / DP-8 / DP-9) — library search, scry and surveil become
//! CR 608.2d player choices.
//!
//! CR 608.2d: "If an effect of a spell or ability offers any choices other than
//! choices already made as part of casting the spell, activating the ability, or
//! otherwise putting the spell or ability on the stack, the player announces
//! these while applying the effect. The player can't choose an option that's
//! illegal or impossible."
//!
//! CR 701.23a: "To search for a card in a zone, look at all cards in that zone
//! (even if it's a hidden zone) and find a card that matches the given
//! description."
//! CR 701.22a: "To 'scry N' means to look at the top N cards of your library,
//! then put any number of them on the bottom of your library in any order and
//! the rest on top of your library in any order."
//! CR 701.25a: "To 'surveil N' means to look at the top N cards of your library,
//! then put any number of them into your graveyard and the rest on top of your
//! library in any order."
//!
//! Before this batch the engine made all three choices itself: a search took the
//! lowest `ObjectId`, a scry put **every** looked-at card on the bottom and a
//! surveil put **every** one into the graveyard. The last two INVERTED the
//! printed mechanic — keeping a card on top was unreachable, and `Surveil N` was
//! exactly `Mill N`.

use mtg_engine::cards::card_definition::{PlayerTarget, TargetFilter, ZoneTarget};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::rules::engine::BlockingDecision;
use mtg_engine::state::turn::Step;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, EffectChoiceAnswer, EffectChoiceQuestion, GameEvent, GameState,
    GameStateBuilder, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, TypeLine, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

// ── Shared helpers ───────────────────────────────────────────────────────────

/// Pass priority once per listed player. **No pump** — every test in this file
/// wants to see the block.
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

/// Answer the outstanding CR 608.2d choice with the engine's own default,
/// through `process_command`.
///
/// **Panics if nothing is pending** — so it can never mask a missing block (the
/// `answer_pending_trigger_targets` precedent from PB-DP8).
fn answer_pending_effect_choice(state: GameState) -> (GameState, Vec<GameEvent>) {
    let entry = state
        .pending_effect_choice()
        .expect("no CR 608.2d resolution-time choice is pending");
    let player = entry.player;
    let choice_id = entry.choice_id;
    let answer = mtg_engine::effects::default_effect_choice_answer(&entry.question);
    process_command(
        state,
        Command::AnswerEffectChoice {
            player,
            choice_id,
            answer,
        },
    )
    .expect("the engine must accept its own default answer (SR-38)")
}

/// Answer the outstanding choice with an explicitly supplied answer.
fn answer_with(state: GameState, answer: EffectChoiceAnswer) -> (GameState, Vec<GameEvent>) {
    let entry = state
        .pending_effect_choice()
        .expect("no CR 608.2d resolution-time choice is pending");
    let player = entry.player;
    let choice_id = entry.choice_id;
    process_command(
        state,
        Command::AnswerEffectChoice {
            player,
            choice_id,
            answer,
        },
    )
    .expect("answer should be accepted")
}

/// The outstanding question, cloned.
fn outstanding_question(state: &GameState) -> EffectChoiceQuestion {
    state
        .pending_effect_choice()
        .expect("a CR 608.2d choice should be outstanding")
        .question
        .clone()
}

fn zone_of(state: &GameState, name: &str) -> Option<ZoneId> {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name)
        .map(|o| o.zone)
}

fn names_in_library(state: &GameState, player: PlayerId) -> Vec<String> {
    state
        .zones()
        .get(&ZoneId::Library(player))
        .expect("library exists")
        .object_ids()
        .into_iter()
        .filter_map(|id| {
            state
                .objects()
                .get(&id)
                .map(|o| o.characteristics.name.clone())
        })
        .collect()
}

fn name_of(state: &GameState, id: ObjectId) -> String {
    state
        .objects()
        .get(&id)
        .map(|o| o.characteristics.name.clone())
        .unwrap_or_else(|| "<gone>".to_string())
}

fn spell_def(name: &str, card_id: &str, effect: Effect) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(card_id.to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Sorcery],
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

fn cast(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
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
    }))
}

/// A two-player fixture whose only spell is `def`, in p1's hand, plus whatever
/// `extra` objects the caller wants. p1 has 5 colourless floating.
fn fixture(def: CardDefinition, extra: Vec<ObjectSpec>) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        );
    for spec in extra {
        builder = builder.object(spec);
    }
    let mut state = builder.build().unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(p(1));
    state
}

fn library_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).in_zone(ZoneId::Library(owner))
}

/// "Search your library for a creature card and put it into your hand."
/// A STATED-QUALITY search (CR 701.23b), so failing to find is legal.
fn creature_tutor() -> CardDefinition {
    spell_def(
        "Creature Tutor",
        "dp9-creature-tutor",
        Effect::SearchLibrary {
            player: PlayerTarget::Controller,
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

/// "Search your library for a card and put it into your hand."
/// A QUANTITY-ONLY search (CR 701.23d), so finding is MANDATORY.
fn any_card_tutor() -> CardDefinition {
    spell_def(
        "Any Card Tutor",
        "dp9-any-card-tutor",
        Effect::SearchLibrary {
            player: PlayerTarget::Controller,
            filter: TargetFilter::default(),
            reveal: false,
            destination: ZoneTarget::Hand {
                owner: PlayerTarget::Controller,
            },
            shuffle_before_placing: false,
            also_search_graveyard: false,
        },
    )
}

fn scry_spell(n: i32) -> CardDefinition {
    spell_def(
        "Scry Spell",
        "dp9-scry-spell",
        Effect::Scry {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(n),
        },
    )
}

fn surveil_spell(n: i32) -> CardDefinition {
    spell_def(
        "Surveil Spell",
        "dp9-surveil-spell",
        Effect::Surveil {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(n),
        },
    )
}

/// Cast p1's only spell and pass both players — leaving the game at whatever the
/// resolution produced (blocked, or complete).
fn cast_and_resolve(state: GameState, spell_name: &str) -> (GameState, Vec<GameEvent>) {
    let spell_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == spell_name && o.zone == ZoneId::Hand(p(1)))
        .map(|(id, _)| *id)
        .expect("spell should be in p1's hand");
    let (state, _) = process_command(state, cast(p(1), spell_id)).expect("cast should succeed");
    pass_all(state, &[p(1), p(2)])
}

// ── T1 / T2 / T3 / T4 — library search (CR 701.23) ───────────────────────────

#[test]
/// CR 608.2d / CR 701.23a — a library search BLOCKS, and the whole resolution is
/// rolled back to the moment before it began.
///
/// Fail-before probe: on `main` the spell has already resolved and a card is in
/// hand by this point. Asserting the spell is STILL ON THE STACK fails there.
fn test_dp9_search_blocks_and_rolls_back() {
    let state = fixture(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
            library_creature(p(1), "Gamma"),
        ],
    );
    let lib_before = names_in_library(&state, p(1));
    let (state, events) = cast_and_resolve(state, "Creature Tutor");

    assert!(
        state.pending_effect_choice().is_some(),
        "CR 608.2d: the searching player must be asked"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 608.1: the roll-back puts the resolving spell back on the stack"
    );
    assert_eq!(
        names_in_library(&state, p(1)),
        lib_before,
        "CR 701.23a: nothing may move before the announcement"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellResolved { .. })),
        "the aborted pass must leak no events; got {events:?}"
    );
    let questions: Vec<&GameEvent> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::EffectChoiceRequired { .. }))
        .collect();
    assert_eq!(
        questions.len(),
        1,
        "exactly one question, got {questions:?}"
    );
    match questions[0] {
        GameEvent::EffectChoiceRequired {
            player, question, ..
        } => {
            assert_eq!(*player, p(1));
            match question {
                EffectChoiceQuestion::SearchLibrary {
                    candidates,
                    may_fail_to_find,
                } => {
                    assert_eq!(candidates.len(), 3, "all three creatures match the filter");
                    assert!(
                        *may_fail_to_find,
                        "CR 701.23b: a stated-quality search may decline to find"
                    );
                }
                other => panic!("wrong question shape: {other:?}"),
            }
        }
        other => panic!("wrong event: {other:?}"),
    }
    // Nobody has priority while blocked, and nobody may take it.
    assert!(
        matches!(
            state.blocking_decision(),
            Some(BlockingDecision::EffectChoice { .. })
        ),
        "the CR 608.2d entry must gate the engine"
    );
}

#[test]
/// CR 701.23a — the ANSWER decides which card is found, not the engine.
///
/// Fail-before probe: on `main` the engine took `min_by_key(|id| id.0)`, so the
/// HIGHEST-id candidate could never be found. This test names it.
fn test_dp9_chosen_card_is_found_not_the_lowest_id() {
    let state = fixture(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
            library_creature(p(1), "Gamma"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Creature Tutor");
    let candidates = match outstanding_question(&state) {
        EffectChoiceQuestion::SearchLibrary { candidates, .. } => candidates,
        other => panic!("expected a search question, got {other:?}"),
    };
    let highest = *candidates.last().expect("three candidates");
    let lowest = candidates[0];
    let highest_name = name_of(&state, highest);
    let lowest_name = name_of(&state, lowest);
    assert_ne!(highest, lowest, "the fixture must offer a real choice");

    let (state, _) = answer_with(
        state,
        EffectChoiceAnswer::SearchLibrary {
            found: Some(highest),
        },
    );

    assert!(
        matches!(zone_of(&state, &highest_name), Some(ZoneId::Hand(_))),
        "CR 701.23a: the ANNOUNCED card is the one found; library now {:?}",
        names_in_library(&state, p(1))
    );
    assert!(
        matches!(zone_of(&state, &lowest_name), Some(ZoneId::Library(_))),
        "the lowest-id candidate -- the pre-PB-DP9 auto-pick -- must stay in the library"
    );
    assert!(
        state.pending_effect_choice().is_none(),
        "the entry is cleared once answered"
    );
    assert!(
        state.stack_objects().is_empty(),
        "CR 608.2m: the resolution completes and the sorcery leaves the stack"
    );
    assert!(
        state.turn().priority_holder.is_some(),
        "CR 117.3b: priority is granted after the resolution completes"
    );
}

#[test]
/// CR 701.23b — "that player isn't required to find some or all of those cards
/// even if they're present in that zone." Failing to find is legal for a
/// stated-quality search.
fn test_dp9_legal_fail_to_find() {
    let state = fixture(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Creature Tutor");
    let lib_before = names_in_library(&state, p(1));

    let (state, _) = answer_with(state, EffectChoiceAnswer::SearchLibrary { found: None });

    assert_eq!(
        names_in_library(&state, p(1)),
        lib_before,
        "CR 701.23b: declining to find moves nothing"
    );
    assert!(
        state
            .zones()
            .get(&ZoneId::Hand(p(1)))
            .map(|z| z.is_empty())
            .unwrap_or(false),
        "nothing reaches hand when the player fails to find"
    );
    assert!(
        matches!(
            zone_of(&state, "Creature Tutor"),
            Some(ZoneId::Graveyard(_))
        ),
        "CR 608.2m: the sorcery still finishes resolving and goes to the graveyard"
    );
    assert!(
        state.turn().priority_holder.is_some(),
        "CR 117.3b: priority is granted"
    );
}

#[test]
/// CR 701.23d — "If a player is searching a hidden zone simply for a quantity of
/// cards, such as 'a card' [...] that player must find that many cards." So an
/// unrestricted search may NOT fail to find, and the engine rejects the attempt.
fn test_dp9_unrestricted_search_may_not_fail_to_find() {
    let state = fixture(
        any_card_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Any Card Tutor");
    match outstanding_question(&state) {
        EffectChoiceQuestion::SearchLibrary {
            may_fail_to_find, ..
        } => assert!(
            !may_fail_to_find,
            "CR 701.23d: a quantity-only search must find"
        ),
        other => panic!("expected a search question, got {other:?}"),
    }

    let hash_before = state.public_state_hash();
    let entry = state.pending_effect_choice().unwrap();
    let cmd = Command::AnswerEffectChoice {
        player: entry.player,
        choice_id: entry.choice_id,
        answer: EffectChoiceAnswer::SearchLibrary { found: None },
    };
    let err = process_command(state.clone(), cmd).expect_err("fail-to-find must be rejected");
    assert!(
        format!("{err:?}").contains("701.23d"),
        "the rejection must cite CR 701.23d; got {err:?}"
    );
    assert_eq!(
        state.public_state_hash(),
        hash_before,
        "a rejected command must leave the state untouched"
    );
}

#[test]
/// CR 701.23d — a quantity-only search with exactly ONE candidate has exactly one
/// legal answer, so the announcement is determined and no choice is offered.
fn test_dp9_forced_quantity_search_asks_nothing() {
    let state = fixture(any_card_tutor(), vec![library_creature(p(1), "Only")]);
    let (state, events) = cast_and_resolve(state, "Any Card Tutor");

    assert!(
        state.pending_effect_choice().is_none(),
        "CR 701.23d: no choice exists, so none is offered"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::EffectChoiceRequired { .. })),
        "no question event; got {events:?}"
    );
    assert!(
        matches!(zone_of(&state, "Only"), Some(ZoneId::Hand(_))),
        "the one legal card is found directly"
    );
}

// ── T5 / T6 — scry and surveil (CR 701.22a / 701.25a) ────────────────────────

#[test]
/// CR 701.22a — "put any number of them on the bottom [...] and THE REST ON TOP
/// of your library in any order." Keeping cards on top was unreachable before
/// PB-DP9.
///
/// Also pins PB-RS1's orientation: `Zone::top_n` is top-first, the library's top
/// is the LAST element and its bottom is index 0.
///
/// Fail-before probe: on `main` all three scried cards go to the bottom, so
/// asserting any of them is still on top fails.
fn test_dp9_scry_keeps_cards_on_top_in_a_chosen_order() {
    // Builder appends, so the library vector is [Filler, C, B, A] and A is the
    // TOP card (last element). `top_n(3)` therefore yields [A, B, C].
    let state = fixture(
        scry_spell(3),
        vec![
            library_creature(p(1), "Filler"),
            library_creature(p(1), "C"),
            library_creature(p(1), "B"),
            library_creature(p(1), "A"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Scry Spell");
    let looked_at = match outstanding_question(&state) {
        EffectChoiceQuestion::Scry { looked_at } => looked_at,
        other => panic!("expected a scry question, got {other:?}"),
    };
    let names: Vec<String> = looked_at.iter().map(|&id| name_of(&state, id)).collect();
    assert_eq!(
        names,
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        "CR 701.22a: the question offers the top N, top-first"
    );
    let (a, b, c) = (looked_at[0], looked_at[1], looked_at[2]);

    // Announce: B to the bottom; C then A back on top (C becomes the new top).
    let (state, _) = answer_with(
        state,
        EffectChoiceAnswer::Scry {
            bottom: vec![b],
            top: vec![c, a],
        },
    );

    let lib = names_in_library(&state, p(1));
    assert_eq!(
        lib,
        vec![
            "B".to_string(),
            "Filler".to_string(),
            "A".to_string(),
            "C".to_string()
        ],
        "CR 701.22a / PB-RS1: index 0 is the BOTTOM (B) and the last element is \
         the TOP (C, because `top[0]` is top-most)"
    );
    assert!(
        state.stack_objects().is_empty(),
        "the resolution completes after the announcement"
    );
}

#[test]
/// CR 701.25a — "put any number of them into your graveyard and THE REST ON TOP."
/// Before PB-DP9 `Surveil N` was unconditionally `Mill N`.
///
/// CR 701.25d — the `Surveilled` event fires after the process completes, "even
/// if some or all of those actions were impossible".
///
/// Fail-before probe: on `main` both surveilled cards are milled, so asserting
/// that one is still in the library fails.
fn test_dp9_surveil_keeps_cards_on_top() {
    // Library vector [Filler, B, A]; top_n(2) == [A, B].
    let state = fixture(
        surveil_spell(2),
        vec![
            library_creature(p(1), "Filler"),
            library_creature(p(1), "B"),
            library_creature(p(1), "A"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Surveil Spell");
    let looked_at = match outstanding_question(&state) {
        EffectChoiceQuestion::Surveil { looked_at } => looked_at,
        other => panic!("expected a surveil question, got {other:?}"),
    };
    assert_eq!(
        looked_at
            .iter()
            .map(|&id| name_of(&state, id))
            .collect::<Vec<_>>(),
        vec!["A".to_string(), "B".to_string()],
        "CR 701.25a: the top N, top-first"
    );
    let (a, b) = (looked_at[0], looked_at[1]);

    let (state, events) = answer_with(
        state,
        EffectChoiceAnswer::Surveil {
            graveyard: vec![b],
            top: vec![a],
        },
    );

    assert!(
        matches!(zone_of(&state, "B"), Some(ZoneId::Graveyard(_))),
        "CR 701.25a: the announced card goes to the graveyard"
    );
    let lib = names_in_library(&state, p(1));
    assert_eq!(
        lib.last().map(String::as_str),
        Some("A"),
        "CR 701.25a: the rest stays ON TOP; library is {lib:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Surveilled { count: 2, .. })),
        "CR 701.25d: the Surveilled event still fires, with the looked-at count"
    );
}

#[test]
/// CR 701.22b / CR 701.25c — "If a player is instructed to scry 0 [surveil 0], no
/// scry [surveil] event occurs."
///
/// The surveil arm already had this guard; the SCRY arm did not, and emitted
/// `Scried { count: 0 }` — so a "whenever you scry" trigger would have fired off a
/// Scry 0. That half is a fix, not a regression guard.
fn test_dp9_scry_zero_and_surveil_zero_ask_nothing() {
    for (def, name) in [
        (scry_spell(0), "Scry Spell"),
        (surveil_spell(0), "Surveil Spell"),
    ] {
        let state = fixture(def, vec![library_creature(p(1), "Top")]);
        let (state, events) = cast_and_resolve(state, name);
        assert!(
            state.pending_effect_choice().is_none(),
            "{name}: no choice is offered for a zero-count instruction"
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, GameEvent::Scried { .. } | GameEvent::Surveilled { .. })),
            "CR 701.22b / 701.25c: no event occurs for {name}; got {events:?}"
        );
        assert!(
            matches!(zone_of(&state, "Top"), Some(ZoneId::Library(_))),
            "{name}: nothing moves"
        );
    }
}

// ── T7 / T13 — answer validation (the SR-29 trust boundary) ──────────────────

#[test]
/// CR 608.2d — the rejection classes, each asserted by its specific message, with
/// the state hash unchanged after every one. Split from the partition cases below
/// so each test names one question shape.
fn test_dp9_search_answer_rejections() {
    let state = fixture(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Creature Tutor");
    let entry = state.pending_effect_choice().unwrap().clone();
    let candidates = match &entry.question {
        EffectChoiceQuestion::SearchLibrary { candidates, .. } => candidates.clone(),
        other => panic!("expected a search question, got {other:?}"),
    };
    let hash = state.public_state_hash();

    let cases: Vec<(&str, PlayerId, u64, EffectChoiceAnswer, &str)> = vec![
        (
            "an id that is not a candidate",
            entry.player,
            entry.choice_id,
            EffectChoiceAnswer::SearchLibrary {
                found: Some(ObjectId(9_999)),
            },
            "701.23a",
        ),
        (
            "an answer from the wrong player (SR-29 trust boundary)",
            p(2),
            entry.choice_id,
            EffectChoiceAnswer::SearchLibrary {
                found: Some(candidates[0]),
            },
            "608.2d",
        ),
        (
            "a stale choice_id (the MOMENT guard)",
            entry.player,
            entry.choice_id + 1,
            EffectChoiceAnswer::SearchLibrary {
                found: Some(candidates[0]),
            },
            "stale choice",
        ),
        (
            "an answer of the wrong variant",
            entry.player,
            entry.choice_id,
            EffectChoiceAnswer::Scry {
                bottom: vec![],
                top: vec![],
            },
            "does not answer question",
        ),
    ];
    for (label, player, choice_id, answer, expect) in cases {
        let mut probe = state.clone();
        let err =
            mtg_engine::effects::handle_answer_effect_choice(&mut probe, player, choice_id, answer)
                .expect_err(label);
        let msg = format!("{err:?}");
        assert!(
            msg.contains(expect),
            "{label}: expected a message naming {expect:?}, got {msg}"
        );
        assert_eq!(
            probe.public_state_hash(),
            hash,
            "{label}: a rejected answer must leave the state untouched"
        );
    }

    // ...and the control: an ACCEPTED answer DOES move the hash, so the pins
    // above cannot be passing for the wrong reason.
    let mut accepted = state.clone();
    mtg_engine::effects::handle_answer_effect_choice(
        &mut accepted,
        entry.player,
        entry.choice_id,
        EffectChoiceAnswer::SearchLibrary {
            found: Some(candidates[0]),
        },
    )
    .expect("a legal answer must be accepted");
    assert_ne!(
        accepted.public_state_hash(),
        hash,
        "an accepted answer must change the state"
    );
}

#[test]
/// CR 701.22a — the announced halves must PARTITION the cards looked at: same
/// multiset, no duplicates within or across them, nothing else. The ORDER is the
/// player's (CR 401.4); the multiset is a constraint the engine enforces.
fn test_dp9_scry_partition_rejections() {
    let state = fixture(
        scry_spell(2),
        vec![
            library_creature(p(1), "Filler"),
            library_creature(p(1), "B"),
            library_creature(p(1), "A"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Scry Spell");
    let entry = state.pending_effect_choice().unwrap().clone();
    let looked_at = match &entry.question {
        EffectChoiceQuestion::Scry { looked_at } => looked_at.clone(),
        other => panic!("expected a scry question, got {other:?}"),
    };
    let hash = state.public_state_hash();

    let cases: Vec<(&str, EffectChoiceAnswer, &str)> = vec![
        (
            "a partition missing an id",
            EffectChoiceAnswer::Scry {
                bottom: vec![looked_at[0]],
                top: vec![],
            },
            "exactly once",
        ),
        (
            "a partition with a duplicate",
            EffectChoiceAnswer::Scry {
                bottom: vec![looked_at[0]],
                top: vec![looked_at[0]],
            },
            "more than once",
        ),
        (
            "a partition naming a card that was not looked at",
            EffectChoiceAnswer::Scry {
                bottom: vec![looked_at[0]],
                top: vec![ObjectId(9_999)],
            },
            "not one of the cards looked at",
        ),
    ];
    for (label, answer, expect) in cases {
        let mut probe = state.clone();
        let err = mtg_engine::effects::handle_answer_effect_choice(
            &mut probe,
            entry.player,
            entry.choice_id,
            answer,
        )
        .expect_err(label);
        let msg = format!("{err:?}");
        assert!(
            msg.contains(expect),
            "{label}: expected a message naming {expect:?}, got {msg}"
        );
        assert_eq!(
            probe.public_state_hash(),
            hash,
            "{label}: a rejected answer must leave the state untouched"
        );
    }

    // No entry at all.
    let mut fresh = fixture(scry_spell(1), vec![library_creature(p(1), "X")]);
    let err = mtg_engine::effects::handle_answer_effect_choice(
        &mut fresh,
        p(1),
        1,
        EffectChoiceAnswer::Scry {
            bottom: vec![],
            top: vec![],
        },
    )
    .expect_err("answering with no entry outstanding must be rejected");
    assert!(
        format!("{err:?}").contains("no resolution-time choice is pending"),
        "got {err:?}"
    );
}

// ── T8 / T16 — the admission gate ────────────────────────────────────────────

#[test]
/// CR 608.2d / CR 608.1 — while the resolution is rolled back nobody has
/// priority, so `process_command` rejects every command except the answer from
/// the named player and `Concede`. The state hash is unchanged in every case.
///
/// Also documents plan §1.5 exit 5 as a PROPERTY rather than a comment: the
/// suspended object cannot leave the stack, because nothing that could remove it
/// is admitted.
fn test_dp9_admission_gate_while_blocked() {
    let state = fixture(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Creature Tutor");
    let hash = state.public_state_hash();
    let stack_len = state.stack_objects().len();
    assert_eq!(stack_len, 1, "the resolving spell is back on the stack");

    let rejected: Vec<(&str, Command)> = vec![
        ("PassPriority p1", Command::PassPriority { player: p(1) }),
        ("PassPriority p2", Command::PassPriority { player: p(2) }),
        (
            "PlayLand p1",
            Command::PlayLand {
                player: p(1),
                card: ObjectId(1),
            },
        ),
        (
            "TapForMana p2",
            Command::TapForMana {
                player: p(2),
                source: ObjectId(1),
                ability_index: 0,
                chosen_color: None,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        ),
        ("CastSpell p1", cast(p(1), ObjectId(1))),
        (
            "AnswerEffectChoice from the WRONG player",
            Command::AnswerEffectChoice {
                player: p(2),
                choice_id: state.pending_effect_choice().unwrap().choice_id,
                answer: EffectChoiceAnswer::SearchLibrary { found: None },
            },
        ),
    ];
    for (label, cmd) in rejected {
        let err = process_command(state.clone(), cmd)
            .expect_err(&format!("{label} must be rejected while blocked"));
        assert!(
            format!("{err:?}").contains("BlockedByPendingDecision"),
            "{label}: expected BlockedByPendingDecision, got {err:?}"
        );
        assert_eq!(
            state.public_state_hash(),
            hash,
            "{label}: the state must be untouched"
        );
        assert_eq!(
            state.stack_objects().len(),
            stack_len,
            "{label}: the suspended object cannot leave the stack (plan §1.5 exit 5)"
        );
    }
}

// ── T9 / T10 / T12 — the roll-back and the replay ────────────────────────────

#[test]
/// CR 608.2c / CR 608.2d — the roll-back is TOTAL: an effect that ran BEFORE the
/// choice is undone too, and after the answer it runs exactly ONCE.
///
/// Fail-before probe: the "damage not yet dealt at the moment the search would
/// ask" half is new surface; the "exactly once" half is the guard against a
/// partial-apply bug.
fn test_dp9_rollback_is_total() {
    let def = spell_def(
        "Bolt Then Tutor",
        "dp9-bolt-then-tutor",
        Effect::Sequence(vec![
            Effect::DealDamage {
                target: mtg_engine::CardEffectTarget::EachOpponent,
                amount: EffectAmount::Fixed(3),
                source: None,
            },
            Effect::SearchLibrary {
                player: PlayerTarget::Controller,
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
        ]),
    );
    let state = fixture(
        def,
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let life_before = state.players()[&p(2)].life_total;
    let lib_before = names_in_library(&state, p(1));

    let (state, _) = cast_and_resolve(state, "Bolt Then Tutor");
    assert!(
        state.pending_effect_choice().is_some(),
        "the search must ask"
    );
    assert_eq!(
        state.players()[&p(2)].life_total,
        life_before,
        "CR 608.2d: the roll-back undoes the damage the Sequence had already dealt"
    );
    assert_eq!(
        names_in_library(&state, p(1)),
        lib_before,
        "the library is untouched"
    );

    let (state, _) = answer_pending_effect_choice(state);
    assert_eq!(
        state.players()[&p(2)].life_total,
        life_before - 3,
        "after the answer the damage is dealt EXACTLY once (not zero, not twice)"
    );
}

#[test]
/// CR 608.2c — "the instructions are followed in the order written". A resolution
/// containing TWO choices asks them one at a time, with different `choice_id`s,
/// and applies both.
///
/// This is the replay mechanism's own test: the second pass re-executes the
/// search from the top and consumes the banked answer at the choice point.
fn test_dp9_two_choices_in_one_resolution() {
    let def = spell_def(
        "Tutor Then Scry",
        "dp9-tutor-then-scry",
        Effect::Sequence(vec![
            Effect::SearchLibrary {
                player: PlayerTarget::Controller,
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
            Effect::Scry {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
        ]),
    );
    let state = fixture(
        def,
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
            library_creature(p(1), "Gamma"),
        ],
    );
    let (state, events1) = cast_and_resolve(state, "Tutor Then Scry");
    assert_eq!(
        state.effect_choice_answers().len(),
        0,
        "no answer is banked before the first is given"
    );
    let q1 = outstanding_question(&state);
    assert!(
        matches!(q1, EffectChoiceQuestion::SearchLibrary { .. }),
        "CR 608.2c: the SEARCH is asked first -- it is written first"
    );
    let id1 = state.pending_effect_choice().unwrap().choice_id;

    let (state, events2) = answer_pending_effect_choice(state);
    assert_eq!(
        state.effect_choice_answers().len(),
        1,
        "the first answer is banked for the replay"
    );
    let q2 = outstanding_question(&state);
    assert!(
        matches!(q2, EffectChoiceQuestion::Scry { .. }),
        "the replay reaches the SCRY next"
    );
    let id2 = state.pending_effect_choice().unwrap().choice_id;
    assert_ne!(id1, id2, "each question gets its own MOMENT guard");

    let (state, events3) = answer_pending_effect_choice(state);
    assert!(
        state.effect_choice_answers().is_empty(),
        "the bank's life ends with the resolution"
    );
    assert!(
        state.pending_effect_choice().is_none(),
        "no choice is outstanding once the resolution completes"
    );

    // Each question was emitted exactly once across the whole sequence; the
    // discarded passes leaked nothing.
    let all: Vec<&GameEvent> = events1
        .iter()
        .chain(events2.iter())
        .chain(events3.iter())
        .collect();
    let questions = all
        .iter()
        .filter(|e| matches!(e, GameEvent::EffectChoiceRequired { .. }))
        .count();
    assert_eq!(questions, 2, "exactly two questions, got {questions}");
    let scried = all
        .iter()
        .filter(|e| matches!(e, GameEvent::Scried { .. }))
        .count();
    assert_eq!(scried, 1, "the scry happened exactly once, got {scried}");
}

#[test]
/// CR 608.2c — a choice nested inside `Conditional` inside `Sequence`: after the
/// answer, the instructions AFTER the choice run, and the instruction BEFORE it
/// does not run twice.
///
/// This is the nesting proof. There is no cursor: the replay re-evaluates the
/// `Conditional` (same state ⇒ same branch) and re-collects the `Sequence`, which
/// is *correct* rather than merely convenient because both are pure functions of
/// a state the roll-back restored.
fn test_dp9_choice_inside_conditional_and_sequence() {
    let inner = Effect::Sequence(vec![
        Effect::SearchLibrary {
            player: PlayerTarget::Controller,
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
        Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(5),
        },
    ]);
    let def = spell_def(
        "Nested Tutor",
        "dp9-nested-tutor",
        Effect::Sequence(vec![
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            Effect::Conditional {
                condition: mtg_engine::Condition::Always,
                if_true: Box::new(inner),
                if_false: Box::new(Effect::Nothing),
            },
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(7),
            },
        ]),
    );
    let state = fixture(
        def,
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let life_before = state.players()[&p(1)].life_total;

    let (state, _) = cast_and_resolve(state, "Nested Tutor");
    assert!(
        state.pending_effect_choice().is_some(),
        "the nested search must ask"
    );
    assert_eq!(
        state.players()[&p(1)].life_total,
        life_before,
        "the roll-back undoes the +1 the Sequence had already applied"
    );

    let (state, _) = answer_pending_effect_choice(state);
    assert_eq!(
        state.players()[&p(1)].life_total,
        life_before + 1 + 5 + 7,
        "after the answer every instruction runs EXACTLY once: the +1 before the \
         choice, the +5 after it inside the Conditional, and the +7 after the \
         Conditional"
    );
}

#[test]
/// CR 608.2e / CR 101.4 / CR 701.23i — "each player searches their library":
/// every player is asked in turn, each with their OWN candidates, and all the
/// answers are applied.
///
/// **Recorded deviation (OOS-DP9-8):** CR 701.22c / 701.23i / 608.2e require the
/// per-player decisions to be made in **APNAP** order.
/// `effects::resolve_player_target_list` iterates `state.players.keys()` — an
/// `imbl::OrdMap`, i.e. ascending `PlayerId`. That is pre-existing (it governs
/// every `ForEach::EachPlayer` effect, far beyond this roster), and PB-DP9 makes
/// it *observable* for the first time because the questions are now asked in that
/// order. Not fixed here; this test asserts the order the engine actually has.
fn test_dp9_choice_inside_for_each_each_player() {
    let def = spell_def(
        "Everyone Tutors",
        "dp9-everyone-tutors",
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
    );
    let state = fixture(
        def,
        vec![
            library_creature(p(1), "P1 Alpha"),
            library_creature(p(1), "P1 Beta"),
            library_creature(p(2), "P2 Alpha"),
            library_creature(p(2), "P2 Beta"),
        ],
    );

    let (mut state, _) = cast_and_resolve(state, "Everyone Tutors");
    let mut asked: Vec<PlayerId> = Vec::new();
    let mut chosen: Vec<String> = Vec::new();
    let mut guard = 0;
    while let Some(entry) = state.pending_effect_choice() {
        asked.push(entry.player);
        let candidates = match &entry.question {
            EffectChoiceQuestion::SearchLibrary { candidates, .. } => candidates.clone(),
            other => panic!("expected a search question, got {other:?}"),
        };
        assert_eq!(
            candidates.len(),
            2,
            "each player is offered only their OWN library's matches"
        );
        // Announce the SECOND candidate, which the pre-PB-DP9 auto-pick could
        // never have taken.
        let pick = candidates[1];
        chosen.push(name_of(&state, pick));
        let (s, _) = answer_with(
            state,
            EffectChoiceAnswer::SearchLibrary { found: Some(pick) },
        );
        state = s;
        guard += 1;
        assert!(guard < 8, "the per-player questions did not converge");
    }

    assert_eq!(
        asked,
        vec![p(1), p(2)],
        "every player named by the effect is asked, one at a time. The ORDER is \
         ascending PlayerId, which is what the engine actually does -- CR 608.2e / \
         701.23i want APNAP (seed OOS-DP9-8)."
    );
    assert_eq!(
        chosen,
        vec!["P1 Beta".to_string(), "P2 Beta".to_string()],
        "each player's own announcement is the one applied"
    );
    for name in ["P1 Beta", "P2 Beta"] {
        assert!(
            matches!(zone_of(&state, name), Some(ZoneId::Hand(_))),
            "{name} should have been found"
        );
    }
    for name in ["P1 Alpha", "P2 Alpha"] {
        assert!(
            matches!(zone_of(&state, name), Some(ZoneId::Library(_))),
            "{name} -- the pre-PB-DP9 auto-pick -- must stay in the library"
        );
    }
}

// ── T13 — the moment guard ───────────────────────────────────────────────────

#[test]
/// CR 608.2d — the `choice_id` MOMENT guard. An answer quoting a superseded id is
/// rejected, including the id of the PREVIOUS choice in the same resolution
/// (PB-DP7 lesson 2 / PB-DP8 HIGH-1: bind positionally, never lazily).
fn test_dp9_stale_choice_id_rejected() {
    let def = spell_def(
        "Tutor Then Scry Again",
        "dp9-tutor-then-scry-2",
        Effect::Sequence(vec![
            Effect::SearchLibrary {
                player: PlayerTarget::Controller,
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
            Effect::Scry {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
        ]),
    );
    let state = fixture(
        def,
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Tutor Then Scry Again");
    let first_id = state.pending_effect_choice().unwrap().choice_id;
    let (state, _) = answer_pending_effect_choice(state);
    let second = state.pending_effect_choice().unwrap().clone();
    assert_ne!(first_id, second.choice_id);
    let hash = state.public_state_hash();

    for (label, id) in [
        ("choice_id + 1", second.choice_id + 1),
        ("the PREVIOUS choice's id", first_id),
    ] {
        let looked_at = match &second.question {
            EffectChoiceQuestion::Scry { looked_at } => looked_at.clone(),
            other => panic!("expected a scry question, got {other:?}"),
        };
        let err = process_command(
            state.clone(),
            Command::AnswerEffectChoice {
                player: second.player,
                choice_id: id,
                answer: EffectChoiceAnswer::Scry {
                    bottom: vec![],
                    top: looked_at,
                },
            },
        )
        .expect_err(&format!("{label} must be rejected"));
        assert!(
            format!("{err:?}").contains("stale choice"),
            "{label}: got {err:?}"
        );
        assert_eq!(
            state.public_state_hash(),
            hash,
            "{label}: the state must be untouched"
        );
    }
}

// ── T14 / T15 — the concede exits (plan §1.5) ────────────────────────────────

#[test]
/// CR 104.3a / CR 800.4j / CR 608.2d — the entry's OWN player concedes while the
/// choice is outstanding.
///
/// Without the discharge this is an unrecoverable deadlock: `priority_holder` is
/// `None`, `players_passed` is full, nobody can pass and nothing else drives
/// `handle_all_passed`. This is PB-DP8's exact bug class, which it shipped three
/// times — so this test asserts against the hazardous state (a live player can
/// actually ACT), not merely that the entry is gone.
fn test_dp9_owner_concedes_mid_choice() {
    let state = fixture(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Creature Tutor");
    assert!(state.pending_effect_choice().is_some());

    let (state, _) =
        process_command(state, Command::Concede { player: p(1) }).expect("concede should succeed");

    assert!(
        state.pending_effect_choice().is_none(),
        "the departed player's entry must be cleared"
    );
    assert!(
        state.effect_choice_answers().is_empty(),
        "the answer bank is DROPPED, not kept: the concede mutated the board"
    );
    assert!(
        state.blocking_decision().is_none(),
        "the game must not stay blocked on a player who has left"
    );
    // With only p2 left the game is over (CR 104.2b), which is its own valid
    // exit -- so assert the game is either over or genuinely playable, never
    // deadlocked in between.
    let over = state
        .players()
        .values()
        .filter(|pl| !pl.has_lost && !pl.has_conceded)
        .count()
        <= 1;
    if !over {
        let holder = state
            .turn()
            .priority_holder
            .expect("a live player must hold priority");
        assert_ne!(holder, p(1), "priority must not name the conceded seat");
        process_command(state, Command::PassPriority { player: holder })
            .expect("the holder must be able to act -- this is the deadlock assertion");
    }
}

#[test]
/// CR 104.3a / CR 603.3b — a FOREIGN concede must not step over the block.
///
/// This pins PB-DP8's obligation-5 gate generalising for free: `handle_concede`
/// refuses to advance priority or the turn while `blocking_decision(state)` is
/// `Some(..)`, and it reads the predicate rather than any one field.
fn test_dp9_foreign_concede_does_not_step_over_the_block() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![creature_tutor()]))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), "Creature Tutor")
                .with_card_id(CardId("dp9-creature-tutor".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        );
    for name in ["Alpha", "Beta"] {
        builder = builder.object(library_creature(p(1), name));
    }
    let mut state = builder.build().unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(p(1));

    let spell_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Creature Tutor")
        .map(|(id, _)| *id)
        .unwrap();
    let (state, _) = process_command(state, cast(p(1), spell_id)).unwrap();
    let (state, _) = pass_all(state, &[p(1), p(2), p(3)]);
    let entry_id = state
        .pending_effect_choice()
        .expect("p1 must be asked")
        .choice_id;

    let (state, events) = process_command(state, Command::Concede { player: p(2) })
        .expect("a foreign concede is always admitted");

    assert_eq!(
        state
            .pending_effect_choice()
            .expect("the entry survives a foreign concede")
            .choice_id,
        entry_id,
        "the block persists, correctly"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::PriorityGiven { .. })),
        "no priority may be granted while the block stands; got {events:?}"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "no stack resolution may happen either"
    );

    // ...and p1's answer still completes the resolution.
    let (state, _) = answer_pending_effect_choice(state);
    assert!(state.pending_effect_choice().is_none());
    assert!(state.stack_objects().is_empty());
}

// ── T17 — the defaults, and the deliberate flip ──────────────────────────────

#[test]
/// CR 608.2d — the three exported defaults.
///
/// The SEARCH default is byte-identical to the pre-PB-DP9 `min_by_key(|id| id.0)`
/// auto-pick, so that half of the batch is zero-churn. The SCRY and SURVEIL
/// defaults are the IDENTITY and are explicitly **NOT** the pre-PB-DP9 behaviour
/// (bottom-everything / mill-everything). Both directions are asserted here so
/// the flip is documented by a test rather than by a commit message.
fn test_dp9_defaults_reproduce_the_stated_behaviour() {
    use mtg_engine::effects::{
        default_effect_choice_answer, default_scry_answer, default_search_answer,
        default_surveil_answer,
    };
    let ids = vec![ObjectId(7), ObjectId(11), ObjectId(23)];

    let q = EffectChoiceQuestion::SearchLibrary {
        candidates: ids.clone(),
        may_fail_to_find: true,
    };
    assert_eq!(
        default_search_answer(&q),
        EffectChoiceAnswer::SearchLibrary {
            found: Some(ObjectId(7))
        },
        "CR 701.23a: the default is the lowest ObjectId -- the pre-PB-DP9 auto-pick"
    );
    assert_eq!(default_effect_choice_answer(&q), default_search_answer(&q));

    let q = EffectChoiceQuestion::Scry {
        looked_at: ids.clone(),
    };
    assert_eq!(
        default_scry_answer(&q),
        EffectChoiceAnswer::Scry {
            bottom: vec![],
            top: ids.clone()
        },
        "CR 701.22a: the default is the IDENTITY -- keep everything on top"
    );
    assert_ne!(
        default_scry_answer(&q),
        EffectChoiceAnswer::Scry {
            bottom: ids.clone(),
            top: vec![]
        },
        "and it is deliberately NOT the pre-PB-DP9 bottom-everything"
    );
    assert_eq!(default_effect_choice_answer(&q), default_scry_answer(&q));

    let q = EffectChoiceQuestion::Surveil {
        looked_at: ids.clone(),
    };
    assert_eq!(
        default_surveil_answer(&q),
        EffectChoiceAnswer::Surveil {
            graveyard: vec![],
            top: ids.clone()
        },
        "CR 701.25a: the default is the IDENTITY -- keep everything on top"
    );
    assert_ne!(
        default_surveil_answer(&q),
        EffectChoiceAnswer::Surveil {
            graveyard: ids.clone(),
            top: vec![]
        },
        "and it is deliberately NOT the pre-PB-DP9 mill-everything -- `Surveil N` \
         was exactly `Mill N`, which can also deck a player under CR 704.5b"
    );
    assert_eq!(default_effect_choice_answer(&q), default_surveil_answer(&q));
}

// ── T20 — hidden information (Architecture Invariant 7) ──────────────────────

#[test]
/// Architecture Invariant 7 / CR 401.2 — every id in an `EffectChoiceRequired`
/// names a card in a HIDDEN zone, so the event is private to the asked seat.
///
/// **There is no network filter to enforce this yet** — the M10 centralized
/// server is the intended consumer of `private_to()` and does not exist. This
/// test pins the DECLARATION, and the batch must not read as though the leak is
/// closed.
fn test_dp9_private_to_leak_probe() {
    let state = fixture(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (_state, events) = cast_and_resolve(state, "Creature Tutor");
    let ev = events
        .iter()
        .find(|e| matches!(e, GameEvent::EffectChoiceRequired { .. }))
        .expect("the question must be emitted");

    assert_eq!(
        ev.private_to(),
        Some(p(1)),
        "the searching player is the only seat that may see the candidate list"
    );
    assert_ne!(ev.private_to(), Some(p(2)), "no other seat");
    assert!(
        ev.reveals_hidden_info(),
        "knowing WHICH library cards match a filter is hidden information"
    );

    // The event carries only ObjectIds -- never a card name or CardId.
    let json = serde_json::to_string(ev).expect("serialize");
    assert!(
        !json.contains("Alpha") && !json.contains("Beta"),
        "the question must not carry card identity; got {json}"
    );

    // CR 514.1 (PB-DP7): the same declaration for the cleanup discard, whose
    // `hand` field is the exact ObjectId composition of a hidden zone.
    let discard = GameEvent::CleanupDiscardChoiceRequired {
        player: p(2),
        count: 1,
        hand: vec![ObjectId(1)],
    };
    assert_eq!(discard.private_to(), Some(p(2)));

    // ...and a public event is public.
    assert_eq!(GameEvent::AllPlayersPassed.private_to(), None);
}

// ── T21 — the loop-detection deviation ───────────────────────────────────────

#[test]
/// CR 726 (PB-DP9's deliberate deviation from the PB-DP7/PB-DP8 precedent).
///
/// The three CR 608.2d fields are in `public_state_hash` but NOT in
/// `loop_detection.rs`'s mandatory-state fingerprint. This test pins both
/// directions at once, using the one construction that isolates them: a
/// rolled-back-and-blocked resolution has a board BYTE-IDENTICAL to the moment
/// before the resolving pass (that is what the roll-back means), so the two
/// states differ only in the choice fields plus the priority bookkeeping the
/// mandatory hash already ignores.
///
/// If the fields were folded in, two structurally identical CR 726 positions
/// would fingerprint differently and a mandatory loop could be silently masked.
fn test_dp9_loop_detection_fingerprint_excludes_the_choice_state() {
    use mtg_engine::rules::loop_detection::compute_mandatory_state_hash;

    let state = fixture(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let spell_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Creature Tutor")
        .map(|(id, _)| *id)
        .unwrap();
    let (state, _) = process_command(state, cast(p(1), spell_id)).unwrap();
    // p1 passes; the spell is on the stack and nothing has resolved.
    let (before, _) = pass_all(state, &[p(1)]);
    let mandatory_before = compute_mandatory_state_hash(&before);
    let public_before = before.public_state_hash();

    // p2 passes; the resolution starts, hits the search, and is rolled back.
    let (blocked, _) = pass_all(before, &[p(2)]);
    assert!(
        blocked.pending_effect_choice().is_some(),
        "sanity: the resolution must be blocked"
    );

    assert_eq!(
        compute_mandatory_state_hash(&blocked),
        mandatory_before,
        "CR 726: a rolled-back, blocked resolution is the SAME mandatory-loop \
         position as the moment before it -- the choice fields are excluded"
    );
    assert_ne!(
        blocked.public_state_hash(),
        public_before,
        "...and a DIFFERENT public position -- the choice fields ARE hashed there"
    );
}

// ── T22 / T23 — rosters (SR-36) ──────────────────────────────────────────────

mod roster {
    use mtg_card_defs::all_cards;
    use mtg_card_types::cards::card_definition::{AbilityDefinition, Completeness, Effect};

    /// Walk an `Effect` tree, including every nesting combinator, and report
    /// whether any node satisfies `pred`.
    ///
    /// A flat scan UNDERCOUNTS: the three asking effects live inside `Sequence`,
    /// `ForEach`, `Conditional`, `Repeat`, `MayPayThenEffect`, `Choose`, the
    /// coin-flip arms and so on all over the corpus.
    pub fn effect_contains(effect: &Effect, pred: &dyn Fn(&Effect) -> bool) -> bool {
        if pred(effect) {
            return true;
        }
        let mut kids: Vec<&Effect> = Vec::new();
        match effect {
            Effect::Sequence(v) => kids.extend(v.iter()),
            Effect::Conditional {
                if_true, if_false, ..
            } => {
                kids.push(if_true);
                kids.push(if_false);
            }
            Effect::ForEach { effect, .. } => kids.push(effect),
            Effect::Repeat { effect, .. } => kids.push(effect),
            Effect::Choose { choices, .. } => kids.extend(choices.iter()),
            Effect::MayPayOrElse { or_else, .. } => kids.push(or_else),
            Effect::MayPayThenEffect { then, .. } => kids.push(then),
            Effect::RollDice { results, .. } => {
                kids.extend(results.iter().map(|(_, _, e)| e));
            }
            _ => {}
        }
        kids.into_iter().any(|k| effect_contains(k, pred))
    }

    /// Every `Effect` an `AbilityDefinition` can carry.
    pub fn ability_effects(a: &AbilityDefinition) -> Vec<&Effect> {
        match a {
            AbilityDefinition::Spell { effect, .. }
            | AbilityDefinition::Triggered { effect, .. }
            | AbilityDefinition::Activated { effect, .. } => vec![effect],
            _ => vec![],
        }
    }

    pub struct Roster {
        pub complete: Vec<String>,
        pub other: usize,
    }

    pub fn collect(pred: &dyn Fn(&Effect) -> bool) -> Roster {
        let mut complete = Vec::new();
        let mut other = 0usize;
        for def in all_cards() {
            let mut faces = vec![&def.abilities];
            if let Some(f) = def.back_face.as_ref() {
                faces.push(&f.abilities);
            }
            if let Some(f) = def.adventure_face.as_ref() {
                faces.push(&f.abilities);
            }
            let hit = faces.iter().any(|abilities| {
                abilities.iter().any(|a| {
                    ability_effects(a)
                        .into_iter()
                        .any(|e| effect_contains(e, pred))
                })
            });
            if !hit {
                continue;
            }
            if def.completeness == Completeness::Complete {
                complete.push(def.name.clone());
            } else {
                other += 1;
            }
        }
        complete.sort();
        Roster { complete, other }
    }
}

#[test]
/// SR-36 — the PB-DP9 rosters, ENUMERATED from `all_cards()` rather than grepped.
///
/// The audit claims 74 search / 16 scry / 8 surveil `Complete` defs. PB-DP8's row
/// records that both the audit's number and the planner's grep were wrong there,
/// so these three printed counts are the deliverable: they become the fact.
///
/// The assertions are `>=` on purpose — the authoring campaign adds cards
/// continuously and an `==` pin would redden on unrelated work.
fn test_dp9_roster_enumeration() {
    use mtg_card_types::cards::card_definition::Effect;

    let search = roster::collect(&|e| matches!(e, Effect::SearchLibrary { .. }));
    let scry = roster::collect(&|e| matches!(e, Effect::Scry { .. }));
    let surveil = roster::collect(&|e| matches!(e, Effect::Surveil { .. }));

    println!(
        "PB-DP9 roster (SR-36, enumerated from all_cards()):\n  \
         SearchLibrary: {} Complete (+{} non-Complete)\n  \
         Scry:          {} Complete (+{} non-Complete)\n  \
         Surveil:       {} Complete (+{} non-Complete)",
        search.complete.len(),
        search.other,
        scry.complete.len(),
        scry.other,
        surveil.complete.len(),
        surveil.other
    );
    println!("  search roster: {:?}", search.complete);
    println!("  scry roster:   {:?}", scry.complete);
    println!("  surveil roster: {:?}", surveil.complete);

    assert!(
        search.complete.len() >= 50,
        "SearchLibrary roster collapsed to {}",
        search.complete.len()
    );
    assert!(
        scry.complete.len() >= 10,
        "Scry roster collapsed to {}",
        scry.complete.len()
    );
    assert!(
        surveil.complete.len() >= 5,
        "Surveil roster collapsed to {}",
        surveil.complete.len()
    );
}

#[test]
/// CR 605.1b / CR 605.4a — the mana-ability gate.
///
/// A mana ability resolves immediately and OUTSIDE the stack, so there is no
/// stack object to roll back to and PB-DP9's suspension cannot apply. The
/// `EffectContext::effect_choice_gate_closed` flag makes the three arms take
/// their deterministic default there instead.
///
/// That branch skips an obligation (offering the choice), so this test is where
/// the obligation is discharged: **no `Complete` card def puts one of the three
/// asking effects inside a mana ability.** If this ever reddens, the branch has
/// become live and the card needs a rules decision, not a silent default.
///
/// The behavioural half is asserted directly: with the gate closed the effect
/// applies the default and records NO entry.
fn test_dp9_mana_ability_gate() {
    use mtg_card_types::cards::card_definition::Effect as CEffect;

    // (a) The roster obligation.
    // `ManaAbility` carries no `Effect` tree of its own, so the ONLY route into
    // the gated path is a `WhenTappedForMana` TRIGGERED ability whose effect is
    // mana-producing -- `rules::mana.rs`'s CR 605.4a branch, which is the single
    // site that closes the gate. Scan those.
    let mut offenders: Vec<String> = Vec::new();
    for def in mtg_card_defs::all_cards() {
        let mut faces = vec![&def.abilities];
        if let Some(f) = def.back_face.as_ref() {
            faces.push(&f.abilities);
        }
        for abilities in faces {
            for a in abilities {
                if let mtg_card_types::cards::card_definition::AbilityDefinition::Triggered {
                    trigger_condition,
                    effect,
                    ..
                } = a
                {
                    if !matches!(
                        trigger_condition,
                        mtg_card_types::cards::card_definition::TriggerCondition::WhenTappedForMana { .. }
                    ) {
                        continue;
                    }
                    let asks = roster::effect_contains(effect, &|e| {
                        matches!(
                            e,
                            CEffect::SearchLibrary { .. }
                                | CEffect::Scry { .. }
                                | CEffect::Surveil { .. }
                        )
                    });
                    if asks {
                        offenders.push(def.name.clone());
                    }
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "CR 605.1b: these defs put a CR 608.2d choice inside a mana ability, where \
         CR 605.4a leaves no room to announce it: {offenders:?}"
    );

    // (b) The behaviour, asserted directly.
    let mut state = fixture(
        scry_spell(1),
        vec![
            library_creature(p(1), "Filler"),
            library_creature(p(1), "Top"),
        ],
    );
    let mut ctx = mtg_engine::effects::EffectContext::new(p(1), ObjectId(9_999), vec![]);
    ctx.effect_choice_gate_closed = true;
    let events = mtg_engine::effects::execute_effect(
        &mut state,
        &Effect::Scry {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
        &mut ctx,
    );
    assert!(
        state.pending_effect_choice().is_none(),
        "CR 605.4a: a gated effect must not suspend -- there is nothing to roll back to"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Scried { count: 1, .. })),
        "the effect still completes, with the default answer"
    );
    assert_eq!(
        names_in_library(&state, p(1)).last().map(String::as_str),
        Some("Top"),
        "the default is the identity: the looked-at card stays on top"
    );
}
