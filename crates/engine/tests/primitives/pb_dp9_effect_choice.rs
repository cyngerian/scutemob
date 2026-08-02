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

/// A **three-player** fixture, otherwise identical to [`fixture`].
///
/// Fix-cycle Finding 1 (HIGH): the concede tests cannot use the 2-player
/// [`fixture`], because a concede there ends the game (CR 104.2b) and every
/// post-concede code path takes its `is_game_over` early exit. A test built on
/// it reads as coverage of the recovery path while executing none of it.
fn fixture_3p(def: CardDefinition, extra: Vec<ObjectSpec>) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
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

/// [`cast_and_resolve`] for [`fixture_3p`].
fn cast_and_resolve_3p(state: GameState, spell_name: &str) -> (GameState, Vec<GameEvent>) {
    let spell_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == spell_name && o.zone == ZoneId::Hand(p(1)))
        .map(|(id, _)| *id)
        .expect("spell should be in p1's hand");
    let (state, _) = process_command(state, cast(p(1), spell_id)).expect("cast should succeed");
    pass_all(state, &[p(1), p(2), p(3)])
}

fn library_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).in_zone(ZoneId::Library(owner))
}

/// A library creature carrying a real CR 205.4a stated quality (the Legendary
/// supertype), for the CR 701.23b half of
/// `test_dp9_may_fail_to_find_ignores_non_quality_filter_axes`.
fn legendary_library_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1)
        .with_supertypes(vec![mtg_engine::SuperType::Legendary])
        .in_zone(ZoneId::Library(owner))
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
///
/// **Fix-cycle Finding 1 (HIGH): the fixture must be THREE-player.** On the
/// 2-player `fixture()` the concede ends the game (CR 104.2b), so
/// `discharge_effect_choice_on_concede` takes its `is_game_over` early exit and
/// returns *before* `resolve_top_of_stack` — the whole "drive the rolled-back
/// resolution, do not merely clear it" behaviour never executed, while the
/// entry/bank/`blocking_decision` assertions all passed off the clear-only half.
/// That is PB-DP8's transferable rule (ii) — a test that constructs a hazardous
/// state and does not assert against it is worse than no test, because it reads
/// as coverage. Every assertion below is now unconditional.
fn test_dp9_owner_concedes_mid_choice() {
    let state = fixture_3p(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve_3p(state, "Creature Tutor");
    assert_eq!(state.pending_effect_choice().unwrap().player, p(1));
    let alpha = zone_of(&state, "Alpha").expect("Alpha exists");
    assert_eq!(
        alpha,
        ZoneId::Library(p(1)),
        "sanity: nothing has moved yet"
    );

    let (state, _) =
        process_command(state, Command::Concede { player: p(1) }).expect("concede should succeed");

    // The game SURVIVES the concede (p2 and p3 are still in), so the discharge's
    // resolve-and-drive half really runs.
    assert_eq!(
        state
            .players()
            .values()
            .filter(|pl| !pl.has_lost && !pl.has_conceded)
            .count(),
        2,
        "sanity: with 3 seats, one concede does not end the game -- if this ever \
         reddens the test has gone vacuous again"
    );

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

    // The resolution was DRIVEN, not merely unblocked.
    assert!(
        state.stack_objects().is_empty(),
        "the suspended spell must actually finish resolving -- clearing the entry \
         alone leaves it on the stack with nobody able to pass"
    );
    assert_eq!(
        zone_of(&state, "Alpha"),
        Some(ZoneId::Hand(p(1))),
        "CR 104.3a: a departed player announces nothing, so the search applied \
         the engine's default (the lowest-ObjectId candidate)"
    );

    // ...and the hazardous state itself: priority is RECOVERABLE.
    let holder = state
        .turn()
        .priority_holder
        .expect("a live player must hold priority -- this is the deadlock assertion");
    assert_ne!(holder, p(1), "priority must not name the conceded seat");
    assert!(
        state
            .players()
            .get(&holder)
            .map(|pl| !pl.has_lost && !pl.has_conceded)
            .unwrap_or(false),
        "the holder must be a live seat"
    );
    process_command(state, Command::PassPriority { player: holder })
        .expect("the holder must be able to act -- this is the deadlock assertion");
}

#[test]
/// CR 104.3a / CR 603.3b / CR 608.2d — a FOREIGN concede must not step over the
/// block, and must RE-ASK rather than resume the old moment.
///
/// This pins PB-DP8's obligation-5 gate generalising for free: `handle_concede`
/// refuses to advance priority or the turn while `blocking_decision(state)` is
/// `Some(..)`, and it reads the predicate rather than any one field.
///
/// Fix-cycle Finding 2 (HIGH) changed the *moment*: the concede mutated the
/// board, so the outstanding question is abandoned with the rest of the bank and
/// the still-live owner is re-asked with a fresh `choice_id` against the
/// post-concede board (CR 608.2d: the announcement is made "while applying the
/// effect"). The block itself persists, which is what this test guards.
fn test_dp9_foreign_concede_does_not_step_over_the_block() {
    let state = fixture_3p(
        creature_tutor(),
        vec![
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve_3p(state, "Creature Tutor");
    let entry_id = state
        .pending_effect_choice()
        .expect("p1 must be asked")
        .choice_id;

    let (state, events) = process_command(state, Command::Concede { player: p(2) })
        .expect("a foreign concede is always admitted");

    let entry = state
        .pending_effect_choice()
        .expect("the BLOCK survives a foreign concede");
    assert_eq!(entry.player, p(1), "still p1's choice to make");
    assert_ne!(
        entry.choice_id, entry_id,
        "but a NEW moment: the pre-concede question was asked against a board \
         that no longer exists (CR 608.2d)"
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
    // The re-ask is announced, so a client is never left holding a dead id.
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::EffectChoiceRequired { choice_id, .. } if *choice_id == entry.choice_id)
        ),
        "the fresh question must be emitted; got {events:?}"
    );

    // ...and p1's answer still completes the resolution.
    let (state, _) = answer_pending_effect_choice(state);
    assert!(state.pending_effect_choice().is_none());
    assert!(state.stack_objects().is_empty());
}

#[test]
/// CR 608.2d / CR 104.3a — a **foreign** concede invalidates the answer bank.
///
/// Fix-cycle Finding 2 (HIGH), fail-before probe. The abort-and-replay design
/// banks each answer and re-executes the whole resolution against the state the
/// questions were asked against. `Concede` is the only other command admitted
/// while a choice is outstanding, and it MUTATES that state — so every banked
/// answer stops being an answer to a question the replay still asks.
///
/// Before the fix the bank was dropped only when the ENTRY'S OWN player left, so
/// this sequence survived with a stale bank and the replay compared p2's
/// recomputed question against p1's banked answer:
/// `debug_assert!(false, "replay determinism violation")` — a panic in every
/// debug, test and fuzzer build, reached by a legal command sequence.
fn test_dp9_foreign_concede_invalidates_a_non_empty_bank() {
    let def = spell_def(
        "Everyone Tutors Thrice",
        "dp9-everyone-tutors-3p",
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
    let state = fixture_3p(
        def,
        vec![
            library_creature(p(1), "P1 Alpha"),
            library_creature(p(1), "P1 Beta"),
            library_creature(p(2), "P2 Alpha"),
            library_creature(p(2), "P2 Beta"),
            library_creature(p(3), "P3 Alpha"),
            library_creature(p(3), "P3 Beta"),
        ],
    );
    let (state, _) = cast_and_resolve_3p(state, "Everyone Tutors Thrice");

    // 1. p1 is asked first (ascending PlayerId — the OOS-DP9-8 deviation) and
    //    answers, so the bank is non-empty.
    assert_eq!(state.pending_effect_choice().unwrap().player, p(1));
    let p1_pick = match &outstanding_question(&state) {
        EffectChoiceQuestion::SearchLibrary { candidates, .. } => candidates[1],
        other => panic!("expected a search question, got {other:?}"),
    };
    let (state, _) = answer_with(
        state,
        EffectChoiceAnswer::SearchLibrary {
            found: Some(p1_pick),
        },
    );
    assert_eq!(
        state.effect_choice_answers().len(),
        1,
        "the bank is non-empty"
    );

    // 2. p2's question is now outstanding...
    let p2_choice_id = state.pending_effect_choice().unwrap().choice_id;
    assert_eq!(state.pending_effect_choice().unwrap().player, p(2));

    // 3. ...and p1 — whose answer is IN the bank, and who is not the outstanding
    //    entry's owner — concedes.
    let (state, _) = process_command(state, Command::Concede { player: p(1) })
        .expect("a concede is always admitted while blocked");

    assert!(
        state.effect_choice_answers().is_empty(),
        "CR 608.2d: the concede mutated the board, so every banked answer is \
         abandoned -- the surviving players are re-asked against the board as it \
         NOW is"
    );
    let re_asked = state
        .pending_effect_choice()
        .expect("the resolution is re-driven and p2 is re-asked");
    assert_eq!(re_asked.player, p(2), "p2 still owes an answer");
    assert_ne!(
        re_asked.choice_id, p2_choice_id,
        "the re-ask is a NEW moment: the stale choice_id must not be honoured"
    );
    assert_eq!(
        re_asked.index, 0,
        "with the bank dropped, p2's question is now the resolution's FIRST \
         choice -- a departed player is skipped without consuming a bank slot"
    );

    // 4. The rest of the resolution completes, with no panic and no stale answer.
    let mut state = state;
    let mut asked: Vec<PlayerId> = Vec::new();
    let mut guard = 0;
    while let Some(entry) = state.pending_effect_choice() {
        asked.push(entry.player);
        let candidates = match &entry.question {
            EffectChoiceQuestion::SearchLibrary { candidates, .. } => candidates.clone(),
            other => panic!("expected a search question, got {other:?}"),
        };
        let (s, _) = answer_with(
            state,
            EffectChoiceAnswer::SearchLibrary {
                found: Some(candidates[1]),
            },
        );
        state = s;
        guard += 1;
        assert!(guard < 8, "the per-player questions did not converge");
    }
    assert_eq!(
        asked,
        vec![p(2), p(3)],
        "only the SURVIVING players are asked; p1 takes the default without \
         being asked (CR 104.3a: a player who has left announces nothing)"
    );
    assert!(state.stack_objects().is_empty(), "the resolution completed");
    for name in ["P2 Beta", "P3 Beta"] {
        assert!(
            matches!(zone_of(&state, name), Some(ZoneId::Hand(_))),
            "{name}: the surviving player's own announcement is applied"
        );
    }
    // CR 800.4a vs CR 704.5a (closing-review LOW-4, seed OOS-DP9-18): this pins a
    // KNOWN DEVIATION, not correct behaviour. `ask_or_consume_effect_choice`
    // treats `has_lost || has_conceded` as "announces nothing, take the default",
    // but `resolve_player_target_list` filters only on `has_lost` -- so a
    // *conceded* player is still in the effect's player list and still has the
    // effect applied to them. CR 800.4a says all objects owned by a departed
    // player leave the game, so there should be no library left to search and no
    // hand to put a card into. The engine has no CR 800.4a object sweep at all
    // (it marks `has_conceded` and leaves the board alone), so this assertion
    // records what the engine does today.
    assert!(
        matches!(zone_of(&state, "P1 Alpha"), Some(ZoneId::Hand(_))),
        "DEVIATION (OOS-DP9-18): p1 has left the game, yet its library is still \
         searched and a card still lands in its hand -- with the DEFAULT pick, \
         not the answer it banked before conceding"
    );

    // ...and the hazardous state itself. p1 is the ACTIVE player of `fixture_3p`,
    // so this sequence is closing-review HIGH-1's: the resolution's CR 117.3b
    // tail used to grant priority to `turn.active_player` unconditionally, i.e.
    // to a seat that had already left the game (CR 800.4j).
    assert_recoverable(&state, "after an active-player concede under p2's block");
}

/// CR 800.4j / CR 104.3a — the three assertions every "somebody left the game"
/// test in this file owes: priority names a LIVE seat, and that seat can act.
///
/// Factored out because closing-review HIGH-1 was reachable only because
/// `test_dp9_foreign_concede_invalidates_a_non_empty_bank` built the exact
/// hazardous state and then asserted nothing about it (PB-DP8's transferable
/// rule (ii)).
fn assert_recoverable(state: &GameState, label: &str) {
    let holder = state.turn().priority_holder.unwrap_or_else(|| {
        panic!("{label}: a live player must hold priority -- this is the deadlock assertion")
    });
    assert!(
        state
            .players()
            .get(&holder)
            .map(|pl| !pl.has_lost && !pl.has_conceded)
            .unwrap_or(false),
        "{label}: CR 800.4j -- priority must not name a seat that has left the \
         game (holder {holder:?})"
    );
    process_command(state.clone(), Command::PassPriority { player: holder }).unwrap_or_else(|e| {
        panic!("{label}: the holder must be able to act -- this is the deadlock assertion: {e:?}")
    });
}

#[test]
/// CR 800.4j / CR 117.3b / CR 608.2d — the **ACTIVE** player concedes while a
/// FOREIGN seat's resolution-time choice is outstanding.
///
/// Closing-review HIGH-1, fail-before probe. The fourth appearance of the
/// "a guard that skips work inherits the obligation of what it skipped" class in
/// this suite; PB-DP7 and PB-DP8 each shipped it twice.
///
/// The sequence, all of it legal:
///
/// 1. p2's CR 608.2d question is outstanding, so `priority_holder` is `None` and
///    `players_passed` is full (the roll-back restored the resolving moment).
/// 2. p1 -- the ACTIVE player -- concedes. `Concede` is always admitted
///    (CR 104.3a). `discharge_effect_choice_on_concede` re-drives, p2 is re-asked,
///    so `blocking_decision` is `Some(EffectChoice)` again.
/// 3. `handle_concede`'s `blocking_decision(state).is_none()` gate therefore skips
///    its whole priority/turn block -- including `advance_turn` for p1's own turn.
/// 4. p2 (and p3) answer, the resolution completes, and
///    `resolve_top_of_stack_inner`'s CR 117.3b tail granted priority to
///    `turn.active_player` **unconditionally** -- i.e. to p1, who has left.
///
/// `blocking_decision` is then `None`, so `PassPriority` is *admitted* -- and
/// answers `PlayerEliminated` from p1 and `NotPriorityHolder` from everyone else.
/// Every driving loop (`LocalGame::advance`, `GameDriver`, the TUI auto-pass, the
/// fuzzer) dies there; only another `Concede` unsticks it.
///
/// The fix is CR 800.4j itself -- "If the active player would receive priority,
/// instead the next player in turn order receives priority" -- applied at the
/// grant, which is the same idiom `enter_step` has used at its two grant sites all
/// along. See `rules::priority::grant_priority_to_active_player`.
///
/// This probe uses an **empty** answer bank (p1 owns no creature card, so it is
/// never asked) to keep it disjoint from the bank-invalidation probe above.
fn test_dp9_active_player_concedes_under_a_foreign_block() {
    let def = spell_def(
        "Everyone Tutors",
        "dp9-everyone-tutors-active-concede",
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
    // p1 (the active player and the caster) has NO creature card, so its own
    // search finds no candidates and asks no question: the bank stays empty and
    // the first and only outstanding entry belongs to a FOREIGN seat.
    let state = fixture_3p(
        def,
        vec![
            library_creature(p(2), "P2 Alpha"),
            library_creature(p(2), "P2 Beta"),
            library_creature(p(3), "P3 Alpha"),
            library_creature(p(3), "P3 Beta"),
        ],
    );
    assert_eq!(state.turn().active_player, p(1), "sanity: p1 is active");
    let (state, _) = cast_and_resolve_3p(state, "Everyone Tutors");
    assert_eq!(
        state.pending_effect_choice().unwrap().player,
        p(2),
        "sanity: the block belongs to a seat OTHER than the active player"
    );
    assert!(
        state.effect_choice_answers().is_empty(),
        "sanity: the bank is empty, so this probe is disjoint from the \
         bank-invalidation one"
    );
    assert_eq!(
        state.turn().priority_holder,
        None,
        "sanity: nobody holds priority while a CR 608.2d choice is outstanding"
    );

    let (state, _) = process_command(state, Command::Concede { player: p(1) })
        .expect("CR 104.3a: a concede is always admitted");
    assert!(
        state.pending_effect_choice().is_some(),
        "the foreign block survives the concede (re-asked against the new board)"
    );

    // p2 and p3 answer to completion.
    let mut state = state;
    let mut guard = 0;
    while state.pending_effect_choice().is_some() {
        let (s, _) = answer_pending_effect_choice(state);
        state = s;
        guard += 1;
        assert!(guard < 8, "the per-player questions did not converge");
    }
    assert!(state.stack_objects().is_empty(), "the resolution completed");

    // FAIL-BEFORE: this is `Some(p(1))` on the pre-fix engine.
    assert_recoverable(&state, "after the resolution completed");

    // ...and the next step boundary must not re-strand it either: `enter_step`'s
    // grant has been CR 800.4j-aware since long before this batch, and this pins
    // that the skipped `advance_turn` (step 3 above) does not turn into a
    // deadlock one boundary out -- CR 800.4j: the turn continues without an
    // active player.
    let live: Vec<PlayerId> = state
        .players()
        .iter()
        .filter(|(_, pl)| !pl.has_lost && !pl.has_conceded)
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(live, vec![p(2), p(3)], "sanity: two seats remain");
    let step_before = state.turn().step;
    let (state, _) = pass_all(state, &live);
    assert_ne!(
        state.turn().step,
        step_before,
        "the step must advance once both live seats pass"
    );
    assert_recoverable(&state, "after the step boundary");
}

#[test]
/// CR 800.4j / CR 117.3b / CR 704.5a — the SAME defect as
/// `test_dp9_active_player_concedes_under_a_foreign_block`, reached with **no
/// CR 608.2d choice anywhere in the sequence**.
///
/// Closing-review HIGH-1 offered two candidate fixes: call
/// `abilities::repair_departed_priority_holder` at the tail of
/// `Command::AnswerEffectChoice`, or make the resolution's grant liveness-aware.
/// This probe is why the second one is the right one and the first would have
/// been a patch on one of three call sites.
///
/// `resolve_top_of_stack_inner` runs `sba::check_and_apply_sbas` a few lines
/// before it grants priority, so a resolution that kills the ACTIVE player
/// reaches the grant with `active.has_lost` already true. Here p1 -- the active
/// player and the caster -- resolves its own "you lose 99 life": CR 704.5a marks
/// it as having lost during the resolution, and the CR 117.3b tail then handed
/// priority straight back to it. No concede, no effect choice, no
/// `AnswerEffectChoice` command; nothing PB-DP9 added is on this path, which
/// makes it the evidence that the bug is the grant's and not the batch's.
///
/// Fail-before: `holder` is `Some(p(1))` on the pre-fix engine.
fn test_dp9_resolution_grant_skips_an_active_player_killed_by_an_sba() {
    let def = spell_def(
        "Terminal Introspection",
        "dp9-active-player-sba-death",
        Effect::LoseLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(99),
        },
    );
    let state = fixture_3p(def, vec![]);
    assert_eq!(state.turn().active_player, p(1), "sanity: p1 is active");
    let (state, _) = cast_and_resolve_3p(state, "Terminal Introspection");

    assert!(
        state
            .players()
            .get(&p(1))
            .map(|pl| pl.has_lost)
            .unwrap_or(false),
        "CR 704.5a: the active player lost during its own spell's resolution"
    );
    assert!(
        state.pending_effect_choice().is_none(),
        "sanity: this path has no CR 608.2d choice on it at all"
    );
    assert_recoverable(
        &state,
        "after an SBA killed the active player mid-resolution",
    );
}

#[test]
/// CR 701.23b / CR 701.23d — a `TargetFilter` field that is a runtime BOARD
/// property, not a card quality, must not buy the searcher a fail-to-find.
///
/// Closing-review LOW-5, fail-before probe. `may_fail_to_find` was
/// `*filter != TargetFilter::default()`, i.e. *any* non-default field counted as
/// a "stated quality" in CR 701.23b's sense. `controller`, `exclude_self`,
/// `is_token`, `is_nontoken`, `is_attacking` and `is_blocking` are not
/// characteristics of a card — a card in a library has no controller and is not a
/// token, an attacker or the source — and each is documented at its declaration
/// as invisible to `matches_filter()`. So setting one narrowed nothing and yet
/// flipped the search from CR 701.23d ("find as many as possible", declining is
/// ILLEGAL) to CR 701.23b (declining is legal), over the entire library.
///
/// Both halves are asserted, because the value of the fix is that the two
/// otherwise-identical filters now disagree:
///
///   * `TargetFilter { controller: You, .. }`  → quantity-only, MUST find.
///   * `TargetFilter { legendary: true, .. }`  → stated quality, MAY decline.
///
/// No `all_cards()` def sets one of the six on a `SearchLibrary` filter today
/// (`test_dp9_roster_enumeration` walks that corpus), so this is the only place
/// the narrowing is observable.
fn test_dp9_may_fail_to_find_ignores_non_quality_filter_axes() {
    use mtg_engine::cards::card_definition::TargetController;

    // Half 1: a non-quality axis. CR 701.23d — the decline must be REFUSED.
    let def = spell_def(
        "Controller Tutor",
        "dp9-controller-tutor",
        Effect::SearchLibrary {
            player: PlayerTarget::Controller,
            filter: TargetFilter {
                controller: TargetController::You,
                exclude_self: true,
                is_nontoken: true,
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
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Controller Tutor");
    match outstanding_question(&state) {
        EffectChoiceQuestion::SearchLibrary {
            candidates,
            may_fail_to_find,
        } => {
            assert_eq!(
                candidates.len(),
                2,
                "sanity: none of the three fields narrowed anything -- \
                 `matches_filter` cannot see any of them"
            );
            assert!(
                !may_fail_to_find,
                "CR 701.23d: `controller` / `exclude_self` / `is_nontoken` are \
                 board properties, not stated qualities, so this is still a \
                 quantity-only search"
            );
        }
        other => panic!("expected a search question, got {other:?}"),
    }
    let (asker, choice_id) = {
        let entry = state.pending_effect_choice().expect("a choice is pending");
        (entry.player, entry.choice_id)
    };
    let err = process_command(
        state,
        Command::AnswerEffectChoice {
            player: asker,
            choice_id,
            answer: EffectChoiceAnswer::SearchLibrary { found: None },
        },
    )
    .expect_err("CR 701.23d: declining must be rejected");
    assert!(
        format!("{err:?}").contains("701.23d"),
        "expected the CR 701.23d rejection, got {err:?}"
    );

    // Half 2: a real stated quality on the same shape. CR 701.23b — the decline
    // is LEGAL, so the narrowing has not simply disabled fail-to-find.
    let def = spell_def(
        "Legend Tutor",
        "dp9-legend-tutor",
        Effect::SearchLibrary {
            player: PlayerTarget::Controller,
            filter: TargetFilter {
                controller: TargetController::You,
                legendary: true,
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
            legendary_library_creature(p(1), "Legend One"),
            legendary_library_creature(p(1), "Legend Two"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Legend Tutor");
    match outstanding_question(&state) {
        EffectChoiceQuestion::SearchLibrary {
            may_fail_to_find, ..
        } => assert!(
            may_fail_to_find,
            "CR 701.23b: `legendary` IS a stated quality, so the decline stays legal"
        ),
        other => panic!("expected a search question, got {other:?}"),
    }
    let (state, _) = answer_with(state, EffectChoiceAnswer::SearchLibrary { found: None });
    assert!(
        state.stack_objects().is_empty(),
        "CR 701.23b: the decline resolves the search with nothing found"
    );
    assert_eq!(
        zone_of(&state, "Legend One"),
        Some(ZoneId::Library(p(1))),
        "nothing may move on a legal fail-to-find"
    );

    // Half 3: `is_tapped`, added by the second closing review (LOW-3). The
    // original exclusion list left it in on the stated grounds that the tapped
    // pair "*are* checked against library cards ... via `matches_filter`" and so
    // empty the candidate list. That was false: `matches_filter` takes a
    // `&Characteristics` and contains zero occurrences of either field, so it
    // cannot see tapped state -- the list came back UNNARROWED *and* carried a
    // CR 701.23d-forbidden decline over the whole library.
    let def = spell_def(
        "Tapped Tutor",
        "dp9-tapped-tutor",
        Effect::SearchLibrary {
            player: PlayerTarget::Controller,
            filter: TargetFilter {
                is_tapped: true,
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
            library_creature(p(1), "Alpha"),
            library_creature(p(1), "Beta"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Tapped Tutor");
    match outstanding_question(&state) {
        EffectChoiceQuestion::SearchLibrary {
            candidates,
            may_fail_to_find,
        } => {
            assert_eq!(
                candidates.len(),
                2,
                "the hazardous state itself: `is_tapped` narrows NOTHING, because \
                 `matches_filter` cannot see it -- both untapped library cards match"
            );
            assert!(
                !may_fail_to_find,
                "CR 701.23d: `is_tapped` is a runtime board property, not a stated \
                 quality, so this is still a quantity-only search and the decline \
                 must be illegal"
            );
        }
        other => panic!("expected a search question, got {other:?}"),
    }
    let (asker, choice_id) = {
        let entry = state.pending_effect_choice().expect("a choice is pending");
        (entry.player, entry.choice_id)
    };
    let err = process_command(
        state,
        Command::AnswerEffectChoice {
            player: asker,
            choice_id,
            answer: EffectChoiceAnswer::SearchLibrary { found: None },
        },
    )
    .expect_err("CR 701.23d: declining must be rejected on an `is_tapped` filter");
    assert!(
        format!("{err:?}").contains("701.23d"),
        "expected the CR 701.23d rejection, got {err:?}"
    );
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
    use mtg_card_types::cards::card_definition::Completeness;

    /// **PB-DP10 note (OOS-DP10-1):** this walk matches OBJECT KEYS ONLY, so it is blind
    /// to a UNIT `Effect` variant (`Effect::Proliferate`, `Effect::TheRingTemptsYou`
    /// serialize as bare JSON strings, not object keys). Harmless here -- `SearchLibrary`,
    /// `Scry` and `Surveil` are all struct variants, so this roster's own three targets are
    /// unaffected -- but a hazard the moment this function is reused for a unit-variant
    /// row. `crates/engine/tests/core/decision_site_walk.rs::json_contains_variant` is the
    /// canonical, unit-variant-aware version of this exact walk; it cannot be shared
    /// directly across the SR-9a integration-test-target boundary (a group `main.rs` may
    /// declare only bare `mod x;` lines, no `#[path]`), so this copy is kept, documented,
    /// and cross-checked BY VALUE against the canonical walk's own rosters
    /// (`decision_gate::canonical_walk_reproduces_pb_dp9_rosters`) rather than by text.
    ///
    /// Does this serialized subtree contain an externally-tagged enum variant
    /// named `variant`?
    ///
    /// # Why a serde walk and not a hand-written `match` (fix-cycle Finding 5)
    ///
    /// The first version of this roster walked the tree by hand: an
    /// `effect_contains` that matched `Sequence`/`Conditional`/`ForEach`/
    /// `Repeat`/`Choose`/`MayPay*`/`RollDice`, and an `ability_effects` that
    /// returned the single `effect` field of `Spell`/`Triggered`/`Activated`.
    /// Both were incomplete, and the review found the misses:
    /// `AbilityDefinition::{Spell,Triggered,Activated}::modes` — where every
    /// modal card's real effects live — was never walked at all, and
    /// `Effect::CoinFlip`'s two arms were missing while the doc comment claimed
    /// they were covered. Four `Complete` defs carrying a mode-nested
    /// `SearchLibrary` (`evolution_charm`, `insatiable_avarice`,
    /// `thirsting_roots`, `tooth_and_nail`) were silently absent, and the
    /// resulting counts were published into the audit as fact. That is exactly
    /// the defect SR-36 exists to prevent, wearing an enumeration's authority
    /// instead of a grep's.
    ///
    /// A hand-written match cannot be trusted here: `AbilityDefinition` has ~55
    /// variants and `Effect` has hundreds, and neither is compiler-forced in a
    /// `_ => {}`-shaped walk. Serializing the `CardDefinition` and searching the
    /// resulting tree is **structurally complete by construction** — it visits
    /// every field of every variant, at every nesting depth, and stays complete
    /// as the DSL grows. `card_definition.rs` carries no `#[serde(skip)]` or
    /// `skip_serializing*` attribute (checked), so nothing is hidden from it,
    /// and serde's default externally-tagged representation makes an enum
    /// variant an object key: `Effect::Scry { .. }` is `{"Scry": { .. }}`.
    /// Field names are lowercase, so a variant-name key cannot collide with one,
    /// and no other enum in the DSL has a `SearchLibrary`, `Scry` or `Surveil`
    /// variant (checked; `TriggerCondition::WheneverYouSurveil` is a different
    /// name).
    fn json_contains_variant(v: &serde_json::Value, variant: &str) -> bool {
        match v {
            serde_json::Value::Object(map) => map
                .iter()
                .any(|(k, child)| k == variant || json_contains_variant(child, variant)),
            serde_json::Value::Array(items) => {
                items.iter().any(|c| json_contains_variant(c, variant))
            }
            _ => false,
        }
    }

    /// [`json_contains_variant`] against any serializable DSL node.
    pub fn contains_variant<T: serde::Serialize>(node: &T, variant: &str) -> bool {
        let json = serde_json::to_value(node).expect("the card DSL is Serialize");
        json_contains_variant(&json, variant)
    }

    pub struct Roster {
        pub complete: Vec<String>,
        pub other_names: Vec<String>,
    }

    impl Roster {
        pub fn other(&self) -> usize {
            self.other_names.len()
        }
        pub fn contains(&self, name: &str) -> bool {
            self.complete.iter().any(|n| n == name) || self.other_names.iter().any(|n| n == name)
        }
    }

    /// Every def whose `CardDefinition` — including both faces, the adventure
    /// face, every `modes` list and every nested effect arm — carries `variant`.
    pub fn collect(variant: &str) -> Roster {
        let mut complete = Vec::new();
        let mut other_names = Vec::new();
        for def in all_cards() {
            if !contains_variant(&def, variant) {
                continue;
            }
            if def.completeness == Completeness::Complete {
                complete.push(def.name.clone());
            } else {
                other_names.push(def.name.clone());
            }
        }
        complete.sort();
        other_names.sort();
        Roster {
            complete,
            other_names,
        }
    }
}

#[test]
/// SR-36 — the PB-DP9 rosters, ENUMERATED from `all_cards()` rather than grepped.
///
/// The audit claimed 74 search / 16 scry / 8 surveil `Complete` defs; PB-DP9 as
/// shipped printed 69 / 16 / 7 and those went into the audit as fact. Both were
/// wrong: the shipped walk skipped `AbilityDefinition::*::modes` and
/// `Effect::CoinFlip` (fix-cycle Finding 5). These printed counts, from a
/// structurally complete serde walk, are the deliverable — they become the fact.
///
/// The assertions are `>=` on purpose — the authoring campaign adds cards
/// continuously and an `==` pin would redden on unrelated work. The four
/// mode-nested defs the old walk missed are pinned BY NAME, because "the walk
/// reaches inside `modes`" is the property this test now guards.
fn test_dp9_roster_enumeration() {
    let search = roster::collect("SearchLibrary");
    let scry = roster::collect("Scry");
    let surveil = roster::collect("Surveil");

    println!(
        "PB-DP9 roster (SR-36, enumerated from all_cards()):\n  \
         SearchLibrary: {} Complete (+{} non-Complete)\n  \
         Scry:          {} Complete (+{} non-Complete)\n  \
         Surveil:       {} Complete (+{} non-Complete)",
        search.complete.len(),
        search.other(),
        scry.complete.len(),
        scry.other(),
        surveil.complete.len(),
        surveil.other()
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

    // Fix-cycle Finding 5 regression guard: every one of these carries its
    // `Effect::SearchLibrary` inside `AbilityDefinition::Spell { modes }`, so a
    // walk that stops at the ability's own `effect` field misses all four.
    //
    // Three are `Complete`; `Tooth and Nail` is `partial` (its own note: "'up to
    // two' search -- SearchLibrary finds one card", OOS-DP9-3), so it lands in
    // `other_names`. The review recorded all four as `Complete` -- it was right
    // that all four were MISSING, wrong about that one's marker, which is why
    // this guard asserts roster membership rather than the `Complete` half.
    for name in [
        "Evolution Charm",
        "Insatiable Avarice",
        "Thirsting Roots",
        "Tooth and Nail",
    ] {
        assert!(
            search.contains(name),
            "{name} carries a mode-nested SearchLibrary and must be in the roster"
        );
    }
    for name in ["Evolution Charm", "Insatiable Avarice", "Thirsting Roots"] {
        assert!(
            search.complete.iter().any(|n| n == name),
            "{name} is Complete and must be in the Complete half of the roster"
        );
    }
}

#[test]
/// CR 605.1b / CR 605.4a — the mana-ability gate.
///
/// A mana ability resolves immediately and OUTSIDE the stack, so there is no
/// stack object to roll back to and PB-DP9's suspension cannot apply. The
/// `EffectContext::effect_choice_gate_closed` flag makes the four arms take
/// their deterministic default there instead.
///
/// That branch skips an obligation (offering the choice), so this test is where
/// the obligation is discharged: **no `Complete` card def puts one of the four
/// asking effects inside a mana ability.** If this ever reddens, the branch has
/// become live and the card needs a rules decision, not a silent default.
///
/// The behavioural half is asserted directly: with the gate closed the effect
/// applies the default and records NO entry.
fn test_dp9_mana_ability_gate() {
    // (a) The roster obligation.
    // `ManaAbility` carries no `Effect` tree of its own, so the ONLY route into
    // the gated path is a `WhenTappedForMana` TRIGGERED ability whose effect is
    // mana-producing -- `rules::mana.rs`'s CR 605.4a branch, which is the single
    // site that closes the gate. Scan those.
    //
    // The subtree search is the same structurally-complete serde walk the roster
    // uses (fix-cycle Finding 5): the old hand-written `effect_contains`
    // inherited the `modes` / `CoinFlip` gap here too, so a mana trigger nesting
    // a scry inside a coin flip would not have been found.
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
                    ..
                } = a
                {
                    if !matches!(
                        trigger_condition,
                        mtg_card_types::cards::card_definition::TriggerCondition::WhenTappedForMana { .. }
                    ) {
                        continue;
                    }
                    let asks = ["SearchLibrary", "Scry", "Surveil", "DiscardCards"]
                        .iter()
                        .any(|v| roster::contains_variant(a, v));
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
