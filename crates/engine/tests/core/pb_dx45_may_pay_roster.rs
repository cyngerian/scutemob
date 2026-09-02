//! PB-DX45 (`OOS-DX24-9` ≡ `OOS-DX27-5`): the `Effect::MayPayThenEffect` roster gates.
//!
//! CR 118.12 makes an optional cost a **player** decision. Before this batch
//! `effects/mod.rs`'s `MayPayThenEffect` arm called `try_pay_optional_cost`
//! unconditionally for every eligible payer, so the engine paid whenever it
//! could. This file is the census that bounds the repaired class, and the
//! ratchet that stops it regrowing.
//!
//! * **R1** — the exact set of corpus defs carrying `Effect::MayPayThenEffect`,
//!   pinned by NAME, with the `Complete` (deck-legal) subset called out
//!   separately. A pin, so a 15th use is a deliberate act.
//! * **R2** — every corpus `MayPayThenEffect` cost is one `can_pay_optional_cost`
//!   can decide, i.e. NOT one of the nine arms that fall through to its tail.
//!   **That tail returns `false`, not `true`**, and the consequence is the
//!   opposite of what this doc claimed in its first draft: with PB-DX45's
//!   `if !can_pay_optional_cost(..) { continue; }` short-circuit, such a cost is
//!   **never asked about and the whole `then` arm silently never runs** — a
//!   silent no-op, not a harmless over-ask. Corrected by the batch's own
//!   `/review`, which proved it by executing
//!   `MayPayThenEffect { cost: Cost::Tap, then: GainLife(2) }` and observing
//!   `pending=None life=20 events=0`. The needle set and the gate were always
//!   right; only the stated reason was wrong, and wrong in the direction that
//!   would make a future author triage a real defect as a formality.
//! * **R3** — **inverse axis** (dispatch hygiene 6: the memo's figure is a
//!   FLOOR, so a second axis is mandatory). Starts from the printed oracle text
//!   rather than the DSL: every `Complete` def whose printed text carries an
//!   optional-cost idiom ("you may pay X", "you may sacrifice …", "you may
//!   discard …") and which does NOT carry `MayPayThenEffect`. That population is
//!   **filed, not taken** (`OOS-DX45-3`) — it is `OOS-DP10-9` / PB-DX8 territory
//!   — and is pinned here so it cannot grow in silence.
//! * **R4** — the SECOND `try_pay_optional_cost` call site. The seed names
//!   `Effect::MayPayThenEffect` and the engine has **two** callers of that
//!   helper: the `MayPayThenEffect` arm and `Effect::LookAtTopThenPlace`'s
//!   `place_cost` (`effects/mod.rs:6365`), which is the identical CR 118.12
//!   pay-when-able decision one function over, live on one deck-legal
//!   `Complete` def (`birthing_ritual`). A site list is a FLOOR
//!   (dispatch hygiene 6); this row pins the second site's population so a
//!   third cannot appear unnoticed.
//! * **`t_census_report`** — PRINTS every axis. Every population figure this
//!   batch publishes is read off this test's output, never transcribed
//!   (PB-DX8's rule; PB-DX28's execution notes quoted two fingerprints that had
//!   never existed in any source file).
//!
//! Reuses `decision_site_walk.rs`'s canonical serialized-JSON walk rather than a
//! second hand-written tree walk, for PB-DP10's reason: a hand-written walk is a
//! reachability claim and needs the same enumeration a match arm does.

use crate::decision_site_walk::{
    def_contains_variant, find_variant_nodes, is_effectively_complete,
};
use mtg_engine::all_cards;
use mtg_engine::CardDefinition;
use serde_json::Value;
use std::collections::BTreeSet;

// ── R1: the pinned roster ────────────────────────────────────────────────────

/// Every corpus def carrying `Effect::MayPayThenEffect`, by name — **14**.
const MAY_PAY_MEMBERS: &[&str] = &[
    "Crossway Troublemakers",
    "Disciple of Freyalise",
    "Ezuri, Stalker of Spheres",
    "Hazoret's Monument",
    "Kalastria Highborn",
    "Leaf-Crowned Visionary",
    "Mana Vault",
    "Miara, Thorn of the Glade",
    "Nadir Kraken",
    "Nether Traitor",
    "Ruthless Technomancer",
    "Springbloom Druid",
    "Tainted Observer",
    "Vampire Gourmand",
];

/// The `Complete` (deck-legal) subset of [`MAY_PAY_MEMBERS`] — the population
/// whose printed "you may" a real game could actually reach.
///
/// **TEN, and the v4 memo says ELEVEN.** `memory/primitives/seed-rerank-2026-08-14.md`
/// §1d records *"Two independent measurements of this task both returned 11
/// deck-legal `Complete` defs for the same population, which is what proves they
/// are one thing"* — and PB-DX45 re-derived it at HEAD, twice (this roster and
/// the `decision_gate.rs` `BASELINE`, which carries exactly ten
/// `may_pay_then_effect` entries), and got **10** both times. No corpus def in
/// this class has changed its `completeness` marker since PB-DX27
/// (`3390b6a9`, 2026-08-13), i.e. **before** the memo's census, so the corpus
/// did not move underneath it.
///
/// The memo's CONCLUSION (that `OOS-DX24-9` and `OOS-DX27-5` name one defect)
/// is right — they name the same `Effect` variant and the same handler — but the
/// EVIDENCE it offered for it was two agreeing wrong numbers. Recorded here
/// rather than silently corrected, because "a member list is a floor" has been
/// this queue's standing lesson for six batches and this is the first time a
/// published figure has been an over-count instead.
///
/// `vampire_gourmand` joins this set as part of PB-DX45 (the policy
/// re-adjudication, AC 7242) and is listed in [`MAY_PAY_COMPLETE_AFTER_DX45`].
const MAY_PAY_COMPLETE_BEFORE_DX45: &[&str] = &[
    "Crossway Troublemakers",
    "Disciple of Freyalise",
    "Hazoret's Monument",
    "Kalastria Highborn",
    "Leaf-Crowned Visionary",
    "Miara, Thorn of the Glade",
    "Nadir Kraken",
    "Nether Traitor",
    "Springbloom Druid",
    "Tainted Observer",
];

/// [`MAY_PAY_COMPLETE_BEFORE_DX45`] plus the one marker this batch flips.
///
/// **Only ONE of the four non-`Complete` members flips, and `OOS-DX27-5` claims
/// two.** That row says PB-DX27 *"left `ruthless_technomancer` and
/// `vampire_gourmand` at `partial` on the same shape"*. Read at HEAD, only
/// `vampire_gourmand`'s marker note cites the pay-when-able deviation;
/// `ruthless_technomancer`'s cites its **activated** ability ("no `Cost` variant
/// for a player-chosen variable-X sacrifice count"), which this batch does not
/// touch and which stays live. `ezuri_stalker_of_spheres` and `mana_vault` are
/// likewise blocked on unrelated gaps. The correction is carried back into the
/// registry row.
const MAY_PAY_COMPLETE_AFTER_DX45: &[&str] = &[
    "Crossway Troublemakers",
    "Disciple of Freyalise",
    "Hazoret's Monument",
    "Kalastria Highborn",
    "Leaf-Crowned Visionary",
    "Miara, Thorn of the Glade",
    "Nadir Kraken",
    "Nether Traitor",
    "Springbloom Druid",
    "Tainted Observer",
    "Vampire Gourmand",
];

fn may_pay_defs() -> Vec<CardDefinition> {
    all_cards()
        .into_iter()
        .filter(|d| def_contains_variant(d, "MayPayThenEffect"))
        .collect()
}

fn names(defs: &[CardDefinition]) -> BTreeSet<String> {
    defs.iter().map(|d| d.name.clone()).collect()
}

#[test]
fn r1_may_pay_roster_is_pinned() {
    let live = names(&may_pay_defs());
    let pinned: BTreeSet<String> = MAY_PAY_MEMBERS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        live, pinned,
        "PB-DX45 R1: the Effect::MayPayThenEffect roster moved. Every member's printed \
         \"you may\" is a CR 118.12 player decision served by \
         EffectChoiceQuestion::PayOptionalCost; a new member is a deliberate act, so \
         update MAY_PAY_MEMBERS (and the Complete subset below) in the same commit."
    );
    assert!(
        live.len() >= 10,
        "PB-DX45 R1 non-vacuity floor: the walk found {} members, which is fewer than the \
         ten this batch measured as deck-legal alone -- the walk is broken, not the corpus",
        live.len()
    );
}

#[test]
fn r1b_may_pay_complete_subset_is_pinned() {
    let live: BTreeSet<String> = may_pay_defs()
        .into_iter()
        .filter(is_effectively_complete)
        .map(|d| d.name)
        .collect();
    let pinned: BTreeSet<String> = MAY_PAY_COMPLETE_AFTER_DX45
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        live, pinned,
        "PB-DX45 R1b: the deck-legal Complete subset moved. See this constant's doc for \
         why it is ELEVEN after this batch and TEN before it, and why the v4 memo's \
         pre-batch figure of 11 does not reproduce."
    );
    let before: BTreeSet<String> = MAY_PAY_COMPLETE_BEFORE_DX45
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        pinned.difference(&before).cloned().collect::<Vec<_>>(),
        vec!["Vampire Gourmand".to_string()],
        "PB-DX45 R1b: exactly one marker flip was predicted before regeneration; the \
         two lists must differ by exactly Vampire Gourmand"
    );
}

// ── R2: every corpus cost is one `can_pay_optional_cost` can decide ──────────

/// The `Cost` variant tags `can_pay_optional_cost` decides on their own merits.
///
/// Its final arm (`Cost::Tap | SacrificeSelf | ExileSelf | Forage |
/// RemoveCounter | DiscardSelf | ExileFromHand | ExileSelfFromHand | Exert`)
/// returns an unconditional **`false`** (`effects/mod.rs`, the tail of
/// `can_pay_optional_cost`), so a `MayPayThenEffect` carrying one of those is
/// **never payable**, is therefore never asked about under PB-DX45's
/// determined short-circuit, and its `then` arm **never runs at all** — a silent
/// no-op on a printed card, which is a defect and not a formality.
///
/// **This doc said `true` and "asked unconditionally" in its first draft**, i.e.
/// it described the harmless failure instead of the real one. Corrected by the
/// batch's `/review`, which executed the case rather than reading the arm. The
/// needle set below is unchanged: it was always the right set, for a reason
/// stronger than the one originally given.
const DECIDABLE_COST_TAGS: &[&str] = &["Mana", "PayLife", "DiscardCard", "Sacrifice", "Sequence"];

/// The tag of a serialized `Cost`: an object key for a struct/tuple variant, the
/// string itself for a unit variant (`"DiscardCard"`).
fn cost_tag(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(m) => m.keys().next().cloned(),
        _ => None,
    }
}

fn corpus_cost_tags() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for def in may_pay_defs() {
        let json = serde_json::to_value(&def).expect("CardDefinition serializes");
        for node in find_variant_nodes(&json, "MayPayThenEffect") {
            if let Some(tag) = node.get("cost").and_then(cost_tag) {
                out.push((def.name.clone(), tag));
            }
        }
    }
    out
}

#[test]
fn r2_every_corpus_cost_is_decidable() {
    let tags = corpus_cost_tags();
    for (name, tag) in &tags {
        assert!(
            DECIDABLE_COST_TAGS.contains(&tag.as_str()),
            "PB-DX45 R2: {name}'s MayPayThenEffect carries Cost::{tag}, which \
             can_pay_optional_cost does not decide -- it falls through to that \
             function's tail, which returns FALSE. So this cost is never payable, \
             PB-DX45's determined short-circuit never asks about it, and the effect's \
             whole `then` arm silently never runs. Extend can_pay_optional_cost to \
             decide the cost, or add the tag here with a reason saying why a silent \
             no-op is acceptable for it."
        );
    }
    // Non-vacuity floors: the walk found something, and the set spans more than
    // one cost KIND (a walk that only ever saw `Mana` would pass this test while
    // being blind to the sacrifice arm, which is the expensive half of the seed).
    assert!(
        tags.len() >= 14,
        "PB-DX45 R2 non-vacuity: only {} cost nodes found; the walk is broken",
        tags.len()
    );
    let kinds: BTreeSet<&str> = tags.iter().map(|(_, t)| t.as_str()).collect();
    assert!(
        kinds.len() >= 4 && kinds.contains("Sacrifice") && kinds.contains("Mana"),
        "PB-DX45 R2 non-vacuity: cost kinds {kinds:?} -- the corpus is known to span \
         Mana, PayLife, DiscardCard, Sacrifice and Sequence"
    );
}

// ── R3: the inverse axis (the WIDER class, filed not taken) ─────────────────

/// Printed idioms that name an optional cost with a real price attached — the
/// exact shape `Effect::MayPayThenEffect` models.
///
/// Deliberately NOT the bare word "may": PB-DX8 measured **287** oracle-positive
/// `may` defs against **72** effectively-`Complete` ones with nothing in the DSL
/// able to express them, and that whole class is `OOS-DP10-9` / PB-DX8
/// territory. These needles are the *priced* sub-class, which is what this batch
/// is scoped to and therefore what its inverse axis must measure.
const OPTIONAL_COST_IDIOMS: &[&str] = &[
    "you may pay",
    "you may sacrifice",
    "you may discard",
    "may pay ",
];

/// Collect every `oracle_text` string reachable from a serialized
/// `CardDefinition` — the front face's AND every `CardFace`'s.
///
/// **Not `def.oracle_text` alone.** PB-DX8's census axis read exactly that and
/// was blind to every transformed face and Adventure half; the fix was found by
/// an inverse method rather than by a test. Walking the serialized tree reaches
/// them all by construction.
fn all_oracle_text(def: &CardDefinition) -> String {
    fn walk(v: &Value, out: &mut String) {
        match v {
            Value::Object(m) => {
                for (k, child) in m {
                    if k == "oracle_text" {
                        if let Some(s) = child.as_str() {
                            out.push_str(s);
                            out.push('\n');
                        }
                    }
                    walk(child, out);
                }
            }
            Value::Array(items) => items.iter().for_each(|i| walk(i, out)),
            _ => {}
        }
    }
    let json = serde_json::to_value(def).expect("CardDefinition serializes");
    let mut out = String::new();
    walk(&json, &mut out);
    out.to_lowercase()
}

fn inverse_axis() -> BTreeSet<String> {
    all_cards()
        .into_iter()
        .filter(is_effectively_complete)
        .filter(|d| !def_contains_variant(d, "MayPayThenEffect"))
        .filter(|d| {
            let text = all_oracle_text(d);
            OPTIONAL_COST_IDIOMS.iter().any(|n| text.contains(n))
        })
        .map(|d| d.name)
        .collect()
}

/// Every `Complete` def printing a PRICED optional cost that this batch does
/// **not** repair, because it is not expressed as `Effect::MayPayThenEffect`.
///
/// **This is a FILING, not a fix** (`OOS-DX45-3`). AC 7243 scopes PB-DX45 to
/// `MayPayThenEffect`; a wider auto-taken-"may" class is `OOS-DP10-9` /
/// PB-DX8 territory and is to be filed rather than taken. The list is pinned so
/// that (a) the number this batch publishes is measured rather than asserted and
/// (b) the class cannot grow in silence between now and the batch that takes it.
///
/// Each member reaches its optional cost through some OTHER construct — an
/// as-enters replacement (`EntersTappedUnlessPayLife`), a keyword's own cost, an
/// `Effect::Conditional`, or nothing at all. Sorted by name; the members
/// themselves are printed by `t_census_report`.
const INVERSE_AXIS_MEMBERS: &[&str] = &[
    "Birthing Ritual",
    "Blood Crypt",
    "Breeding Pool",
    "Crypt Ghast",
    "Force of Will",
    "Galadhrim Brigade",
    "Godless Shrine",
    "Grim Harvest",
    "Hallowed Fountain",
    "Huddle Up",
    "Make Disappear",
    "Mind Games",
    "Nullpriest of Oblivion",
    "Overgrown Tomb",
    "Predator Dragon",
    "Sacred Foundry",
    "Saw It Coming",
    "Sea Gate Restoration",
    "Searing Touch",
    "Slickshot Show-Off",
    "Steam Vents",
    "Stomping Ground",
    "Syndic of Tithes",
    "Temple Garden",
    "Teneb, the Harvester",
    "Ultramarines Honour Guard",
    "Watery Grave",
];

#[test]
fn r3_inverse_axis_is_pinned() {
    let live = inverse_axis();
    let pinned: BTreeSet<String> = INVERSE_AXIS_MEMBERS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        live, pinned,
        "PB-DX45 R3 (inverse axis, OOS-DX45-3): the set of Complete defs printing a \
         PRICED optional cost without carrying Effect::MayPayThenEffect moved. This class \
         is FILED, not fixed, by PB-DX45 -- a new member means either a def that should \
         have used MayPayThenEffect, or a genuine growth of OOS-DX45-3's population. \
         Decide which, then re-pin."
    );
}

// ── R4: the SECOND try_pay_optional_cost call site ──────────────────────────

/// Every corpus def whose `Effect::LookAtTopThenPlace` sets a `place_cost` —
/// the engine's OTHER CR 118.12 pay-when-able decision.
///
/// `effects/mod.rs` has exactly two callers of `try_pay_optional_cost`: the
/// `MayPayThenEffect` arm the seed names, and this one. Both were unconditional
/// before PB-DX45 and both now ask; pinning the second site's population is what
/// makes "every `try_pay_optional_cost` caller asks" a measured statement rather
/// than an assertion about the two the batch happened to read.
///
/// One member, and it is deck-legal `Complete`: `birthing_ritual`'s printed
/// *"Then you may sacrifice a creature. If you do, …"*. Five further defs use
/// `LookAtTopThenPlace` with `place_cost: None` and are correctly outside this
/// set — an absent cost is not an optional one.
const PLACE_COST_MEMBERS: &[&str] = &["Birthing Ritual"];

fn place_cost_defs() -> BTreeSet<String> {
    all_cards()
        .into_iter()
        .filter(|d| {
            let json = serde_json::to_value(d).expect("CardDefinition serializes");
            find_variant_nodes(&json, "LookAtTopThenPlace")
                .iter()
                .any(|n| n.get("place_cost").map(|c| !c.is_null()).unwrap_or(false))
        })
        .map(|d| d.name)
        .collect()
}

#[test]
fn r4_second_pay_site_population_is_pinned() {
    let live = place_cost_defs();
    let pinned: BTreeSet<String> = PLACE_COST_MEMBERS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        live, pinned,
        "PB-DX45 R4: the Effect::LookAtTopThenPlace `place_cost` population moved. That \
         field is the engine's SECOND CR 118.12 optional-cost payment, served by the same \
         EffectChoiceQuestion::PayOptionalCost as MayPayThenEffect."
    );
    // Non-vacuity: the walk must actually reach `LookAtTopThenPlace` nodes, not
    // merely return an empty set because the key name is wrong.
    //
    // **FIVE, and a source grep says six** — SR-36's exact failure mode, fired
    // by this floor on its first run. `grep -rn LookAtTopThenPlace
    // crates/card-defs/src/defs/` returns six FILES, but `muxus_goblin_grandee`
    // only names the effect in a comment. The compiled corpus carries five:
    // `birthing_ritual` (the one with a `place_cost`), `grisly_salvage`,
    // `growing_rites_of_itlimoc`, `risen_reef`, `satyr_wayfinder`. The floor is
    // written against the `all_cards()` walk, which is the ground truth.
    let carriers = all_cards()
        .into_iter()
        .filter(|d| def_contains_variant(d, "LookAtTopThenPlace"))
        .count();
    assert!(
        carriers >= 5,
        "PB-DX45 R4 non-vacuity: only {carriers} defs carry LookAtTopThenPlace; the walk \
         is broken, not the corpus"
    );
}

// ── The census report ────────────────────────────────────────────────────────

#[test]
fn t_census_report() {
    let defs = may_pay_defs();
    let complete: BTreeSet<String> = defs
        .iter()
        .filter(|d| is_effectively_complete(d))
        .map(|d| d.name.clone())
        .collect();
    println!("=== PB-DX45 census (printed, never transcribed) ===");
    println!(
        "forward axis (Effect::MayPayThenEffect): {} defs",
        defs.len()
    );
    println!("  all:         {:?}", names(&defs));
    println!(
        "  deck-legal Complete: {} -> {:?}",
        complete.len(),
        complete
    );
    let mut kinds: Vec<(String, String)> = corpus_cost_tags();
    kinds.sort();
    println!("  cost kinds:");
    for (name, tag) in &kinds {
        println!("    {name:<32} Cost::{tag}");
    }
    println!(
        "second pay site (LookAtTopThenPlace.place_cost): {:?}",
        place_cost_defs()
    );
    let inv = inverse_axis();
    println!(
        "inverse axis (printed priced optional cost, NO MayPayThenEffect): {} defs",
        inv.len()
    );
    for n in &inv {
        println!("    {n}");
    }
    println!("=== END ===");
}
