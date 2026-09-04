//! PB-DX35 Half B (`OOS-DX4-5`): the census for the "you may look at / put" family,
//! **printed** rather than transcribed.
//!
//! `Effect::LookAtTopThenPlace.optional` was inert — the executor destructured it away —
//! so five `Complete` defs recorded a printed "you may" the engine never asked. Half B
//! makes the flag real. This file is the census that bounds the served class and the
//! ratchet that stops it regrowing, on TWO axes, because dispatch hygiene 6 says a filed
//! member list is a FLOOR:
//!
//! * **B1** — the exact set of corpus defs carrying `Effect::LookAtTopThenPlace`, by NAME,
//!   with `optional` / `place_cost` / marker for each. Pinned, so a sixth use is a
//!   deliberate act.
//! * **B2** — every one of them sets `optional: true` and is `Complete`. That is the
//!   population this batch served, and it is what makes "0 flips" checkable rather than
//!   asserted: a member that was NOT already `Complete` would have made Half B a coverage
//!   change.
//! * **B3** — the **inverse axis**, from PRINTED ORACLE TEXT rather than from the DSL:
//!   defs that print the same look-then-optionally-place shape and do **not** use the
//!   primitive. That population is FILED, not taken (`OOS-DX35-5`), and pinned here so it
//!   cannot grow in silence.
//! * **B4** — the corpus-wide "you may" residual, ratcheted, with an explicit statement
//!   that PB-DX35 did not widen into it.
//! * **`t_census_report`** — PRINTS every axis. Every population figure PB-DX35 publishes
//!   is read off this test's output, never transcribed (PB-DX8's rule, after PB-DX28's
//!   execution notes quoted two fingerprints that had never existed in any source file).
//!
//! **Why the oracle axis is a phrase pair and not a single needle.** The naive needle
//! `"you may put"` sweeps in a completely different family — *"you may put a land card
//! **from your hand** onto the battlefield"* (Burgeoning, Uro, Chulane, …), which is
//! landfall-shaped and has nothing to do with looking at cards. Measured: the naive needle
//! returns **29** defs, of which 18 are that family. So [`prints_look_then_place`] takes a
//! CONJUNCTION — a look/reveal/mill/exile-the-top verb somewhere in the text AND a
//! `you may put|reveal … onto the battlefield|into your hand|from among` clause that does
//! **not** name a different source zone. That axis was derived independently twice (a
//! throwaway script over the def files, and this test over `all_cards()`) and both returned
//! the same 5 + 11; the agreement is the evidence, not either run alone.
//!
//! Reuses `decision_site_walk.rs`'s canonical serialized-JSON walk rather than a second
//! hand-written tree walk, for PB-DP10's reason: a hand-written walk silently misses the
//! nesting sites it was not taught.

use crate::decision_site_walk::{
    def_contains_variant, find_variant_nodes, is_effectively_complete,
};
use mtg_engine::all_cards;
use mtg_engine::CardDefinition;
use serde_json::Value;
use std::collections::BTreeSet;

// ── The two axes ─────────────────────────────────────────────────────────────

/// Every corpus def carrying `Effect::LookAtTopThenPlace`, with the two fields that
/// decide whether it asks.
struct Carrier {
    name: String,
    /// **`all`, not `any`** — see `b2`'s doc for the executed defeat that forced it.
    optional: bool,
    place_cost: bool,
    complete: bool,
    /// How many `LookAtTopThenPlace` nodes the def carries. A def with two is legal and the
    /// corpus has none; the count is reported so a second node cannot arrive unnoticed.
    nodes: usize,
}

fn carriers() -> Vec<Carrier> {
    let mut out: Vec<Carrier> = Vec::new();
    for def in all_cards() {
        if !def_contains_variant(&def, "LookAtTopThenPlace") {
            continue;
        }
        let json = serde_json::to_value(&def).expect("CardDefinition serializes");
        let nodes = find_variant_nodes(&json, "LookAtTopThenPlace");
        // A def could in principle carry more than one; fold them, since the gate's
        // question is "does this def ask" rather than "does this NODE ask".
        // **`all`, not `any`.** The first draft used `any`, and this batch's own `/review`
        // defeated it by execution: adding a SECOND `LookAtTopThenPlace` node with
        // `optional: false` to `grisly_salvage` (a real, deck-legal `Complete` carrier) inside a
        // `Sequence` left `b1` AND `b2` green — while that node keeps the pre-batch
        // deterministic take-when-able behaviour on a printed "you may", i.e. the very defect
        // this batch closed, alive on a `Complete` def. `b2`'s own docstring then asserted
        // something false about a green tree.
        let optional = nodes
            .iter()
            .all(|n| n.get("optional").and_then(Value::as_bool) == Some(true));
        let place_cost = nodes
            .iter()
            .any(|n| !matches!(n.get("place_cost"), None | Some(Value::Null)));
        out.push(Carrier {
            name: def.name.clone(),
            optional,
            place_cost,
            complete: is_effectively_complete(&def),
            nodes: nodes.len(),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// The oracle-text axis: does this def PRINT a "look/reveal/mill, then you may put or
/// reveal one of them" shape?
///
/// Deliberately a conjunction — see the module doc for the 29-vs-11 measurement that
/// forced it. The excluded-source list is what keeps the landfall family out: a clause
/// naming a DIFFERENT origin ("from your hand", "from the command zone", a graveyard) is
/// not this shape however optional it is.
fn prints_look_then_place(def: &CardDefinition) -> Option<String> {
    let o = def.oracle_text.to_lowercase();
    const LOOK_VERBS: [&str; 4] = ["look at the", "reveal the top", "mill", "exile the top"];
    if !LOOK_VERBS.iter().any(|v| o.contains(v)) {
        return None;
    }
    const OPENERS: [&str; 2] = ["you may put", "you may reveal"];
    const CLOSERS: [&str; 3] = ["onto the battlefield", "into your hand", "from among"];
    const EXCLUDED_SOURCES: [&str; 3] =
        ["from your hand", "from the command zone", "graveyard onto"];
    for opener in OPENERS {
        let mut from = 0usize;
        while let Some(rel) = o[from..].find(opener) {
            let start = from + rel;
            // A generous but BOUNDED window: long enough for "you may put up to two
            // creature and/or land cards from among the milled cards", short enough that
            // it cannot reach an unrelated later sentence. Bounded on a char boundary so
            // a multi-byte oracle text cannot panic here (PB-DX49's `r5b` lesson: a
            // fixed-width byte slice PANICS rather than failing with its own message).
            let mut end = (start + 160).min(o.len());
            while end > start && !o.is_char_boundary(end) {
                end -= 1;
            }
            let clause = &o[start..end];
            let clause = match CLOSERS
                .iter()
                .filter_map(|c| clause.find(c).map(|i| i + c.len()))
                .min()
            {
                Some(i) => &clause[..i],
                None => {
                    from = start + opener.len();
                    continue;
                }
            };
            if !EXCLUDED_SOURCES.iter().any(|e| clause.contains(e)) {
                return Some(clause.trim().to_string());
            }
            from = start + opener.len();
        }
    }
    None
}

// ── B1: the pinned carrier roster ────────────────────────────────────────────

/// The five corpus defs carrying `Effect::LookAtTopThenPlace`, pinned BY NAME.
///
/// Not a "roughly five" — a sixth use is a deliberate act, and whoever makes it must
/// decide whether their def's printed text says "may" before adding it here.
///
/// **`muxus_goblin_grandee` is deliberately NOT a member and that is worth stating**: it
/// matches a text grep for the variant name and carries ZERO occurrences of the effect —
/// the name appears only in a comment and in its `partial` note explaining why
/// `RevealAndRoute` is the right primitive for it instead. SR-36's rule (walk
/// `all_cards()`, never grep source) is what separates the two, and this is the def that
/// proves the difference is not theoretical.
const CARRIERS: [&str; 5] = [
    "Birthing Ritual",
    "Grisly Salvage",
    "Growing Rites of Itlimoc",
    "Risen Reef",
    "Satyr Wayfinder",
];

#[test]
fn b1_the_look_at_top_then_place_carriers_are_pinned() {
    let live: BTreeSet<String> = carriers().into_iter().map(|c| c.name).collect();
    let pinned: BTreeSet<String> = CARRIERS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        live, pinned,
        "the Effect::LookAtTopThenPlace population moved. PB-DX35 made `optional` a real \
         CR 608.2d question for every one of these; a new member inherits that, so add it \
         here deliberately and check its PRINTED text says \"may\" before setting \
         `optional: true`."
    );
    // Non-vacuity: the walk must actually reach the nodes, not return an empty set
    // because the variant key was renamed. A gate whose denominator can silently go to
    // zero is not a gate (`OOS-DX8-7`).
    assert_eq!(
        live.len(),
        5,
        "non-vacuity: the serialized walk found {} carriers, expected 5",
        live.len()
    );
}

// ── B2: every carrier is `optional: true` AND `Complete` ─────────────────────

#[test]
/// The population Half B served, and the reason its coverage delta is ZERO.
///
/// Both halves of this matter. `optional: true` on all five is why the batch's engine
/// change is behaviourally live rather than latent; `Complete` on all five is why closing
/// `OOS-DX4-5` moved no marker and re-dealt no seeded fixture. If a future member arrives
/// `partial`, this reddens and whoever added it owes a coverage prediction.
fn b2_every_carrier_is_optional_and_complete() {
    let cs = carriers();
    let not_optional: Vec<&str> = cs
        .iter()
        .filter(|c| !c.optional)
        .map(|c| c.name.as_str())
        .collect();
    assert!(
        not_optional.is_empty(),
        "these carriers have at least one `LookAtTopThenPlace` node with `optional: false`, so \
         PB-DX35's ask does not fire for it and it keeps the deterministic take-when-able \
         winner: {not_optional:?}. That is a legal value (pinned behaviourally by \
         `primitives::pb_dx35_optional_placement::t1`) but no corpus def had it when the class \
         was closed, so a new one is a decision to record. **The fold is `all`, not `any`** -- \
         with `any`, a SECOND node carrying `optional: false` hid behind a first one carrying \
         `true`, which this batch's `/review` proved by execution on `grisly_salvage`."
    );
    // Every carrier holds exactly one node today. Reported by `t_census_report` and asserted
    // here, because the `all` fold above is only as informative as the node count it folds over.
    let multi: Vec<(&str, usize)> = cs
        .iter()
        .filter(|c| c.nodes != 1)
        .map(|c| (c.name.as_str(), c.nodes))
        .collect();
    assert!(
        multi.is_empty(),
        "these carriers hold more than one `Effect::LookAtTopThenPlace` node: {multi:?}. That is \
         legal, and the `all` fold above handles it correctly -- but the corpus had exactly one \
         per def when this class was closed, so a second is a deliberate act worth reading."
    );
    let not_complete: Vec<&str> = cs
        .iter()
        .filter(|c| !c.complete)
        .map(|c| c.name.as_str())
        .collect();
    assert!(
        not_complete.is_empty(),
        "these carriers are not `Complete`: {not_complete:?}. PB-DX35 published \"0 flips\" \
         for Half B on the strength of all five already being `Complete`; a non-`Complete` \
         member makes that claim false."
    );
    // Exactly one carrier sets a `place_cost`, and it is the one that asks TWO questions
    // in one resolution (CR 118.12 twice over: "you may sacrifice", then "you may put").
    let with_cost: Vec<&str> = cs
        .iter()
        .filter(|c| c.place_cost)
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(
        with_cost,
        vec!["Birthing Ritual"],
        "the `place_cost` sub-population moved. It is pinned separately by \
         `core::pb_dx45_may_pay_roster::r4`; this assertion exists because Birthing Ritual \
         is the ONLY def that exercises the two-question ordering \
         (`primitives::pb_dx35_optional_placement::t6`), and if it stops being the only one \
         that probe stops being a complete account of the ordering."
    );
}

// ── B3: the inverse axis — printed shape, different (or no) primitive ────────

/// Defs that PRINT the look-then-optionally-place shape and do **not** use
/// `Effect::LookAtTopThenPlace`. Filed as `OOS-DX35-5`, deliberately NOT repaired here.
///
/// Two structural signals this list carries, both worth reading before taking the seed:
///
/// * Four of them (`Bounty of Skemfar`, `Harald, King of Skemfar`, `Narset, Parter of
///   Veils`, `Six`) substitute `Effect::RevealAndRoute`, which routes EVERY match and has
///   no optionality axis at all — it is the uncapped, mandatory cousin of the primitive
///   this batch just repaired.
/// * Three more (`Carth the Lion`, `Turntimber Symbiosis`, `Ureni of the Unwritten`) cite
///   that same cap/optionality gap as their BLOCKING reason **while
///   `Effect::LookAtTopThenPlace` already closes it** and has since PB-OS8. Three stale
///   blocker notes against a primitive that exists — PB-DX27's class, reached from a
///   different direction.
///
/// Exactly one member is `Complete`, and it is `Complete` for an unrelated reason:
/// `Xenagos, the Reveler` ships two of its three loyalty abilities as `Effect::Nothing`
/// under `// TODO` comments (`OOS-DX35-6`).
const INVERSE_MEMBERS: [&str; 11] = [
    "Bounty of Skemfar",
    "Carth the Lion",
    "Harald, King of Skemfar",
    "Herald's Horn",
    "Narset, Parter of Veils",
    "Six",
    "Smuggler's Surprise",
    "Turntimber Symbiosis // Turntimber, Serpentine Wood",
    "Ureni of the Unwritten",
    "Wrenn and Realmbreaker",
    "Xenagos, the Reveler",
];

#[test]
fn b3_the_inverse_oracle_axis_is_pinned() {
    let mut live: BTreeSet<String> = BTreeSet::new();
    let mut carrier_hits = 0usize;
    for def in all_cards() {
        if prints_look_then_place(&def).is_none() {
            continue;
        }
        if def_contains_variant(&def, "LookAtTopThenPlace") {
            carrier_hits += 1;
        } else {
            live.insert(def.name.clone());
        }
    }
    // The axis must find all five carriers too, or it is not measuring the shape it
    // claims to measure — it would be measuring "defs the primitive missed" by accident.
    // This is the non-vacuity floor that makes the inverse count meaningful.
    assert_eq!(
        carrier_hits, 5,
        "the oracle axis found only {carrier_hits} of the 5 defs that DO use the \
         primitive, so it is not recognising the printed shape and its inverse count \
         cannot be trusted"
    );
    let pinned: BTreeSet<String> = INVERSE_MEMBERS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        live, pinned,
        "the inverse-axis population moved (OOS-DX35-5). These defs print a \"look/reveal/\
         mill, then you MAY put or reveal one of them\" clause without using \
         `Effect::LookAtTopThenPlace`. PB-DX35 deliberately did not repair them; the list \
         is pinned so the class cannot grow in silence."
    );
}

// ── B4: the corpus-wide "you may" residual, ratcheted ───────────────────────

/// PB-DX35 closed the DSL's DEDICATED optionality flag (5 defs). It did **not** widen into
/// the corpus-wide "you may" sweep, which is `OOS-DX8-1`'s successor, restated as
/// `OOS-DX35-5`.
///
/// These are CEILINGS, not equalities, and the reason is stated rather than assumed: the
/// corpus grows, and a new def printing "you may" is ordinary card authoring, not a
/// regression. What they catch is the class DOUBLING unnoticed — which is what would
/// happen if a templating wave landed a hundred optional-trigger cards on a DSL that
/// cannot express them. Measured at close-out; re-measure rather than trust these.
const MAX_YOU_MAY_DEFS: usize = 400;
const MAX_YOU_MAY_COMPLETE: usize = 200;

#[test]
fn b4_the_corpus_wide_you_may_residual_is_ratcheted_not_closed() {
    let (mut total, mut complete) = (0usize, 0usize);
    for def in all_cards() {
        if !def.oracle_text.to_lowercase().contains("you may") {
            continue;
        }
        total += 1;
        if is_effectively_complete(&def) {
            complete += 1;
        }
    }
    // Non-vacuity: the substring must actually be found. A zero here means `oracle_text`
    // stopped being populated, not that the corpus stopped printing "you may".
    assert!(
        total > 100,
        "non-vacuity: only {total} defs print \"you may\"; the oracle-text field is \
         probably not being read"
    );
    assert!(
        total <= MAX_YOU_MAY_DEFS && complete <= MAX_YOU_MAY_COMPLETE,
        "the corpus-wide \"you may\" population grew past its ratchet ({total} defs, \
         {complete} of them `Complete`, ceilings {MAX_YOU_MAY_DEFS}/{MAX_YOU_MAY_COMPLETE}). \
         PB-DX35 served only the 5 defs carrying the DSL's dedicated `optional` flag; the \
         rest is OOS-DX35-5 / OOS-DX8-1 and is explicitly NOT closed."
    );
}

// ── The census report ────────────────────────────────────────────────────────

#[test]
/// PRINTS every population above. Run with `--nocapture`.
///
/// PB-DX8's rule: publish the figure, do not transcribe it. Every Half B population figure
/// in `memory/primitives/pb-DX35-execution-notes.md` and in CLAUDE.md is read off this
/// output.
fn t_census_report() {
    eprintln!("\n=== PB-DX35 Half B census (walked from all_cards(), never grepped) ===");
    let cs = carriers();
    eprintln!("B1 -- Effect::LookAtTopThenPlace carriers: {}", cs.len());
    for c in &cs {
        eprintln!(
            "  {:<28} optional(all)={:<5} place_cost={:<5} complete={:<5} nodes={}",
            c.name, c.optional, c.place_cost, c.complete, c.nodes
        );
    }
    let mut inverse: Vec<(String, String, bool)> = Vec::new();
    let (mut you_may, mut you_may_complete) = (0usize, 0usize);
    for def in all_cards() {
        if def.oracle_text.to_lowercase().contains("you may") {
            you_may += 1;
            if is_effectively_complete(&def) {
                you_may_complete += 1;
            }
        }
        if let Some(clause) = prints_look_then_place(&def) {
            if !def_contains_variant(&def, "LookAtTopThenPlace") {
                inverse.push((def.name.clone(), clause, is_effectively_complete(&def)));
            }
        }
    }
    inverse.sort();
    eprintln!(
        "\nB3 -- prints the shape, does NOT use the primitive (OOS-DX35-5): {}",
        inverse.len()
    );
    for (n, clause, complete) in &inverse {
        eprintln!("  {n:<28} complete={complete:<5} \"{clause}\"");
    }
    eprintln!(
        "\nB4 -- corpus-wide \"you may\" residual (NOT closed by this batch): {you_may} defs, \
         {you_may_complete} of them Complete"
    );
    eprintln!("=== end PB-DX35 Half B census ===\n");
}
