//! UI-2 (playtest triage F9) roster sweep (SR-36 -- enumerate `all_cards()`, never
//! grep source): the two additional-cost kinds the browser client can now announce.
//!
//! The batch's whole premise is that `StubProvider` must offer exactly what
//! `casting.rs` will accept (SR-38). Every one of these gates pins a fact the
//! *provider* and the *play-server view* rely on, so that the day the corpus stops
//! having that shape, this file fails rather than the offer quietly becoming wrong.
//!
//! * **R1** -- the exact set of defs declaring a `spell_additional_costs` entry
//!   (CR 118.8). `legal_actions.rs::build_additional_cost_plan` builds a required
//!   sacrifice descriptor for exactly these, and suppresses their `CastSpell` offer
//!   when nothing eligible exists.
//! * **R2** -- **at most one** requirement per def. `casting.rs:3300-3369` validates
//!   `required_costs[0]` and consumes one sacrifice id, in its own words "For now,
//!   we support exactly one mandatory sacrifice cost". The provider therefore reads
//!   `.first()` alone. A def declaring two would be silently under-asked; this is
//!   the gate that makes that impossible to ship in silence.
//! * **R3** -- the exact set of defs carrying `AbilityDefinition::Squad { cost }`
//!   (CR 702.157a), which is what the provider detects on.
//! * **R4** -- every Squad def's cost is **non-zero** and carries **no hybrid or
//!   Phyrexian pip**. Non-zero: `legal_actions.rs::squad_max_count` returns 0 for a
//!   zero mana value, because the affordability walk would otherwise be unbounded.
//!   No hybrid/Phyrexian: **this half's original reason is CLOSED** -- it was that
//!   `view.rs::format_mana_cost_compact` rendered neither pip kind, and PB-DX29 taught
//!   it both (CR 107.4e/107.4f) plus `{X}` (CR 107.3). The assertion is KEPT as a
//!   surprise-detector on the Squad roster rather than deleted, but a failure now means
//!   "a Squad cost got more complicated, go look", not "the label will lie".
//!   The way that limitation surfaced is worth reading:
//!   `core::pb_dx29_additional_cost_roster::r3`'s doc records it. This gate could never
//!   have found it, because the counter-example (`brokkos_apex_of_forever`, a HYBRID
//!   mutate cost) was in the corpus the whole time and is not a Squad def.
//! * **R5** -- no def declares an additional cost (sacrifice or Squad) together with
//!   an `{X}` or a modal spell ability. This is the premise the frontend's picker
//!   ordering rests on: CR 601.2b's own internal order is modes -> additional costs
//!   -> X, while `ActionBar.svelte` bundles modes and X into one `ValuePrompt` stage
//!   that runs BEFORE the cost stage. That ordering is unobservable exactly as long
//!   as no card needs both, and this gate is what says so.
//! * **R6** -- non-vacuity floors for R1 and R3, and a floor on the corpus itself.
//!
//! What membership asserts, and does NOT (PB-DX4's `BASELINE` lesson, same wording):
//! membership means only that this def declares this specific shape. It says nothing
//! about whether the def is otherwise oracle-correct.

use mtg_engine::{AbilityDefinition, CardDefinition, ManaCost};
use std::collections::BTreeSet;

/// R1: defs declaring at least one `SpellAdditionalCost` (CR 118.8).
fn roster_r1(defs: &[CardDefinition]) -> BTreeSet<String> {
    defs.iter()
        .filter(|d| !d.spell_additional_costs.is_empty())
        .map(|d| d.name.clone())
        .collect()
}

/// R3: defs carrying the cost-bearing `AbilityDefinition::Squad { cost }` variant.
///
/// Deliberately NOT `KeywordAbility::Squad`: the marker synthesises no cost, and a
/// def carrying only the marker is refused by `casting.rs`'s `get_squad_cost` the
/// moment a non-zero count is announced. R3b below pins that the two sets agree, so
/// the marker-only shape (which `galadhrim_brigade` shipped until UI-2 repaired it)
/// cannot come back unnoticed.
fn roster_r3(defs: &[CardDefinition]) -> BTreeSet<String> {
    defs.iter()
        .filter(|d| {
            d.abilities
                .iter()
                .any(|a| matches!(a, AbilityDefinition::Squad { .. }))
        })
        .map(|d| d.name.clone())
        .collect()
}

/// Defs carrying the `KeywordAbility::Squad` presence marker.
fn roster_squad_marker(defs: &[CardDefinition]) -> BTreeSet<String> {
    defs.iter()
        .filter(|d| {
            d.abilities.iter().any(|a| {
                matches!(
                    a,
                    AbilityDefinition::Keyword(mtg_engine::KeywordAbility::Squad)
                )
            })
        })
        .map(|d| d.name.clone())
        .collect()
}

fn squad_cost_of(def: &CardDefinition) -> Option<&ManaCost> {
    def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Squad { cost } = a {
            Some(cost)
        } else {
            None
        }
    })
}

/// Does this def declare an `{X}` in its mana cost, or a modal spell ability?
///
/// Both are announced in CR 601.2b, and both route through `ActionBar.svelte`'s
/// `ValuePrompt` stage. See R5.
fn declares_x_or_modes(def: &CardDefinition) -> bool {
    let has_x = def
        .mana_cost
        .as_ref()
        .map(|c| c.x_count > 0)
        .unwrap_or(false);
    let has_modes = def.abilities.iter().any(|a| match a {
        AbilityDefinition::Spell { modes, .. } => modes.is_some(),
        _ => false,
    });
    has_x || has_modes
}

/// R1 -- pinned EXACT (not a floor). A new def declaring a CR 118.8 additional cost
/// must fail this gate until a human confirms the provider's suppression gate is
/// right for it.
#[test]
fn r1_spell_additional_cost_roster_is_pinned() {
    let defs = mtg_engine::all_cards();
    let found = roster_r1(&defs);
    let expected: BTreeSet<String> = [
        "Abjure",
        "Altar of Bone",
        "Corrupted Conviction",
        "Crop Rotation",
        "Culling the Weak",
        "Deadly Dispute",
        "Diabolic Intent",
        "Eldritch Evolution",
        "Goblin Grenade",
        "Harrow",
        "Life's Legacy",
        "Momentous Fall",
        "Natural Order",
        "Village Rites",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(
        found, expected,
        "R1 (defs with a non-empty `spell_additional_costs`) has changed. Membership asserts \
         ONLY that the def declares a CR 118.8 additional cost -- nothing about whether it is \
         otherwise oracle-correct. If a card was ADDED, confirm that \
         `legal_actions.rs::build_additional_cost_plan` builds a sensible eligibility set for \
         its filter and that suppressing the offer when nothing is eligible is right for it \
         (SR-38), then update this pinned set. If REMOVED, confirm that was \
         intentional.\nFound:    {found:?}\nExpected: {expected:?}"
    );
}

/// R2 -- **at most one** requirement per def, because `casting.rs` validates only
/// `required_costs[0]` and the provider therefore reads only `.first()`.
#[test]
fn r2_no_def_declares_more_than_one_spell_additional_cost() {
    let defs = mtg_engine::all_cards();
    let offenders: Vec<(String, usize)> = defs
        .iter()
        .filter(|d| d.spell_additional_costs.len() > 1)
        .map(|d| (d.name.clone(), d.spell_additional_costs.len()))
        .collect();
    assert!(
        offenders.is_empty(),
        "`casting.rs`'s spell-additional-cost block validates `required_costs[0]` ALONE (its \
         own comment: \"For now, we support exactly one mandatory sacrifice cost\"), and \
         `legal_actions.rs::build_additional_cost_plan` mirrors that by reading `.first()`. A \
         def declaring more than one is therefore SILENTLY under-asked -- the player pays the \
         first cost and the rest are never checked. Either teach both sides to handle several, \
         or split the def. Offenders: {offenders:?}"
    );
}

/// R3 -- pinned EXACT set of Squad defs (the cost-bearing variant).
#[test]
fn r3_squad_cost_roster_is_pinned() {
    let defs = mtg_engine::all_cards();
    let found = roster_r3(&defs);
    let expected: BTreeSet<String> = ["Galadhrim Brigade", "Ultramarines Honour Guard"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        found, expected,
        "R3 (defs with `AbilityDefinition::Squad {{ cost }}`) has changed. If a card was \
         ADDED, confirm its squad cost is the printed one and update this set; the provider \
         detects Squad on THIS variant, so a def added with only the \
         `KeywordAbility::Squad` marker is never offered Squad at all (see \
         R3b).\nFound:    {found:?}\nExpected: {expected:?}"
    );
}

/// R3b -- the marker set and the cost set must be **the same set**.
///
/// This is the gate UI-2 wrote because the corpus failed it: `galadhrim_brigade`
/// shipped `Complete` and deck-legal carrying `KeywordAbility::Squad` and no
/// `AbilityDefinition::Squad { cost }`, so `casting.rs`'s `get_squad_cost` returned
/// `None` and EVERY `squad_count > 0` cast was refused with "spell has squad keyword
/// but no squad cost defined" -- on the very card the first human playtest tried to
/// Squad. Nothing could fail, because nothing checked.
#[test]
fn r3b_squad_marker_and_squad_cost_declare_the_same_defs() {
    let defs = mtg_engine::all_cards();
    let marker = roster_squad_marker(&defs);
    let cost = roster_r3(&defs);
    let marker_only: Vec<&String> = marker.difference(&cost).collect();
    let cost_only: Vec<&String> = cost.difference(&marker).collect();
    assert!(
        marker_only.is_empty(),
        "these defs carry `KeywordAbility::Squad` but no `AbilityDefinition::Squad {{ cost }}`, \
         so their squad cost is UNPAYABLE -- `casting.rs::get_squad_cost` returns `None` and \
         every non-zero count is refused. Author the cost from the printed \"Squad {{N}}\" \
         (`ultramarines_honour_guard.rs` is the reference: both variants, always): {marker_only:?}"
    );
    assert!(
        cost_only.is_empty(),
        "these defs carry a squad COST but not the `KeywordAbility::Squad` marker. \
         `casting.rs` gates on `chars.keywords.contains(&KeywordAbility::Squad)` BEFORE it \
         looks the cost up, so the cost is dead and any announced count is refused with \
         \"spell does not have squad\": {cost_only:?}"
    );
}

/// R4 -- every Squad cost is non-zero, and carries no hybrid or Phyrexian pip.
///
/// The hybrid/Phyrexian half's original justification is closed (PB-DX29 taught the
/// formatter both pip kinds); see this file's module doc. Kept as a surprise-detector.
/// The failure message below is corrected in place rather than left asserting a
/// limitation that no longer exists -- a gate whose reason has gone stale tells the next
/// reader to fix the wrong thing, which is `OOS-CARDS2-8`'s class in a test file.
#[test]
fn r4_every_squad_cost_is_nonzero_and_has_no_hybrid_or_phyrexian_pip() {
    let defs = mtg_engine::all_cards();
    for def in defs.iter().filter(|d| squad_cost_of(d).is_some()) {
        let cost = squad_cost_of(def).expect("filtered above");
        assert!(
            cost.mana_value() > 0,
            "{}: a zero-mana-value squad cost makes `legal_actions.rs::squad_max_count`'s \
             affordability walk unbounded (every N stays equally free), so it returns 0 and \
             the cost can never be offered. Either the def's cost is wrong or that helper \
             needs a real answer for the free-squad case.",
            def.name
        );
        assert!(
            cost.hybrid.is_empty() && cost.phyrexian.is_empty(),
            "{}: this Squad cost carries a hybrid or Phyrexian pip. \
             `view.rs::format_mana_cost_compact` CAN render both since PB-DX29, so this is no \
             longer a display bug -- but a Squad cost is paid N times and \
             `legal_actions.rs::repeated_cost_max_count` bounds N through \
             `ManaCost::mana_value`, which resolves a hybrid pip to its LARGEST component \
             (CR 202.3f). Confirm the offered max_count is the one the payment path will \
             accept before pinning this card. cost: {cost:?}",
            def.name
        );
    }
}

/// R5 -- no def declares an additional cost together with `{X}` or modes.
///
/// The premise behind `ActionBar.svelte`'s stage order: CR 601.2b's own internal
/// order is modes -> additional costs -> X, but the client bundles modes and X into
/// one `ValuePrompt` that runs *before* the cost stage. Harmless exactly while no
/// card needs both. This gate is the "check the reachability argument, not the
/// guard" lesson from SIM-1, applied ahead of time.
#[test]
fn r5_no_def_mixes_an_additional_cost_with_x_or_modes() {
    let defs = mtg_engine::all_cards();
    let offenders: Vec<String> = defs
        .iter()
        .filter(|d| !d.spell_additional_costs.is_empty() || squad_cost_of(d).is_some())
        .filter(|d| declares_x_or_modes(d))
        .map(|d| d.name.clone())
        .collect();
    assert!(
        offenders.is_empty(),
        "these defs declare BOTH a CR 118.8/702.157 additional cost and an `{{X}}` or a modal \
         spell ability. `tools/play-server/frontend/src/lib/ActionBar.svelte` runs its \
         `ValuePrompt` (modes + X) stage BEFORE the `CostPicker` stage, which inverts CR \
         601.2b's own order (modes -> additional costs -> X). That was unobservable while this \
         set was empty. It is not any more -- either split `ValuePrompt` so modes precede and \
         X follows the cost stage, or confirm the inversion is harmless for these cards and \
         say so here: {offenders:?}"
    );
}

/// R6 -- non-vacuity floors. Without these, a walk that silently stopped finding
/// anything would make R2/R4/R5 pass by examining nothing.
#[test]
fn r6_rosters_are_not_vacuous() {
    let defs = mtg_engine::all_cards();
    assert!(
        defs.len() > 1_500,
        "the corpus itself looks empty ({} defs) -- `all_cards()` is broken, and every gate \
         in this file would pass vacuously",
        defs.len()
    );
    assert_eq!(
        roster_r1(&defs).len(),
        14,
        "R1 floor: 14 defs declare a CR 118.8 additional cost. If this changed legitimately, \
         update R1's pinned set and this number together."
    );
    assert_eq!(
        roster_r3(&defs).len(),
        2,
        "R3 floor: 2 defs carry a squad cost. If this changed legitimately, update R3's pinned \
         set and this number together."
    );
    assert!(
        defs.iter().any(declares_x_or_modes),
        "R5's `declares_x_or_modes` predicate matches NOTHING in the corpus, so R5 passes \
         vacuously -- the predicate is broken, not the corpus"
    );
}
