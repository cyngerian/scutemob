//! PB-DX44 (`OOS-DX29-3`) — the pitch alt cost (CR 118.9), through the SAME
//! `LocalGame`/`HumanChoice`/`params.rs` channel a human client uses.
//!
//! Force of Will and Force of Negation are each cast at their printed pitch
//! cost with a NON-DEFAULT pitched card — `eligible[0]` is the provider's own
//! default (the lowest `ObjectId`, i.e. the FIRST pitch-eligible card built
//! into the fixture), and every test below picks `eligible[1]` instead, so a
//! probe that accepted the default could not tell a human's choice from the
//! engine's. Force of Vigor is exercised in BOTH directions of its
//! `opponents_turn_only` gate — offered and resolved on an opponent's turn
//! (T2), absent on the caster's own turn (T3).
//!
//! # Misdirection is NOT exercised end-to-end here
//!
//! It is the fourth deck-legal pitch member
//! (`pb_dx44_uncastable_roster.rs::r1`). Its target
//! (`TargetSpellWithSingleTarget`) needs the identical stack-spell fixture
//! Force of Will already builds below, and the offer-layer machinery all four
//! members share (`offerable_pitch_plan`, `eligible_pitch_cards`,
//! `pitch_ability_of`) has no per-card branch — so a fourth copy of the same
//! fixture would prove the same function a fourth time, not a new one.
//! Recorded as a floor rather than silently dropped.
//!
//! CR index: CR 118.9 (pitch), CR 119.4 (life payment gating), CR 702.138a
//! (Force of Negation's exile-instead-of-graveyard clause).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, enrich_spec_from_def, AdditionalCost, AltCostKind, CardDefinition, CardId,
    CardRegistry, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Target, ZoneId,
};
use mtg_simulator::{
    ActionParams, AdvanceOutcome, HumanChoice, LegalAction, LocalGame, LocalGameLimits,
    PendingDecision, StubProvider,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// ── Fixture plumbing (mirrors `pb_dx44_spree_mode_costs.rs`'s conventions) ──────────

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

fn is_pitch_cast_of(card: ObjectId) -> impl Fn(&LegalAction) -> bool {
    move |a| {
        matches!(
            a,
            LegalAction::CastSpell {
                card: c,
                alt_cost: Some(AltCostKind::Pitch),
                ..
            } if *c == card
        )
    }
}

fn is_ordinary_cast_of(card: ObjectId) -> impl Fn(&LegalAction) -> bool {
    move |a| {
        matches!(
            a,
            LegalAction::CastSpell {
                card: c,
                alt_cost: None,
                ..
            } if *c == card
        )
    }
}

/// Pass priority for WHICHEVER seat is asked, until a decision offers an action
/// matching `pred` — player-agnostic, mirrors
/// `pb_dx44_spree_mode_costs.rs::drive_until`, duplicated per that file's own
/// precedent (a shared helper across test binaries is not this project's idiom).
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    pred: impl Fn(&LegalAction) -> bool,
) -> PendingDecision {
    for _ in 0..400 {
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
    panic!("no decision offered a matching action within 400 priority windows");
}

fn drain_stack(game: &mut LocalGame<StubProvider>) {
    for _ in 0..20 {
        if game.state().stack_objects().is_empty() {
            return;
        }
        let decision = expect_decision(game);
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
        .expect("passing to resolve is always legal");
    }
    panic!("stack did not empty within budget");
}

fn start(state: GameState) -> LocalGame<StubProvider> {
    let human_seats: BTreeSet<PlayerId> = [P1, P2].into_iter().collect();
    let (game, _events) = LocalGame::start(
        state,
        1,
        StubProvider,
        HashMap::new(),
        human_seats,
        limits(),
        true,
    )
    .expect("game starts");
    game
}

/// Drive P2 (human) to cast Lightning Bolt targeting `target`, funded fresh at
/// cast time (`auto_tap: true`).
fn cast_bolt(game: &mut LocalGame<StubProvider>, bolt: ObjectId, target: PlayerId) {
    let decision = drive_until(game, is_ordinary_cast_of(bolt));
    let idx = index_of(&decision.actions, is_ordinary_cast_of(bolt));
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets: vec![Target::Player(target)],
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .expect("Lightning Bolt must be castable with a Mountain out");
}

/// Two-player state, P2 ACTIVE FIRST — the only shape that can legally OFFER an
/// `opponents_turn_only` pitch cost, and harmless for a pitch cost with no such
/// restriction: P1 still gets a priority window during P2's own turn (CR 116).
///
/// `force_name` is P1's pitch spell; `eligible` are the pitch-eligible cards
/// ALSO placed in P1's hand, in the order given — `eligible[0]` is the
/// provider's own default (lowest `ObjectId`, built first), so every caller
/// below picks `eligible[1]` as the NON-DEFAULT answer. `p2_mountains` funds
/// Lightning Bolt ({R}), the target-spell fixture Force of Will and Force of
/// Negation both need; pass `0` for a pitch spell that targets permanents
/// instead (Force of Vigor).
fn pitch_state(force_name: &str, eligible: &[&str], p1_life: i32, p2_mountains: u32) -> GameState {
    let defs = defs_by_name();
    let mut builder = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P2)
        .player_life(P1, p1_life)
        .object(corpus_object(&defs, P1, force_name, ZoneId::Hand(P1)));
    for name in eligible {
        builder = builder.object(corpus_object(&defs, P1, name, ZoneId::Hand(P1)));
    }
    if p2_mountains > 0 {
        builder = builder.object(corpus_object(&defs, P2, "Lightning Bolt", ZoneId::Hand(P2)));
    }
    for _ in 0..p2_mountains {
        builder = builder.object(corpus_object(&defs, P2, "Mountain", ZoneId::Battlefield));
    }
    for i in 0..5 {
        builder = builder
            .object(ObjectSpec::card(P1, &format!("P1 Filler {i}")).in_zone(ZoneId::Library(P1)));
        builder = builder
            .object(ObjectSpec::card(P2, &format!("P2 Filler {i}")).in_zone(ZoneId::Library(P2)));
    }
    builder.build().expect("state builds")
}

// ═══════════════════════════════════════════════════════════════════════════
// T1 — Force of Will: life + colour, targeting a real spell on the stack.
// ═══════════════════════════════════════════════════════════════════════════

/// **T1** — Force of Will, pitched: pays 1 life and exiles the NON-DEFAULT
/// blue card, counters Lightning Bolt, and charges NO mana (CR 118.9a).
#[test]
fn t1_force_of_will_pitch_end_to_end_with_a_non_default_card_and_life_paid() {
    let state = pitch_state("Force of Will", &["Brainstorm", "Counterspell"], 20, 1);
    let force_id = id_of(&state, "Force of Will");
    let bolt_id = id_of(&state, "Lightning Bolt");
    // The NON-DEFAULT card: `eligible[0]` (Brainstorm, lower ObjectId, built
    // first) is the provider's own default; this test deliberately picks
    // `eligible[1]` instead.
    let non_default_card_id = id_of(&state, "Counterspell");
    let default_card_id = id_of(&state, "Brainstorm");

    let mut game = start(state);
    cast_bolt(&mut game, bolt_id, P1);

    let decision = drive_until(&mut game, is_pitch_cast_of(force_id));
    let idx = index_of(&decision.actions, is_pitch_cast_of(force_id));

    // CR 400.7: casting minted Lightning Bolt a FRESH `ObjectId` on the move to
    // the stack, so `bolt_id` (captured pre-cast) is now dead. `TargetSpell` /
    // `TargetSpellWithFilter` validate via `state.objects.get(&id)` with
    // `zone == Stack` (`casting.rs:6582-6598`) -- the CARD's id, not the
    // `StackObject.id` `StackObjectKind::Spell { source_object }` links to it.
    // Re-resolve by name for exactly that reason.
    let bolt_stack_id = id_of(game.state(), "Lightning Bolt");

    let p1_life_before = game.state().player(P1).expect("p1 exists").life_total;

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets: vec![Target::Object(bolt_stack_id)],
                additional_costs: vec![AdditionalCost::ExileFromHand {
                    card: non_default_card_id,
                }],
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .unwrap_or_else(|e| panic!("Force of Will pitch cast must be accepted: {e:?}"));

    // CR 119.4: exactly 1 life paid.
    assert_eq!(
        game.state().player(P1).expect("p1 exists").life_total,
        p1_life_before - 1,
        "CR 118.9: pitching Force of Will pays exactly 1 life"
    );
    // CR 118.9a: the printed mana cost was NEVER paid — P1 had no mana pool to
    // begin with, so reaching this point at all already proves it, and the
    // pool stays empty rather than going negative or erroring.
    assert!(
        game.state()
            .player(P1)
            .expect("p1 exists")
            .mana_pool
            .is_empty(),
        "CR 118.9a: Force of Will's printed {{3}}{{U}}{{U}} must never be charged"
    );
    // The NON-DEFAULT card, and only it, is exiled. CR 400.7: the move minted a
    // FRESH `ObjectId` for it, so this is checked by NAME rather than by
    // reusing the pre-cast `non_default_card_id` -- that id is what the
    // submitted `AdditionalCost::ExileFromHand` correctly named (the card's id
    // AT SUBMIT time, before the move happened), and is dead afterward.
    let exile_names: Vec<String> = game
        .state()
        .objects_in_zone(&ZoneId::Exile)
        .into_iter()
        .map(|o| o.characteristics.name.clone())
        .collect();
    assert_eq!(
        exile_names,
        vec!["Counterspell".to_string()],
        "exactly the chosen (non-default) card must be exiled: {exile_names:?}"
    );
    assert_eq!(
        game.state()
            .object(default_card_id)
            .expect("the default card was never moved, so its id must still be live")
            .zone,
        ZoneId::Hand(P1),
        "the DEFAULT card (never chosen) must still be in hand"
    );

    drain_stack(&mut game);

    // CR 608: resolution. Bolt is countered, to P2's graveyard (its owner).
    let graveyard: Vec<String> = game
        .state()
        .objects_in_zone(&ZoneId::Graveyard(P2))
        .into_iter()
        .map(|o| o.characteristics.name.clone())
        .collect();
    assert!(
        graveyard.contains(&"Lightning Bolt".to_string()),
        "Force of Will's CounterSpell must have sent Lightning Bolt to its \
         owner's graveyard: {graveyard:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T2/T3 — Force of Vigor: `opponents_turn_only`, both directions.
// ═══════════════════════════════════════════════════════════════════════════

/// Two-player state for Force of Vigor: P1 holds the pitch spell plus two
/// green pitch-eligible cards; P2 controls two synthetic permanents (an
/// artifact and an enchantment — Force of Vigor's own effect has TWO
/// `DeclaredTarget` slots, `Sequence([DestroyPermanent{0}, DestroyPermanent{1}])`,
/// so a two-target announcement matches the def's own authoring exactly).
/// `active` decides whose turn it is — the whole variable T2/T3 differ on.
fn force_of_vigor_state(active: PlayerId) -> GameState {
    let defs = defs_by_name();
    GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(active)
        .player_life(P1, 20)
        .object(corpus_object(&defs, P1, "Force of Vigor", ZoneId::Hand(P1)))
        .object(corpus_object(&defs, P1, "Llanowar Elves", ZoneId::Hand(P1)))
        .object(corpus_object(&defs, P1, "Rampant Growth", ZoneId::Hand(P1)))
        .object(ObjectSpec::artifact(P2, "Doomed Artifact").in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::enchantment(P2, "Doomed Enchantment").in_zone(ZoneId::Battlefield))
        .build()
        .expect("state builds")
}

/// **T2** — Force of Vigor, pitched on P2's turn (legal: CR 118.9's "if it's
/// not your turn" is met), destroying two real permanents. No `PayLife`
/// component in this card's `costs` — proves that branch charges no life at
/// all (the mirror image of T1's life-paid assertion).
#[test]
fn t2_force_of_vigor_pitch_end_to_end_on_an_opponents_turn() {
    let state = force_of_vigor_state(P2);
    let force_id = id_of(&state, "Force of Vigor");
    let artifact_id = id_of(&state, "Doomed Artifact");
    let enchantment_id = id_of(&state, "Doomed Enchantment");
    // NON-DEFAULT: `eligible[0]` (Llanowar Elves) is the provider's default;
    // this test picks Rampant Growth instead.
    let non_default_card_id = id_of(&state, "Rampant Growth");
    let default_card_id = id_of(&state, "Llanowar Elves");

    let mut game = start(state);

    let decision = drive_until(&mut game, is_pitch_cast_of(force_id));
    let idx = index_of(&decision.actions, is_pitch_cast_of(force_id));
    let p1_life_before = game.state().player(P1).expect("p1 exists").life_total;

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets: vec![Target::Object(artifact_id), Target::Object(enchantment_id)],
                additional_costs: vec![AdditionalCost::ExileFromHand {
                    card: non_default_card_id,
                }],
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .unwrap_or_else(|e| panic!("Force of Vigor pitch cast must be accepted: {e:?}"));

    assert_eq!(
        game.state().player(P1).expect("p1 exists").life_total,
        p1_life_before,
        "CR 118.9: Force of Vigor's pitch cost has no life component"
    );
    // CR 400.7: the exile move minted a FRESH `ObjectId`, so this is checked by
    // NAME -- see T1's identical note.
    let exile_names: Vec<String> = game
        .state()
        .objects_in_zone(&ZoneId::Exile)
        .into_iter()
        .map(|o| o.characteristics.name.clone())
        .collect();
    assert_eq!(
        exile_names,
        vec!["Rampant Growth".to_string()],
        "exactly the chosen (non-default) green card must be exiled: {exile_names:?}"
    );
    assert_eq!(
        game.state()
            .object(default_card_id)
            .expect("the default card was never moved, so its id must still be live")
            .zone,
        ZoneId::Hand(P1),
        "the DEFAULT green card (never chosen) must still be in hand"
    );

    drain_stack(&mut game);

    assert!(
        !game.state().objects().contains_key(&artifact_id),
        "CR 601.2c: the announced artifact must be destroyed"
    );
    assert!(
        !game.state().objects().contains_key(&enchantment_id),
        "CR 601.2c: the announced enchantment must be destroyed"
    );
    let graveyard: Vec<String> = game
        .state()
        .objects_in_zone(&ZoneId::Graveyard(P2))
        .into_iter()
        .map(|o| o.characteristics.name.clone())
        .collect();
    assert!(
        graveyard.contains(&"Doomed Artifact".to_string())
            && graveyard.contains(&"Doomed Enchantment".to_string()),
        "both destroyed permanents must be in the graveyard, not merely gone \
         from the battlefield: {graveyard:?}"
    );
}

/// **T3** — the negative control: on P1's OWN turn, Force of Vigor's pitch
/// action must not be offered at all (CR 118.9's "if it's not your turn").
/// The ORDINARY cast (paying the printed `{2}{G}{G}`) is untouched by this
/// gate and stays offered whenever it is otherwise legal to cast — this test
/// does not assert on it either way, since P1 has no green mana in this
/// fixture and the ordinary cast is not this test's subject.
#[test]
fn t3_force_of_vigor_pitch_is_not_offered_on_the_casters_own_turn() {
    let state = force_of_vigor_state(P1);
    let force_id = id_of(&state, "Force of Vigor");
    let mut game = start(state);

    // Drive to the first real priority window and confirm the pitch action is
    // absent -- not merely "not chosen yet". `drive_until` cannot be reused
    // here (its whole contract is "wait until offered"); walk one window
    // directly instead.
    let decision = expect_decision(&mut game);
    assert_eq!(
        decision.player, P1,
        "sanity: P1 is active first and gets the first priority window"
    );
    assert!(
        !decision.actions.iter().any(is_pitch_cast_of(force_id)),
        "CR 118.9: Force of Vigor's pitch action must be ABSENT on the caster's \
         own turn (SR-38) -- got {:?}",
        decision.actions
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T4 — Force of Negation: `opponents_turn_only` + CR 702.138a's exile-instead
// clause, on the SAME stack-spell fixture Force of Will (T1) uses.
// ═══════════════════════════════════════════════════════════════════════════

/// **T4** — Force of Negation, pitched on P2's turn, countering Lightning Bolt
/// (a noncreature spell, satisfying `TargetSpellWithFilter { non_creature:
/// true }`) with the NON-DEFAULT blue card. CR 702.138a: the countered spell
/// goes to EXILE, not its owner's graveyard -- the one assertion that
/// distinguishes this from T1's plain `CounterSpell`.
#[test]
fn t4_force_of_negation_pitch_end_to_end_exiles_the_countered_spell() {
    let state = pitch_state("Force of Negation", &["Brainstorm", "Counterspell"], 20, 1);
    let force_id = id_of(&state, "Force of Negation");
    let bolt_id = id_of(&state, "Lightning Bolt");
    let non_default_card_id = id_of(&state, "Counterspell");

    let mut game = start(state);
    cast_bolt(&mut game, bolt_id, P1);

    let decision = drive_until(&mut game, is_pitch_cast_of(force_id));
    let idx = index_of(&decision.actions, is_pitch_cast_of(force_id));
    // CR 400.7: casting minted Lightning Bolt a FRESH `ObjectId` on the move to
    // the stack, so `bolt_id` (captured pre-cast) is now dead. `TargetSpell` /
    // `TargetSpellWithFilter` validate via `state.objects.get(&id)` with
    // `zone == Stack` (`casting.rs:6582-6598`) -- the CARD's id, not the
    // `StackObject.id` `StackObjectKind::Spell { source_object }` links to it.
    // Re-resolve by name for exactly that reason.
    let bolt_stack_id = id_of(game.state(), "Lightning Bolt");

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets: vec![Target::Object(bolt_stack_id)],
                additional_costs: vec![AdditionalCost::ExileFromHand {
                    card: non_default_card_id,
                }],
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .unwrap_or_else(|e| panic!("Force of Negation pitch cast must be accepted: {e:?}"));

    assert!(
        game.state()
            .player(P1)
            .expect("p1 exists")
            .mana_pool
            .is_empty(),
        "CR 118.9a: the printed {{1}}{{U}}{{U}} must never be charged"
    );
    // CR 400.7: the exile move minted a FRESH `ObjectId`, so this is checked by
    // NAME -- see T1's identical note.
    let exile_names_after_cast: Vec<String> = game
        .state()
        .objects_in_zone(&ZoneId::Exile)
        .into_iter()
        .map(|o| o.characteristics.name.clone())
        .collect();
    assert_eq!(
        exile_names_after_cast,
        vec!["Counterspell".to_string()],
        "exactly the chosen (non-default) blue card must be exiled to pay the \
         pitch cost: {exile_names_after_cast:?}"
    );

    drain_stack(&mut game);

    // CR 702.138a: exiled, NOT sent to the graveyard.
    let graveyard: Vec<String> = game
        .state()
        .objects_in_zone(&ZoneId::Graveyard(P2))
        .into_iter()
        .map(|o| o.characteristics.name.clone())
        .collect();
    assert!(
        !graveyard.contains(&"Lightning Bolt".to_string()),
        "CR 702.138a: Force of Negation exiles the countered spell instead of \
         sending it to the graveyard: {graveyard:?}"
    );
    let exile: Vec<String> = game
        .state()
        .objects_in_zone(&ZoneId::Exile)
        .into_iter()
        .map(|o| o.characteristics.name.clone())
        .collect();
    assert!(
        exile.contains(&"Lightning Bolt".to_string()),
        "the countered spell must be in exile: {exile:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T5 — the AUTO-TAP prediction itself, isolated from the charge.
// ═══════════════════════════════════════════════════════════════════════════

/// **T5** — `effective_cast_cost_with_additional`'s `Pitch` branch, discriminated
/// from the ENGINE's own `{0}` charge.
///
/// T1's assertions ("pool stays empty", "life drops by exactly 1") pass even if
/// the auto-tap PREDICTS the printed cost instead of `{0}`: the fixtures there
/// have no mana source able to pay the FULL printed cost, so a wrong (higher)
/// prediction just makes `mana_solver::solve_mana_payment_with_pool` find NO
/// plan, `auto_tap_commands_for` taps nothing, and the ACTUAL charge (decided
/// by `casting.rs` alone, never by this prediction) is still `{0}` either way
/// -- a silent non-discrimination, proven by an executed revert during this
/// stage's own verification (see the execution notes). This test closes that
/// gap: Force of Vigor is pitched with EXACTLY enough Forests on the board to
/// pay its printed `{2}{G}{G}` (4 mana), announcing ZERO targets (CR 601.2c
/// `UpToN`'s legal minimum, so no stack-spell fixture is needed) -- a wrong
/// prediction WOULD find a plan and tap them, and a correct one leaves every
/// Forest untapped.
#[test]
fn t5_pitch_cost_prediction_taps_nothing_even_when_it_could() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P2)
        .player_life(P1, 20)
        .object(corpus_object(&defs, P1, "Force of Vigor", ZoneId::Hand(P1)))
        .object(corpus_object(&defs, P1, "Llanowar Elves", ZoneId::Hand(P1)))
        .object(corpus_object(&defs, P1, "Forest", ZoneId::Battlefield))
        .object(corpus_object(&defs, P1, "Forest", ZoneId::Battlefield))
        .object(corpus_object(&defs, P1, "Forest", ZoneId::Battlefield))
        .object(corpus_object(&defs, P1, "Forest", ZoneId::Battlefield))
        .build()
        .expect("state builds");
    let force_id = id_of(&state, "Force of Vigor");
    let pitch_card_id = id_of(&state, "Llanowar Elves");

    let human_seats: BTreeSet<PlayerId> = [P1].into_iter().collect();
    let (mut game, _events) = LocalGame::start(
        state,
        5,
        StubProvider,
        HashMap::new(),
        human_seats,
        limits(),
        true,
    )
    .expect("game starts");

    let decision = drive_until(&mut game, is_pitch_cast_of(force_id));
    let idx = index_of(&decision.actions, is_pitch_cast_of(force_id));

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets: vec![],
                additional_costs: vec![AdditionalCost::ExileFromHand {
                    card: pitch_card_id,
                }],
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .unwrap_or_else(|e| {
        panic!("Force of Vigor pitch cast (zero announced targets) must be accepted: {e:?}")
    });

    let untapped_forests = game
        .state()
        .objects_in_zone(&ZoneId::Battlefield)
        .into_iter()
        .filter(|o| o.characteristics.name == "Forest" && !o.status.tapped)
        .count();
    assert_eq!(
        untapped_forests, 4,
        "CR 118.9a: Force of Vigor's printed {{2}}{{G}}{{G}} must never be \
         charged, so `auto_tap: true` must tap NONE of the 4 available \
         Forests -- a wrong (printed-cost) prediction would have found a \
         plan and tapped them"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T6 — Misdirection: the ONLY pitch member whose cost list has no life
// component, driven end to end for that reason.
// ═══════════════════════════════════════════════════════════════════════════

/// **T6** — Misdirection, pitched: exiles the NON-DEFAULT blue card, pays NO
/// life, charges NO mana, and redirects Lightning Bolt off its announced target
/// onto the caster's opponent (CR 115.7a).
///
/// # Why this exists, and a correction to this test's own first draft
///
/// This file's first draft recorded Misdirection as a stated FLOOR, on the
/// ground that `offerable_pitch_plan` / `eligible_pitch_cards` /
/// `pitch_ability_of` have no per-card branch. The probe was then written with
/// a doc comment claiming Misdirection is *"the only deck-legal member whose
/// cost list contains no `Cost::PayLife` at all"*.
///
/// **That claim was exactly backwards, and executing a revert is what said so.**
/// Rewriting `offerable_pitch_plan`'s tolerant `for … if let Cost::PayLife`
/// loop into a `find_map(PayLife)?` — i.e. making a life component MANDATORY —
/// was expected to redden this test alone. It reddened **four**. Reading the
/// defs afterwards: `force_of_will` pays `Cost::PayLife(1)`, and
/// `force_of_negation`, `force_of_vigor` and `misdirection` pay **no life at
/// all**. Force of Will is the exception, not the rule, and this comment had
/// the population inverted 1-vs-3.
///
/// So the honest account of what T6 buys:
/// * **Coverage of the fourth named member.** `pb_dx44_uncastable_roster::r1`
///   pins the deck-legal pitch population at four and this batch's acceptance
///   criterion names all four; three of four is not four.
/// * **The only pitch cast in this file whose spell effect is a target CHANGE**
///   (CR 115.7a, `Effect::ChangeTargets`) rather than a counter or a destroy —
///   the path PB-DX25c rebuilt, reached for the first time through the pitch
///   channel.
/// * It is the only member combining a life-free cost list with
///   `opponents_turn_only: false`, so it is the only pitch offer that must
///   appear on the CASTER'S OWN TURN with nothing to check but the colour.
///
/// **It isolates no code branch T1-T5 miss**, and that is stated rather than
/// implied: it reddens on the shared-channel reverts (`params.rs` forwarding
/// `alt_cost`, the Pitch cost-prediction branch) exactly as its siblings do,
/// and there is no revert of a line this batch owns that reddens T6 alone.
///
/// **The redirect half is deliberately asserted by DAMAGE, not by the stack.**
/// `Effect::ChangeTargets` rewriting a `StackObject.targets` entry is
/// observable before resolution, and a probe that stopped there would pass
/// against a redirect that never takes effect. Bolt is drained to resolution
/// and the life totals decide it.
#[test]
fn t6_misdirection_pitch_end_to_end_with_no_life_component() {
    // P2 is active (see `pitch_state`), holds Lightning Bolt and a Mountain to
    // cast it with; P1 holds Misdirection plus two blue pitch candidates.
    let state = pitch_state("Misdirection", &["Brainstorm", "Counterspell"], 20, 1);
    let mis_id = id_of(&state, "Misdirection");
    let bolt_id = id_of(&state, "Lightning Bolt");
    // `eligible[0]` (Brainstorm) is the provider's own default; pick the other.
    let non_default_card_id = id_of(&state, "Counterspell");
    let default_card_id = id_of(&state, "Brainstorm");

    let mut game = start(state);
    // P2 casts Bolt at P1. Misdirection will move it to P2.
    cast_bolt(&mut game, bolt_id, P1);

    let decision = drive_until(&mut game, is_pitch_cast_of(mis_id));
    let idx = index_of(&decision.actions, is_pitch_cast_of(mis_id));

    // CR 400.7: the cast minted Bolt a fresh `ObjectId`; re-resolve by name, for
    // the reason T1 documents at length.
    let bolt_stack_id = id_of(game.state(), "Lightning Bolt");

    let p1_life_before = game.state().player(P1).expect("p1 exists").life_total;
    let p2_life_before = game.state().player(P2).expect("p2 exists").life_total;

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets: vec![Target::Object(bolt_stack_id)],
                additional_costs: vec![AdditionalCost::ExileFromHand {
                    card: non_default_card_id,
                }],
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .unwrap_or_else(|e| panic!("Misdirection pitch cast must be accepted: {e:?}"));

    // The point of this probe: NO life is paid, because Misdirection's cost list
    // has no `Cost::PayLife` member at all.
    assert_eq!(
        game.state().player(P1).expect("p1 exists").life_total,
        p1_life_before,
        "CR 118.9: Misdirection's pitch cost is a card and NOTHING else -- no \
         life may be paid"
    );
    // CR 118.9a: the printed {3}{U}{U} is never charged. P1 has no mana source
    // at all, so reaching this point proves it; the empty pool proves nothing
    // was conjured.
    assert!(
        game.state()
            .player(P1)
            .expect("p1 exists")
            .mana_pool
            .is_empty(),
        "CR 118.9a: Misdirection's printed {{3}}{{U}}{{U}} must never be charged"
    );
    // The NON-DEFAULT card, and only it, is exiled -- checked by NAME because
    // CR 400.7 minted it a fresh id on the move.
    let exile_names: Vec<String> = game
        .state()
        .objects_in_zone(&ZoneId::Exile)
        .into_iter()
        .map(|o| o.characteristics.name.clone())
        .collect();
    assert_eq!(
        exile_names,
        vec!["Counterspell".to_string()],
        "exactly the chosen (non-default) card must be exiled: {exile_names:?}"
    );
    assert_eq!(
        game.state()
            .object(default_card_id)
            .expect("the default card was never moved, so its id must still be live")
            .zone,
        ZoneId::Hand(P1),
        "the DEFAULT card (never chosen) must still be in hand"
    );

    drain_stack(&mut game);

    // CR 115.7a: the redirect landed. Bolt was announced at P1 and dealt its 3
    // to P2 instead. Asserted on BOTH seats -- "P2 lost 3" alone would also hold
    // if the spell had somehow hit both.
    assert_eq!(
        game.state().player(P2).expect("p2 exists").life_total,
        p2_life_before - 3,
        "CR 115.7a: Misdirection moved Lightning Bolt's target to P2, so P2 \
         takes the 3 damage"
    );
    assert_eq!(
        game.state().player(P1).expect("p1 exists").life_total,
        p1_life_before,
        "CR 115.7a: P1 was Bolt's ANNOUNCED target and must take no damage \
         once the target is changed -- and pays no life for the pitch either, \
         so this total is unmoved from before the cast"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T7 — the DEFERRED half of `OOS-DX29-3`, pinned wrong-way-round.
// ═══════════════════════════════════════════════════════════════════════════

/// **T7** — an Escape card sitting in a graveyard is offered NOTHING today, so
/// nothing about it can be refused. Pinned so the day a graveyard cast loop is
/// added, this test says out loud what that loop is coupled to.
///
/// # What is deferred, and why it is deferred rather than shipped
///
/// `OOS-DX29-3` has two halves. The pitch half is CLOSED by this batch (T1-T6).
/// The other half is a graveyard cast loop — `StubProvider`'s two cast loops
/// walk `ZoneId::Hand(player)` and `ZoneId::Command(player)` and **no
/// graveyard**, so Retrace, Jump-Start, Escape and Flashback casts are never
/// offered at all.
///
/// It is deferred because it is not one feature, it is two that must land
/// together, and the seed says so: **`casting.rs` AUTO-DETECTS escape.**
/// `casting.rs:283` is
/// `casting_from_graveyard && card_has_escape_keyword && !casting_with_flashback`,
/// with no opt-in from the caller — so the moment a graveyard loop exists, an
/// Escape card in a graveyard becomes an escape cast that then demands an
/// exact-count `AdditionalCost::EscapeExile` the offer layer has no channel to
/// supply. **A graveyard loop shipped alone converts "never offered" into "a
/// hard refusal"**, which is strictly worse than the status quo and is exactly
/// the SR-38 shape this batch exists to delete. Adding the `EscapeExile`
/// channel is a second cost picker with its own eligibility arithmetic (CR
/// 702.138a's exact count, from a zone the pitch picker never reads), and this
/// batch has already taken a wire bump and four other channels.
///
/// # The deferral is doubly safe today, and both halves are measured
///
/// 1. This test: the provider offers **zero** casts for a graveyard-resident
///    Escape card, so no refusal exists to fix.
/// 2. `pb_dx44_uncastable_roster::r8`: the deck-legal `Complete` Escape
///    population is **zero** — all four corpus Escape defs are `partial` or
///    `known_wrong` — so even a graveyard loop could not reach one from a legal
///    deck. The seed's row does not mention this and it is the difference
///    between a latent defect and an unreachable one.
///
/// **This test is a FLOOR, not an approval.** It goes red the day a graveyard
/// cast loop is added, and that redness is the reminder to bring the
/// `EscapeExile` channel with it.
#[test]
fn t7_an_escape_card_in_a_graveyard_is_offered_no_cast_today() {
    let defs = defs_by_name();
    // `nethergoyf` is `partial`, so a real deck could not contain it; a fixture
    // can, which is what makes the coupling observable at all today.
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .object(corpus_object(
            &defs,
            P1,
            "Nethergoyf",
            ZoneId::Graveyard(P1),
        ))
        // Plenty of mana, so a suppressed offer cannot be mistaken for an
        // unaffordable one.
        .object(corpus_object(&defs, P1, "Swamp", ZoneId::Battlefield))
        .object(corpus_object(&defs, P1, "Forest", ZoneId::Battlefield))
        .object(corpus_object(&defs, P1, "Mountain", ZoneId::Battlefield))
        .build()
        .expect("state builds");

    let goyf = id_of(&state, "Nethergoyf");
    assert_eq!(
        state.object(goyf).expect("goyf exists").zone,
        ZoneId::Graveyard(P1),
        "fixture precondition: the Escape card must actually be in the graveyard"
    );

    let actions =
        mtg_simulator::legal_actions::LegalActionProvider::legal_actions(&StubProvider, &state, P1);
    let casts_of_goyf: Vec<&LegalAction> = actions
        .iter()
        .filter(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == goyf))
        .collect();
    assert!(
        casts_of_goyf.is_empty(),
        "`OOS-DX29-3` (deferred half): the provider walks Hand and Command only, \
         so a graveyard-resident Escape card must be offered NO cast at all. \
         Offering one without an `EscapeExile` channel turns 'never offered' \
         into a HARD REFUSAL, because `casting.rs:283` auto-detects escape from \
         the zone alone. Got: {casts_of_goyf:?}"
    );

    // Non-vacuity: the provider is working and DOES offer casts from hand in
    // this same state -- an empty `actions` list would satisfy the assertion
    // above while proving nothing.
    assert!(
        !actions.is_empty(),
        "non-vacuity floor: the provider returned no actions at all, so the \
         assertion above is vacuous"
    );
}
