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

use mtg_engine::testing::replay_harness::build_face_ability_vectors;
use mtg_engine::{
    all_cards, check_and_apply_sbas, enrich_spec_from_def, CardDefinition, CardRegistry,
    GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;

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
