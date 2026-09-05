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

// ─────────────────────────────────────────────────────────────────────────────
// PB-DX57 (`OOS-DX28-1`) — the shared MATCH-ARM parser, and R2's declaration pin
// ─────────────────────────────────────────────────────────────────────────────

/// **Why a match-arm parser lives here.**
///
/// `core::pb_dx57_declared_source` is the one parser for *type declarations*
/// (`pub enum X { .. }`, `pub struct X { .. }`), and its module doc states, as a
/// deliberate omission, that it holds no helper for a FUNCTION's match arms —
/// arms have guards, `|` alternatives and nested matches, and a half-right
/// shared one would be worse than the specific ones.
///
/// Two `OOS-DX28-1` members need exactly that, though, and they need the *same*
/// shape: `DECIDABLE_COST_TAGS` below must be derived from
/// `effects::can_pay_optional_cost`'s arms, and
/// `pb_dx28_chosen_object_roster::SUPPORTED_ARMS` from
/// `effects::resolve_pending_object_choices`'s. Writing it twice would be this
/// seed's own mistake at a smaller scale, so it is written once, here, and
/// `pb_dx28_chosen_object_roster` calls it.
///
/// **Scope, stated rather than implied.** This handles the ONE arm shape both
/// subjects use: heads at a fixed indentation naming `Enum::Variant`, possibly
/// `|`-alternated across lines, with the body running to the next head. It does
/// not model guards (`if`), nested matches or `@` bindings, and it does not try
/// to. `pb_dx36_deals_damage_roster::extract_function_body` and
/// `pb_dx39_source_relative_roster::effect_applies_to_arms` keep their own
/// hand-written extractors: rewiring two working, differently-shaped derivations
/// is not this row's scope, and is recorded here so the next reader knows the
/// duplication is a decision rather than an oversight.
pub(crate) const EFFECTS_MOD_RS: &str = "crates/engine/src/effects/mod.rs";

/// One `match` arm: every enum variant its pattern names (more than one for a
/// `|`-alternation) and the text of its body.
pub(crate) struct MatchArm {
    pub(crate) names: Vec<String>,
    pub(crate) body: String,
}

/// Strip `//` line comments and `/* */` block comments, preserving byte length
/// (comment bytes become spaces, newlines survive) so line-indentation parsing
/// still works on the result.
///
/// BOTH kinds, not just `//`: PB-DX8's `/* */` defeat is on record — the
/// byte-identical sentence reddened as a line comment and left every test green
/// as a block comment. String literals are respected so a `//` inside one is not
/// deleted as if it were code.
pub(crate) fn strip_comments_preserving_length(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i] as char;
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < b.len() {
                out.push(b[i + 1] as char);
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if c == '/' && b.get(i + 1) == Some(&b'*') {
            let mut depth = 1usize;
            out.push_str("  ");
            i += 2;
            while i < b.len() && depth > 0 {
                if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                } else {
                    out.push(if b[i] == b'\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// The brace-matched body of `{` at `open`, INCLUDING both braces.
fn brace_body(src: &str, open: usize) -> &str {
    assert_eq!(
        src.as_bytes().get(open),
        Some(&b'{'),
        "brace_body must be given the offset of a `{{`"
    );
    let mut depth = 0usize;
    for (off, ch) in src[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[open..open + off + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced braces — the body starting at {open} is never closed");
}

/// The comment-stripped body of `fn <fn_name>(` in the workspace-relative file
/// `rel`, braces included.
pub(crate) fn function_body(rel: &str, fn_name: &str) -> String {
    let raw = crate::pb_dx57_declared_source::read_workspace_file(rel);
    let src = strip_comments_preserving_length(&raw);
    let marker = format!("fn {fn_name}(");
    let at = src.find(&marker).unwrap_or_else(|| {
        panic!(
            "`fn {fn_name}(` not found in {rel}. The function was renamed or moved — \
             re-point this pin at wherever it now lives, and do NOT delete the pin and \
             keep the hand-written list, which is the defect OOS-DX28-1 names."
        )
    });
    let open = src[at..]
        .find('{')
        .map(|i| i + at)
        .unwrap_or_else(|| panic!("`fn {fn_name}` in {rel} has no body"));
    brace_body(&src, open).to_string()
}

/// The arms of `<match_header>` inside `fn <fn_name>(` in `rel`, keyed on head
/// lines that name `<enum_prefix>::Variant` at exactly `indent` spaces.
///
/// Panics on an empty parse: an arm list that comes back `[]` makes every
/// `assert_eq!` against it trivially satisfiable, which is `OOS-DX28-1`'s own
/// failure mode re-entering through its fix.
pub(crate) fn match_arms(
    rel: &str,
    fn_name: &str,
    match_header: &str,
    enum_prefix: &str,
    indent: usize,
) -> Vec<MatchArm> {
    let fn_body = function_body(rel, fn_name);
    let head_at = fn_body.find(match_header).unwrap_or_else(|| {
        panic!(
            "`{match_header}` not found inside `fn {fn_name}` in {rel} — the match was \
                rewritten, and this pin is measuring nothing until it is re-pointed"
        )
    });
    let open = fn_body[head_at..]
        .find('{')
        .map(|i| i + head_at)
        .expect("a match header ends in `{`");
    // The INNER text, without the match's own `{`/`}`: leaving the closing brace in
    // would append `\n    }` to the final arm's body, and every "is this body an
    // unconditional `false`" test would then be false for the last arm — measured,
    // not guessed: the first draft of this parser classified all 14 `Cost` variants
    // as decidable for exactly that reason.
    let whole = brace_body(&fn_body, open);
    let body = &whole[1..whole.len() - 1];

    let head_prefix = format!("{enum_prefix}::");
    let alt_prefix = format!("| {enum_prefix}::");

    let mut arms: Vec<MatchArm> = Vec::new();
    let mut pending: Vec<String> = Vec::new();
    let mut cur_body: Option<String> = None;
    for line in body.lines() {
        let trimmed = line.trim_start();
        let ind = line.len() - trimmed.len();
        let is_head = ind == indent
            && (trimmed.starts_with(&head_prefix) || trimmed.starts_with(&alt_prefix));
        if !is_head {
            if let Some(b) = cur_body.as_mut() {
                b.push('\n');
                b.push_str(line);
            }
            continue;
        }
        if let Some(b) = cur_body.take() {
            arms.push(MatchArm {
                names: std::mem::take(&mut pending),
                body: b,
            });
        }
        let mut from = 0usize;
        while let Some(i) = trimmed[from..].find(&head_prefix) {
            let at = from + i + head_prefix.len();
            let name: String = trimmed[at..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                pending.push(name);
            }
            from = at;
        }
        if let Some(a) = trimmed.find("=>") {
            cur_body = Some(trimmed[a + 2..].to_string());
        }
    }
    if let Some(b) = cur_body.take() {
        arms.push(MatchArm {
            names: pending,
            body: b,
        });
    }

    assert!(
        !arms.is_empty(),
        "match_arms({rel}, {fn_name}, {match_header:?}, {enum_prefix}, indent {indent}) parsed \
         ZERO arms. Every assert_eq! against this set would be trivially true — the \
         arm-head indentation convention changed, and this pin is measuring nothing."
    );
    arms
}

/// Does this arm body return an unconditional `false` — i.e. is the variant one
/// the function declines to decide?
fn body_is_unconditional_false(body: &str) -> bool {
    body.trim().trim_end_matches(',').trim() == "false"
}

/// **`DECIDABLE_COST_TAGS` is now DERIVED from `can_pay_optional_cost` itself,
/// and the two halves catch opposite failures.**
///
/// `OOS-DX28-1` census row 11. The census's own account of the danger, which
/// `r2` above cannot see:
///
/// > Correct at HEAD (5 + 9 = 14 ✓). The dangerous direction is **shrinkage, not
/// > growth**: if `can_pay_optional_cost` stops handling e.g. `Sacrifice`, the
/// > const still lists it, `r2` stays green, and every `MayPayThenEffect`
/// > carrying that cost becomes a silent never-payable no-op — which is
/// > precisely the defect `r2`'s docstring says it exists to catch.
///
/// So this row asserts **two** things, and only the two together bound the class:
///
/// 1. `DECIDABLE_COST_TAGS == { arms of can_pay_optional_cost whose body is not
///    an unconditional `false` }`. A variant moving from a deciding arm into the
///    `false` alternation reddens HERE and nowhere else — `r2` keeps comparing
///    the corpus against a list that still names it.
/// 2. `decidable ∪ undecidable == pub enum Cost`'s declared variants, and the
///    two are disjoint. Rust's exhaustiveness already forces the union when the
///    match has no wildcard; this assertion is what notices the day someone adds
///    `_ => false` and a new `Cost` variant silently becomes undecidable without
///    anyone writing its name down. That is the only edit of this shape that
///    COMPILES, and it is therefore the only one worth a test.
///
/// **Revert to watch red**: move `Cost::Sacrifice` into the trailing `false`
/// alternation (leg 1), or replace that whole alternation with `_ => false`
/// (leg 2).
#[test]
fn r2b_decidable_cost_tags_are_derived_from_can_pay_optional_cost() {
    // **STATED RESIDUAL — this derivation is SYNTACTIC, not semantic, and the adversarial pass
    // defeated it on exactly that.** It decides "decidable" by whether the arm's body is the
    // literal `false`; planting `Cost::PayLife(n) => *n > u32::MAX` leaves this gate GREEN while
    // `PayLife` refuses every payment it is ever asked about. No source-level derivation can
    // close that — deciding whether an arm's body is semantically constant is the halting
    // problem in miniature — so it is recorded rather than papered over.
    //
    // **What DOES cover it, measured**: the same plant reddens SEVEN probes in
    // `primitives::pb_dx45_optional_cost` plus `bare_lookup_ratchet`. So the behaviour is well
    // covered by behavioural tests and this gate's job is narrower than its name suggests — it
    // holds the LIST against the FUNCTION's arm structure, which is what stops a variant
    // silently leaving the decidable set by deletion. Saying which half is covered by what is
    // the point (`OOS-DX49-6`: a comment asserting a property the code does not enforce).
    let arms = match_arms(
        EFFECTS_MOD_RS,
        "can_pay_optional_cost",
        "match cost {",
        "Cost",
        8,
    );

    let mut decidable: BTreeSet<String> = BTreeSet::new();
    let mut undecidable: BTreeSet<String> = BTreeSet::new();
    for arm in &arms {
        let bucket = if body_is_unconditional_false(&arm.body) {
            &mut undecidable
        } else {
            &mut decidable
        };
        for n in &arm.names {
            bucket.insert(n.clone());
        }
    }

    println!(
        "PB-DX57 row 11: can_pay_optional_cost has {} arm group(s); decides {:?}; \
         declines {:?}",
        arms.len(),
        decidable,
        undecidable
    );

    let pinned: BTreeSet<String> = DECIDABLE_COST_TAGS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert_eq!(
        decidable, pinned,
        "DECIDABLE_COST_TAGS no longer matches the arms of \
         `effects::can_pay_optional_cost` that actually decide a cost. A variant that \
         LEAVES the decidable set while staying on this list makes every corpus \
         MayPayThenEffect carrying it a silent never-payable no-op, with r2 above still \
         green — that is OOS-DX28-1's failure mode on this constant. Re-derive the census \
         before re-pinning the list."
    );

    let declared = crate::pb_dx57_declared_source::declared_enum_variants(
        crate::pb_dx57_declared_source::CARD_DEFINITION_RS,
        "Cost",
    );
    assert!(
        decidable.is_disjoint(&undecidable),
        "a Cost variant is classified BOTH decidable and undecidable: {:?}",
        decidable.intersection(&undecidable).collect::<Vec<_>>()
    );
    let union: BTreeSet<String> = decidable.union(&undecidable).cloned().collect();
    assert_eq!(
        union,
        declared,
        "`can_pay_optional_cost` no longer names every declared `Cost` variant. Either a \
         wildcard arm was added (so a new variant is silently undecidable and nobody wrote \
         its name down), or this parser's arm-head convention broke. declared-but-unclassified \
         = {:?}, classified-but-undeclared = {:?}",
        declared.difference(&union).collect::<Vec<_>>(),
        union.difference(&declared).collect::<Vec<_>>()
    );
    // Non-vacuity: the `false` tail is the half r2 cannot see, so it must be
    // non-empty for this row to be measuring the interesting direction at all.
    assert!(
        undecidable.len() >= 5,
        "non-vacuity: only {} Cost variant(s) parsed as undecidable (measured 9 at HEAD); \
         the arm walk has stopped seeing the `|`-alternated tail: {undecidable:?}",
        undecidable.len()
    );
}
