//! PB-DX50 half 2 (`OOS-DX29-2`) — CR 702.140c makes the over/under choice a
//! **resolution** choice, and the engine took it at announcement.
//!
//! > CR 702.140c: *"As a mutating creature spell resolves, if its target is legal
//! > … **The spell's controller chooses whether the spell is put on top of the
//! > creature or on the bottom.**"*
//!
//! "As it resolves." Before this batch the value was captured at cast time and
//! rode in `AdditionalCost::Mutate { on_top }`, with two consequences that are
//! different from each other and both wrong:
//!
//! 1. the **opponent** learned the choice before deciding whether to respond
//!    (`Command::CastSpell` carries it, and `hash.rs` hashed it onto the stack
//!    object), and
//! 2. the **controller** could not change it after seeing the responses.
//!
//! CR 702.140e is what makes it load-bearing rather than cosmetic: the topmost
//! card supplies the merged permanent's name, mana cost, colours, types and
//! power/toughness.
//!
//! The choice now suspends onto PB-DP9's CR 608.2d channel as
//! `EffectChoiceQuestion::MutateOnTop { host }`, asked **inside the legal-target
//! branch only** (CR 702.140c's own "if its target is legal") and **before any
//! state mutation**.
//!
//! ## §8's trap, and why `t_trap_*` exists
//!
//! Adding a seventh `EffectChoiceQuestion` variant is **not compile-forced** at
//! two sites in `effects::handle_answer_effect_choice`:
//! `variants_agree` was a `matches!` over six hardcoded `(question, answer)`
//! pairs (an unlisted pair silently returns `false`, so every legal answer is
//! REJECTED), and the per-variant legality match below it ended in
//! `_ => unreachable!()` (so fixing only the first turns the rejection into a
//! release PANIC). Measured before the fix: the whole workspace compiled green
//! with both traps unrepaired, across **eight** genuinely compile-forced sites.
//! Both are now one exhaustive `match` on the pair; `t_trap_a`/`t_trap_b` are
//! the behavioural pins.

use mtg_engine::effects::default_effect_choice_answer;
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::state::types::AltCostKind;
use mtg_engine::AdditionalCost;
use mtg_engine::{
    process_command, AbilityDefinition, AnsweredEffectChoice, CardDefinition, CardId, CardRegistry,
    CardType, Color, Command, Effect, EffectAmount, EffectChoiceAnswer, EffectChoiceQuestion,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PendingEffectChoice, PlayerId, PlayerTarget, Step, SubType, TriggerCondition,
    TypeLine, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name}"))
}

const BEAST: &str = "DX50b Mutating Beast";
const HOST: &str = "DX50b Wolf Host";

/// `with_trigger` adds a CR 702.140d "whenever this creature mutates" ability that
/// gains 1 life -- untargeted, so nothing about target selection can confound the
/// COUNT `t7` measures.
fn beast_def_with_trigger(with_trigger: bool) -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx50b-mutating-beast".to_string()),
        name: BEAST.to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Beast".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Mutate {1}{G}{G}".to_string(),
        abilities: {
            let mut a = vec![
                AbilityDefinition::Keyword(KeywordAbility::Mutate),
                AbilityDefinition::MutateCost {
                    cost: ManaCost {
                        generic: 1,
                        green: 2,
                        ..Default::default()
                    },
                },
            ];
            if with_trigger {
                a.push(AbilityDefinition::Triggered {
                    once_per_turn: false,
                    trigger_condition: TriggerCondition::WhenMutates,
                    effect: Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    intervening_if: None,
                    targets: vec![],
                    modes: None,
                    trigger_zone: Default::default(),
                });
            }
            a
        },
        // P/T and NAME both differ from the host, so CR 702.140e's "the topmost card
        // supplies …" is observable in BOTH directions rather than only one.
        power: Some(4),
        toughness: Some(4),
        ..Default::default()
    }
}

fn host_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx50b-wolf-host".to_string()),
        name: HOST.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Wolf".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(3),
        ..Default::default()
    }
}

/// **Enriched from the `CardDefinition`, deliberately** (`memory/gotchas-infra.md`):
/// `ObjectSpec::card` mints a NAKED object, and `t7`'s CR 702.140d trigger has to reach
/// `Characteristics::triggered_abilities` for `check_triggers` to see it after the merge.
/// A hand-built spec measures nothing there -- and measures it GREEN, which is worse.
fn beast_spec(
    owner: PlayerId,
    defs: &std::collections::HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let mut s = mtg_engine::enrich_spec_from_def(base_beast_spec(owner), defs);
    s.power = Some(4);
    s.toughness = Some(4);
    s
}

fn base_beast_spec(owner: PlayerId) -> ObjectSpec {
    let mut s = ObjectSpec::card(owner, BEAST)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("dx50b-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_colors(vec![Color::Green])
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        });
    s.power = Some(4);
    s.toughness = Some(4);
    s
}

fn host_spec(owner: PlayerId) -> ObjectSpec {
    let mut s = ObjectSpec::card(owner, HOST)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("dx50b-wolf-host".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    s.power = Some(2);
    s.toughness = Some(3);
    s
}

/// Two seats, p1 holding the mutator and owning+controlling the host, with the mutate
/// cost affordable from the POOL (so no `mana_solver` planning is entangled with what
/// these probes measure).
fn board() -> GameState {
    board_with(false)
}

fn board_with(mutate_trigger: bool) -> GameState {
    let p1 = p(1);
    let p2 = p(2);
    let bdef = beast_def_with_trigger(mutate_trigger);
    let defs: std::collections::HashMap<String, CardDefinition> = [
        (BEAST.to_string(), bdef.clone()),
        (HOST.to_string(), host_def()),
    ]
    .into_iter()
    .collect();
    let registry = CardRegistry::new(vec![bdef, host_def()]);
    let beast = beast_spec(p1, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(beast)
        .object(host_spec(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("fixture builds");
    let pool = &mut state.players_mut().get_mut(&p1).unwrap().mana_pool;
    pool.add(ManaColor::Green, 4);
    pool.add(ManaColor::Colorless, 4);
    state.turn_mut().priority_holder = Some(p1);
    state
}

/// Cast the mutate spell (CR 702.140a). Returns the state and the host id.
fn cast_mutate() -> (GameState, ObjectId) {
    cast_mutate_on(board())
}

fn cast_mutate_on(state: GameState) -> (GameState, ObjectId) {
    let card = find_object(&state, BEAST);
    let host = find_object(&state, HOST);
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p(1),
            card,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate { target: host }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .expect("mutate cast is legal");
    (state, host)
}

/// Both seats pass, resolving the top of the stack (CR 117.4).
fn resolve_top(state: GameState) -> (GameState, Vec<GameEvent>) {
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

// ── t1: the ask happens at RESOLUTION, not at announcement ───────────────────

/// CR 702.140c -- "As a mutating creature spell resolves … the spell's controller
/// chooses whether the spell is put on top of the creature or on the bottom."
///
/// The whole batch in one assertion: after the CAST there is no pending choice
/// (the opponent gets priority knowing nothing), and after the RESOLUTION starts
/// there is one, naming the host.
#[test]
fn t1_the_over_under_choice_is_asked_at_resolution_and_not_at_announcement() {
    let (state, host) = cast_mutate();
    assert!(
        state.pending_effect_choice().is_none(),
        "CR 702.140c: announcing a mutate spell must not ask the over/under \
         question -- the opponent has priority and must not learn the choice yet"
    );
    // The stack object carries no over/under answer at all any more.
    let so = state
        .stack_objects()
        .last()
        .expect("the mutate spell is on the stack");
    assert!(
        matches!(so.kind, StackObjectKind::MutatingCreatureSpell { .. }),
        "fixture sanity: the cast produced a MutatingCreatureSpell"
    );
    let (state, _) = resolve_top(state);
    let pending = state
        .pending_effect_choice()
        .expect("CR 702.140c: resolution must ask the over/under question");
    assert_eq!(
        pending.question,
        EffectChoiceQuestion::MutateOnTop { host },
        "CR 702.140c: the question names the creature being mutated onto"
    );
    assert_eq!(
        pending.player,
        p(1),
        "CR 702.140c: the SPELL'S CONTROLLER chooses"
    );
}

/// CR 608.2d -- the suspend applies NOTHING. The spell is still on the stack and
/// the mutator's card has not moved.
#[test]
fn t2_the_suspended_resolution_applies_nothing() {
    let (state, _host) = cast_mutate();
    let mutator_card = state
        .stack_objects()
        .last()
        .and_then(|so| match so.kind {
            StackObjectKind::MutatingCreatureSpell { source_object, .. } => Some(source_object),
            _ => None,
        })
        .expect("mutate spell on the stack");
    let stack_len_before = state.stack_objects().len();
    let (state, _) = resolve_top(state);
    assert_eq!(
        state.stack_objects().len(),
        stack_len_before,
        "CR 608.2d: a suspended resolution leaves the spell on the stack"
    );
    let card = state
        .objects()
        .get(&mutator_card)
        .expect("card still exists");
    assert_eq!(
        card.zone,
        ZoneId::Stack,
        "CR 608.2d: nothing moved -- the mutating card is still in the stack zone"
    );
}

// ── t3/t4: BOTH answers reach a different, observable board (CR 702.140e) ────

fn answer_and_finish(state: GameState, on_top: bool) -> GameState {
    let choice_id = state
        .pending_effect_choice()
        .expect("a question is pending")
        .choice_id;
    let (state, _) = process_command(
        state,
        Command::AnswerEffectChoice {
            player: p(1),
            choice_id,
            answer: EffectChoiceAnswer::MutateOnTop { on_top },
        },
    )
    .expect("both answers are legal, always (CR 702.140c states no restriction)");
    state
}

/// CR 702.140e -- "the merged permanent has the name, mana cost, color, card
/// type, subtype and power/toughness of the topmost card."
#[test]
fn t3_answering_on_top_puts_the_mutator_on_top() {
    let (state, host) = cast_mutate();
    let (state, _) = resolve_top(state);
    let state = answer_and_finish(state, true);
    let merged = state.objects().get(&host).expect("merged permanent exists");
    assert_eq!(
        merged.characteristics.name, BEAST,
        "CR 702.140e: on top -- the mutator's name is the merged permanent's name"
    );
    assert_eq!(
        merged.characteristics.power,
        Some(4),
        "CR 702.140e: on top -- the mutator's P/T"
    );
}

/// CR 702.140e, the OTHER answer -- and this is the one the pre-batch engine
/// could not reach at resolution time from any channel.
#[test]
fn t4_answering_under_keeps_the_hosts_characteristics() {
    let (state, host) = cast_mutate();
    let (state, _) = resolve_top(state);
    let state = answer_and_finish(state, false);
    let merged = state.objects().get(&host).expect("merged permanent exists");
    assert_eq!(
        merged.characteristics.name, HOST,
        "CR 702.140e: under -- the HOST's name survives on the merged permanent"
    );
    assert_eq!(
        merged.characteristics.power,
        Some(2),
        "CR 702.140e: under -- the HOST's P/T"
    );
}

/// The two answers must reach DIFFERENT boards, or `t3`/`t4` are two spellings of
/// one measurement. Asserted as a pair, on one fixture, so a regression that
/// collapses the choice to a constant reddens here even if both halves above
/// somehow agreed.
#[test]
fn t5_the_two_answers_are_observably_different() {
    let (a, host) = cast_mutate();
    let (a, _) = resolve_top(a);
    let a = answer_and_finish(a, true);
    let (b, _) = cast_mutate();
    let (b, _) = resolve_top(b);
    let b = answer_and_finish(b, false);
    assert_ne!(
        a.objects().get(&host).unwrap().characteristics.name,
        b.objects().get(&host).unwrap().characteristics.name,
        "CR 702.140c/e: the choice must be observable, or it is not a choice"
    );
}

// ── t6: the default recovers the pre-batch value ─────────────────────────────

/// PB-DX50 §5: `default_effect_choice_answer` returns the exact pre-batch
/// hard-coded value, which is what keeps every bot game, every recorded fuzz seed
/// and `combat/192_mutate_gemrazer.json` behaviourally identical while only the
/// COMMAND TRACE grows.
#[test]
fn t6_the_default_answer_is_on_top() {
    assert_eq!(
        default_effect_choice_answer(&EffectChoiceQuestion::MutateOnTop { host: ObjectId(1) }),
        EffectChoiceAnswer::MutateOnTop { on_top: true },
        "the default must reproduce the pre-batch `on_top: true`, or every seeded \
         fixture in the tree re-deals"
    );
}

// ── t7/t8: §8's two traps ────────────────────────────────────────────────────

/// Seed a bare `MutateOnTop` question with no resolution behind it, so the probe
/// measures `handle_answer_effect_choice`'s check 4 / check 5 and nothing else.
fn state_with_bare_question() -> GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .pending_effect_choice(PendingEffectChoice {
            choice_id: 77,
            player: p(1),
            source: ObjectId(1),
            question: EffectChoiceQuestion::MutateOnTop { host: ObjectId(2) },
            index: 0,
        })
        .build()
        .expect("fixture builds")
}

/// §8 trap 1 -- `variants_agree`. Before the fix this was a `matches!` over six
/// hardcoded pairs, so a `MutateOnTop` answer to a `MutateOnTop` question was
/// rejected with *"answer … does not answer question …"*: a clean offer followed
/// by a guaranteed refusal, the SR-38 shape from the engine side.
///
/// **Measured, not argued**: with the variant added and `variants_agree`
/// untouched, `cargo check --workspace --all-targets` was GREEN.
#[test]
fn t_trap_a_a_matching_mutate_answer_is_accepted() {
    let state = state_with_bare_question();
    let r = process_command(
        state,
        Command::AnswerEffectChoice {
            player: p(1),
            choice_id: 77,
            answer: EffectChoiceAnswer::MutateOnTop { on_top: false },
        },
    );
    // The resolution behind this bare question is empty (nothing on the stack),
    // so the command may still fail LATER -- what must not happen is a
    // variant-agreement rejection.
    if let Err(e) = &r {
        let msg = format!("{e:?}");
        assert!(
            !msg.contains("does not answer question"),
            "PB-DX50 §8 trap 1: a MutateOnTop answer to a MutateOnTop question \
             must pass `variants_agree`, got {msg}"
        );
    }
}

/// §8 trap 2 -- the per-variant legality match's `_ => unreachable!()` tail. A
/// batch that fixed trap 1 and not trap 2 would turn the rejection into a
/// **release panic**. There is no wildcard tail any more, so this arm is reached
/// by an exhaustive pattern; the probe is the behavioural witness that no panic
/// occurs on the path check 4 now admits.
#[test]
fn t_trap_b_answering_does_not_panic() {
    let state = state_with_bare_question();
    // Deliberately NOT `.expect(..)`: the point is that the call RETURNS.
    let _ = process_command(
        state,
        Command::AnswerEffectChoice {
            player: p(1),
            choice_id: 77,
            answer: EffectChoiceAnswer::MutateOnTop { on_top: true },
        },
    );
}

/// The genuine wrong-question case still rejects, and rejects for the right
/// reason. Without this, `t_trap_a` alone is satisfied by deleting check 4.
#[test]
fn t_trap_c_a_wrong_answer_to_a_mutate_question_is_still_rejected() {
    let state = state_with_bare_question();
    let err = process_command(
        state,
        Command::AnswerEffectChoice {
            player: p(1),
            choice_id: 77,
            answer: EffectChoiceAnswer::PayOptionalCost { pay: true },
        },
    )
    .expect_err("CR 608.2d: a PayOptionalCost answer does not answer a MutateOnTop question");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("does not answer question"),
        "the wrong-question diagnosis must survive the structural fix, got {msg}"
    );
}

/// The inverse: a `MutateOnTop` answer to some OTHER question is rejected too.
/// Pins that the exhaustive pair-match did not accidentally admit the variant
/// against every question.
#[test]
fn t_trap_d_a_mutate_answer_to_another_question_is_rejected() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .pending_effect_choice(PendingEffectChoice {
            choice_id: 78,
            player: p(1),
            source: ObjectId(1),
            question: EffectChoiceQuestion::Scry {
                looked_at: Vec::new(),
            },
            index: 0,
        })
        .build()
        .expect("fixture builds");
    let err = process_command(
        state,
        Command::AnswerEffectChoice {
            player: p(1),
            choice_id: 78,
            answer: EffectChoiceAnswer::MutateOnTop { on_top: true },
        },
    )
    .expect_err("CR 608.2d: a MutateOnTop answer does not answer a Scry question");
    assert!(
        format!("{err:?}").contains("does not answer question"),
        "the wrong-question diagnosis must fire in both directions"
    );
}

/// The banked answer round-trips through `AnsweredEffectChoice`, which is what
/// `ask_or_consume_effect_choice`'s structural question-equality check consumes
/// on the replay. A `MutateOnTop` question that did not compare equal to itself
/// would livelock the replay.
#[test]
fn t9_the_question_compares_equal_to_itself_across_a_clone() {
    let q = EffectChoiceQuestion::MutateOnTop {
        host: ObjectId(1234),
    };
    let banked = AnsweredEffectChoice {
        question: q.clone(),
        answer: EffectChoiceAnswer::MutateOnTop { on_top: false },
    };
    assert_eq!(
        banked.question, q,
        "CR 608.2d: the replay's question-equality check compares this structurally"
    );
    assert_ne!(
        q,
        EffectChoiceQuestion::MutateOnTop { host: ObjectId(1) },
        "two different hosts must be two different questions"
    );
}

// ── t7 — OOS-DX50-1: the answer must not double-dispatch the resolution's triggers ──

/// CR 603.3 (`OOS-DX50-1`) — a trigger produced by the REPLAYED resolution goes on the
/// stack **exactly once**.
///
/// **This is a PRE-EXISTING engine defect that PB-DX50 is merely the first batch to
/// reach.** `Command::AnswerEffectChoice` used to call `check_and_flush_triggers` over the
/// events `handle_answer_effect_choice` returned — but those events came straight out of
/// `resolve_top_of_stack`, whose own tail already runs `check_triggers_with_timing` +
/// `check_and_apply_sbas` + `flush_pending_triggers` over the identical slice. Two sweeps,
/// one event slice, two copies of every trigger.
///
/// `handle_all_passed`, the ordinary CR 608.1 path, calls `resolve_top_of_stack` and then
/// calls nothing, which is what makes the extra sweep provably redundant rather than
/// arguably so.
///
/// It was invisible before this batch only because no suspending resolution in the tree
/// had a fixture whose replay emitted a trigger-producing event. Any of them can:
/// ENG-1's `Effect::DiscardCards` feeding a madness or "whenever you discard" trigger is
/// the same shape. Found by `combat/192_mutate_gemrazer.json` putting TWO copies of
/// Gemrazer's `WhenMutates` trigger on the stack.
///
/// **The count is `== 1`, never `>= 1`**: PB-DX48's own headline was that a `>= 1`
/// assertion passes on the double-dispatch design.
///
/// **Revert to watch red**: restore the `check_and_flush_triggers` call (under its
/// `blocking_decision(&state).is_none()` guard) in `rules/engine.rs`'s
/// `Command::AnswerEffectChoice` arm.
#[test]
fn t7_answering_queues_each_trigger_exactly_once() {
    let (state, _host) = cast_mutate_on(board_with(true));
    let (state, _) = resolve_top(state);
    let pending = state
        .pending_effect_choice()
        .cloned()
        .expect("CR 702.140c: the question is pending");
    let (state, events) = process_command(
        state,
        Command::AnswerEffectChoice {
            player: p(1),
            choice_id: pending.choice_id,
            answer: EffectChoiceAnswer::MutateOnTop { on_top: true },
        },
    )
    .expect("the answer is legal");

    // Non-vacuity: the merge really did happen and really did emit CR 702.140d's event,
    // so a zero here would be a broken fixture rather than a fixed defect.
    let mutated = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureMutated { .. }))
        .count();
    assert_eq!(
        mutated, 1,
        "precondition (CR 702.140d): the replayed resolution merges exactly once"
    );

    let stack_triggers = state
        .stack_objects()
        .iter()
        .filter(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { .. }))
        .count();
    assert_eq!(
        stack_triggers,
        1,
        "CR 603.3 (`OOS-DX50-1`): the CR 702.140d trigger goes on the stack ONCE. Two \
         means `Command::AnswerEffectChoice` swept the same event slice that \
         `resolve_top_of_stack`'s own tail already swept. Stack: {:?}",
        state
            .stack_objects()
            .iter()
            .map(|so| format!("{:?}", so.kind))
            .collect::<Vec<_>>()
    );
}
