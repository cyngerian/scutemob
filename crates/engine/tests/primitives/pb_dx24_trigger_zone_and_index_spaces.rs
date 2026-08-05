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

use mtg_engine::state::stubs::PendingTriggerKind;
use mtg_engine::testing::replay_harness::build_face_ability_vectors;
use mtg_engine::{
    all_cards, check_and_apply_sbas, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ManaPool, ObjectId, ObjectSpec,
    PlayerId, PlayerTarget, Step, TriggerCondition, TriggerEvent, ZoneId,
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
        "CR 118.12: with {{B}} available, Nether Traitor's MayPayThenEffect \
         must pay (CR 118.12 pay-when-able) and return it to the battlefield"
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
