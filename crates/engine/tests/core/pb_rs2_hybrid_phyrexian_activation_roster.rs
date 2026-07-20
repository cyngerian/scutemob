//! PB-RS2 roster sweep (SR-36 — enumerate `all_cards()`, never grep source): which card
//! defs carry a hybrid or Phyrexian pip inside an **activation cost**
//! (`AbilityDefinition::Activated`'s `Cost`, recursing through `Cost::Sequence` — this
//! covers both stack-using activated abilities and, pre-lowering, mana abilities, since
//! `mana_ability_lowering` in `testing/replay_harness.rs` derives `ManaAbility` from the
//! same `AbilityDefinition::Activated` entries).
//!
//! This is deliberately narrower than a whole-`CardDefinition` JSON walk: a card's own
//! printed `mana_cost` (the CAST path, e.g. Birthing Pod's `{3}{G/P}`, or
//! `ajani_sleeper_agent`'s hybrid-Phyrexian pip) is out of scope — `casting.rs` already
//! flattened that path correctly before this PB (§0.1 of the plan). Only the activation
//! side was the free-pip bug (OOS-RS-2), so only the activation side belongs in this
//! roster.
//!
//! Expected pinned set after PB-RS2: the 7 filter lands (their `{Hybrid},{T}: ...` mana
//! ability) + Birthing Pod (its newly-authored `{1}{G/P}, {T}, Sacrifice...` activated
//! ability) = 8. Pinning an EXACT set (not a floor, contrast `pb_rs1_roster_sweep.rs`)
//! is deliberate here: this is a narrow, specific primitive shape, not an
//! actively-growing authoring target, so the next card that adds one should fail this
//! test until a human confirms its activation cost is actually charged (the whole point
//! of the residue guard + this sweep working together).

use mtg_engine::{AbilityDefinition, CardDefinition, Cost};
use std::collections::BTreeSet;

/// True if `cost` contains, anywhere in its (possibly nested `Sequence`) shape, a
/// `Cost::Mana(ManaCost)` with a non-empty `hybrid` or `phyrexian` field.
fn cost_has_hybrid_or_phyrexian_pip(cost: &Cost) -> bool {
    match cost {
        Cost::Mana(mc) => !mc.hybrid.is_empty() || !mc.phyrexian.is_empty(),
        Cost::Sequence(costs) => costs.iter().any(cost_has_hybrid_or_phyrexian_pip),
        _ => false,
    }
}

/// Every card whose `AbilityDefinition::Activated` cost (recursing through
/// `Cost::Sequence`) carries a hybrid or Phyrexian pip.
fn roster(defs: &[CardDefinition]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in defs {
        for ability in &def.abilities {
            if let AbilityDefinition::Activated { cost, .. } = ability {
                if cost_has_hybrid_or_phyrexian_pip(cost) {
                    out.insert(def.name.clone());
                }
            }
        }
    }
    out
}

#[test]
fn every_hybrid_or_phyrexian_pip_in_an_activation_cost_is_accounted_for() {
    let defs = mtg_engine::all_cards();
    let found = roster(&defs);

    let expected: BTreeSet<String> = [
        "Twilight Mire",
        "Graven Cairns",
        "Sunken Ruins",
        "Flooded Grove",
        "Rugged Prairie",
        "Fetid Heath",
        "Cascade Bluffs",
        "Birthing Pod",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    assert_eq!(
        found, expected,
        "the roster of cards with a hybrid/Phyrexian pip in an activation cost has changed. \
         If a card was ADDED to this set, confirm its activation cost is actually charged \
         (PB-RS2 fixed the payment path; the §6 residue guard will panic in debug tests if \
         it isn't flattened before reaching ManaPool::can_spend/spend) and update this pinned \
         set. If a card was REMOVED, confirm that was intentional.\nFound:    {found:?}\n\
         Expected: {expected:?}"
    );
}

/// PB-RS2's one coverage flip has no other ratchet: `birthing_pod_activation_charges_the_phyrexian_pip`
/// (in `crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs`) exercises the card's
/// activated ability directly and loudly, but that test proves *behavior*, not *the
/// `Completeness` marker itself* — nothing in this file previously read `completeness` at all
/// (a fact review finding #10 caught `memory/primitive-wip.md` misstating). This test pins the
/// marker: if a future change reverts Birthing Pod to `inert`/`partial`, this fails loudly
/// instead of the sibling test silently self-skipping.
#[test]
fn birthing_pod_completeness_is_pinned_complete() {
    let defs = mtg_engine::all_cards();
    let pod = defs
        .iter()
        .find(|d| d.name == "Birthing Pod")
        .expect("Birthing Pod card def must exist in the corpus");
    assert!(
        pod.completeness.is_complete(),
        "Birthing Pod's completeness regressed to {:?} — this is PB-RS2's only coverage flip \
         (OOS-OS8-1). If this is intentional, also revert \
         `birthing_pod_activation_charges_the_phyrexian_pip` back to a self-skip and update \
         this assertion's comment accordingly; do not just delete this test.",
        pod.completeness
    );
}

/// Non-vacuity: the walk must actually reach real `AbilityDefinition::Activated` costs.
/// If the corpus scan or the DSL shape changed such that this always returns empty, the
/// exact-set assertion above would degrade into "expected == {} == found" only by both
/// sides going empty — this floor makes that failure mode loud instead of silent.
#[test]
fn the_roster_walk_is_not_vacuous() {
    let defs = mtg_engine::all_cards();
    let mut any_activated_cost_seen = false;
    for def in &defs {
        for ability in &def.abilities {
            if matches!(ability, AbilityDefinition::Activated { .. }) {
                any_activated_cost_seen = true;
                break;
            }
        }
    }
    assert!(
        any_activated_cost_seen,
        "no AbilityDefinition::Activated entries found anywhere in the corpus — the walk is \
         broken (this is a real, populated corpus; PB-RS2's roster gate would be vacuous)"
    );
}
