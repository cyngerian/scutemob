//! PB-DX24 (`OOS-DX1-3` + `OOS-DX1-4`): the lowering drops `trigger_zone`, and the
//! queue/resolution index spaces disagree.
//!
//! `nether_traitor.rs` pairs `TriggerCondition::WheneverCreatureDies` with
//! `trigger_zone: Some(TriggerZone::Graveyard)` (CR 113.6b / CR 113.6m — the
//! ability functions ONLY from the graveyard, because its effect moves the card
//! out of the graveyard and its trigger condition does not put it there). Before
//! this batch, `build_face_ability_vectors` (`crates/engine/src/testing/
//! replay_harness.rs`) lowered the ability onto the BATTLEFIELD object anyway
//! (swallowing `trigger_zone` through a `..` rest pattern), and
//! `collect_graveyard_carddef_triggers` (`crates/engine/src/rules/abilities.rs`)
//! had no dispatch arm for `WheneverCreatureDies` at all — so the ability fired
//! from the wrong zone (battlefield) and never fired from the right one
//! (graveyard). See `memory/primitives/pb-plan-DX24.md` and
//! `memory/primitives/pb-DX24-stage0.md`.
//!
//! T1 and T7 (this file, stage 2) are the mandatory fail-before probes for the
//! LOWERING half (Change 1/2). T2-T6 (stage 4) cover the graveyard DISPATCH
//! half (Change 3). T10-family (stage 5) covers the index-space fixes
//! (Change 4, OOS-DX1-4).

use mtg_engine::rules::abilities::check_triggers;
use mtg_engine::state::stubs::PendingTriggerKind;
use mtg_engine::testing::replay_harness::build_face_ability_vectors;
use mtg_engine::{
    all_cards, check_and_apply_sbas, enrich_spec_from_def, process_command, AltCostKind,
    AttackTarget, CardDefinition, CardFace, CardId, CardRegistry, CardType, CastSpellData, Color,
    CombatDamageAssignment, CombatDamageTarget, Command, CounterType, Effect, EffectAmount,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ManaPool,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, SubType, TriggerCondition, TriggerEvent,
    TypeLine, ZoneId,
};
use std::collections::HashMap;

#[allow(unused_imports)]
use mtg_card_types::cards::card_definition::{
    AbilityDefinition as CardDefAbilityDefinition, TriggerZone,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    let cards = all_cards();
    cards.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{name}' not found in state"))
}

/// Find an object in the graveyard by name -- used to pick up a just-died
/// creature's NEW `ObjectId` (CR 400.7) after `check_and_apply_sbas`.
fn find_in_graveyard(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name && matches!(o.zone, ZoneId::Graveyard(_)))
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{name}' not found in any graveyard"))
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

/// Re-derive Nether Traitor's `WheneverCreatureDies` ability's CARD-DEF index
/// from `all_cards()` -- never hard-coded (the plan's own instruction, T2).
fn nether_traitor_death_ability_index(defs: &HashMap<String, CardDefinition>) -> usize {
    let def = defs.get("Nether Traitor").unwrap();
    def.abilities
        .iter()
        .position(|a| {
            matches!(
                a,
                CardDefAbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WheneverCreatureDies { .. },
                    ..
                }
            )
        })
        .expect("Nether Traitor must have a WheneverCreatureDies ability")
}

/// Pass priority for all listed players once, accumulating events. Mirrors
/// `pb_ef10_sacrifice_driven_amounts.rs::pass_all`.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    let (current, pump_events) =
        mtg_engine::testing::replay_harness::auto_answer_blocking_decisions(current);
    all_events.extend(pump_events);
    (current, all_events)
}

/// Drain the stack completely (repeated `pass_all` rounds).
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        let (s, _) = pass_all(state, players);
        state = s;
        guard += 1;
        assert!(
            guard < 20,
            "drain_stack: stack did not empty after 20 rounds"
        );
    }
    state
}

// ── T1: Nether Traitor must NOT trigger from the battlefield ────────────────

/// CR 113.6 / CR 113.6m — Nether Traitor's ability moves the card OUT of the
/// graveyard, and its trigger condition does not put it into the graveyard, so
/// CR 113.6m confines the ability to functioning only from the graveyard. A
/// Nether Traitor sitting on the BATTLEFIELD when another creature dies must
/// produce ZERO pending triggers sourced at it.
///
/// Non-vacuity: the SAME death must produce >=1 trigger from a real
/// battlefield-scoped death-watcher (Blood Artist) in the SAME fixture, so a
/// test that fires nothing at all cannot pass.
#[test]
fn test_dx24_nether_traitor_does_not_trigger_from_the_battlefield() {
    let p1 = p(1);
    let defs = load_defs();
    let defs_vec: Vec<CardDefinition> = defs.values().cloned().collect();

    let nether_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Nether Traitor").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let blood_artist_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Blood Artist").in_zone(ZoneId::Battlefield),
        &defs,
    );
    // Fodder: a vanilla creature with toughness 0 that dies to SBA 704.5f the
    // moment check_and_apply_sbas runs.
    let fodder_spec = ObjectSpec::creature(p1, "Fodder", 1, 0).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(defs_vec))
        .object(nether_spec)
        .object(blood_artist_spec)
        .object(fodder_spec)
        .build()
        .unwrap();

    let nether_id = find_by_name(&state, "Nether Traitor");
    let blood_artist_id = find_by_name(&state, "Blood Artist");

    check_and_apply_sbas(&mut state);

    let nether_triggers: Vec<_> = state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == nether_id)
        .collect();
    assert!(
        nether_triggers.is_empty(),
        "CR 113.6 / CR 113.6m: Nether Traitor on the BATTLEFIELD must not trigger \
         when another creature dies -- the ability functions only from the \
         graveyard. Got {} trigger(s) sourced at the battlefield object: {:?}",
        nether_triggers.len(),
        nether_triggers
    );

    let blood_artist_triggers: Vec<_> = state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == blood_artist_id)
        .collect();
    assert!(
        !blood_artist_triggers.is_empty(),
        "non-vacuity: Blood Artist's ordinary battlefield death-watcher trigger \
         must fire on the SAME event batch, or this fixture proves nothing. Got \
         {} trigger(s).",
        blood_artist_triggers.len()
    );
}

// ── T7: differential over the whole corpus ───────────────────────────────────

/// CR 113.6b / CR 113.6m — structural probe: for EVERY card in `all_cards()`,
/// lowering `def.abilities` through `build_face_ability_vectors` must produce
/// the IDENTICAL `triggered_abilities` vector as lowering `def.abilities` with
/// every `trigger_zone: Some(_)` ability manually stripped out first. If any
/// lowering arm swallows `trigger_zone` (installs a zone-scoped ability onto
/// the battlefield object's runtime trigger vector), the two vectors diverge
/// and this test names the card.
///
/// Non-vacuity: at least one def (in fact, exactly the 3-def `trigger_zone:
/// Some(_)` population measured at stage 1) must have a non-identity INPUT
/// under the removal, and at least one of those must have a non-empty
/// DIFFERENCE today -- otherwise the differential is trivially true.
#[test]
fn test_dx24_lowering_drops_every_zone_scoped_ability_over_the_corpus() {
    use mtg_card_types::cards::card_definition::AbilityDefinition;

    let cards = all_cards();
    let mut non_identity_inputs = 0usize;
    let mut divergent_defs: Vec<String> = Vec::new();

    for def in &cards {
        let filtered: Vec<AbilityDefinition> = def
            .abilities
            .iter()
            .filter(|a| {
                !matches!(
                    a,
                    AbilityDefinition::Triggered {
                        trigger_zone: Some(_),
                        ..
                    }
                )
            })
            .cloned()
            .collect();
        if filtered.len() != def.abilities.len() {
            non_identity_inputs += 1;
        }

        let (_, _, full_triggered) = build_face_ability_vectors(&def.abilities);
        let (_, _, filtered_triggered) = build_face_ability_vectors(&filtered);

        if full_triggered != filtered_triggered {
            divergent_defs.push(def.name.clone());
        }

        // Fix cycle (review Finding 10): the ORIGINAL version fed ONLY
        // `def.abilities` into this differential -- `build_face_ability_vectors`
        // is called on the BACK face too (`face.rs:104`'s rebuild,
        // `resolution.rs:888`'s disturb rebuild), so a back-face
        // `trigger_zone: Some(_)` ability was never differentiated here.
        if let Some(back) = &def.back_face {
            let back_filtered: Vec<AbilityDefinition> = back
                .abilities
                .iter()
                .filter(|a| {
                    !matches!(
                        a,
                        AbilityDefinition::Triggered {
                            trigger_zone: Some(_),
                            ..
                        }
                    )
                })
                .cloned()
                .collect();
            if back_filtered.len() != back.abilities.len() {
                non_identity_inputs += 1;
            }
            let (_, _, back_full_triggered) = build_face_ability_vectors(&back.abilities);
            let (_, _, back_filtered_triggered) = build_face_ability_vectors(&back_filtered);
            if back_full_triggered != back_filtered_triggered {
                divergent_defs.push(format!("{} (back face)", def.name));
            }
        }
    }

    assert!(
        non_identity_inputs >= 1,
        "non-vacuity: at least one corpus def must carry a `trigger_zone: Some(_)` \
         ability (measured at PB-DX24 stage 1: 3 defs -- Bloodghast, Squee Goblin \
         Nabob, Nether Traitor), or this differential exercises nothing. This floor \
         holds REGARDLESS of whether the lowering is fixed -- it only asserts that \
         removal actually shrinks at least one def's input."
    );
    // Non-vacuity note (fail-before record, not re-asserted here): stage 2 of
    // PB-DX24 watched this test fail on the unmodified tree with
    // divergent_defs == ["Nether Traitor"] -- the filter removal reproduces
    // that observation (see the revert recipe in the assertion message below
    // and memory/primitives/pb-DX24-execution-notes.md). Asserting
    // "divergent_defs must be non-empty" HERE would make this test
    // permanently red after the fix it exists to gate, which is the opposite
    // of its purpose.
    assert!(
        divergent_defs.is_empty(),
        "CR 113.6b / CR 113.6m: build_face_ability_vectors must lower a def's \
         abilities IDENTICALLY whether or not its trigger_zone: Some(_) abilities \
         are present in the input -- a lowering arm is installing a zone-scoped \
         ability onto the battlefield object's runtime trigger vector. Divergent \
         defs: {:?}",
        divergent_defs
    );
}

// ── Stage 4 shared fixture ───────────────────────────────────────────────────

/// Nether Traitor already in `p1`'s graveyard, with `black_mana` floating in
/// `p1`'s pool, and a vanilla "Fodder" creature (controlled by
/// `fodder_controller`, toughness 0 so SBA 704.5f kills it on the next
/// `check_and_apply_sbas`/`pass_all`).
fn build_nether_in_graveyard_fixture(
    defs: &HashMap<String, CardDefinition>,
    black_mana: u32,
    fodder_controller: PlayerId,
) -> GameState {
    let p1 = p(1);
    let nether_card_id = defs.get("Nether Traitor").unwrap().card_id.clone();
    let nether_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Nether Traitor")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(nether_card_id),
        defs,
    );
    let fodder_spec =
        ObjectSpec::creature(fodder_controller, "Fodder", 1, 0).in_zone(ZoneId::Battlefield);
    let defs_vec: Vec<CardDefinition> = defs.values().cloned().collect();
    GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(defs_vec))
        .player_mana(
            p1,
            ManaPool {
                black: black_mana,
                ..Default::default()
            },
        )
        .object(nether_spec)
        .object(fodder_spec)
        .build()
        .unwrap()
}

// ── T2: Nether Traitor triggers from the graveyard ───────────────────────────

/// CR 603.6c / CR 113.6b / CR 108.4a — a creature `p1` controls dies while
/// Nether Traitor is in `p1`'s OWN graveyard: exactly one trigger, sourced at
/// the graveyard object, `kind == CardDefETB`, `controller == p1` (the card's
/// OWNER, per CR 108.4a -- a graveyard card has no controller), keyed on
/// `AnyCreatureDies`, carrying the dying creature's new graveyard id, at the
/// ability's real card-def index (re-derived, never hard-coded).
#[test]
fn test_dx24_nether_traitor_triggers_from_the_graveyard() {
    let p1 = p(1);
    let defs = load_defs();
    let expected_ability_index = nether_traitor_death_ability_index(&defs);

    let mut state = build_nether_in_graveyard_fixture(&defs, 1, p1);
    let nether_gy_id = find_by_name(&state, "Nether Traitor");

    check_and_apply_sbas(&mut state);

    let fodder_new_id = find_in_graveyard(&state, "Fodder");

    let nether_triggers: Vec<_> = state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == nether_gy_id)
        .collect();
    assert_eq!(
        nether_triggers.len(),
        1,
        "CR 603.6c / CR 113.6b: Nether Traitor in the GRAVEYARD must trigger \
         exactly once when a creature its owner controls dies. Got {} \
         trigger(s): {:?}",
        nether_triggers.len(),
        nether_triggers
    );
    let t = nether_triggers[0];
    assert_eq!(
        t.kind,
        PendingTriggerKind::CardDefETB,
        "kind must be CardDefETB"
    );
    assert_eq!(
        t.controller, p1,
        "CR 108.4a: a graveyard card's controller is its owner"
    );
    assert_eq!(
        t.triggering_event,
        Some(TriggerEvent::AnyCreatureDies),
        "must dispatch through AnyCreatureDies"
    );
    assert_eq!(
        t.entering_object_id,
        Some(fodder_new_id),
        "must carry the dying creature's NEW graveyard ObjectId (CR 400.7)"
    );
    assert_eq!(
        t.ability_index, expected_ability_index,
        "ability_index must be the CARD-DEF index of the WheneverCreatureDies \
         ability, re-derived from all_cards() rather than hard-coded"
    );
}

// ── T3: end-to-end return, with a paired no-mana negative ───────────────────

/// CR 603.3 / CR 603.3a / CR 118.12 (`MayPayThenEffect`) — driven through
/// `process_command`: a creature dies, the trigger flushes, resolves, and (if
/// `{B}` is available) Nether Traitor returns to the battlefield. The paired
/// negative -- zero black mana available -- proves the assertion discriminates
/// the RETURN, not merely the trigger firing.
///
/// Fix cycle (review Finding 9): CR 118.12 itself makes this a PLAYER CHOICE
/// ("checks whether the player CHOSE to pay an optional cost") -- it does not
/// say "pay when able." The engine's OWN deviation is pay-when-able,
/// documented at its one implementation site
/// (`effects/mod.rs:4299-4301`, `try_pay_optional_cost`) as a deliberate M7
/// simplification, not a CR requirement; `engine.rs:1568` already names this
/// the "DP-19 (`MayPayThenEffect`) bug class." This test pins THAT engine
/// deviation, not CR 118.12 -- see `OOS-DX24-9`.
#[test]
fn test_dx24_nether_traitor_returns_itself_end_to_end() {
    let p1 = p(1);
    let all = [p(1), p(2), p(3), p(4)];
    let defs = load_defs();

    // Positive: 1 black mana floating. Fodder already has toughness 0 at build
    // time, so it must be killed via a DIRECT check_and_apply_sbas +
    // flush_pending_triggers pair, NOT via a priority-pass round: an initial
    // pass_all with an empty stack would hit CR 500.4's step-advance branch
    // BEFORE the SBA check that kills Fodder ever runs (SBA is checked inside
    // enter_step, not on every PassPriority), clearing the mana pool a step
    // early and starving the very payment this test exists to observe.
    // Confirmed by a throwaway debug trace during authoring: without this,
    // Nether Traitor's OWN {B} showed 0 already at MayPayThenEffect time.
    let mut state = build_nether_in_graveyard_fixture(&defs, 1, p1);
    check_and_apply_sbas(&mut state);
    mtg_engine::rules::abilities::flush_pending_triggers(&mut state);
    let state = drain_stack(state, &all);
    assert!(
        on_battlefield(&state, "Nether Traitor"),
        "engine deviation (NOT CR 118.12, which makes this a player choice --\
         see OOS-DX24-9): with {{B}} available, the engine's pay-when-able \
         MayPayThenEffect handler (effects/mod.rs:4299-4301) must pay and \
         return Nether Traitor to the battlefield"
    );
    assert!(
        !in_graveyard(&state, "Nether Traitor", p1),
        "Nether Traitor must have LEFT the graveyard"
    );

    // Negative: zero black mana floating -- the cost cannot be paid, so the
    // `then` arm (MoveZone to battlefield) never runs.
    let mut state_no_mana = build_nether_in_graveyard_fixture(&defs, 0, p1);
    check_and_apply_sbas(&mut state_no_mana);
    mtg_engine::rules::abilities::flush_pending_triggers(&mut state_no_mana);
    let state_no_mana = drain_stack(state_no_mana, &all);
    assert!(
        in_graveyard(&state_no_mana, "Nether Traitor", p1),
        "with NO black mana available, Nether Traitor must stay in the \
         graveyard -- the trigger fired but the optional cost went unpaid"
    );
    assert!(
        !on_battlefield(&state_no_mana, "Nether Traitor"),
        "Nether Traitor must NOT be on the battlefield when the cost went \
         unpaid"
    );
}

// ── T4: CR 603.10a simultaneity ──────────────────────────────────────────────

/// CR 603.10a + the Gatherer simultaneity ruling (plan §1.5) — Nether Traitor
/// and another creature dying in the SAME event batch must NOT trigger Nether
/// Traitor: immediately prior to the event it was on the battlefield, where
/// (CR 113.6m) the ability did not function. Non-vacuity: a second sub-case,
/// where Nether Traitor was ALREADY in the graveyard before the batch, fires
/// exactly one trigger from the SAME helper -- proving the fixture can fire.
#[test]
fn test_dx24_simultaneous_death_does_not_trigger() {
    let p1 = p(1);
    let defs = load_defs();
    let defs_vec: Vec<CardDefinition> = defs.values().cloned().collect();

    // Nether Traitor and Fodder BOTH die in the same SBA batch: both start on
    // the battlefield with toughness 0. `.with_card_id(...)` is REQUIRED even
    // though the object starts on the battlefield -- `move_object_to_zone`
    // carries `card_id` through the zone change, but `card_id` is never
    // auto-populated by `enrich_spec_from_def` for a battlefield object, and
    // `collect_graveyard_carddef_triggers` looks the def up by `card_id`; with
    // it unset, EVERY graveyard object in this fixture is silently invisible
    // to the dispatch (found by an intermediate debug trace during authoring:
    // pending_triggers stayed `[]` even with the look-back guard forcibly
    // disabled, which is what exposed this).
    let nether_card_id = defs.get("Nether Traitor").unwrap().card_id.clone();
    let nether_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Nether Traitor")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(nether_card_id),
        &defs,
    );
    let mut nether_spec = nether_spec;
    nether_spec.toughness = Some(0);
    let fodder_spec = ObjectSpec::creature(p1, "Fodder", 1, 0).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(defs_vec.clone()))
        .player_mana(
            p1,
            ManaPool {
                black: 1,
                ..Default::default()
            },
        )
        .object(nether_spec)
        .object(fodder_spec)
        .build()
        .unwrap();

    check_and_apply_sbas(&mut state);
    let nether_gy_id_simultaneous = find_in_graveyard(&state, "Nether Traitor");
    let simultaneous_triggers: Vec<_> = state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == nether_gy_id_simultaneous)
        .collect();
    assert!(
        simultaneous_triggers.is_empty(),
        "CR 603.10a: Nether Traitor and another creature dying in the SAME \
         batch must NOT trigger Nether Traitor -- it was on the battlefield \
         immediately prior. Got {} trigger(s): {:?}",
        simultaneous_triggers.len(),
        simultaneous_triggers
    );

    // Non-vacuity: Nether Traitor ALREADY in the graveyard (a prior, separate
    // batch), then a creature dies -- must fire exactly once.
    let mut state2 = build_nether_in_graveyard_fixture(&defs, 1, p1);
    let nether_gy_id_already = find_by_name(&state2, "Nether Traitor");
    check_and_apply_sbas(&mut state2);
    let already_triggers: Vec<_> = state2
        .pending_triggers()
        .iter()
        .filter(|t| t.source == nether_gy_id_already)
        .collect();
    assert_eq!(
        already_triggers.len(),
        1,
        "non-vacuity: Nether Traitor ALREADY in the graveyard before the \
         batch must fire exactly once when a controlled creature dies -- the \
         same helper must be capable of firing, or the simultaneity assertion \
         above proves nothing."
    );
}

// ── T5: exclude_self compares the GRAVEYARD identity ─────────────────────────

/// CR 400.7 / CR 603.10a — Nether Traitor dying ALONE (no other creature) must
/// not trigger itself.
///
/// **Finding, recorded honestly per the plan's runner obligations**: the
/// plan's §3.3 table predicted this test's revert (comparing `exclude_self`
/// against `pre_death_id` alone, dropping `new_grave_id`) would "fire,
/// because the two id spaces never meet." Executing that revert (during
/// stage 4 authoring) did NOT redden this test -- it stayed green. Root
/// cause, proven rather than argued: for a GRAVEYARD-dispatched
/// `WheneverCreatureDies` trigger, `new_grave_id == obj_id` can only be true
/// when the trigger's OWN source is the object that just died THIS batch --
/// and `arrived_in_graveyard_this_batch` (CR 603.10a look-back, T4) is built
/// from the SAME `events` slice `collect_graveyard_carddef_triggers` is
/// invoked per-event from, so that exact id is ALWAYS already a member of
/// the look-back set. The two guards are therefore logically overlapping for
/// every state reachable through the public API: there is no game state
/// where dropping the `new_grave_id` comparison changes observable behavior,
/// for THIS corpus's only `exclude_self: true` graveyard-scoped card. The
/// `new_grave_id` comparison (`abilities.rs`, the `WheneverCreatureDies` arm
/// of `collect_graveyard_carddef_triggers`) is kept anyway -- it is still the
/// CR 400.7-correct comparison (a `pre_death_id`-only comparison is silently
/// wrong in principle, matching the ETB arm's identical "moot but kept for
/// symmetry" comment) and it is defense-in-depth against a future narrowing
/// of the look-back guard's scope. This test therefore verifies the
/// OBSERVABLE outcome (self-death does not self-trigger), not the id-space
/// choice in isolation -- the id-space choice is a source-level fact, verified
/// by reading `abilities.rs`, not by an integration-level revert.
#[test]
fn test_dx24_exclude_self_compares_the_graveyard_identity() {
    let p1 = p(1);
    let defs = load_defs();
    let nether_card_id = defs.get("Nether Traitor").unwrap().card_id.clone();
    let defs_vec: Vec<CardDefinition> = defs.values().cloned().collect();

    // Nether Traitor itself is the dying object: it starts on the battlefield
    // with toughness 0, dies via SBA, and its OWN new_grave_id is what
    // exclude_self must catch. `.with_card_id(...)` is REQUIRED (see T4's
    // comment for why -- without it the graveyard object is invisible to
    // collect_graveyard_carddef_triggers and this test passes VACUOUSLY).
    let mut nether_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Nether Traitor")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(nether_card_id.clone()),
        &defs,
    );
    nether_spec.toughness = Some(0);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(defs_vec))
        .player_mana(
            p1,
            ManaPool {
                black: 1,
                ..Default::default()
            },
        )
        .object(nether_spec)
        .build()
        .unwrap();

    check_and_apply_sbas(&mut state);
    let nether_gy_id = find_in_graveyard(&state, "Nether Traitor");
    let self_triggers: Vec<_> = state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == nether_gy_id)
        .collect();
    assert!(
        self_triggers.is_empty(),
        "CR 400.7 / exclude_self: Nether Traitor dying by itself (no OTHER \
         creature) must not trigger itself. Got {} trigger(s): {:?}",
        self_triggers.len(),
        self_triggers
    );
}

// ── T6: graveyard death filters mirror the battlefield path ─────────────────

/// CR 108.4a / CR 111.7 / CR 603.10a / CR 613.1d — (a) an OPPONENT's creature
/// dying does not trigger Nether Traitor (`controller: Some(You)`), with its
/// positive counterpart (a creature P1 CONTROLS dying DOES trigger it, proven
/// by T2 already, so (a) here only needs the negative half plus a same-fixture
/// positive control to prove the fixture can fire at all).
#[test]
fn test_dx24_graveyard_death_filters_mirror_the_battlefield_path() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let nether_card_id = defs.get("Nether Traitor").unwrap().card_id.clone();
    let defs_vec: Vec<CardDefinition> = defs.values().cloned().collect();

    // (a) negative: an OPPONENT's creature dies -- must NOT trigger.
    let nether_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Nether Traitor")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(nether_card_id.clone()),
        &defs,
    );
    let opponent_fodder = ObjectSpec::creature(p2, "Fodder", 1, 0).in_zone(ZoneId::Battlefield);
    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(defs_vec.clone()))
        .player_mana(
            p1,
            ManaPool {
                black: 1,
                ..Default::default()
            },
        )
        .object(nether_spec)
        .object(opponent_fodder)
        .build()
        .unwrap();
    let nether_gy_id = find_by_name(&state, "Nether Traitor");
    check_and_apply_sbas(&mut state);
    let opp_triggers: Vec<_> = state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == nether_gy_id)
        .collect();
    assert!(
        opp_triggers.is_empty(),
        "CR 108.4a: an opponent's creature dying must NOT trigger Nether \
         Traitor (controller: Some(You) is an OWNER-scoped check via \
         death_controller vs owner). Got {} trigger(s): {:?}",
        opp_triggers.len(),
        opp_triggers
    );

    // (a) positive control, SAME fixture shape: p1's OWN creature dies.
    let (state_pos, nether_gy_id_pos) = {
        let nether_spec = enrich_spec_from_def(
            ObjectSpec::card(p1, "Nether Traitor")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(nether_card_id),
            &defs,
        );
        let own_fodder = ObjectSpec::creature(p1, "Fodder", 1, 0).in_zone(ZoneId::Battlefield);
        let mut s = GameStateBuilder::four_player()
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .with_registry(CardRegistry::new(defs_vec))
            .player_mana(
                p1,
                ManaPool {
                    black: 1,
                    ..Default::default()
                },
            )
            .object(nether_spec)
            .object(own_fodder)
            .build()
            .unwrap();
        let id = find_by_name(&s, "Nether Traitor");
        check_and_apply_sbas(&mut s);
        (s, id)
    };
    let pos_triggers: Vec<_> = state_pos
        .pending_triggers()
        .iter()
        .filter(|t| t.source == nether_gy_id_pos)
        .collect();
    assert_eq!(
        pos_triggers.len(),
        1,
        "non-vacuity: p1's OWN creature dying, same fixture shape, must fire \
         exactly once -- otherwise (a)'s negative proves nothing."
    );
}

/// Build a synthetic graveyard-scoped `WheneverCreatureDies` def (mirrors
/// `pb_ac7_ability_index_desync.rs`'s synthetic-def idiom) with `nontoken_only`
/// or `filter` set, so (b)/(c) can be tested without a corpus card carrying
/// them alongside `trigger_zone: Some(Graveyard)`.
fn synthetic_graveyard_watcher_def(
    name: &str,
    nontoken_only: bool,
    filter: Option<mtg_engine::TargetFilter>,
) -> CardDefinition {
    CardDefinition {
        card_id: mtg_engine::CardId(format!("dx24-synthetic-{name}")),
        name: name.to_string(),
        types: mtg_engine::cards::helpers::creature_types(&["Spirit"]),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![CardDefAbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverCreatureDies {
                controller: None,
                exclude_self: true,
                nontoken_only,
                filter,
            },
            effect: mtg_engine::Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: mtg_engine::EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: Some(TriggerZone::Graveyard),
        }],
        completeness: mtg_engine::Completeness::Complete,
        ..Default::default()
    }
}

/// CR 111.7 (`nontoken_only`) — a TOKEN dying must not trigger a graveyard
/// watcher whose `nontoken_only` is set; a NONTOKEN death (same fixture
/// shape) must.
#[test]
fn test_dx24_graveyard_death_filter_nontoken_only() {
    let p1 = p(1);
    let watcher_def = synthetic_graveyard_watcher_def("DX24 Nontoken Watcher", true, None);

    for (fodder_is_token, expect_fires) in [(true, false), (false, true)] {
        let watcher_spec = ObjectSpec::card(p1, "DX24 Nontoken Watcher")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(watcher_def.card_id.clone());
        let watcher_spec = enrich_spec_from_def(
            watcher_spec,
            &[("DX24 Nontoken Watcher".to_string(), watcher_def.clone())]
                .into_iter()
                .collect(),
        );
        let mut fodder_spec = ObjectSpec::creature(p1, "Fodder", 1, 0).in_zone(ZoneId::Battlefield);
        if fodder_is_token {
            fodder_spec = fodder_spec.token();
        }
        let mut state = GameStateBuilder::four_player()
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .with_registry(CardRegistry::new(vec![watcher_def.clone()]))
            .object(watcher_spec)
            .object(fodder_spec)
            .build()
            .unwrap();
        let watcher_id = find_by_name(&state, "DX24 Nontoken Watcher");
        check_and_apply_sbas(&mut state);
        let fired = state
            .pending_triggers()
            .iter()
            .any(|t| t.source == watcher_id);
        assert_eq!(
            fired, expect_fires,
            "CR 111.7: nontoken_only watcher, fodder_is_token={fodder_is_token} \
             -- expected fires={expect_fires}, got {fired}"
        );
    }
}

/// CR 613.1d (subtype `filter`) — a dying creature that does NOT match the
/// watcher's subtype filter must not trigger it; a matching subtype (same
/// fixture shape) must.
#[test]
fn test_dx24_graveyard_death_filter_subtype_filter() {
    let p1 = p(1);
    let filter = mtg_engine::TargetFilter {
        has_subtype: Some(mtg_engine::SubType("Zombie".to_string())),
        ..Default::default()
    };
    let watcher_def = synthetic_graveyard_watcher_def("DX24 Zombie Watcher", false, Some(filter));

    for (fodder_subtypes, expect_fires) in [
        (vec![mtg_engine::SubType("Human".to_string())], false),
        (vec![mtg_engine::SubType("Zombie".to_string())], true),
    ] {
        let watcher_spec = ObjectSpec::card(p1, "DX24 Zombie Watcher")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(watcher_def.card_id.clone());
        let watcher_spec = enrich_spec_from_def(
            watcher_spec,
            &[("DX24 Zombie Watcher".to_string(), watcher_def.clone())]
                .into_iter()
                .collect(),
        );
        let mut fodder_spec = ObjectSpec::creature(p1, "Fodder", 1, 0).in_zone(ZoneId::Battlefield);
        fodder_spec.subtypes = fodder_subtypes.clone();
        let mut state = GameStateBuilder::four_player()
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .with_registry(CardRegistry::new(vec![watcher_def.clone()]))
            .object(watcher_spec)
            .object(fodder_spec)
            .build()
            .unwrap();
        let watcher_id = find_by_name(&state, "DX24 Zombie Watcher");
        check_and_apply_sbas(&mut state);
        let fired = state
            .pending_triggers()
            .iter()
            .any(|t| t.source == watcher_id);
        assert_eq!(
            fired, expect_fires,
            "CR 613.1d: subtype-filtered watcher, fodder_subtypes={fodder_subtypes:?} \
             -- expected fires={expect_fires}, got {fired}"
        );
    }
}

// ── T10-family (stage 5): OOS-DX1-4 Q1/Q3/Q4/Q6 — the two index spaces ──────
//
// Q1/Q3/Q4/Q6 are genuinely reachable index-space disagreements (§4.1 of the
// plan): the queue side indexed `def.abilities` (front face always) while the
// resolution side indexes `def.effective_abilities(is_transformed)`
// (face-aware). §4.2's corpus measurement found 0 real cards exercising any
// of the 7 shapes on a back face, so every probe below uses a SYNTHETIC
// `CardDefinition`/`CardFace`, mirroring `pb_rs4_face_aware_residuals.rs`
// (the precedent for this whole half of the batch).
//
// Q2 (stack) and Q7 (graveyard) are DEFENSIVE fixes: §4.0 establishes that
// `is_transformed` is reachable-true on a BATTLEFIELD permanent only (set at
// exactly one production site, resolution.rs's disturb ETB), so a stack
// object or a graveyard object can never actually show the divergence a
// behavioral probe would need to discriminate -- reverting either site
// produces the SAME observable behavior post-revert as pre-revert, because
// `effective_abilities(false) == &self.abilities` by `effective_abilities`'s
// own `(_, _) => &self.abilities` match arm. Per the task's explicit
// allowance, these two are pinned at the STRUCTURAL level instead
// (`test_dx24_q2_and_q7_queue_sites_call_effective_abilities` below), plus a
// pin on the §4.0 invariant itself that the fix's "zero behaviour change"
// claim depends on.

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

// ── Q1: Backup, via a disturb DFC (the one production path that sets
//    is_transformed == true on an ENTERING permanent) ──────────────────────

/// A minimal Disturb DFC: front is a vanilla {W} 1/1 Human with Disturb and NO
/// Backup ability; the back face's ONLY ability is `Keyword(Backup(2))`.
fn q1_disturb_backup_def() -> CardDefinition {
    CardDefinition {
        card_id: cid("dx24-q1-backup-disturb"),
        name: "DX24 Q1 Backup Front".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Human".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![
            CardDefAbilityDefinition::Keyword(KeywordAbility::Disturb),
            CardDefAbilityDefinition::Disturb {
                cost: ManaCost {
                    white: 1,
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "DX24 Q1 Backup Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature, CardType::Enchantment]
                    .into_iter()
                    .collect(),
                subtypes: [SubType("Spirit".to_string())].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: String::new(),
            abilities: vec![CardDefAbilityDefinition::Keyword(KeywordAbility::Backup(2))],
            power: Some(3),
            toughness: Some(2),
            color_indicator: Some(vec![Color::White]),
        }),
        ..Default::default()
    }
}

fn empty_cast_spell_disturb(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
        alt_cost: Some(AltCostKind::Disturb),
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        face_down_kind: None,
        additional_costs: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }))
}

/// CR 702.165a / OOS-DX1-4 Q1: a disturb DFC whose BACK face is the only face
/// declaring `Backup(2)` must fire its ETB Backup trigger once it enters
/// back-face-up -- Q1's queue site must read the SAME face the permanent is
/// actually showing, not always the front `def.abilities`.
///
/// The pushed trigger's default target is the source itself (abilities.rs's
/// `ETBBackup { target: *object_id, .. }`), so a FIXED queue site puts
/// exactly 2 +1/+1 counters on the entered permanent once the trigger
/// resolves; a BROKEN queue site (reading the front list, which has no
/// Backup ability at all) queues nothing, so the permanent ends with 0
/// counters. Revert (restore `def.abilities` at the Q1 site): counters == 0.
#[test]
fn test_dx24_backup_lowering_reads_the_visible_face_of_a_disturbed_dfc() {
    let p1 = p(1);
    let p2 = p(2);
    let def = q1_disturb_backup_def();
    let registry = CardRegistry::new(vec![def.clone()]);

    let mut spec = ObjectSpec::card(p1, &def.name)
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(def.card_id.clone())
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Disturb)
        .with_mana_cost(ManaCost {
            white: 1,
            ..Default::default()
        });
    spec.power = Some(1);
    spec.toughness = Some(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let card_id = def.card_id.clone();
    let beggar_id = find_by_name(&state, &def.name);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = process_command(state, empty_cast_spell_disturb(p1, beggar_id))
        .unwrap_or_else(|e| panic!("cast with disturb should succeed: {:?}", e));
    let state = drain_stack(state, &[p1, p2]);

    let entered_id = state
        .objects()
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.card_id == Some(card_id.clone())
                && obj.is_transformed
        })
        .map(|(id, _)| *id)
        .expect("back face should be on the battlefield");
    // Resolving the Backup trigger's KeywordTrigger stack object requires
    // another drain pass (the ETB trigger itself resolves onto the stack
    // separately from the permanent it came from).
    let state = drain_stack(state, &[p1, p2]);

    let counters = state.objects()[&entered_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 2,
        "CR 702.165a / OOS-DX1-4 Q1: the back face's Backup(2) ETB trigger must \
         fire and put 2 +1/+1 counters on the entered (back-face) permanent -- \
         Q1's queue site must read the visible (back) face, not always \
         def.abilities. Got {counters} counters."
    );
}

// ── Q3: WhenExertedAsAttacks, via Command::Transform on a battlefield DFC ───

fn q3_exert_dfc_def() -> CardDefinition {
    CardDefinition {
        card_id: cid("dx24-q3-exert-transform"),
        name: "DX24 Q3 Front".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Transform".to_string(),
        abilities: vec![CardDefAbilityDefinition::Keyword(KeywordAbility::Transform)],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "DX24 Q3 Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Horror".to_string())].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "You may exert this creature as it attacks. When you do, \
                           you gain 7 life."
                .to_string(),
            abilities: vec![
                CardDefAbilityDefinition::Keyword(KeywordAbility::Exert),
                CardDefAbilityDefinition::Triggered {
                    once_per_turn: false,
                    trigger_condition: TriggerCondition::WhenExertedAsAttacks,
                    intervening_if: None,
                    effect: Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(7),
                    },
                    targets: vec![],
                    modes: None,
                    trigger_zone: None,
                },
            ],
            power: Some(4),
            toughness: Some(4),
            color_indicator: Some(vec![Color::Black]),
        }),
        ..Default::default()
    }
}

/// CR 701.43d / OOS-DX1-4 Q3: the BACK face's ONLY ability is `Exert` +
/// `WhenExertedAsAttacks`. Once the permanent is `Command::Transform`ed, an
/// attack declared with `exert_choices: [obj_id]` is legal ONLY because
/// `calculate_characteristics` (layer-resolved, already face-aware -- an
/// EARLIER, unrelated mechanism) reports the back face's `Exert` keyword; the
/// LINKED trigger itself is Q3's own subject. Revert (restore `def.abilities`
/// at the Q3 site): the back face's `WhenExertedAsAttacks` ability is never
/// found (front declares none), so no life is gained.
#[test]
fn test_dx24_when_exerted_as_attacks_reads_the_visible_face_of_a_transformed_attacker() {
    let p1 = p(1);
    let p2 = p(2);
    let def = q3_exert_dfc_def();
    let mut defs = HashMap::new();
    defs.insert(def.name.clone(), def.clone());
    let registry = CardRegistry::new(vec![def.clone()]);

    let spec = enrich_spec_from_def(
        ObjectSpec::card(p1, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let obj_id = find_by_name(&state, &def.name);
    let (state, _) = process_command(
        state,
        Command::Transform {
            player: p1,
            permanent: obj_id,
        },
    )
    .expect("Transform should succeed");
    assert!(state.objects()[&obj_id].is_transformed);

    let life_before = state.players()[&p1].life_total;
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(obj_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![obj_id],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("declare attackers with exert should succeed (back face declares Exert)");
    let state = drain_stack(state, &[p1, p2]);

    let life_after = state.players()[&p1].life_total;
    assert_eq!(
        life_after,
        life_before + 7,
        "CR 701.43d / OOS-DX1-4 Q3: the back face's WhenExertedAsAttacks \
         trigger must fire and gain 7 life -- Q3's queue site must read the \
         visible (back) face. life_before={life_before}, life_after={life_after}"
    );
}

// ── Q4: WhenDealsCombatDamageToPlayer, checked directly against
//    check_triggers (see the doc comment on the test for why) ──────────────

fn q4_combat_damage_dfc_def() -> CardDefinition {
    CardDefinition {
        card_id: cid("dx24-q4-combat-damage-transform"),
        name: "DX24 Q4 Front".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Transform".to_string(),
        abilities: vec![CardDefAbilityDefinition::Keyword(KeywordAbility::Transform)],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "DX24 Q4 Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Horror".to_string())].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "Whenever this creature deals combat damage to a player, \
                           you gain 5 life."
                .to_string(),
            abilities: vec![CardDefAbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                intervening_if: None,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(5),
                },
                targets: vec![],
                modes: None,
                trigger_zone: None,
            }],
            power: Some(4),
            toughness: Some(4),
            color_indicator: Some(vec![Color::Black]),
        }),
        ..Default::default()
    }
}

/// CR 510.3a / OOS-DX1-4 Q4: checked directly against `check_triggers`, NOT
/// end-to-end through combat -- `WhenDealsCombatDamageToPlayer` is ALSO
/// lowered into the runtime Channel-A vector by `build_face_ability_vectors`
/// (already face-aware via `apply_face_change`, an EARLIER and unrelated
/// mechanism -- PB-OS4b/PB-RS4), so an end-to-end life-total assertion would
/// be satisfied by Channel A alone and would NOT discriminate Q4's own raw
/// card-registry scan in `abilities.rs`. Filtering the returned
/// `PendingTrigger`s by `kind == PendingTriggerKind::CardDefETB` isolates
/// exactly the code path this batch touches. Revert (restore `def.abilities`
/// at the Q4 site): zero `CardDefETB` hits (front declares no such ability).
#[test]
fn test_dx24_when_deals_combat_damage_to_player_reads_the_visible_face_of_a_transformed_attacker() {
    let p1 = p(1);
    let p2 = p(2);
    let def = q4_combat_damage_dfc_def();
    let mut defs = HashMap::new();
    defs.insert(def.name.clone(), def.clone());
    let registry = CardRegistry::new(vec![def.clone()]);

    let spec = enrich_spec_from_def(
        ObjectSpec::card(p1, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_by_name(&state, &def.name);
    let (state, _) = process_command(
        state,
        Command::Transform {
            player: p1,
            permanent: obj_id,
        },
    )
    .expect("Transform should succeed");
    assert!(state.objects()[&obj_id].is_transformed);

    // Re-derive the expected CARD-DEF index of the back face's
    // WhenDealsCombatDamageToPlayer ability -- never hard-code it (T2's own
    // convention).
    let back_face = def.back_face.as_ref().unwrap();
    let expected_index = back_face
        .abilities
        .iter()
        .position(|a| {
            matches!(
                a,
                CardDefAbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                    ..
                }
            )
        })
        .expect("back face must declare WhenDealsCombatDamageToPlayer");

    let event = GameEvent::CombatDamageDealt {
        assignments: vec![CombatDamageAssignment {
            source: obj_id,
            target: CombatDamageTarget::Player(p2),
            amount: 3,
        }],
    };
    let triggers = check_triggers(&state, &[event]);
    let carddef_hits: Vec<_> = triggers
        .iter()
        .filter(|t| t.source == obj_id && t.kind == PendingTriggerKind::CardDefETB)
        .collect();

    assert_eq!(
        carddef_hits.len(),
        1,
        "CR 510.3a / OOS-DX1-4 Q4: exactly one CardDefETB trigger must be \
         queued from the back face's WhenDealsCombatDamageToPlayer ability -- \
         Q4's queue site must read the visible (back) face. Got: {:?}",
        carddef_hits
    );
    assert_eq!(
        carddef_hits[0].ability_index, expected_index,
        "the queued CardDefETB trigger's ability_index must be the BACK \
         face's card-def index ({expected_index}), not a front-face index"
    );
}

// ── Q6: WheneverRingTemptsYou, checked directly against check_triggers ──────

fn q6_ring_tempts_dfc_def() -> CardDefinition {
    CardDefinition {
        card_id: cid("dx24-q6-ring-tempts-transform"),
        name: "DX24 Q6 Front".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Transform".to_string(),
        abilities: vec![CardDefAbilityDefinition::Keyword(KeywordAbility::Transform)],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "DX24 Q6 Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Horror".to_string())].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "Whenever the Ring tempts you, you gain 3 life.".to_string(),
            abilities: vec![CardDefAbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverRingTemptsYou,
                intervening_if: None,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![],
                modes: None,
                trigger_zone: None,
            }],
            power: Some(4),
            toughness: Some(4),
            color_indicator: Some(vec![Color::Black]),
        }),
        ..Default::default()
    }
}

/// CR 701.54d / OOS-DX1-4 Q6: `WheneverRingTemptsYou` is not lowered into
/// Channel A (it has no arm in `build_face_ability_vectors`), so -- unlike
/// Q4 -- this one CAN be checked end-to-end at the `check_triggers` level
/// without a masking second dispatch path. Revert (restore `def.abilities`
/// at the Q6 site): zero `CardDefETB` hits (front declares no such ability).
#[test]
fn test_dx24_whenever_ring_tempts_you_reads_the_visible_face_of_a_transformed_permanent() {
    let p1 = p(1);
    let p2 = p(2);
    let def = q6_ring_tempts_dfc_def();
    let mut defs = HashMap::new();
    defs.insert(def.name.clone(), def.clone());
    let registry = CardRegistry::new(vec![def.clone()]);

    let spec = enrich_spec_from_def(
        ObjectSpec::card(p1, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_by_name(&state, &def.name);
    let (state, _) = process_command(
        state,
        Command::Transform {
            player: p1,
            permanent: obj_id,
        },
    )
    .expect("Transform should succeed");
    assert!(state.objects()[&obj_id].is_transformed);

    let back_face = def.back_face.as_ref().unwrap();
    let expected_index = back_face
        .abilities
        .iter()
        .position(|a| {
            matches!(
                a,
                CardDefAbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WheneverRingTemptsYou,
                    ..
                }
            )
        })
        .expect("back face must declare WheneverRingTemptsYou");

    let event = GameEvent::RingTempted {
        player: p1,
        new_level: 1,
    };
    let triggers = check_triggers(&state, &[event]);
    let carddef_hits: Vec<_> = triggers
        .iter()
        .filter(|t| t.source == obj_id && t.kind == PendingTriggerKind::CardDefETB)
        .collect();

    assert_eq!(
        carddef_hits.len(),
        1,
        "CR 701.54d / OOS-DX1-4 Q6: exactly one CardDefETB trigger must be \
         queued from the back face's WheneverRingTemptsYou ability -- Q6's \
         queue site must read the visible (back) face. Got: {:?}",
        carddef_hits
    );
    assert_eq!(
        carddef_hits[0].ability_index, expected_index,
        "the queued CardDefETB trigger's ability_index must be the BACK \
         face's card-def index ({expected_index}), not a front-face index"
    );
}

// ── §4.0 invariant pins + Q2/Q7 structural pin (fix cycle, review Findings 2
//    / 11) -- both rewritten after the review showed the originals did not
//    gate what they claimed ─────────────────────────────────────────────────

/// PB-DX24 §4.0 fix cycle (review Finding 2): the ORIGINAL version of this
/// test matched only the two literal strings `is_transformed = true` /
/// `is_transformed: true` and so missed `face.rs:104`'s COMPUTED write
/// (`obj_mut.is_transformed = new_is_transformed;`, the site
/// `Command::Transform` -- and every other flip -- routes through). That
/// write already existed in the tree when the original pin shipped, so the
/// gate was GREEN while its own stated failure condition ("a second site")
/// was already true. Widened here to catch ANY write whose right-hand side
/// is not the literal `false` (the CR 712.8a reset writes in `state/mod.rs`
/// are excluded on purpose -- they are the RESET half of the invariant, not
/// a candidate for setting `is_transformed` true off the battlefield, and are
/// covered by the runtime probe below instead). This is a drift gate, not
/// the load-bearing proof -- the two runtime probes that follow are, because
/// this structural scan cannot detect the REMOVAL of a guard (only the
/// ADDITION of a write), which is exactly the shape of revert that defeated
/// the original test (deleting `face.rs`'s battlefield check adds no new
/// `is_transformed` write line at all).
#[test]
fn test_dx24_is_transformed_writes_are_confined_to_resolution_and_face_rs() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits: Vec<String> = Vec::new();
    scan_dir_for_is_transformed_writes(&root, &mut hits);
    let other_hits: Vec<_> = hits
        .iter()
        .filter(|h| !h.contains("resolution.rs") && !h.contains("face.rs"))
        .collect();
    assert!(
        other_hits.is_empty(),
        "PB-DX24 §4.0: a write to `is_transformed` (outside a comment, RHS \
         not literally `false`) was found somewhere other than resolution.rs \
         (the disturb ETB) or face.rs (`apply_face_change`'s flip) -- \
         is_transformed may be reachable off the battlefield through a path \
         Q2/Q7's classification never accounted for. Unexpected: {other_hits:?}. \
         Full set: {hits:?}"
    );
    let resolution_hits = hits.iter().filter(|h| h.contains("resolution.rs")).count();
    let face_hits = hits.iter().filter(|h| h.contains("face.rs")).count();
    assert_eq!(
        resolution_hits, 1,
        "expected exactly one resolution.rs write (the disturb ETB); full set: {hits:?}"
    );
    assert_eq!(
        face_hits, 1,
        "expected exactly one face.rs write (apply_face_change's flip); full set: {hits:?}"
    );
}

/// Scans for a WRITE to the `is_transformed` field: either the field-mutation
/// form (`<expr>.is_transformed = <rhs>;`, requires a leading `.` so a
/// same-suffix local binding like `entering_is_transformed = ...` can never
/// match) or the struct-literal form (`is_transformed: <rhs>,`, requires the
/// character immediately before `is_transformed` to NOT be an identifier
/// character, so `new_is_transformed: bool` -- a parameter declaration, not a
/// write -- is excluded by construction rather than by an ad hoc filter).
/// Excludes any hit whose right-hand side is literally `false` (the
/// CR 712.8a reset writes) or `bool` (a type annotation, not a value).
fn scan_dir_for_is_transformed_writes(dir: &std::path::Path, hits: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_for_is_transformed_writes(&path, hits);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (lineno, line) in contents.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            let code_part = match line.find("//") {
                Some(idx) => &line[..idx],
                None => line,
            };
            for (needle, dot_prefixed) in
                [(".is_transformed = ", true), ("is_transformed: ", false)]
            {
                let mut search_from = 0usize;
                while let Some(rel) = code_part[search_from..].find(needle) {
                    let idx = search_from + rel;
                    search_from = idx + needle.len();
                    let boundary_ok = if dot_prefixed {
                        true
                    } else {
                        let before = code_part[..idx].chars().next_back();
                        !matches!(before, Some(c) if c.is_alphanumeric() || c == '_')
                    };
                    if !boundary_ok {
                        continue;
                    }
                    let rhs = code_part[idx + needle.len()..]
                        .split([',', ';'])
                        .next()
                        .unwrap_or("")
                        .trim();
                    if rhs != "false" && rhs != "bool" {
                        hits.push(format!("{}:{}", path.display(), lineno + 1));
                    }
                }
            }
        }
    }
}

/// PB-DX24 §4.0 fix cycle (review Finding 2, part (a)): the REAL invariant
/// behind Q2/Q7's "defensive" classification is CR 712.8a / CR 400.7 --
/// `is_transformed` resets to `false` when a battlefield permanent leaves the
/// battlefield, because the destination-zone object is a NEW object built by
/// `state::GameState::move_object_to_zone` with `is_transformed: false`
/// hard-coded (never carried over from the departing object). This pins that
/// mechanism directly: transform a DFC on the battlefield (back face
/// toughness 2, front face toughness 5, 3 damage marked from the start so it
/// is inert pre-transform and lethal post-transform), let it die via SBA, and
/// assert the NEW graveyard object -- found by the FRONT face's name, per
/// CR 712.8a's "front face in all non-battlefield zones" -- is not
/// transformed. Revert: change `state/mod.rs`'s `move_object_to_zone`
/// literal `is_transformed: false,` to carry the departing object's own
/// value instead.
#[test]
fn test_dx24_transform_state_resets_on_zone_change_to_graveyard() {
    let p1 = p(1);
    let p2 = p(2);
    let def = f2_probe_dfc_def();
    let mut defs = HashMap::new();
    defs.insert(def.name.clone(), def.clone());
    let registry = CardRegistry::new(vec![def.clone()]);

    let spec = enrich_spec_from_def(
        ObjectSpec::card(p1, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Battlefield)
            .with_damage(3),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_by_name(&state, &def.name);
    assert!(
        on_battlefield(&state, &def.name),
        "sanity: alive pre-transform (3 damage is inert against the FRONT \
         face's toughness 5; the builder never checks SBAs at build time)"
    );

    // CR 704.3: `transform_permanent_in_place` checks SBAs immediately after
    // the flip, so the die-from-transform and the zone-change happen inside
    // this ONE `Command::Transform` call -- there is no separate window in
    // which to observe "transformed and still alive". The 3 damage marked
    // from the start is lethal against the BACK face's toughness 2, inert
    // against the FRONT face's toughness 5.
    let (state, events) = process_command(
        state,
        Command::Transform {
            player: p1,
            permanent: obj_id,
        },
    )
    .expect("Transform should succeed");

    let died_as_transformed = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CreatureDied {
                object_id,
                pre_death_characteristics: Some(c),
                ..
            } if *object_id == obj_id && c.toughness == Some(2)
        )
    });
    assert!(
        died_as_transformed,
        "sanity: the transform's own CR 704.3 SBA check must kill the \
         permanent AS the transformed (toughness-2 back face) permanent -- \
         otherwise this fixture isn't exercising the transformed-then-zone- \
         changed case this test needs. Got events: {events:?}"
    );

    let new_id = find_in_graveyard(&state, &def.name);
    assert!(
        !state.objects()[&new_id].is_transformed,
        "CR 712.8a / CR 400.7: the graveyard object is a NEW object and the \
         front face is used in all non-battlefield zones -- is_transformed \
         must reset to false. Got is_transformed=true on the new graveyard \
         object {new_id:?}."
    );
}

/// PB-DX24 §4.0 fix cycle (review Finding 2, part (b)): the OTHER half of
/// Q2/Q7's "defensive" classification is `rules::face::apply_face_change`'s
/// OWN battlefield gate (`face.rs:67-69` at authoring time, since renumbered
/// by this fix cycle's own comment edits -- re-find by symbol, not by line).
/// No PRODUCTION call site ever invokes `apply_face_change` on a
/// non-battlefield object (every one either checks the zone first, like
/// `Command::Transform`'s `handle_transform`, or has just moved the object
/// TO the battlefield in the same call, like the craft return-from-exile
/// path) -- so that gate was previously unreachable by ANY test using only
/// the public command API, and a revert deleting it left every PB-DX24 test
/// green (review Finding 2's own measured claim). `apply_face_change` was
/// promoted `pub(crate)` -> `pub` (mirroring `build_face_ability_vectors`'s
/// identical PB-DX24 promotion, T7's access problem) so this test can call
/// it DIRECTLY on a graveyard object and observe the gate itself. Revert:
/// delete the `if obj.zone != ZoneId::Battlefield { return; }` guard.
#[test]
fn test_dx24_apply_face_change_is_a_noop_off_the_battlefield() {
    let p1 = p(1);
    let def = f2_probe_dfc_def();
    let mut defs = HashMap::new();
    defs.insert(def.name.clone(), def.clone());
    let registry = CardRegistry::new(vec![def.clone()]);

    let spec = enrich_spec_from_def(
        ObjectSpec::card(p1, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Graveyard(p1)),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_by_name(&state, &def.name);
    assert_eq!(
        state.objects()[&obj_id].zone,
        ZoneId::Graveyard(p1),
        "sanity: the object must be off the battlefield before the call"
    );
    assert!(
        !state.objects()[&obj_id].is_transformed,
        "sanity: starts false"
    );

    mtg_engine::rules::face::apply_face_change(&mut state, obj_id, true);

    assert!(
        !state.objects()[&obj_id].is_transformed,
        "face.rs:63-69: apply_face_change must be a no-op for an object that \
         is not on the battlefield -- CR 712.8a governs only battlefield \
         permanents. Got is_transformed=true after calling apply_face_change \
         on a graveyard object."
    );
}

fn f2_probe_dfc_def() -> CardDefinition {
    CardDefinition {
        card_id: cid("dx24-f2-transform-reset-probe"),
        name: "DX24 F2 Probe Front".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Transform".to_string(),
        abilities: vec![CardDefAbilityDefinition::Keyword(KeywordAbility::Transform)],
        power: Some(1),
        toughness: Some(5),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "DX24 F2 Probe Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: String::new(),
            abilities: vec![],
            power: Some(1),
            toughness: Some(2),
            color_indicator: Some(vec![Color::Black]),
        }),
        ..Default::default()
    }
}

/// OOS-DX1-4 Q2/Q7: both are DEFENSIVE fixes (§4.0 -- `is_transformed` can
/// never be true at either site, so no behavioral probe can discriminate
/// them). Pinned structurally: within the first statement after each site's
/// `OOS-DX1-4 Q<n>` anchor comment in `abilities.rs`, the code must call
/// `effective_abilities(` and must NOT fall back to the bare
/// `.abilities.iter().enumerate()` shape the batch replaced.
///
/// Fix cycle (review Finding 11): the ORIGINAL version scanned raw source
/// (no comment stripping) over a hard 8-line window. Rewritten to (1) strip
/// line AND block comments before checking content -- mirroring
/// `pb_dx24_trigger_zone_roster.rs`'s `strip_comments` idiom (duplicated here
/// rather than shared, since `primitives` and `core` are separate test
/// binaries with no common support crate) -- so a `/* effective_abilities( */`
/// stub comment can no longer produce a false PASS, and (2) end the window at
/// the first statement boundary (a `;` or `{` outside a comment) after the
/// anchor's own comment block, rather than an arbitrary line count, so a
/// `cargo fmt` reflow that moves the call past line 8 can no longer produce a
/// false FAIL.
#[test]
fn test_dx24_q2_and_q7_queue_sites_call_effective_abilities() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules/abilities.rs");
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let lines: Vec<&str> = contents.lines().collect();

    for anchor in ["OOS-DX1-4 Q2", "OOS-DX1-4 Q7"] {
        let anchor_line = lines
            .iter()
            .position(|l| l.contains(anchor))
            .unwrap_or_else(|| panic!("anchor comment `{anchor}` not found in abilities.rs"));
        // Skip past the anchor's own (possibly multi-line) `//` comment block.
        let mut stmt_start = anchor_line;
        while stmt_start < lines.len() && lines[stmt_start].trim_start().starts_with("//") {
            stmt_start += 1;
        }
        // Scan forward to the end of the FIRST statement: the first line
        // (inclusive) whose comment-stripped, trimmed text ends with `;`
        // (a `let` binding, Q2's shape) or `{` (a `for`/`if` header, Q7's
        // shape). Capped as a sanity backstop, not a real limit.
        let mut stmt_end = stmt_start;
        while stmt_end < lines.len() && stmt_end < stmt_start + 40 {
            let code = match lines[stmt_end].find("//") {
                Some(i) => &lines[stmt_end][..i],
                None => lines[stmt_end],
            };
            let trimmed = code.trim_end();
            if trimmed.ends_with(';') || trimmed.ends_with('{') {
                break;
            }
            stmt_end += 1;
        }
        let window_end = (stmt_end + 1).min(lines.len());
        let window_text = strip_comments(&lines[anchor_line..window_end].join("\n"));
        assert!(
            window_text.contains("effective_abilities("),
            "{anchor}: the queue site's first statement after the anchor \
             comment must call `effective_abilities(`. Window \
             (comment-stripped):\n{window_text}"
        );
        assert!(
            !window_text.contains(".abilities.iter().enumerate()"),
            "{anchor}: the queue site must NOT fall back to the bare \
             `def.abilities.iter().enumerate()` shape this batch replaced. \
             Window (comment-stripped):\n{window_text}"
        );
    }
}

/// Mirrors `pb_dx24_trigger_zone_roster.rs`'s `strip_line_comments` (PB-DX32
/// M8 lesson: line-comment stripping alone lets a block-commented row escape
/// detection).
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Mirrors `pb_dx24_trigger_zone_roster.rs`'s `strip_block_comments`.
fn strip_block_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("*/") {
            Some(end) => rest = &after[end + 2..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

fn strip_comments(src: &str) -> String {
    strip_block_comments(&strip_line_comments(src))
}
