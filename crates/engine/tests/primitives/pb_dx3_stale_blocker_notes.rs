//! PB-DX3 (OOS-DP6-3): two stale blocker notes — `garruks_uprising` and
//! `inventors_fair` both carried TODOs claiming `InterveningIf` (the OLD
//! two-variant `InterveningIf` runtime enum) had no board-state variant, when
//! the DEF-LEVEL field these two cards actually use is
//! `AbilityDefinition::Triggered { intervening_if: Option<Condition>, .. }`
//! and `Condition::YouControlNOrMoreWithFilter { count, filter }` has existed
//! since well before this batch (21 other shipped defs already use it) and is
//! queue-time evaluable (`effects::condition_is_queue_time_evaluable`). This
//! file loads both defs from the real corpus via `all_cards()` and drives
//! them through the full command/resolution pipeline — never a re-declared
//! copy of the def.
//!
//! Garruk's Uprising — {2}{G} Enchantment:
//!   "When this enchantment enters, if you control a creature with power 4 or
//!   greater, draw a card." (first ability; this batch's edit)
//!   "Creatures you control have trample." (static; untouched)
//!   "Whenever a creature you control with power 4 or greater enters, draw a
//!   card." (third ability; untouched, T4 pins it as a regression guard)
//! Ruling 2024-11-08: "If you don't control a creature with power 4 or
//! greater immediately after Garruk's Uprising enters, its first ability
//! won't trigger. If you don't control one as the ability resolves, you
//! don't draw a card. They don't have to be the same creature both times."
//!
//! Inventors' Fair — Legendary Land:
//!   "At the beginning of your upkeep, if you control three or more
//!   artifacts, you gain 1 life." (upkeep trigger; ADDED by this batch — it
//!   did not exist in the pre-fix def at all)
//!   "{T}: Add {C}." (mana ability; untouched)
//!   "{4}, {T}, Sacrifice Inventors' Fair: Search your library for an
//!   artifact card, reveal it, put it into your hand, then shuffle. Activate
//!   only if you control three or more artifacts." (search ability;
//!   `activation_condition` ADDED by this batch)
//! Ruling 2016-09-20 #1: "No player may take actions in a turn before
//! Inventors' Fair's triggered ability checks to see if it should trigger.
//! If you don't control three or more artifacts, it won't trigger."
//! Ruling 2016-09-20 #2: "If you control three artifacts as the ability
//! resolves, you gain 1 life... If you don't control three artifacts at that
//! time, you won't gain life."
//! Ruling 2016-09-20 #3: "When using Inventors' Fair's activated ability, the
//! number of artifacts you control is checked only as you activate it. It's
//! not checked again as the ability resolves."
//!
//! ## Pre-fix observations (recorded before the card-def edit, per plan §3)
//!
//! - **T1** (Garruk's Uprising, no power-4+ creature): PRE-FIX, the ETB
//!   trigger queued unconditionally (`intervening_if: None`) and a card WAS
//!   drawn even with no qualifying creature on the battlefield — the
//!   `stack_objects().is_empty()` assertion failed (1 object queued, not 0)
//!   and the post-resolution hand count was 1, not 0.
//! - **T3** (Garruk's Uprising, creature present at queue time, removed
//!   before resolution): PRE-FIX, the trigger still queued (as with T1's
//!   sibling T2) but the RESOLUTION-time re-check also read `None` for
//!   `intervening_if`, so the draw happened unconditionally regardless of
//!   the creature's absence at resolution — a card was drawn even though the
//!   qualifying creature had already left. Post-fix, no draw happens.
//! - **T5** (Inventors' Fair, 2 artifacts, upkeep): PRE-FIX, the upkeep
//!   trigger DID NOT EXIST AT ALL — no ability in `abilities` had
//!   `TriggerCondition::AtBeginningOfYourUpkeep`. Advancing to Upkeep queued
//!   nothing and life stayed unchanged for the "other reason" (missing
//!   ability, not a correctly-gated one) — the same observable state as the
//!   post-fix negative case, which is exactly why T6 (the positive case)
//!   is the disambiguating probe (see below).
//! - **T7** (Inventors' Fair, 3 artifacts then one leaves before
//!   resolution): PRE-FIX, this probe was UNREACHABLE for the stated reason
//!   — there was no upkeep trigger to queue at all (same root cause as T5),
//!   so "no life gained" held vacuously, not because of a resolution-time
//!   re-check. Post-fix, the trigger genuinely queues (3 artifacts) and then
//!   genuinely fizzles at resolution (down to 2) — a real re-check, not an
//!   absent ability.
//! - **T8** (Inventors' Fair, activate the search ability with only 2
//!   artifacts): PRE-FIX, `activation_condition: None` meant the activation
//!   was PERMITTED — `process_command` returned `Ok`, not `Err`. Post-fix it
//!   returns `Err` with a message containing "activation condition not met".
//!
//! T6/T9/T10 are non-regression / positive-direction pins: T6 disambiguates
//! T5 (proves the upkeep trigger exists and fires correctly with 3
//! artifacts, so T5's "no life gained" is a real gate, not vacuous absence);
//! T9 exercises the activated ability end to end through PB-DP9's CR 608.2d
//! channel; T10 pins ruling 2016-09-20 #3 (no resolution-time re-check) so a
//! future "fix" wrapping the search in `Effect::Conditional` would be wrong.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, CardDefinition, CardEffectTarget, Command,
    EffectChoiceAnswer, EffectChoiceQuestion, GameEvent, GameState, GameStateBuilder, ManaColor,
    ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;

// ── Shared helpers ───────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Load a card def from the real corpus by exact name. Never re-declared
/// inline — the probes must exercise the shipped def.
fn card_def(name: &str) -> CardDefinition {
    all_cards()
        .into_iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("{name} should be in the corpus"))
}

fn one_def_map(def: &CardDefinition) -> HashMap<String, CardDefinition> {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Place `def` on the battlefield under `owner`'s control, fully enriched.
fn place_on_battlefield(owner: PlayerId, def: &CardDefinition) -> ObjectSpec {
    let defs = one_def_map(def);
    enrich_spec_from_def(
        ObjectSpec::card(owner, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    )
}

/// Put `def` in `owner`'s hand, fully enriched (for casting).
fn place_in_hand(owner: PlayerId, def: &CardDefinition) -> ObjectSpec {
    let defs = one_def_map(def);
    enrich_spec_from_def(
        ObjectSpec::card(owner, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Hand(owner)),
        &defs,
    )
}

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

fn advance_to_step(mut state: GameState, target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn().step == target {
            return state;
        }
        guard += 1;
        assert!(
            guard < 500,
            "advance_to_step exceeded safety guard (stuck at {:?}, wanted {:?})",
            state.turn().step,
            target
        );
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or_else(|| panic!("no priority holder at step {:?}", state.turn().step));
        let (new_state, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = new_state;
    }
}

fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

fn empty_cast_spell_data(player: PlayerId, card: ObjectId) -> Command {
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

/// Destroy a specific object directly, bypassing the stack — used to remove a
/// qualifying permanent BETWEEN queue time and resolution time so the
/// resolution-time re-check has something real to fail against.
fn destroy_object(state: &mut GameState, controller: PlayerId, obj_id: ObjectId) {
    execute_effect(
        state,
        &mtg_engine::Effect::DestroyPermanent {
            target: CardEffectTarget::Source,
            cant_be_regenerated: false,
        },
        &mut EffectContext::new(controller, obj_id, vec![]),
    );
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .zones()
        .get(&ZoneId::Hand(player))
        .map(|z| z.object_ids().len())
        .unwrap_or(0)
}

/// A three-artifact, two-player fixture for Inventors' Fair, at the given
/// step, with the land itself and the given number of controlled artifacts
/// already on the battlefield.
fn inventors_fair_fixture(num_artifacts: u32, step: Step) -> (GameState, CardDefinition) {
    let def = card_def("Inventors' Fair");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def));
    for i in 0..num_artifacts {
        builder = builder.object(ObjectSpec::artifact(p(1), &format!("Artifact {i}")));
    }
    let mut state = builder.active_player(p(1)).at_step(step).build().unwrap();
    state.turn_mut().priority_holder = Some(p(1));
    (state, def)
}

fn artifact_count_on_battlefield(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.controller == player
                && o.characteristics
                    .card_types
                    .contains(&mtg_engine::CardType::Artifact)
        })
        .count()
}

// ── T1: Garruk's Uprising -- ETB with NO qualifying creature: no trigger, no draw ──

/// CR 603.4 (ruling 2024-11-08, sentence 1): "If you don't control a creature
/// with power 4 or greater immediately after Garruk's Uprising enters, its
/// first ability won't trigger." Pre-fix: `intervening_if: None` meant the
/// trigger always queued and always drew a card (see module doc's "Pre-fix
/// observations").
#[test]
fn test_dx3_garruks_uprising_etb_no_qualifying_creature_no_trigger() {
    let def = card_def("Garruk's Uprising");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_in_hand(p(1), &def))
        // A creature that does NOT qualify (power 3, not 4+).
        .object(ObjectSpec::creature(p(1), "Small Beast", 3, 3))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn_mut().priority_holder = Some(p(1));

    let spell_id = find_object(&state, "Garruk's Uprising");
    let (state, _) =
        process_command(state, empty_cast_spell_data(p(1), spell_id)).expect("cast should succeed");
    // Baseline captured AFTER the cast (the enchantment has already left the
    // hand for the stack) so the draw is the only remaining variable -- taking
    // the baseline before casting would count the -1 (spell leaves hand) and
    // any +1 (draw) as cancelling out, silently vacating the assertion.
    let hand_before = hand_count(&state, p(1));
    // One `pass_all` resolves the enchantment onto the battlefield and, if the
    // ETB trigger were wrongly queued, would flush it onto the stack too.
    let (state, _) = pass_all(state, &[p(1), p(2)]);

    assert!(
        state.stack_objects().is_empty(),
        "CR 603.4: with no power-4+ creature controlled at queue time, Garruk's \
         Uprising's ETB trigger must not even be queued; stack is {:?}",
        state.stack_objects()
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        hand_count(&state, p(1)),
        hand_before,
        "no card should have been drawn from the ungated ETB ability"
    );
}

// ── T2: Garruk's Uprising -- ETB with a qualifying creature: queues, draws ────

/// Non-regression positive direction: with a genuine 4-power creature
/// controlled, the ETB trigger queues, resolves, and exactly one card is
/// drawn. Pins the positive half so T1 cannot be satisfied by breaking the
/// ability outright.
#[test]
fn test_dx3_garruks_uprising_etb_qualifying_creature_draws_one() {
    let def = card_def("Garruk's Uprising");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_in_hand(p(1), &def))
        .object(ObjectSpec::creature(p(1), "Big Beast", 4, 4))
        .object(ObjectSpec::creature(p(1), "Library Filler", 1, 1).in_zone(ZoneId::Library(p(1))))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn_mut().priority_holder = Some(p(1));

    let spell_id = find_object(&state, "Garruk's Uprising");
    let (state, _) =
        process_command(state, empty_cast_spell_data(p(1), spell_id)).expect("cast should succeed");
    let hand_before = hand_count(&state, p(1));
    let (state, _) = pass_all(state, &[p(1), p(2)]);

    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.4: with a power-4+ creature controlled at queue time, the ETB \
         trigger should be on the stack"
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        hand_count(&state, p(1)),
        hand_before + 1,
        "exactly one card should have been drawn"
    );
}

// ── T3: Garruk's Uprising -- queue-time true, resolution-time false: no draw ──

/// CR 603.4 (ruling 2024-11-08, sentence 2): "If you don't control one as the
/// ability resolves, you don't draw a card." The qualifying creature is
/// present when the ETB trigger queues, then destroyed before the trigger
/// resolves. Pre-fix (see module doc): the resolution-time re-check also read
/// `None`, so the draw happened unconditionally.
#[test]
fn test_dx3_garruks_uprising_creature_leaves_before_resolution_no_draw() {
    let def = card_def("Garruk's Uprising");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_in_hand(p(1), &def))
        .object(ObjectSpec::creature(p(1), "Fleeting Beast", 4, 4))
        // A real library card, so that IF a draw were to wrongly happen, it
        // would be observable rather than silently no-op-ing on an empty
        // library (gotcha: "DrawCards on empty library is silently a no-op").
        .object(ObjectSpec::creature(p(1), "Library Filler", 1, 1).in_zone(ZoneId::Library(p(1))))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn_mut().priority_holder = Some(p(1));

    let spell_id = find_object(&state, "Garruk's Uprising");
    let (state, _) =
        process_command(state, empty_cast_spell_data(p(1), spell_id)).expect("cast should succeed");
    let hand_before = hand_count(&state, p(1));
    let (mut state, _) = pass_all(state, &[p(1), p(2)]);

    assert_eq!(
        state.stack_objects().len(),
        1,
        "the trigger should have queued while the creature was still controlled"
    );

    let beast_id = find_object(&state, "Fleeting Beast");
    destroy_object(&mut state, p(1), beast_id);

    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        hand_count(&state, p(1)),
        hand_before,
        "CR 603.4: the qualifying creature left before the trigger resolved -- \
         no card should have been drawn"
    );
}

// ── T4: Garruk's Uprising -- the untouched third ability still fires ─────────

/// CR 603.4, regression guard: the third ability ("Whenever a creature you
/// control with power 4 or greater enters, draw a card") is untouched by this
/// batch's edit to the FIRST ability. Uses `check_triggers` directly on a
/// `PermanentEnteredBattlefield` event, mirroring the T9 pattern in
/// `pb_dp6_intervening_if_queue_time.rs`.
#[test]
fn test_dx3_garruks_uprising_third_ability_unaffected() {
    let def = card_def("Garruk's Uprising");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .object(ObjectSpec::creature(p(1), "New Beast", 4, 4))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Garruk's Uprising");
    let beast_id = find_object(&state, "New Beast");
    let events = vec![GameEvent::PermanentEnteredBattlefield {
        object_id: beast_id,
        player: p(1),
    }];
    let triggers = mtg_engine::rules::abilities::check_triggers(&state, &events);
    assert!(
        triggers.iter().any(|t| t.source == source_id),
        "the third ability (power-4+ ETB draw trigger) should still fire -- \
         untouched by this batch's edit to the first ability"
    );
}

// ── T5: Inventors' Fair -- upkeep with only 2 artifacts: no trigger, no life ──

/// CR 603.4 (ruling 2016-09-20 #1): "No player may take actions in a turn
/// before Inventors' Fair's triggered ability checks to see if it should
/// trigger. If you don't control three or more artifacts, it won't trigger."
/// Pre-fix (see module doc): the ability did not exist at all, so this
/// probe's negative assertion holds vacuously pre-fix. T6 disambiguates by
/// pinning the positive direction with 3 artifacts.
#[test]
fn test_dx3_inventors_fair_upkeep_two_artifacts_no_trigger() {
    let (state, _def) = inventors_fair_fixture(2, Step::Untap);
    assert_eq!(
        artifact_count_on_battlefield(&state, p(1)),
        2,
        "sanity: exactly 2 artifacts controlled"
    );
    let life_before = state.players()[&p(1)].life_total;

    let state = advance_to_step(state, Step::Upkeep);
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.4: with only 2 artifacts controlled, the upkeep trigger must \
         not be queued; stack is {:?}",
        state.stack_objects()
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        state.players()[&p(1)].life_total,
        life_before,
        "no life should have been gained"
    );
}

// ── T6: Inventors' Fair -- upkeep with 3 artifacts: queues, resolves, +1 life ─

/// Non-regression positive direction, and the disambiguator for T5: proves
/// the upkeep trigger genuinely exists and genuinely fires when the
/// condition is true, so T5's "no trigger" cannot be passing merely because
/// the ability is absent.
#[test]
fn test_dx3_inventors_fair_upkeep_three_artifacts_gains_life() {
    let (state, _def) = inventors_fair_fixture(3, Step::Untap);
    assert_eq!(
        artifact_count_on_battlefield(&state, p(1)),
        3,
        "sanity: exactly 3 artifacts controlled"
    );
    let life_before = state.players()[&p(1)].life_total;

    let state = advance_to_step(state, Step::Upkeep);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.4: with 3 artifacts controlled, the upkeep trigger should be \
         on the stack"
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        state.players()[&p(1)].life_total,
        life_before + 1,
        "CR 603.4 / ruling 2016-09-20 #2: exactly 1 life should have been gained"
    );
}

// ── T7: Inventors' Fair -- 3 at queue time, 2 at resolution: no life gained ──

/// CR 603.4 (ruling 2016-09-20 #2): "If you don't control three artifacts at
/// that time [as the ability resolves], you won't gain life." The trigger
/// queues genuinely (3 artifacts), then one artifact is destroyed before the
/// trigger resolves. Pre-fix (see module doc): this probe was unreachable for
/// the stated reason -- there was no trigger to queue at all.
#[test]
fn test_dx3_inventors_fair_artifact_leaves_before_resolution_no_life() {
    let (mut state, _def) = inventors_fair_fixture(3, Step::Untap);
    let life_before = state.players()[&p(1)].life_total;

    let state_after_queue = advance_to_step(state, Step::Upkeep);
    assert_eq!(
        state_after_queue.stack_objects().len(),
        1,
        "the trigger should have queued while 3 artifacts were controlled"
    );
    state = state_after_queue;

    // Remove one artifact -- down to 2, which fails the resolution-time re-check.
    let artifact_id = find_object(&state, "Artifact 0");
    destroy_object(&mut state, p(1), artifact_id);
    assert_eq!(
        artifact_count_on_battlefield(&state, p(1)),
        2,
        "sanity: only 2 artifacts remain"
    );

    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        state.players()[&p(1)].life_total,
        life_before,
        "CR 603.4: only 2 artifacts remained at resolution -- no life should \
         have been gained"
    );
}

// ── T8: Inventors' Fair -- activation with 2 artifacts is illegal ────────────

/// CR 602.5b "Activate only if [condition]" / ruling 2016-09-20 #3. Pre-fix
/// (see module doc): `activation_condition: None` permitted the activation
/// unconditionally, so `process_command` returned `Ok`. The error message is
/// asserted, not bare `is_err()` -- a wrong ability index or an unpayable
/// cost would also error (PB-DX2's vacuity-discipline lesson).
#[test]
fn test_dx3_inventors_fair_activation_rejected_below_threshold() {
    let (mut state, _def) = inventors_fair_fixture(2, Step::PreCombatMain);
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);

    let source_id = find_object(&state, "Inventors' Fair");
    let err = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect_err("activation with only 2 artifacts must be rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("activation condition not met"),
        "the rejection must name the activation condition (CR 602.5b), not \
         merely be some error; got {msg}"
    );
}

// ── T9: Inventors' Fair -- legal activation resolves end-to-end (CR 608.2d) ──

/// The search ability activated with 3 artifacts controlled: the activation
/// is accepted, the land is sacrificed as part of the COST (CR 602.2c, before
/// resolution), and the ability resolves through PB-DP9's `EffectChoiceRequired`
/// / `Command::AnswerEffectChoice` channel with the ANNOUNCED artifact (not
/// the lowest `ObjectId`) landing in hand.
#[test]
fn test_dx3_inventors_fair_search_ability_end_to_end() {
    let def = card_def("Inventors' Fair");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        // Two artifact cards in the library so the search offers a real choice.
        .object(ObjectSpec::artifact(p(1), "Library Alpha").in_zone(ZoneId::Library(p(1))))
        .object(ObjectSpec::artifact(p(1), "Library Beta").in_zone(ZoneId::Library(p(1))));
    for i in 0..3u32 {
        builder = builder.object(ObjectSpec::artifact(p(1), &format!("Artifact {i}")));
    }
    let mut state = builder
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state.turn_mut().priority_holder = Some(p(1));

    let source_id = find_object(&state, "Inventors' Fair");
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("activation with 3 artifacts should be accepted");

    // CR 602.2c: sacrifice is paid as part of the COST, at activation time --
    // before the ability has resolved at all.
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Inventors' Fair"
                && o.zone == ZoneId::Graveyard(p(1))),
        "Inventors' Fair should already be sacrificed (in the graveyard) right \
         after activation, before the ability resolves"
    );

    let (state, _) = pass_all(state, &[p(1), p(2)]);
    let entry = state
        .pending_effect_choice()
        .expect("CR 608.2d: the search should block for a player choice");
    let candidates = match &entry.question {
        EffectChoiceQuestion::SearchLibrary { candidates, .. } => candidates.clone(),
        other => panic!("expected a search question, got {other:?}"),
    };
    assert_eq!(candidates.len(), 2, "both library artifacts should qualify");
    let announced = candidates[1];
    let announced_name = state
        .objects()
        .get(&announced)
        .map(|o| o.characteristics.name.clone())
        .unwrap();
    let player = entry.player;
    let choice_id = entry.choice_id;
    let (state, _) = process_command(
        state,
        Command::AnswerEffectChoice {
            player,
            choice_id,
            answer: EffectChoiceAnswer::SearchLibrary {
                found: Some(announced),
            },
        },
    )
    .expect("the announced answer should be accepted");

    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == announced_name && o.zone == ZoneId::Hand(p(1))),
        "the ANNOUNCED artifact ({announced_name}) should be in hand"
    );
    assert!(
        state.stack_objects().is_empty(),
        "the ability should have finished resolving"
    );
}

// ── T10: Inventors' Fair -- no resolution-time re-check on the search ability ─

/// CR 602.5b + ruling 2016-09-20 #3: "the number of artifacts you control is
/// checked only as you activate it. It's not checked again as the ability
/// resolves." Activate legally with 3 artifacts, then remove two of them
/// before the ability resolves -- the search must still happen (the question
/// must still be asked). Guards against a "fix" that wraps the search effect
/// in `Effect::Conditional`, which would be wrong per the ruling.
#[test]
fn test_dx3_inventors_fair_no_resolution_time_recheck_on_search() {
    let def = card_def("Inventors' Fair");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .object(ObjectSpec::artifact(p(1), "Library Only").in_zone(ZoneId::Library(p(1))));
    for i in 0..3u32 {
        builder = builder.object(ObjectSpec::artifact(p(1), &format!("Artifact {i}")));
    }
    let mut state = builder
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state.turn_mut().priority_holder = Some(p(1));

    let source_id = find_object(&state, "Inventors' Fair");
    let (mut state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("activation with 3 artifacts should be accepted");

    // Remove two of the three artifacts -- down to 1, well below the
    // threshold -- while the ability sits on the stack.
    for i in 0..2u32 {
        let artifact_id = find_object(&state, &format!("Artifact {i}"));
        destroy_object(&mut state, p(1), artifact_id);
    }
    assert_eq!(
        artifact_count_on_battlefield(&state, p(1)),
        1,
        "sanity: only 1 artifact remains"
    );

    let (state, _) = pass_all(state, &[p(1), p(2)]);
    assert!(
        state.pending_effect_choice().is_some(),
        "ruling 2016-09-20 #3: the count is checked ONLY at activation -- the \
         search must still ask, even though the count has since dropped below \
         the threshold"
    );
    match &state.pending_effect_choice().unwrap().question {
        EffectChoiceQuestion::SearchLibrary { candidates, .. } => {
            assert_eq!(
                candidates.len(),
                1,
                "the one library artifact still qualifies"
            );
        }
        other => panic!("expected a search question, got {other:?}"),
    }
}
