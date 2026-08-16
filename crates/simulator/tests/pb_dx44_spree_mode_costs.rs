//! PB-DX44 (`OOS-DX29-14`) — Spree per-mode costs are now visible to the auto-tap.
//!
//! # What was wrong
//!
//! `crates/engine/src/rules/casting.rs` charges `ModeSelection.mode_costs` once per
//! chosen mode for a `KeywordAbility::Spree` spell — that arithmetic is correct and was
//! never in question. `legal_actions::effective_cast_cost_with_additional` — the
//! function `LocalGame::auto_tap_commands_for` asks how much mana to tap — modelled NO
//! mode costs at all, so `insatiable_avarice` (the ONLY deck-legal `Complete` Spree def)
//! had its base `{B}` tapped and the cast refused with `InsufficientMana`, from BOTH the
//! browser (a human announcing modes) and the bot path (`spell_default_modes` falls back
//! to `[0]`). Fixed by folding `ModeSelection.mode_costs` into
//! `effective_cast_cost_with_additional`, keyed on a new `modes_chosen: &[usize]`
//! parameter, so the auto-tap and `casting.rs` are ONE arithmetic.
//!
//! # CR index
//!
//! CR 700.2h / 702.172a (Spree per-mode costs), CR 601.2f-h (announcing/paying the total
//! cost), CR 118.8d (additional costs don't change mana VALUE).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, CardDefinition, CardId, CardRegistry,
    Command, GameEvent, GameState, GameStateBuilder, ManaCost, ManaPool, ObjectId, ObjectSpec,
    PlayerId, Target, ZoneId,
};
use mtg_simulator::{
    effective_cast_cost, effective_cast_cost_with_additional, ActionParams, AdvanceOutcome, Bot,
    HeuristicBot, HumanChoice, LegalAction, LocalGame, LocalGameLimits, PendingDecision,
    StubProvider,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// ── Fixture plumbing (mirrors `pb_dx29_cost_kind_surface.rs`'s conventions) ─────────

fn defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn corpus_registry() -> std::sync::Arc<CardRegistry> {
    CardRegistry::new(all_cards())
}

fn corpus_card_id(defs: &HashMap<String, CardDefinition>, name: &str) -> CardId {
    defs.get(name)
        .unwrap_or_else(|| panic!("{name:?} is not in `all_cards()`"))
        .card_id
        .clone()
}

/// A real corpus card, fully enriched, in `zone`. See
/// `pb_dx29_cost_kind_surface.rs::corpus_object`'s doc for why `enrich_spec_from_def`
/// and the by-`card_id` lookup (not `card_name_to_id`) are both load-bearing.
fn corpus_object(
    defs: &HashMap<String, CardDefinition>,
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .with_card_id(corpus_card_id(defs, name))
            .in_zone(zone),
        defs,
    )
}

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?}"))
}

/// `Insatiable Avarice` in `P1`'s hand, `n` untapped Swamps on `P1`'s battlefield (a
/// real mana source, so `auto_tap: true` funds the cast fresh at cast time rather than
/// relying on a pre-game mana pool surviving `start_game`'s turn reset), and a small
/// library for both players so no draw step empties an actual library (CR 104.3c is not
/// this file's subject).
fn spree_state(n_swamps: u32) -> GameState {
    let defs = defs_by_name();
    let mut builder = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .object(corpus_object(
            &defs,
            P1,
            "Insatiable Avarice",
            ZoneId::Hand(P1),
        ));
    for _ in 0..n_swamps {
        // Deliberately NOT renamed: `enrich_spec_from_def` looks up the def by the
        // spec's OWN `name` field (`defs.get(&spec.name)`), so a copy renamed before
        // enrichment would silently fail to enrich (no mana ability, no card types) --
        // a fourth instance of the "`ObjectSpec::card()` is naked" gotcha class. Every
        // Swamp stays named "Swamp"; this file never needs to address one individually.
        builder = builder.object(corpus_object(&defs, P1, "Swamp", ZoneId::Battlefield));
    }
    for i in 0..5 {
        builder = builder
            .object(ObjectSpec::card(P1, &format!("P1 Filler {i}")).in_zone(ZoneId::Library(P1)));
        builder = builder
            .object(ObjectSpec::card(P2, &format!("P2 Filler {i}")).in_zone(ZoneId::Library(P2)));
    }
    builder.build().expect("state builds")
}

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 3,
        max_commands: 2000,
        max_consecutive_passes: 500,
        record_journal: true,
    }
}

fn expect_decision(game: &mut LocalGame<StubProvider>) -> PendingDecision {
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {other:?}"),
    }
}

fn index_of(actions: &[LegalAction], pred: impl Fn(&LegalAction) -> bool) -> usize {
    actions
        .iter()
        .position(pred)
        .unwrap_or_else(|| panic!("no matching action in {actions:?}"))
}

/// Pass priority (as `P1`) until a decision offers an action matching `pred`, or give
/// up. `start_game` resets the turn to `Step::Untap` (`reset_turn_state`), so a fixture
/// cannot begin in a main phase — `StubProvider` only offers `CastSpell` for a sorcery
/// in a main phase with an empty stack (CR 307.1). Walking there through real priority
/// passes is the honest way to reach it (mirrors
/// `local_game_human_actions.rs::drive_until`, duplicated rather than shared across
/// test binaries per that file's own precedent).
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    pred: impl Fn(&LegalAction) -> bool,
) -> PendingDecision {
    for _ in 0..200 {
        let decision = expect_decision(game);
        if decision.actions.iter().any(&pred) {
            return decision;
        }
        let pass = index_of(&decision.actions, |a| {
            matches!(a, LegalAction::PassPriority)
        });
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: pass,
                params: ActionParams::default(),
            },
        )
        .expect("passing priority is always legal at a priority window");
    }
    panic!("no decision offered a matching action within 200 priority windows");
}

fn is_insatiable_avarice_cast(a: &LegalAction) -> bool {
    matches!(a, LegalAction::CastSpell { .. })
}

// ═══════════════════════════════════════════════════════════════════════════════
// P — the cost PREDICTION (`effective_cast_cost_with_additional`)
// ═══════════════════════════════════════════════════════════════════════════════

/// **P1** — the prediction, for every one of Insatiable Avarice's three reachable mode
/// combinations, matches what `casting.rs` actually charges: base `{B}` (mana value 1)
/// plus mode 0's `{2}`, mode 1's `{B}{B}`, or both.
#[test]
fn p1_predicted_cost_matches_casting_rs_for_every_mode_combination() {
    let state = spree_state(0);
    let card = id_of(&state, "Insatiable Avarice");
    let base = effective_cast_cost(&state, P1, card).expect("has a mana cost");
    assert_eq!(base.mana_value(), 1, "base cost is {{B}}");

    let mode0 = effective_cast_cost_with_additional(&state, P1, card, &[], &[0], None)
        .expect("mode 0 predicted");
    assert_eq!(
        mode0,
        ManaCost {
            black: 1,
            generic: 2,
            ..Default::default()
        },
        "{{B}} base + {{2}} for mode 0"
    );

    let mode1 = effective_cast_cost_with_additional(&state, P1, card, &[], &[1], None)
        .expect("mode 1 predicted");
    assert_eq!(
        mode1,
        ManaCost {
            black: 3,
            ..Default::default()
        },
        "{{B}} base + {{B}}{{B}} for mode 1"
    );

    let both = effective_cast_cost_with_additional(&state, P1, card, &[], &[0, 1], None)
        .expect("both modes predicted");
    assert_eq!(
        both,
        ManaCost {
            black: 3,
            generic: 2,
            ..Default::default()
        },
        "{{B}} base + {{2}} + {{B}}{{B}} for both modes"
    );

    // Non-vacuity: the un-modal identity path (no modes announced) is untouched.
    let none = effective_cast_cost_with_additional(&state, P1, card, &[], &[], None)
        .expect("identity predicted");
    assert_eq!(none, base, "no announced modes must be the identity");
}

/// **P3** — the Entwine-overrides-`modes_chosen` mirror (CR 702.42a / `casting.rs`'s own
/// Spree arm, `indices_to_charge = if entwine_paid { 0..costs.len() } else {
/// modes_chosen }`), pinned with a SYNTHETIC def.
///
/// **Disclosed syntheticity**: no corpus def carries BOTH `KeywordAbility::Spree` and
/// `KeywordAbility::Entwine` — checked by grep (`grep -rl KeywordAbility::Spree
/// crates/card-defs/src/defs/ | xargs grep -l KeywordAbility::Entwine` returns nothing)
/// — so this property is UNREACHABLE from any real cast today. `casting.rs`'s own
/// generic arithmetic does not gate the combination out (both keywords can coexist on
/// one `AbilityDefinition::Spell` in principle, and its Spree arm reads `entwine_paid`
/// unconditionally, CR-wise a spell simply cannot print both mechanics), so this pins
/// the mirror against `casting.rs`'s actual code shape rather than against a rule that
/// will ever fire on a shipped card.
#[test]
fn p3_entwine_overrides_modes_chosen_and_charges_every_mode_synthetic() {
    use mtg_engine::cards::card_definition::ModeSelection;
    use mtg_engine::state::CardType;
    use mtg_engine::{AbilityDefinition, AdditionalCost, CardDefinition, Effect, KeywordAbility};

    let def = CardDefinition {
        card_id: CardId("pb-dx44-synthetic-spree-entwine".to_string()),
        name: "PB-DX44 Synthetic Spree Entwine".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Spree),
            AbilityDefinition::Keyword(KeywordAbility::Entwine),
            // A real Charm-style Entwine spell prints its own additional cost for the
            // "cast for both/all modes" announcement (CR 702.42a); zero here because
            // this synthetic def's whole point is the SPREE arm's mode-cost mirror, not
            // Entwine's own cost. `AdditionalCost::Entwine` still needs SOME
            // `AbilityDefinition::Entwine { cost }` to exist, or
            // `effective_cast_cost_with_additional`'s pre-existing `entwine_paid` arm
            // (unrelated to this test's subject) returns `None` via its own `?` before
            // the Spree block below is ever reached.
            AbilityDefinition::Entwine {
                cost: ManaCost::default(),
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 2,
                    allow_duplicate_modes: false,
                    mode_costs: Some(vec![
                        ManaCost {
                            generic: 2,
                            ..Default::default()
                        },
                        ManaCost {
                            black: 2,
                            ..Default::default()
                        },
                    ]),
                    modes: vec![Effect::Nothing, Effect::Nothing],
                    mode_targets: None,
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![def.clone()]);
    let defs_map: HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(registry)
        .active_player(P1)
        .object(enrich_spec_from_def(
            ObjectSpec::card(P1, &def.name)
                .with_card_id(def.card_id.clone())
                .in_zone(ZoneId::Hand(P1)),
            &defs_map,
        ))
        .build()
        .expect("state builds");
    let card = id_of(&state, &def.name);

    // Entwine paid, modes_chosen EMPTY: every mode's cost must still be charged.
    let with_entwine = effective_cast_cost_with_additional(
        &state,
        P1,
        card,
        &[AdditionalCost::Entwine],
        &[],
        None,
    )
    .expect("entwine + spree predicted");
    assert_eq!(
        with_entwine,
        ManaCost {
            black: 1 + 2, // base {B} + mode 1's {B}{B}
            generic: 2,   // mode 0's {2}
            ..Default::default()
        },
        "CR 702.42a: entwine charges EVERY mode's Spree cost, ignoring `modes_chosen`"
    );

    // Without entwine, the SAME empty `modes_chosen` charges nothing extra (identity) —
    // the contrast that proves the override is really `entwine_paid`-gated and not a
    // permanent "charge everything" bug.
    let without_entwine = effective_cast_cost_with_additional(&state, P1, card, &[], &[], None)
        .expect("spree with no announcement predicted");
    assert_eq!(
        without_entwine,
        ManaCost {
            black: 1,
            ..Default::default()
        },
        "without entwine and with no modes announced, only the base cost is charged"
    );
}

/// **P2** — prediction-vs-charge parity, in the style of
/// `pb_dx29_cost_kind_surface.rs`'s `assert_prediction_is_exactly_what_the_engine_charges`
/// (lines 1246-1298 at the time this test was written): the SAME cast is driven through
/// the real engine twice — once from a pool holding exactly the predicted mana (must be
/// ACCEPTED) and once one mana short (must be REFUSED with `InsufficientMana`). Proves
/// the two arithmetics agree by EXECUTION, not by re-reading `casting.rs`.
#[test]
fn p2_prediction_vs_charge_parity_for_mode_1() {
    let defs = defs_by_name();
    let build = |pool: ManaPool| {
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Insatiable Avarice",
                ZoneId::Hand(P1),
            ))
            .build()
            .expect("state builds")
    };
    let probe = build(ManaPool::default());
    let card = id_of(&probe, "Insatiable Avarice");
    let predicted = effective_cast_cost_with_additional(&probe, P1, card, &[], &[1], None)
        .expect("mode 1 predicted");
    assert_eq!(predicted.mana_value(), 3);

    let exact_pool = ManaPool {
        black: predicted.black,
        colorless: predicted.generic,
        ..Default::default()
    };
    let exact = build(exact_pool.clone());
    let card = id_of(&exact, "Insatiable Avarice");
    let accepted = cast(exact, P1, card, vec![Target::Player(P2)], vec![1], vec![]);
    assert!(
        accepted.is_ok(),
        "engine refused a cast funded with exactly the predicted {predicted:?}: {:?}",
        accepted.err()
    );

    let mut short_pool = exact_pool;
    // exact_pool is all-black (predicted.generic == 0 for mode 1: {B} + {B}{B} =
    // {B}{B}{B}, no generic component) so take one black mana away.
    assert_eq!(
        short_pool.colorless, 0,
        "sanity: mode 1's cost is pure black"
    );
    short_pool.black -= 1;
    let short = build(short_pool);
    let card = id_of(&short, "Insatiable Avarice");
    let refused = cast(short, P1, card, vec![Target::Player(P2)], vec![1], vec![]);
    assert!(
        matches!(refused, Err(mtg_engine::GameStateError::InsufficientMana)),
        "one mana short of {predicted:?} must be refused for INSUFFICIENT MANA; got {:?}",
        refused.map(|_| "ACCEPTED")
    );
}

/// Cast `Insatiable Avarice` directly through `process_command`, mirroring
/// `pb_dx29_cost_kind_surface.rs::cast`.
fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
    additional_costs: Vec<mtg_engine::AdditionalCost>,
) -> Result<(GameState, Vec<GameEvent>), mtg_engine::GameStateError> {
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen,
            x_value: 0,
            face_down_kind: None,
            additional_costs,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// E — END TO END, through the real human channel (`LocalGame`/`HumanChoice`)
// ═══════════════════════════════════════════════════════════════════════════════

/// **E1** — the decisive probe. A human announces mode `[1]` alone (a NON-DEFAULT
/// selection — `spell_default_modes` would pick `[0]`) through
/// `LocalGame::submit`/`ActionParams { modes_chosen: vec![1], .. }`, with `auto_tap:
/// true` funding the cast from real Swamps. Asserted by the RESOLUTION EFFECT, not the
/// offer: mode 1 makes the target player draw 3 cards and lose 3 life (CR 702.172a).
///
/// Before this batch's fix, `auto_tap_commands_for` priced only the base `{B}` here, the
/// solver tapped one Swamp, and the engine refused the cast with `InsufficientMana` --
/// this test's first draft (run against a deliberately-reverted `local_game.rs`, see the
/// PB-DX44 execution notes for the revert record) reproduced exactly that.
#[test]
fn e1_human_channel_cast_with_mode_1_alone_resolves_its_effect() {
    let state = spree_state(3);
    // Sanity: enough black mana for {B} + {B}{B} = {B}{B}{B}, exactly.
    assert_eq!(
        state
            .objects()
            .values()
            .filter(|o| o.characteristics.name.starts_with("Swamp"))
            .count(),
        3
    );
    let human_seats: BTreeSet<PlayerId> = [P1].into_iter().collect();
    let (mut game, _events) = LocalGame::start(
        state,
        1,
        StubProvider,
        HashMap::new(),
        human_seats,
        limits(),
        true,
    )
    .expect("game starts");

    let decision = drive_until(&mut game, is_insatiable_avarice_cast);
    let card_index = index_of(&decision.actions, is_insatiable_avarice_cast);

    let p2_hand_before = game.state().objects_in_zone(&ZoneId::Hand(P2)).len();
    let p2_life_before = game.state().player(P2).expect("p2 exists").life_total;

    let events = game
        .submit(
            decision.seq,
            HumanChoice {
                action_index: card_index,
                params: ActionParams {
                    modes_chosen: vec![1],
                    targets: vec![Target::Player(P2)],
                    auto_tap: true,
                    ..Default::default()
                },
            },
        )
        .unwrap_or_else(|e| {
            panic!(
                "mode-1 cast must be accepted (funded {:?} exactly): {e:?}",
                e
            )
        });
    assert!(
        !events.is_empty(),
        "a real command sequence (taps + cast) must produce events"
    );

    // The spell must actually be castable to a stack object with modes_chosen == [1] --
    // non-vacuity, checked before draining the stack, so a silently-empty modes list
    // cannot make the resolution assertion below pass for the wrong reason.
    let on_stack = game
        .state()
        .stack_objects()
        .iter()
        .find(|so| matches!(&so.kind, mtg_engine::StackObjectKind::Spell { .. }))
        .expect("the spell must be on the stack after a successful cast");
    assert_eq!(
        on_stack.modes_chosen,
        vec![1],
        "the stack object must record exactly the announced mode"
    );

    // Drain the stack (both players pass) so the spell resolves.
    for _ in 0..10 {
        if game.state().stack_objects().is_empty() {
            break;
        }
        let decision = expect_decision(&mut game);
        let pass = index_of(&decision.actions, |a| {
            matches!(a, LegalAction::PassPriority)
        });
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: pass,
                params: ActionParams::default(),
            },
        )
        .expect("passing to resolve the spell is always legal");
    }
    assert!(
        game.state().stack_objects().is_empty(),
        "the spell must have resolved"
    );

    let p2_hand_after = game.state().objects_in_zone(&ZoneId::Hand(P2)).len();
    let p2_life_after = game.state().player(P2).expect("p2 exists").life_total;
    assert_eq!(
        p2_hand_after,
        p2_hand_before + 3,
        "CR 702.172a mode 1: target player draws 3 cards"
    );
    assert_eq!(
        p2_life_after,
        p2_life_before - 3,
        "CR 702.172a mode 1: target player loses 3 life"
    );
}

/// **E2** — the "separately [0,1]" half of the decisive probe: both modes chosen at
/// once. Asserted the same way as E1 (the resolution effect, not the offer) for mode
/// 1's half, PLUS proof that the total charged really is base + BOTH mode costs (not
/// just one) by construction: the pool holds `{B}{B}{B}` + `{2}` and nothing more, so an
/// under-prediction (charging only one mode) would leave 2 generic mana stranded and an
/// over-prediction would make the cast unaffordable and this whole test fail at
/// `submit`.
#[test]
fn e2_human_channel_cast_with_both_modes_is_accepted_and_resolves_mode_1() {
    let state = spree_state(5); // 5 Swamps: 3 for the black pips, 2 spare for {2} generic
    let human_seats: BTreeSet<PlayerId> = [P1].into_iter().collect();
    let (mut game, _events) = LocalGame::start(
        state,
        2,
        StubProvider,
        HashMap::new(),
        human_seats,
        limits(),
        true,
    )
    .expect("game starts");

    let decision = drive_until(&mut game, is_insatiable_avarice_cast);
    let card_index = index_of(&decision.actions, is_insatiable_avarice_cast);

    let p2_hand_before = game.state().objects_in_zone(&ZoneId::Hand(P2)).len();
    let p2_life_before = game.state().player(P2).expect("p2 exists").life_total;
    let p1_lands_untapped_before = game
        .state()
        .objects_in_zone(&ZoneId::Battlefield)
        .into_iter()
        .filter(|o| o.controller == P1 && !o.status.tapped)
        .count();
    assert_eq!(p1_lands_untapped_before, 5, "sanity: 5 untapped Swamps");

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: card_index,
            params: ActionParams {
                modes_chosen: vec![0, 1],
                targets: vec![Target::Player(P2)],
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .unwrap_or_else(|e| panic!("both-modes cast must be accepted: {e:?}"));

    // CR 700.2h: base {B} + mode 0 {2} + mode 1 {B}{B} = mana value 5. Exactly 5 of the
    // 5 Swamps must now be tapped -- neither more (over-taps, stranding mana the solver
    // has no way to spend) nor fewer (a silent undercharge).
    let p1_lands_tapped_after = game
        .state()
        .objects_in_zone(&ZoneId::Battlefield)
        .into_iter()
        .filter(|o| o.controller == P1 && o.status.tapped)
        .count();
    assert_eq!(
        p1_lands_tapped_after, 5,
        "CR 700.2h: base {{B}} + mode 0 {{2}} + mode 1 {{B}}{{B}} = mana value 5 -- all \
         5 Swamps must be tapped, exactly"
    );

    let on_stack = game
        .state()
        .stack_objects()
        .iter()
        .find(|so| matches!(&so.kind, mtg_engine::StackObjectKind::Spell { .. }))
        .expect("the spell must be on the stack");
    assert_eq!(on_stack.modes_chosen, vec![0, 1]);

    // Drain the stack. Mode 0 (search-then-shuffle-then-put-on-top) raises a
    // `LegalAction::AnswerEffectChoice` decision for a MANDATORY search (CR 701.23d --
    // Insatiable Avarice's mode 0 has no `TargetFilter` quality, so `may_fail_to_find`
    // is false and the provider's own default answer is a real find, never a decline).
    // Submitting the offer's own default answer verbatim resolves it without this test
    // re-deriving which card gets found.
    for _ in 0..10 {
        if game.state().stack_objects().is_empty() {
            break;
        }
        let decision = expect_decision(&mut game);
        let idx = decision
            .actions
            .iter()
            .position(|a| matches!(a, LegalAction::AnswerEffectChoice { .. }))
            .unwrap_or_else(|| {
                index_of(&decision.actions, |a| {
                    matches!(a, LegalAction::PassPriority)
                })
            });
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: idx,
                params: ActionParams::default(),
            },
        )
        .expect("the provider's own default answer must be accepted");
    }
    assert!(
        game.state().stack_objects().is_empty(),
        "the spell must have resolved"
    );

    let p2_hand_after = game.state().objects_in_zone(&ZoneId::Hand(P2)).len();
    let p2_life_after = game.state().player(P2).expect("p2 exists").life_total;
    assert_eq!(
        p2_hand_after,
        p2_hand_before + 3,
        "mode 1 still resolves when chosen alongside mode 0"
    );
    assert_eq!(p2_life_after, p2_life_before - 3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// C — the BOT path (the `spell_default_modes` fallback, CR 700.2a/PB-DP3)
// ═══════════════════════════════════════════════════════════════════════════════

/// **C1** — a bot casting Insatiable Avarice with NO announced modes falls back to
/// `spell_default_modes` (`[0]`, the first `min_modes` indices), and — the point of this
/// test — the cast is FUNDED for that default mode's cost and not merely the base cost.
/// Before this batch, `HeuristicBot` would have this cast rejected with
/// `InsufficientMana` and recorded via `RejectedCommand`, then fall back to
/// `PassPriority` (`LocalGame::advance`'s bot-rejection path).
#[test]
fn c1_bot_default_mode_cast_is_funded_and_not_rejected() {
    let state = spree_state(3);
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(P1, Box::new(HeuristicBot::new(9, "Bot-1".to_string())));
    let (mut game, _events) = LocalGame::start(
        state,
        3,
        StubProvider,
        bots,
        BTreeSet::new(), // no human seats -- P1 is a bot, P2 has no bot (auto-passes)
        limits(),
        true,
    )
    .expect("game starts");

    // `advance()` with no human seats runs the WHOLE game internally (it loops until a
    // human decision, `GameOver`, or a `Halted` safety valve — see its own doc), so one
    // call is the entire drive. `HeuristicBot` scores every real action above
    // `PassPriority` (see `memory/gotchas-infra.md`'s "Simulator / play-client
    // Gotchas"), and casting Insatiable Avarice is the only real action available on
    // this board once the active player reaches a main phase with an empty stack, so
    // it must be cast well within `limits()`'s 3-turn cap.
    match game.advance() {
        AdvanceOutcome::GameOver(_) | AdvanceOutcome::Halted(_) => {}
        AdvanceOutcome::AwaitingHuman(_) => {
            panic!("no human seat exists; nothing should await one")
        }
    }

    let hand_still_has_the_card = game
        .state()
        .objects_in_zone(&ZoneId::Hand(P1))
        .iter()
        .any(|o| o.characteristics.name == "Insatiable Avarice");
    assert!(
        !hand_still_has_the_card,
        "the bot must have cast Insatiable Avarice out of its hand within the turn cap; \
         rejections: {:?}",
        game.rejections()
    );

    // The decisive assertion: no `RejectedCommand` names a mana refusal. A rejection
    // here is exactly the pre-fix defect (`OOS-DX29-14`) -- the bot announcing no
    // modes, falling back to `spell_default_modes` == `[0]`, and being funded for the
    // BASE cost alone, so the mode-0-inclusive cast is refused with
    // `InsufficientMana`.
    let mana_rejections: Vec<&mtg_simulator::RejectedCommand> = game
        .rejections()
        .iter()
        .filter(|r| r.error.to_lowercase().contains("mana"))
        .collect();
    assert!(
        mana_rejections.is_empty(),
        "the bot's cast must not be rejected for a mana reason; rejections: {:?}",
        mana_rejections
    );
}
