//! SIM-2 permanent roster gate: the populations `mana_solver::plannable_tap_ability` and
//! `tap_ability_is_activatable` were written against.
//!
//! Enumerated from `all_cards()` and the real ability-lowering pipeline
//! (`enrich_spec_from_def`), never grepped — SR-36, and for the reason SR-36 gives: the
//! runtime `ManaAbility` a def lowers to is not readable from its source text.
//!
//! # What this gate is for
//!
//! Each filter in the solver is a claim about the corpus as much as about the rules. "20
//! defs carry a mana component, so refusing to plan them costs 20 cards" is a *measured*
//! statement, and the interesting failure is not that a number moved — it is that a NEW
//! shape appeared that the solver has never been asked about. R4 in particular pins a
//! population at **zero**: no def in the corpus lowers to a counter-cost mana ability
//! today, which is exactly the shape that rots silently, so the arm that handles it is
//! covered by a synthetic fixture instead
//! (`sim2_mana_intelligence::t14_counter_cost_source_respects_the_counters_present`).
//!
//! When a number here moves, the fix is to re-derive it AND re-read the matching arm of
//! `plannable_tap_ability` — not to bump the constant.

use std::collections::HashMap;

use mtg_engine::{all_cards, enrich_spec_from_def, ObjectSpec, PlayerId};

/// One row per (def × `{T}` mana ability) in the corpus, after lowering.
struct Row {
    name: String,
    produced: u32,
    any_color: bool,
    has_mana_component: bool,
    has_life_component: bool,
    has_counter_component: bool,
    has_activation_condition: bool,
    is_scaled: bool,
}

fn rows() -> Vec<Row> {
    let defs = all_cards();
    let by_name: HashMap<String, _> = defs.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let mut rows = Vec::new();
    for def in defs.iter() {
        let spec = enrich_spec_from_def(ObjectSpec::card(PlayerId(0), &def.name), &by_name);
        for ability in spec.mana_abilities.iter() {
            if !ability.requires_tap {
                continue;
            }
            rows.push(Row {
                name: def.name.clone(),
                produced: ability.produces.values().sum(),
                any_color: ability.any_color,
                has_mana_component: ability.mana_cost.as_ref().is_some_and(|c| {
                    c.mana_value() > 0 || !c.hybrid.is_empty() || !c.phyrexian.is_empty()
                }),
                has_life_component: ability.life_cost > 0,
                has_counter_component: ability.remove_counter.is_some(),
                has_activation_condition: ability.activation_condition.is_some(),
                is_scaled: ability.scaled_amount.is_some(),
            });
        }
    }
    rows
}

/// R1 — the population the true-production fix exists for: `{T}` mana abilities that make
/// **more than one** mana. Credited as 1 each until SIM-2 (playtest F4).
#[test]
fn r1_multi_mana_sources() {
    let rows = rows();
    let multi: Vec<&Row> = rows
        .iter()
        .filter(|r| !r.any_color && !r.is_scaled && r.produced > 1)
        .collect();
    assert_eq!(
        multi.len(),
        36,
        "expected 36 multi-mana {{T}} abilities; re-derive AND re-read mana_solver's \
         production accounting if this moved. Found: {:?}",
        multi.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    for expected in [
        "Sol Ring",
        "Mana Crypt",
        "Golgari Rot Farm",
        "Llanowar Tribe",
    ] {
        assert!(
            multi.iter().any(|r| r.name == expected),
            "{expected} must be in the multi-mana roster (non-vacuity)"
        );
    }
    // The largest single-activation production in the corpus, which is what bounds the
    // solver's least-waste search: Llanowar Tribe's {G}{G}{G}.
    assert_eq!(multi.iter().map(|r| r.produced).max(), Some(3));
}

/// R2 — abilities with their own mana component. `plannable_tap_ability` refuses to PLAN
/// these (crediting gross production while ignoring the cost would over-credit) while
/// `StubProvider` still offers them, so this number is the size of what the solver gives
/// up: Signets, filter lands, Cabal Coffers, Crypt of Agadeem.
#[test]
fn r2_abilities_with_a_mana_component() {
    let rows = rows();
    let with_mana: Vec<&Row> = rows.iter().filter(|r| r.has_mana_component).collect();
    assert_eq!(
        with_mana.len(),
        20,
        "expected 20 {{T}} mana abilities with a mana component: {:?}",
        with_mana.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    for expected in ["Boros Signet", "Cascade Bluffs", "Cabal Coffers"] {
        assert!(
            with_mana.iter().any(|r| r.name == expected),
            "{expected} must be in the mana-component roster (non-vacuity)"
        );
    }
}

/// R3 — life and activation-condition components, the two arms `StubProvider` shares with
/// the solver (SG-1 and OOS-CARDS2-9 respectively).
#[test]
fn r3_life_and_condition_components() {
    let rows = rows();
    let life: Vec<&Row> = rows.iter().filter(|r| r.has_life_component).collect();
    let conditioned: Vec<&Row> = rows.iter().filter(|r| r.has_activation_condition).collect();
    assert_eq!(
        life.len(),
        8,
        "expected 8 {{T}} mana abilities with a life component: {:?}",
        life.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    assert_eq!(
        conditioned.len(),
        13,
        "expected 13 conditioned {{T}} mana abilities: {:?}",
        conditioned.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    for expected in ["Mox Opal", "Tainted Field", "Temple of the False God"] {
        assert!(
            conditioned.iter().any(|r| r.name == expected),
            "{expected} must be in the conditioned roster (non-vacuity)"
        );
    }
}

/// R4 — **pinned EMPTY**, and that is the whole point of the row.
///
/// No def lowers to a counter-cost mana ability today (`Cost::RemoveCounter` reaches
/// `mana_ability_lowering` from nowhere in the current corpus), so the solver's
/// counter-cost arm has no corpus traffic and would rot unnoticed. The arm is covered by a
/// synthetic fixture instead; this row exists so that the FIRST def to add one turns up as
/// a failing test naming the card, rather than as a silently unplanned mana source.
#[test]
fn r4_counter_cost_mana_abilities_are_absent_from_the_corpus() {
    let rows = rows();
    let with_counters: Vec<&Row> = rows.iter().filter(|r| r.has_counter_component).collect();
    assert!(
        with_counters.is_empty(),
        "a counter-cost mana ability now exists in the corpus ({:?}) — re-read \
         mana_solver::tap_ability_is_activatable's remove_counter arm and give it a \
         corpus-backed test",
        with_counters.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    // Non-vacuity floor: the enumeration itself is working, so "empty" is a fact about
    // counter costs and not about `rows()` returning nothing.
    assert!(
        rows.len() >= 300,
        "the roster enumeration collapsed: only {} {{T}} mana abilities found",
        rows.len()
    );
}

/// R5 — SR-36 scaled abilities carry a `1`-per-colour **marker** in `produces`, not a real
/// count (Gaea's Cradle, Cabal Coffers). The solver credits the marker, which UNDER-counts
/// and can only under-offer; it never over-credits, which is the direction that would
/// produce a plan the engine refuses. Pinned so that a change to the marker convention
/// shows up here rather than as a mysterious over-offer.
#[test]
fn r5_scaled_abilities_carry_a_marker_not_a_count() {
    let rows = rows();
    let scaled: Vec<&Row> = rows.iter().filter(|r| r.is_scaled).collect();
    assert_eq!(
        scaled.len(),
        9,
        "expected 9 scaled {{T}} mana abilities: {:?}",
        scaled.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    for row in &scaled {
        assert_eq!(
            row.produced, 1,
            "{}'s scaled ability must carry the 1-per-colour marker (SR-36)",
            row.name
        );
    }
}
