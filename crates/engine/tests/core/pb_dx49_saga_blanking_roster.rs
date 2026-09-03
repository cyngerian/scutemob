//! PB-DX49 (`OOS-RR4-1`, rider `OOS-RR4-3`): the CR 714 Saga / blanked-abilities
//! **census** gate.
//!
//! `rules::saga::saga_view` answers "is this a Saga", "which chapter abilities does it
//! still have" and "what is its final chapter number" once, after consulting the layer
//! axis, so the five CR 714 decision sites cannot disagree. That is a change to the
//! *engine*. This file measures the **corpus**: which defs are Sagas, which defs can blank
//! a permanent's abilities, which of those pairs are actually reachable, and which source
//! sites consume the query. A population gate, in the SR-36 shape — every roster below is
//! built by walking `all_cards()` or by parsing engine source, **never** by grepping card
//! source for a variant name.
//!
//! ## Why a grep is not allowed here, stated with the receipts
//!
//! Two of this file's six rows had their orientation figure refuted by their own walk, and
//! both refutations are SR-36's rule (*enumerate `all_cards()`, never grep source*) paying
//! for itself:
//!
//! - **`grep -l SagaChapter` returns 4 files; the walk returns 3.** `song_of_freyalise.rs`
//!   declares `abilities: vec![]` and names `SagaChapter` only inside two `// TODO`s and its
//!   `Completeness::inert` note. It is `r2`'s member, not `r1`'s.
//! - **`OOS-RR4-1` says 13 blanker defs / 8 deck-legal; its own 2026-08-14 correction says
//!   9; the walk says 11 / 8.** 13 is a bare `RemoveAllAbilities` grep counting
//!   `blood_moon.rs` and `magus_of_the_moon.rs`, whose declarations PB-DX43 replaced. 9 is
//!   the right count of `RemoveAllAbilities` *declarations* and the wrong count of
//!   *blankers*, because PB-DX43 moved CR 305.7's ability loss into
//!   `LayerModification::SetLandTypes` — so the two moons are blankers again, through a
//!   different variant. Deciding membership by **calling
//!   `layers::modification_blanks_abilities`** rather than by matching a variant name is
//!   what makes that channel visible. See `BLANKER_DEFS`.
//!
//! That is `OOS-CARDS2-7` / `OOS-DX47-2` / `OOS-DX48`'s shape — a source-text grep counting
//! prose as usage — for the fourth consecutive batch in this queue. Every population in
//! this file is a walk.
//!
//! ## Rows
//!
//! * **r1** — the Saga population, structural: every def declaring
//!   `AbilityDefinition::SagaChapter` on **either** face, pinned by name, with the
//!   deck-legal `Complete` subset pinned separately.
//! * **r2** — the Saga population, **INVERSE** (oracle-text axis): every def whose printed
//!   text on any face reads as a Saga (a chapter marker line, or CR 714's reminder
//!   wording), minus r1's set. The two axes **do not nest** — that is PB-DX26's, PB-DX43's,
//!   PB-DX45's and PB-DX47's lesson, and it is why both are here rather than one.
//! * **r3** — the blanker population, structural, decided **by calling**
//!   `rules::layers::modification_blanks_abilities` rather than by matching variant names,
//!   so PB-DX43's CR 305.7 channel — and any fourth channel a later batch adds to that
//!   exhaustive match — is covered by construction.
//! * **r4** — the blanker × Saga pair table, each row keyed on the *mechanism* that decides
//!   whether the blanker can reach an enchantment.
//! * **r5** — the face-down channel's real reach: every def that can put a permanent onto
//!   the battlefield face down through `Effect::Manifest` / `Effect::Cloak` (**3**, two of
//!   them deck-legal), plus the two facts that bound it — that neither arm reaches
//!   CR 714.3a's site (`OOS-DX49-2`), and that the *other* face-down channel (the
//!   morph/disguise cast path, which **does** reach it) has no corpus Saga member.
//! * **r6** — the site roster: exactly which functions consume `saga::saga_view`, and the
//!   CR 113.7a exclusion asserted as a **zero**. Walked over **workspace source** — every
//!   `crates/*/src` and `tools/*/src` bar `crates/card-defs/src` — not over one crate.
//! * **r7** — the ability-blanking **variant-naming** roster: every site in workspace source
//!   that names `LayerModification::RemoveAllAbilities` or `::SetLandTypes` itself, rather
//!   than asking `layers::modification_blanks_abilities`. `replacement.rs` asserts in bold
//!   that *"there must be exactly one blanking predicate in the tree"*; before this row that
//!   claim was **ungated**, and the `/review` proved it by appending a second hand-rolled
//!   predicate to `turn_actions.rs` — the exact pre-PB-DX43 shape whose 26-def regression
//!   `layers.rs`'s own doc comment narrates — with the whole `--test core` target GREEN.
//! * **r8** — `modification_blanks_abilities` classified **exhaustively**: one instance of
//!   every `LayerModification` variant, with the variant set gated against the enum's own
//!   declaration. PB-DX43's `f3_..._recognises_both_channels_and_no_others` pins 2 positives
//!   and 5 hand-picked negatives out of 33, so *"and no others"* was an overclaim, and this
//!   file made it load-bearing at a second site: the `/review` moved
//!   `SwitchPowerToughness` into the `true` arm and the entire `-p mtg-engine` set stayed
//!   green, because r3 only reddens where the **corpus** happens to reach.
//! * **`t_census_report`** — PRINTS every population above, with names, under
//!   `--nocapture`. PB-DX8's rule: a figure that no test prints is a figure that was
//!   transcribed.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mtg_engine::rules::layers::modification_blanks_abilities;
use mtg_engine::{
    all_cards, AbilityDefinition, ActivatedAbility, CardDefinition, CardType, Characteristics,
    Color, EffectAmount, EnchantTarget, KeywordAbility, LayerModification, ManaAbility, ObjectId,
    PlayerId, SubType, SuperType,
};
use serde_json::Value;

use crate::decision_site_walk::is_effectively_complete;

// ─────────────────────────────────────────────────────────────────────────────
// Shared walk helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Every ability list a declaration can hide in: the front face's, and every alternate
/// face's. A `CardFace` carries its **own** `abilities` and its **own** `oracle_text`, and
/// reading only `def.abilities` / `def.oracle_text` is `OOS-DX8`'s exact defect (PB-DX8's
/// oracle axis was blind to every transformed face and Adventure half until it was
/// widened).
fn all_ability_lists(def: &CardDefinition) -> Vec<(&'static str, &[AbilityDefinition])> {
    let mut out: Vec<(&'static str, &[AbilityDefinition])> = vec![("front", &def.abilities)];
    if let Some(face) = def.back_face.as_ref() {
        out.push(("back", &face.abilities));
    }
    if let Some(face) = def.adventure_face.as_ref() {
        out.push(("adventure", &face.abilities));
    }
    out
}

/// Every face's printed text, lowercased, one entry per face.
fn all_oracle_texts(def: &CardDefinition) -> Vec<String> {
    let mut out = vec![def.oracle_text.to_lowercase()];
    for face in [def.back_face.as_ref(), def.adventure_face.as_ref()]
        .into_iter()
        .flatten()
    {
        out.push(face.oracle_text.to_lowercase());
    }
    out
}

/// Collect every JSON value stored under an object key equal to `key`, at any depth.
///
/// Depth-agnostic on purpose: a `ContinuousEffectDef` can sit directly under
/// `AbilityDefinition::Static`, or nested inside `Effect::ApplyContinuousEffect` inside a
/// `Sequence` inside a mode inside a chapter ability. A walk keyed on the parent key
/// measures the parent key (`pb_dx42a_continuous_condition_roster`'s finding).
fn collect_under_key<'a>(v: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(m) => {
            for (k, child) in m {
                if k == key {
                    out.push(child);
                }
                collect_under_key(child, key, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_under_key(item, key, out);
            }
        }
        _ => {}
    }
}

/// Serde variant name of a value: unit variants serialize as a bare string, struct/tuple
/// variants as a single-key object.
fn variant_name(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(m) if m.len() == 1 => m.keys().next().cloned(),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// r1 — Saga population, STRUCTURAL
// ─────────────────────────────────────────────────────────────────────────────

/// One def's declared chapter abilities: `(face label, chapter number)`, in declaration
/// order.
fn declared_chapters(def: &CardDefinition) -> Vec<(&'static str, u32)> {
    let mut out = Vec::new();
    for (label, abilities) in all_ability_lists(def) {
        // Serialize the FACE's ability list, not the whole def: that is what attributes a
        // chapter to a face. The walk is depth-agnostic within the face, so a chapter
        // nested inside (say) a Class level or a modal ability is still found.
        let json = serde_json::to_value(abilities).expect("abilities serialize");
        let mut nodes = Vec::new();
        collect_under_key(&json, "SagaChapter", &mut nodes);
        for node in nodes {
            let ch = node
                .get("chapter")
                .and_then(Value::as_u64)
                .expect("AbilityDefinition::SagaChapter always carries `chapter: u32`")
                as u32;
            out.push((label, ch));
        }
    }
    out
}

struct SagaDef {
    name: String,
    complete: bool,
    chapters: Vec<(&'static str, u32)>,
}

fn saga_defs() -> Vec<SagaDef> {
    let mut out: Vec<SagaDef> = all_cards()
        .iter()
        .filter_map(|def| {
            let chapters = declared_chapters(def);
            if chapters.is_empty() {
                return None;
            }
            Some(SagaDef {
                name: def.name.clone(),
                complete: is_effectively_complete(def),
                chapters,
            })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// The **three** defs declaring `AbilityDefinition::SagaChapter` at HEAD.
///
/// **This corrects the plan's orientation figure of four, and the correction is SR-36's
/// rule paying for itself inside the batch that cites it.** `grep -l SagaChapter
/// crates/card-defs/src/defs/*.rs` returns **four** files; the fourth is
/// `song_of_freyalise.rs`, whose `abilities: vec![]` is **empty** and which names
/// `SagaChapter` only inside two `// TODO` comments and its `Completeness::inert` note. It
/// declares no chapter ability at all. That is `OOS-CARDS2-7` / `OOS-DX47-2` / `OOS-DX48`'s
/// shape for the fourth consecutive batch in this queue — a source grep counting prose as
/// usage — and it is why every population in this file is a walk. `song_of_freyalise` is
/// `r2`'s member, not this row's.
const SAGA_DEFS: &[&str] = &[
    "Binding the Old Gods",
    "Fable of the Mirror-Breaker",
    "Urza's Saga",
];

/// The deck-legal `Complete` Saga subset. **One member.** `binding_the_old_gods` declares
/// no `completeness` field at all, so it takes the `#[default] Completeness::Complete`
/// derive — the same mechanism behind five of the eight live-wrong defs the v3 re-rank
/// found, and the reason "no marker" must never be read as "not deck-legal".
const SAGA_DEFS_DECK_LEGAL: &[&str] = &["Binding the Old Gods"];

/// CR 714.1 / CR 714.2a: a Saga is an enchantment with chapter abilities. This row pins the
/// structural population; a new Saga def must be classified here (and, if it is deck-legal,
/// in `SAGA_DEFS_DECK_LEGAL`) before this row can be re-pinned.
#[test]
fn r1_saga_population_is_pinned() {
    let defs = saga_defs();
    let live: BTreeSet<String> = defs.iter().map(|d| d.name.clone()).collect();
    let pinned: BTreeSet<String> = SAGA_DEFS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        live,
        pinned,
        "PB-DX49 r1: the AbilityDefinition::SagaChapter population moved. live only: {:?}; \
         pinned only: {:?}",
        live.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&live).collect::<Vec<_>>()
    );

    let live_legal: BTreeSet<String> = defs
        .iter()
        .filter(|d| d.complete)
        .map(|d| d.name.clone())
        .collect();
    let pinned_legal: BTreeSet<String> =
        SAGA_DEFS_DECK_LEGAL.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        live_legal,
        pinned_legal,
        "PB-DX49 r1: the DECK-LEGAL (Completeness::Complete) Saga subset moved. This is the \
         population every CR 714 behavioural claim in PB-DX49 rests on. live only: {:?}; \
         pinned only: {:?}",
        live_legal.difference(&pinned_legal).collect::<Vec<_>>(),
        pinned_legal.difference(&live_legal).collect::<Vec<_>>()
    );
}

/// Non-vacuity floor for r1's walk: the chapter numbers must actually be read out of the
/// declarations, not merely counted. A walk that found the key but could not read
/// `chapter` would pass the set assertions above while measuring nothing.
///
/// `binding_the_old_gods` is the worked case — three chapters, I/II/III, all on the front
/// face — because it is the one deck-legal member and therefore the subject of every
/// PB-DX49 behavioural probe.
#[test]
fn r1b_chapter_numbers_are_read_not_merely_counted() {
    let defs = saga_defs();
    let binding = defs
        .iter()
        .find(|d| d.name == "Binding the Old Gods")
        .expect("r1 pins Binding the Old Gods as a Saga def");
    assert_eq!(
        binding.chapters,
        vec![("front", 1u32), ("front", 2), ("front", 3)],
        "CR 714.2d: Binding the Old Gods prints chapters I/II/III on its front face; the \
         walk must read the chapter NUMBERS, since `SagaView::final_chapter` is derived \
         from them"
    );
    // A DFC Saga attributes its chapters to the face that prints them (CR 712.8d/e).
    let fable = defs
        .iter()
        .find(|d| d.name == "Fable of the Mirror-Breaker")
        .expect("r1 pins Fable of the Mirror-Breaker as a Saga def");
    assert!(
        fable.chapters.iter().all(|(face, _)| *face == "front"),
        "Fable of the Mirror-Breaker prints its chapters on its FRONT face (the back face \
         is Reflection of Kiki-Jiki); got {:?}",
        fable.chapters
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r2 — Saga population, INVERSE (oracle-text axis)
// ─────────────────────────────────────────────────────────────────────────────

/// Does this printed line look like a Saga chapter marker — `"I — ..."`, `"II — ..."`,
/// `"I, II — ..."`?
///
/// Decided on the text BEFORE the first em dash: it must be non-empty and consist only of
/// roman-numeral letters, commas and spaces. Keying on the dash rather than on a fixed set
/// of literal prefixes is what makes `"I, II, III —"` (Kaya's / the Kamigawa-style bundled
/// chapters) visible without enumerating every bundling a card might print.
fn line_is_chapter_marker(line: &str) -> bool {
    let Some(idx) = line.find('\u{2014}') else {
        return false;
    };
    let prefix = line[..idx].trim();
    !prefix.is_empty()
        && prefix
            .chars()
            .all(|c| matches!(c, 'i' | 'v' | 'x' | ',' | ' '))
}

/// CR 714.2b's reminder wording, as printed on every Saga.
const SAGA_REMINDER_NEEDLES: &[&str] = &["lore counter", "sacrifice after"];

/// Does any face's printed text read as a Saga?
fn prints_as_saga(def: &CardDefinition) -> bool {
    all_oracle_texts(def).iter().any(|text| {
        SAGA_REMINDER_NEEDLES.iter().any(|n| text.contains(n))
            || text.lines().any(line_is_chapter_marker)
    })
}

/// Every def whose printed text reads as a Saga, sorted.
fn oracle_saga_defs() -> Vec<String> {
    let mut out: Vec<String> = all_cards()
        .iter()
        .filter(|def| prints_as_saga(def))
        .map(|def| def.name.clone())
        .collect();
    out.sort();
    out
}

/// The oracle-axis residual: defs that PRINT a Saga (or print CR 714 lore-counter wording)
/// and declare **no** `AbilityDefinition::SagaChapter`.
///
/// This is not a defect list by construction — a card that merely *references* lore
/// counters ("Sagas you control") lands here too, and over-collection can only make the row
/// redder, never greener. Each member is classified below with a reason.
const ORACLE_AXIS_RESIDUAL: &[(&str, &str)] = &[(
    "Song of Freyalise",
    "Prints the full CR 714 reminder text AND a bundled `I, II \u{2014}` chapter marker, and \
     declares `abilities: vec![]` -- two `// TODO`s where the chapters should be. \
     `Completeness::inert`, so 0 deck-legal blast radius. It is the reason r1's pinned set is \
     THREE and a grep says four, and it is the member that proves these two axes do not nest: \
     the structural axis cannot see it at all.",
)];

/// CR 714: the two Saga axes **do not nest**, and this row exists to keep that visible.
///
/// A structural axis measures the declaration construct it walks; a def that prints a Saga
/// and declares nothing is invisible to it. A def that declares chapters while printing no
/// reminder text is invisible to the oracle axis. PB-DX26 (equip markers vs printed type
/// lines), PB-DX43 (`LayerModification` payloads vs `TokenSpec`s), PB-DX45 (`may` evidence
/// scoped to the def rather than the clause) and PB-DX47 (one syntactic form of a registry
/// scan) each learned this the same way. Neither set is asserted to be a superset of the
/// other; both are pinned.
#[test]
fn r2_oracle_axis_residual_is_pinned() {
    let oracle: BTreeSet<String> = oracle_saga_defs().into_iter().collect();
    let structural: BTreeSet<String> = saga_defs().into_iter().map(|d| d.name).collect();
    let residual: BTreeSet<String> = oracle.difference(&structural).cloned().collect();
    let pinned: BTreeSet<String> = ORACLE_AXIS_RESIDUAL
        .iter()
        .map(|(n, _)| n.to_string())
        .collect();
    assert_eq!(
        residual,
        pinned,
        "PB-DX49 r2: the oracle-text Saga axis's residual moved. A def that PRINTS a Saga \
         (chapter marker or CR 714 lore-counter reminder) and declares no \
         AbilityDefinition::SagaChapter must be classified here with a reason. live only: \
         {:?}; pinned only: {:?}",
        residual.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&residual).collect::<Vec<_>>()
    );
    // Non-vacuity: an empty residual is only meaningful if the axis found anything at all.
    // Pinned EMPTY sets rot silently (PB-DX6's two empty rosters); this floor is what makes
    // "residual == {}" a measurement rather than a broken needle.
    assert!(
        oracle.len() >= 4,
        "PB-DX49 r2: the oracle-text Saga axis matched only {} defs, below the 4 measured at \
         HEAD -- the needles or `line_is_chapter_marker` have gone vacuous, and an empty \
         residual above would then be meaningless",
        oracle.len()
    );
}

/// `line_is_chapter_marker`'s discrimination, proven on synthetic input rather than assumed
/// from the corpus (which today contains no bundled-chapter Saga, so the corpus alone would
/// not exercise the comma branch at all).
#[test]
fn r2b_chapter_marker_predicate_discriminates() {
    assert!(line_is_chapter_marker(
        "i \u{2014} destroy target nonland permanent an opponent controls."
    ));
    assert!(line_is_chapter_marker(
        "iii \u{2014} creatures you control gain deathtouch."
    ));
    assert!(line_is_chapter_marker("i, ii \u{2014} draw a card."));
    // An em dash with a non-numeral prefix is an ordinary printed dash, not a chapter.
    assert!(!line_is_chapter_marker("ward \u{2014} pay 3 life."));
    assert!(!line_is_chapter_marker("destroy target creature."));
    // An ASCII hyphen is not the em dash Saga chapters print, and the predicate says so
    // rather than silently widening: every corpus Saga uses U+2014.
    assert!(!line_is_chapter_marker("i - destroy target creature."));
}

// ─────────────────────────────────────────────────────────────────────────────
// r3 — blanker population, STRUCTURAL, decided by the engine's own predicate
// ─────────────────────────────────────────────────────────────────────────────

/// The characteristics handed to `modification_blanks_abilities`.
///
/// **This row measures "could blank", not "does blank this object".**
/// `modification_blanks_abilities` takes a `&Characteristics` for exactly one reason: CR
/// 305.7's ability loss is scoped to *lands*, so the `SetLandTypes` arm asks whether the
/// object is a land. Passing `Characteristics::default()` would answer "no" for every
/// `SetLandTypes` def and under-count the roster by the roster's own fixture choice —
/// exactly the shape of blindness this file exists to prevent. So the fixture contains
/// `CardType::Land`, and the row's claim is the modification-level one: *this def carries a
/// modification that blanks abilities on some object it could apply to.* Whether it can
/// reach a Saga in particular is r4's question, not this row's.
fn land_characteristics() -> Characteristics {
    Characteristics {
        card_types: [CardType::Land].into_iter().collect(),
        ..Default::default()
    }
}

/// The names of every deck-legal (`Completeness::Complete`) def, computed once.
fn complete_names() -> BTreeSet<String> {
    all_cards()
        .iter()
        .filter(|d| is_effectively_complete(d))
        .map(|d| d.name.clone())
        .collect()
}

/// One blanking modification found on a def, attributed to the **ability** that owns it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BlankingSite {
    card: String,
    /// Which face declares it (`front` / `back` / `adventure`) — a `CardFace` carries its
    /// own ability list, so reading `def.abilities` alone is `OOS-DX8`'s defect.
    face: &'static str,
    /// The owning `AbilityDefinition` variant name.
    ability: String,
    /// The `LayerModification` variant name.
    modification: String,
    /// The sibling `EffectFilter` variant name, when the modification sits inside a
    /// `ContinuousEffectDef` (it always does today; `None` would be a new container shape
    /// and is reported rather than hidden).
    filter: Option<String>,
    /// The `TargetRequirement` variants declared by **this ability**, not by the def.
    ///
    /// Clause-scoped deliberately. PB-DX45's `/review` finding was that evidence scoped to
    /// the DEF rather than the CLAUSE exempts a def by something unrelated to the clause
    /// under test, and `Turn // Burn` is the worked case here: it declares
    /// `TargetRequirement::TargetCreature` on the Turn half (which carries the blanking) and
    /// `TargetAny` on the Burn half. A def-scoped read would let the blanking half be
    /// widened to `TargetPermanent` while a `TargetCreature` elsewhere in the file kept
    /// `r4c` green.
    ability_targets: BTreeSet<String>,
}

/// Every `(def, ability, modification)` triple the engine's own predicate classifies as
/// blanking.
///
/// Keyed on the MECHANISM — a JSON key literally named `modification` whose value
/// deserializes as a `LayerModification` — rather than on a `ContinuousEffectDef` field-set
/// fingerprint, so a `LayerModification` reached through some future container is still
/// seen. `modification_blanks_abilities` is exhaustive over `LayerModification` with no
/// wildcard arm, so a fourth blanking channel is a compile error in the engine and an
/// automatic member of this roster.
///
/// **Stated recall bound**: `ability_targets` is read from the OUTERMOST
/// `AbilityDefinition`'s own `targets` field. A blanking modification nested inside an
/// inner ability (a Class level's granted ability, say) would be attributed to the outer
/// ability's targets. Zero corpus members today, and the shape is reported rather than
/// assumed away.
fn blanking_sites() -> Vec<BlankingSite> {
    let chars = land_characteristics();
    let mut out = Vec::new();
    for def in all_cards().iter() {
        for (face, abilities) in all_ability_lists(def) {
            for ability in abilities {
                let json = serde_json::to_value(ability).expect("AbilityDefinition serializes");
                let ability_name = variant_name(&json).unwrap_or_else(|| "UNKNOWN".to_string());
                let ability_targets: BTreeSet<String> = json
                    .get(&ability_name)
                    .and_then(|body| body.get("targets"))
                    .and_then(|t| t.as_array())
                    .map(|items| items.iter().filter_map(variant_name).collect())
                    .unwrap_or_default();
                let mut nodes = Vec::new();
                collect_modification_nodes(&json, &mut nodes);
                for (node, parent) in nodes {
                    let Ok(m) = serde_json::from_value::<LayerModification>(node.clone()) else {
                        continue;
                    };
                    if !modification_blanks_abilities(&m, &chars) {
                        continue;
                    }
                    out.push(BlankingSite {
                        card: def.name.clone(),
                        face,
                        ability: ability_name.clone(),
                        modification: variant_name(node).unwrap_or_else(|| "UNKNOWN".to_string()),
                        filter: parent.and_then(|p| p.get("filter")).and_then(variant_name),
                        ability_targets: ability_targets.clone(),
                    });
                }
            }
        }
    }
    out.sort();
    out
}

/// Collect `(value under a "modification" key, the containing object)` at any depth.
fn collect_modification_nodes<'a>(v: &'a Value, out: &mut Vec<(&'a Value, Option<&'a Value>)>) {
    match v {
        Value::Object(m) => {
            for (k, child) in m {
                if k == "modification" {
                    out.push((child, Some(v)));
                }
                collect_modification_nodes(child, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_modification_nodes(item, out);
            }
        }
        _ => {}
    }
}

/// The blanker roster: `(card name, LayerModification variant, deck-legal Complete)`.
///
/// **Eleven defs, eight of them deck-legal `Complete`, and BOTH halves of `OOS-RR4-1`'s
/// figure are wrong — including the half its own 2026-08-14 correction "fixed".**
///
/// - The row as filed says *"13 corpus defs, 8 deck-legal `Complete`"*. **13** is a bare
///   `RemoveAllAbilities` grep counting `blood_moon.rs` and `magus_of_the_moon.rs`, which
///   name the variant in comments only since PB-DX43 replaced their declarations.
/// - Its correction re-measures the numerator as **9** by the qualified grep path, which is
///   the right count of `RemoveAllAbilities` **declarations** and the wrong count of
///   *blankers*: PB-DX43 moved CR 305.7's ability loss into `LayerModification::SetLandTypes`,
///   so the two moons are blankers again through a different variant. Deciding membership by
///   **calling `modification_blanks_abilities`** rather than by matching a variant name is
///   what makes that channel visible, and is why this row does not grep.
/// - Deck-legal is **8**, which happens to equal the row's original figure by a different
///   membership: the row's 8 assumed all-but-five of its 13 `RemoveAllAbilities` defs, where
///   the true set is six `RemoveAllAbilities` defs (`final_showdown` is `partial`,
///   `oko_thief_of_crowns` is `known_wrong`, `vraska_betrayals_sting` is `partial`) plus the
///   two moons. **An agreeing number is not an agreeing measurement** — PB-DX45's lesson
///   that a census figure is an estimate in both directions.
const BLANKER_DEFS: &[(&str, &str, bool)] = &[
    ("Blood Moon", "SetLandTypes", true),
    ("Darksteel Mutation", "RemoveAllAbilities", true),
    ("Eaten by Piranhas", "RemoveAllAbilities", true),
    ("Final Showdown", "RemoveAllAbilities", false),
    ("Imprisoned in the Moon", "RemoveAllAbilities", true),
    ("Kasmina's Transmutation", "RemoveAllAbilities", true),
    ("Kenrith's Transformation", "RemoveAllAbilities", true),
    ("Magus of the Moon", "SetLandTypes", true),
    ("Oko, Thief of Crowns", "RemoveAllAbilities", false),
    ("Turn // Burn", "RemoveAllAbilities", true),
    ("Vraska, Betrayal's Sting", "RemoveAllAbilities", false),
];

/// CR 613.1f / CR 305.7 / CR 708.2a: every def that carries an ability-blanking
/// modification, decided by calling `layers::modification_blanks_abilities`.
#[test]
fn r3_blanker_population_is_pinned() {
    let sites = blanking_sites();
    let by_card: BTreeMap<String, BTreeSet<String>> =
        sites.iter().fold(BTreeMap::new(), |mut acc, s| {
            acc.entry(s.card.clone())
                .or_default()
                .insert(s.modification.clone());
            acc
        });
    let complete = complete_names();

    let mut live: BTreeSet<(String, String, bool)> = BTreeSet::new();
    for (card, mods) in &by_card {
        for m in mods {
            live.insert((card.clone(), m.clone(), complete.contains(card)));
        }
    }
    let pinned: BTreeSet<(String, String, bool)> = BLANKER_DEFS
        .iter()
        .map(|(c, m, l)| (c.to_string(), m.to_string(), *l))
        .collect();
    assert_eq!(
        live,
        pinned,
        "PB-DX49 r3: the ability-blanking population moved. Every member must be classified \
         here, and every member needs an r4 reach row. live only: {:?}; pinned only: {:?}",
        live.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&live).collect::<Vec<_>>()
    );
    // Both channels must be represented, or the "decided by calling the predicate" claim is
    // decorative: a variant-name match on `RemoveAllAbilities` alone would produce a roster
    // that passes the set assertion above only if the SetLandTypes rows are also pinned out.
    assert!(
        live.iter().any(|(_, m, _)| m == "RemoveAllAbilities"),
        "CR 613.1f channel must be present"
    );
    assert!(
        live.iter().any(|(_, m, _)| m == "SetLandTypes"),
        "CR 305.7 channel (PB-DX43) must be present -- if it is not, the roster has silently \
         reverted to a RemoveAllAbilities-only measurement, which is exactly OOS-RR4-1's \
         corrected-but-still-wrong numerator"
    );
}

/// The `SetLandTypes` half of r3 is only visible because `land_characteristics()` contains
/// `CardType::Land`. Asserted directly, so the fixture choice cannot silently stop
/// mattering: with a default `Characteristics`, CR 305.7's precondition fails and the two
/// moons vanish from the roster.
#[test]
fn r3b_land_fixture_is_load_bearing_for_the_cr_305_7_channel() {
    let with_land = land_characteristics();
    let without_land = Characteristics::default();
    let payload: LayerModification = LayerModification::SetLandTypes(
        [mtg_engine::SubType("Mountain".to_string())]
            .into_iter()
            .collect(),
    );
    assert!(
        modification_blanks_abilities(&payload, &with_land),
        "CR 305.7: setting a LAND's subtype to a basic land type makes it lose its abilities"
    );
    assert!(
        !modification_blanks_abilities(&payload, &without_land),
        "CR 305.7's own precondition is 'a land's subtype' -- a non-land object is outside \
         the rule, which is why r3's fixture must contain CardType::Land or the roster \
         under-counts itself"
    );
    // And a nonbasic payload is not a blanker on either fixture (CR 305.7's other conjunct).
    let nonbasic = LayerModification::SetLandTypes(
        [mtg_engine::SubType("Gate".to_string())]
            .into_iter()
            .collect(),
    );
    assert!(
        !modification_blanks_abilities(&nonbasic, &with_land),
        "CR 305.7 applies only when the payload names one or more BASIC land types"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r4 — blanker × Saga pairs
// ─────────────────────────────────────────────────────────────────────────────

/// The first `KeywordAbility::Enchant(..)` target this def declares, across every face.
fn declared_enchant_target(def: &CardDefinition) -> Option<EnchantTarget> {
    for (_, abilities) in all_ability_lists(def) {
        for ability in abilities {
            if let AbilityDefinition::Keyword(KeywordAbility::Enchant(t)) = ability {
                return Some(t.clone());
            }
        }
    }
    None
}

/// Every `TargetRequirement` variant name this def declares, at any depth.
fn declared_target_requirement_variants(def: &CardDefinition) -> BTreeSet<String> {
    let json = serde_json::to_value(def).expect("CardDefinition serializes");
    let mut nodes = Vec::new();
    collect_under_key(&json, "targets", &mut nodes);
    let mut out = BTreeSet::new();
    for node in nodes {
        if let Value::Array(items) = node {
            for item in items {
                if let Some(n) = variant_name(item) {
                    out.insert(n);
                }
            }
        }
    }
    out
}

fn def_by_name(name: &str) -> CardDefinition {
    all_cards()
        .iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("corpus contains `{name}`"))
        .clone()
}

/// **Pair A** — `Imprisoned in the Moon` × `Binding the Old Gods`.
///
/// This pair exists **only because of `OOS-DX20-10`**. Imprisoned in the Moon prints
/// *"Enchant creature, land, or planeswalker"* and declares
/// `KeywordAbility::Enchant(EnchantTarget::Permanent)` — `EnchantFilter` has no OR over
/// card types, so PB-DX20 pinned the over-wide declaration wrong-way-round instead of
/// fixing it. An enchantment is a permanent, so the Aura can legally be attached to a Saga
/// today, its Layer-6 `RemoveAllAbilities` applies through `EffectFilter::AttachedPermanent`,
/// and the blanked Saga is the CR 714 subject of this batch.
///
/// **The assertion is keyed on the declared `EnchantTarget`, deliberately.** When
/// `OOS-DX20-10` is fixed, this row goes RED and has to be re-adjudicated, rather than
/// silently vacating every PB-DX49 probe that rests on the pair being reachable. A probe
/// that quietly stops testing anything is worse than one that fails.
#[test]
fn r4a_pair_a_depends_on_oos_dx20_10() {
    let aura = def_by_name("Imprisoned in the Moon");
    assert_eq!(
        declared_enchant_target(&aura),
        Some(EnchantTarget::Permanent),
        "OOS-DX20-10: Imprisoned in the Moon prints 'Enchant creature, land, or \
         planeswalker' and declares EnchantTarget::Permanent. Pair A (Imprisoned x Binding \
         the Old Gods) is reachable ONLY because of that over-wide declaration. If this \
         assertion fails, OOS-DX20-10 has been fixed and Pair A must be re-adjudicated -- \
         do not delete this row, and do not assume the PB-DX49 probes that use the pair are \
         still exercising anything."
    );
    assert!(
        is_effectively_complete(&aura),
        "Pair A is only a LIVE pair if the Aura is deck-legal"
    );
    assert!(
        is_effectively_complete(&def_by_name("Binding the Old Gods")),
        "Pair A is only a LIVE pair if the Saga is deck-legal"
    );
    // The blanking modification must reach the ATTACHED permanent, not just some creature.
    let filters: BTreeSet<Option<String>> = blanking_sites()
        .into_iter()
        .filter(|s| s.card == "Imprisoned in the Moon")
        .map(|s| s.filter)
        .collect();
    assert_eq!(
        filters,
        [Some("AttachedPermanent".to_string())]
            .into_iter()
            .collect(),
        "CR 613.1f: Imprisoned in the Moon's RemoveAllAbilities must apply to the attached \
         PERMANENT (not AttachedCreature), or the pair is not reachable through the Aura"
    );
}

/// **Pair B** — `Reality Shift` × `Binding the Old Gods`, and it is **unconditional**.
///
/// Reality Shift is `Complete` (by the `#[default]` derive) and resolves `Effect::Manifest`.
/// CR 708.2a: a face-down permanent has *"no text, no name, **no subtypes**, and no mana
/// cost"*, so a manifested Saga is not a Saga and has no chapter abilities. No card-def
/// defect is required for this pair — unlike Pair A, it does not sit behind `OOS-DX20-10`,
/// and it cannot be vacated by fixing anything.
///
/// **Where the live symptom is, stated because the seed row implies otherwise.**
/// `OOS-RR4-1` reads as though the surviving CR 714.3a ETB lore counter were the defect.
/// It is not, on this channel: `Effect::Manifest` (`effects/mod.rs:5247-5262`) and
/// `Effect::Cloak` (`:5310-5325`) set `face_down` / `face_down_as` themselves and emit
/// `PermanentEnteredBattlefield` directly — **neither calls
/// `replacement::apply_self_etb_from_definition` at all**, so CR 714.3a's site never runs
/// on this path. Pair B's live symptom is at the CR 714.3b precombat-counter site
/// (`turn_actions.rs`), the CR 714.2b chapter-trigger site and the CR 714.4 sacrifice SBA
/// (`sba.rs`) — a face-down 2/2 accruing lore counters and resolving *"Destroy target
/// nonland permanent an opponent controls"*.
#[test]
fn r4b_pair_b_is_unconditional() {
    let shifter = def_by_name("Reality Shift");
    assert!(
        is_effectively_complete(&shifter),
        "Pair B needs Reality Shift deck-legal; it declares no completeness field and takes \
         the #[default] Complete derive"
    );
    assert!(
        is_effectively_complete(&def_by_name("Binding the Old Gods")),
        "Pair B needs the Saga deck-legal"
    );
    assert!(
        face_down_makers()
            .iter()
            .any(|(n, m)| n == "Reality Shift" && m == "Manifest"),
        "CR 701.40a: Reality Shift's second clause is Effect::Manifest, which is what puts a \
         permanent onto the battlefield face down; without it Pair B has no mechanism"
    );
}

/// One blanker's reach, and the mechanism that decides it.
struct ReachRow {
    card: &'static str,
    /// Can this blanker's modification land on an *enchantment* (and therefore on a Saga)?
    can_reach_enchantment: bool,
    /// The declared `KeywordAbility::Enchant` target, if the def is an Aura.
    enchant: Option<EnchantTarget>,
    /// `TargetRequirement` variants that the BLANKING ABILITY itself must still declare for
    /// the classification to hold. Empty when the classification does not rest on a target
    /// requirement.
    requires_target_variants: &'static [&'static str],
    /// `EffectFilter` variants the blanking modification must still sit behind, when the
    /// classification rests on the filter rather than on a target or an Aura.
    requires_filters: &'static [&'static str],
    reason: &'static str,
}

/// Every blanker classified by reach. The `enchant` / `requires_target_variants` /
/// `requires_filters` columns are the MECHANISM the classification rests on, re-checked on
/// every run — so a def that is later widened (an `EnchantTarget::Creature` becoming
/// `Permanent`, a `TargetCreature` becoming `TargetPermanent`, an `AttachedCreature` filter
/// becoming `AttachedPermanent`) reddens this row instead of quietly acquiring a new pair.
const REACH_ROWS: &[ReachRow] = &[
    ReachRow {
        card: "Blood Moon",
        can_reach_enchantment: true,
        enchant: None,
        requires_target_variants: &[],
        requires_filters: &["AllNonbasicLands"],
        reason: "CR 305.7 is scoped to LANDS, so this reaches an enchantment only when the \
                 enchantment is ALSO a land -- i.e. Urza's Saga, corner case #36's own pair. \
                 Not deck-legal today: r4d pins Urza's Saga as `partial` (OOS-RR4-2).",
    },
    ReachRow {
        card: "Darksteel Mutation",
        can_reach_enchantment: false,
        enchant: Some(EnchantTarget::Creature),
        requires_target_variants: &[],
        requires_filters: &["AttachedCreature"],
        reason: "CR 303.4a: 'Enchant creature' -- the Aura can only legally attach to a \
                 creature, and its RemoveAllAbilities applies through AttachedCreature.",
    },
    ReachRow {
        card: "Eaten by Piranhas",
        can_reach_enchantment: false,
        enchant: Some(EnchantTarget::Creature),
        requires_target_variants: &[],
        requires_filters: &["AttachedCreature"],
        reason: "CR 303.4a: 'Enchant creature'.",
    },
    ReachRow {
        card: "Final Showdown",
        can_reach_enchantment: false,
        enchant: None,
        requires_target_variants: &[],
        requires_filters: &["AllCreatures"],
        reason: "Untargeted mass blank scoped by EffectFilter::AllCreatures. `partial`, so 0 \
                 deck-legal blast radius either way.",
    },
    ReachRow {
        card: "Imprisoned in the Moon",
        can_reach_enchantment: true,
        enchant: Some(EnchantTarget::Permanent),
        requires_target_variants: &[],
        requires_filters: &["AttachedPermanent"],
        reason: "PAIR A. Reachable ONLY because of OOS-DX20-10 -- the printed 'Enchant \
                 creature, land, or planeswalker' is declared as EnchantTarget::Permanent, \
                 because EnchantFilter has no OR over card types. An enchantment is a \
                 permanent, so the Aura can attach to a Saga today. See r4a.",
    },
    ReachRow {
        card: "Kasmina's Transmutation",
        can_reach_enchantment: false,
        enchant: Some(EnchantTarget::Creature),
        requires_target_variants: &[],
        requires_filters: &["AttachedCreature"],
        reason: "CR 303.4a: 'Enchant creature'.",
    },
    ReachRow {
        card: "Kenrith's Transformation",
        can_reach_enchantment: false,
        enchant: Some(EnchantTarget::Creature),
        requires_target_variants: &[],
        requires_filters: &["AttachedCreature"],
        reason: "CR 303.4a: 'Enchant creature'.",
    },
    ReachRow {
        card: "Magus of the Moon",
        can_reach_enchantment: true,
        enchant: None,
        requires_target_variants: &[],
        requires_filters: &["AllNonbasicLands"],
        reason: "Same CR 305.7 land scope as Blood Moon; same Urza's Saga gate.",
    },
    ReachRow {
        card: "Oko, Thief of Crowns",
        can_reach_enchantment: true,
        enchant: None,
        requires_target_variants: &["TargetPermanent"],
        requires_filters: &["DeclaredTarget"],
        reason: "A LIVE over-wide declaration found by this roster and NOT fixed here: Oko's \
                 +1 prints 'target artifact or creature' and declares bare \
                 TargetRequirement::TargetPermanent, so it can blank an enchantment -- \
                 including a Saga. It is `known_wrong` for exactly this reason (its own \
                 marker names the missing has_card_types filter), so the deck-legal blast \
                 radius is 0 and it is not a PB-DX49 pair. Promoting the def without \
                 narrowing the target creates a third blanker x Saga pair.",
    },
    ReachRow {
        card: "Turn // Burn",
        can_reach_enchantment: false,
        enchant: None,
        requires_target_variants: &["TargetCreature"],
        requires_filters: &["DeclaredTarget"],
        reason: "The Turn half declares TargetRequirement::TargetCreature; the Burn half's \
                 TargetAny carries no blanking. Read CLAUSE-scoped (per-ability), not \
                 def-scoped -- see BlankingSite::ability_targets.",
    },
    ReachRow {
        card: "Vraska, Betrayal's Sting",
        can_reach_enchantment: false,
        enchant: None,
        requires_target_variants: &["TargetCreature"],
        requires_filters: &["DeclaredTarget"],
        reason: "The -9's blanking loyalty ability declares TargetRequirement::TargetCreature. \
                 `partial`, so 0 deck-legal blast radius either way.",
    },
];

/// CR 303.4a / CR 115.10: which blankers can actually reach an enchantment.
#[test]
fn r4_blanker_reach_is_pinned() {
    let sites = blanking_sites();
    let live: BTreeSet<String> = sites.iter().map(|s| s.card.clone()).collect();
    let pinned: BTreeSet<String> = REACH_ROWS.iter().map(|r| r.card.to_string()).collect();
    assert_eq!(
        live,
        pinned,
        "PB-DX49 r4: every member of r3's blanker population needs a reach classification. \
         live only: {:?}; pinned only: {:?}",
        live.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&live).collect::<Vec<_>>()
    );
    for row in REACH_ROWS {
        let def = def_by_name(row.card);
        assert_eq!(
            declared_enchant_target(&def),
            row.enchant,
            "PB-DX49 r4 ({}): the declared EnchantTarget moved, so the reach classification \
             (can_reach_enchantment = {}) no longer rests on what it was measured against. \
             Reason on file: {}",
            row.card,
            row.can_reach_enchantment,
            row.reason
        );
        let own: Vec<&BlankingSite> = sites.iter().filter(|s| s.card == row.card).collect();
        assert!(
            !own.is_empty(),
            "PB-DX49 r4 ({}): no blanking site found; r3 and r4 disagree",
            row.card
        );
        for want in row.requires_target_variants {
            assert!(
                own.iter().any(|s| s.ability_targets.contains(*want)),
                "PB-DX49 r4 ({}): the classification rests on the BLANKING ABILITY declaring \
                 TargetRequirement::{}, which it no longer does (clause-scoped declarations: \
                 {:?}). Reason on file: {}",
                row.card,
                want,
                own.iter().map(|s| &s.ability_targets).collect::<Vec<_>>(),
                row.reason
            );
        }
        for want in row.requires_filters {
            assert!(
                own.iter().any(|s| s.filter.as_deref() == Some(*want)),
                "PB-DX49 r4 ({}): the classification rests on EffectFilter::{}, which is no \
                 longer the filter on any of its blanking sites ({:?}). Reason on file: {}",
                row.card,
                want,
                own.iter().map(|s| &s.filter).collect::<Vec<_>>(),
                row.reason
            );
        }
    }
}

/// The blankers that **cannot** reach an enchantment, asserted as a group so that widening
/// any one of them reddens a row that says *why* it mattered.
///
/// **Seven members at HEAD, not the plan's five**, and the mechanism differs across them —
/// which is the reason this row checks a disjunction of three mechanisms rather than one:
///
/// - **four** Auras restricted by CR 303.4a *"Enchant creature"* — `darksteel_mutation`,
///   `eaten_by_piranhas`, `kasminas_transmutation`, `kenriths_transformation`;
/// - **two** targeted abilities restricted by `TargetRequirement::TargetCreature` —
///   `turn` (a Spell) and `vraska_betrayals_sting` (a LoyaltyAbility, `partial`);
/// - **one** untargeted mass blank restricted by `EffectFilter::AllCreatures` —
///   `final_showdown` (`partial`).
///
/// The plan's list of five named only the first four plus `turn`, because it was derived
/// from the deck-legal-ish `RemoveAllAbilities` defs rather than from the reach axis. A
/// single-mechanism check would in turn have measured one of the three mechanisms and
/// called it the class — this queue's most repeated finding, from PB-DX26 through PB-DX47.
#[test]
fn r4c_creature_only_blankers_cannot_reach_an_enchantment() {
    let sites = blanking_sites();
    let unreachable: Vec<&ReachRow> = REACH_ROWS
        .iter()
        .filter(|r| !r.can_reach_enchantment)
        .collect();
    assert_eq!(
        unreachable.len(),
        7,
        "PB-DX49 r4c: seven blankers are classified as unable to reach an enchantment \
         (4 'Enchant creature' Auras + Turn // Burn + Vraska + Final Showdown); got {:?}",
        unreachable.iter().map(|r| r.card).collect::<Vec<_>>()
    );
    // All three restricting mechanisms must still be exercised by the class, or the
    // disjunction below is being carried by one of them and the other two have gone
    // decorative (PB-DX47: an allowlist whose reason is not checked is a comment).
    let mechanisms: BTreeSet<&str> = unreachable
        .iter()
        .map(|r| {
            if matches!(
                declared_enchant_target(&def_by_name(r.card)),
                Some(EnchantTarget::Creature)
            ) {
                "aura"
            } else if !r.requires_target_variants.is_empty() {
                "target"
            } else {
                "filter"
            }
        })
        .collect();
    assert_eq!(
        mechanisms,
        ["aura", "filter", "target"]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        "PB-DX49 r4c: all three creature-restricting mechanisms must be represented in the \
         class; got {mechanisms:?}"
    );
    for row in unreachable {
        let def = def_by_name(row.card);
        let own: Vec<&BlankingSite> = sites.iter().filter(|s| s.card == row.card).collect();
        let restricted_by_aura =
            matches!(declared_enchant_target(&def), Some(EnchantTarget::Creature));
        let restricted_by_target = own.iter().any(|s| {
            row.requires_target_variants
                .iter()
                .any(|v| s.ability_targets.contains(*v))
        });
        let restricted_by_filter = !own.is_empty()
            && own
                .iter()
                .all(|s| s.filter.as_deref() == Some("AllCreatures"));
        assert!(
            restricted_by_aura || restricted_by_target || restricted_by_filter,
            "PB-DX49 r4c ({}): classified as unable to reach an enchantment, but no \
             mechanism restricting it to creatures is present any more (aura={}, \
             target={}, filter={}). Reason on file: {}",
            row.card,
            restricted_by_aura,
            restricted_by_target,
            restricted_by_filter,
            row.reason
        );
    }
}

/// **Clause scoping is load-bearing, proven by measuring both readings of the same def.**
///
/// PB-DX45's `/review` found that evidence scoped to the DEF rather than to the CLAUSE
/// exempts a def by something unrelated to the clause under test. `Turn // Burn` is the
/// worked case in this roster: read def-scoped it declares BOTH
/// `TargetRequirement::TargetCreature` (the Turn half, which carries the blanking) and
/// `TargetAny` (the Burn half, which does not). If `r4` read def-scoped, the blanking half
/// could be widened to `TargetPermanent` and the row would stay green off the *other*
/// half's `TargetCreature`. The clause-scoped read cannot be satisfied that way.
#[test]
fn r4e_clause_scoping_is_load_bearing() {
    let def = def_by_name("Turn // Burn");
    let def_scoped = declared_target_requirement_variants(&def);
    let clause_scoped: BTreeSet<String> = blanking_sites()
        .into_iter()
        .filter(|s| s.card == "Turn // Burn")
        .flat_map(|s| s.ability_targets)
        .collect();
    assert!(
        def_scoped.contains("TargetAny"),
        "def-scoped read of Turn // Burn must see the Burn half's TargetAny; got {def_scoped:?}"
    );
    assert!(
        !clause_scoped.contains("TargetAny"),
        "clause-scoped read must NOT see the Burn half's TargetAny -- if it does, the \
         per-ability attribution has collapsed back to a def-scoped one and r4/r4c can be \
         satisfied by an unrelated clause; got {clause_scoped:?}"
    );
    assert!(
        clause_scoped.contains("TargetCreature"),
        "clause-scoped read must still see the blanking half's own TargetCreature; got \
         {clause_scoped:?}"
    );
    assert!(
        def_scoped.len() > clause_scoped.len(),
        "the two readings must actually differ on this def, or this row proves nothing: \
         def-scoped {def_scoped:?} vs clause-scoped {clause_scoped:?}"
    );
}

/// **The famous pair is NOT deck-legal, and this batch does not make it so.**
///
/// Corner case #36 is *Blood Moon + Urza's Saga*. `urzas_saga` is `partial` at HEAD, so
/// `validate_deck` rejects it and the pair cannot occur in a game. Authoring it is
/// **`OOS-RR4-2`**, ranked separately and **explicitly out of scope for PB-DX49** — this
/// batch closes the ENGINE half of #36 and leaves the card half open. If this row ever goes
/// red, `urzas_saga` has been promoted and #36's card half needs re-adjudicating along with
/// the CR 305.7 × CR 714 interaction it makes reachable.
#[test]
fn r4d_urzas_saga_is_not_deck_legal() {
    let saga = def_by_name("Urza's Saga");
    assert!(
        !is_effectively_complete(&saga),
        "OOS-RR4-2: Urza's Saga is `partial` at HEAD, which is why the famous Blood Moon x \
         Urza's Saga pair is not deck-legal and is NOT what PB-DX49's behavioural probes \
         rest on. Authoring it is out of scope for this batch."
    );
    // Both moons are deck-legal and DO carry a CR 305.7 blanking modification -- the pair is
    // gated on the Saga side alone, which is the half this batch does not take.
    for moon in ["Blood Moon", "Magus of the Moon"] {
        assert!(
            is_effectively_complete(&def_by_name(moon)),
            "{moon} is deck-legal; only the Saga half of corner case #36 is gated"
        );
        assert!(
            blanking_sites().iter().any(|s| s.card == moon),
            "{moon} must be an r3 blanker (CR 305.7 SetLandTypes with a basic payload)"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// r5 — the face-down channel's real reach
// ─────────────────────────────────────────────────────────────────────────────

/// `(card name, "Manifest" | "Cloak")` for every def that can put a permanent onto the
/// battlefield face down.
///
/// Keyed on the `Effect` variant, not on `KeywordAbility::Manifest` / `KeywordAbility::Cloak`
/// — those markers exist but are *presence* markers, and a def can resolve `Effect::Manifest`
/// without declaring one (`reality_shift` does exactly that). PB-DX48's `OOS-DX48-4` is the
/// same shape read the other way round: a grep for a keyword variant measured zero and read
/// like a measurement.
fn contains_face_down_effect(json: &Value, key: &str) -> bool {
    let mut nodes = Vec::new();
    collect_under_key(json, key, &mut nodes);
    // `Effect::Manifest { player }` / `Effect::Cloak { player }` are the only shapes that
    // carry a `player` field. The same two names exist as UNIT `KeywordAbility` variants
    // (discriminants 156/157) and as unit `FaceDownKind` / `TurnFaceUpMethod` variants, all
    // of which serialize as bare STRINGS — never as an object key whose value has a
    // `player` inside. The conjunct is what separates them, and
    // `r5c_face_down_effect_matcher_discriminates` proves it on synthetic input, because
    // **no corpus def declares either keyword marker**, so the corpus alone would exercise
    // only the positive half.
    nodes.iter().any(|n| n.get("player").is_some())
}

fn face_down_makers() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for def in all_cards().iter() {
        let json = serde_json::to_value(def).expect("CardDefinition serializes");
        for key in ["Manifest", "Cloak"] {
            if contains_face_down_effect(&json, key) {
                out.push((def.name.clone(), key.to_string()));
            }
        }
    }
    out.sort();
    out
}

/// The CR 708.2a channel's corpus reach.
const FACE_DOWN_MAKERS: &[(&str, &str)] = &[
    ("Cryptic Coat", "Cloak"),
    ("Reality Shift", "Manifest"),
    ("Write into Being", "Manifest"),
];

/// CR 701.40a / CR 701.58a / CR 708.2a: the face-down blanking channel is not reachable
/// "in theory" — it is reachable from these defs and no others.
#[test]
fn r5_face_down_channel_population_is_pinned() {
    let live: BTreeSet<(String, String)> = face_down_makers().into_iter().collect();
    let pinned: BTreeSet<(String, String)> = FACE_DOWN_MAKERS
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    assert_eq!(
        live,
        pinned,
        "PB-DX49 r5: the Effect::Manifest / Effect::Cloak population moved. live only: {:?}; \
         pinned only: {:?}",
        live.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&live).collect::<Vec<_>>()
    );
}

/// `contains_face_down_effect`'s discrimination, on synthetic input.
///
/// The negative half cannot be exercised by the corpus: **zero** defs declare
/// `KeywordAbility::Manifest` or `KeywordAbility::Cloak`, so a matcher that keyed on the
/// bare name would measure the same three defs today and start over-counting the day a
/// presence marker is authored. PB-DX48's `OOS-DX48-4` is the same shape read the other way
/// round — a grep for a keyword variant that measured zero and read like a measurement.
#[test]
fn r5c_face_down_effect_matcher_discriminates() {
    // `Effect::Manifest { player: PlayerTarget::Controller }`.
    let effect = serde_json::json!({ "Manifest": { "player": "Controller" } });
    assert!(contains_face_down_effect(&effect, "Manifest"));
    // `AbilityDefinition::Keyword(KeywordAbility::Manifest)` — a unit variant, no payload.
    let marker = serde_json::json!({ "Keyword": "Manifest" });
    assert!(
        !contains_face_down_effect(&marker, "Manifest"),
        "a KeywordAbility::Manifest presence marker is not a way to put a permanent onto \
         the battlefield face down; counting it would over-state the CR 708.2a channel"
    );
    // Nested arbitrarily deep — the walk must not be parent-key-scoped.
    let nested = serde_json::json!({
        "Triggered": { "effect": { "Sequence": [ { "DrawCards": { "count": 1 } },
            { "Cloak": { "player": "Controller" } } ] } }
    });
    assert!(contains_face_down_effect(&nested, "Cloak"));
    assert!(!contains_face_down_effect(&nested, "Manifest"));
}

/// **What r5 does NOT measure, stated rather than left to be discovered.**
///
/// `Effect::Manifest` / `Effect::Cloak` are not the only way a face-down permanent reaches
/// the battlefield: the **morph / megamorph / disguise cast path** does it too, and
/// `resolution.rs` deliberately restores `face_down_as` *"before any ETB processing"*, so
/// unlike the manifest arms it **does** reach CR 714.3a's site. r5's population is the
/// EFFECT axis alone; the cast axis is keyword-declared and is latent here only because no
/// corpus Saga prints morph — an enchantment cannot, today, be cast face down. This row
/// pins that premise so the claim stops being free the moment it stops being true.
#[test]
fn r5d_no_saga_def_prints_a_face_down_cast_keyword() {
    let saga_names: BTreeSet<String> = saga_defs().into_iter().map(|d| d.name).collect();
    for def in all_cards().iter() {
        if !saga_names.contains(&def.name) {
            continue;
        }
        for (face, abilities) in all_ability_lists(def) {
            for ability in abilities {
                // BOTH spellings, because a face-down cast keyword is declared twice: the
                // presence marker `AbilityDefinition::Keyword(KeywordAbility::Morph)` (a
                // unit variant) and the cost carrier `AbilityDefinition::Morph { cost }`.
                // Checking one of the two would measure one of the two -- this file's own
                // thesis, and the mistake PB-DX26 made with equip markers.
                let is_face_down_cast = matches!(
                    ability,
                    AbilityDefinition::Keyword(
                        KeywordAbility::Morph
                            | KeywordAbility::Megamorph
                            | KeywordAbility::Disguise
                    ) | AbilityDefinition::Morph { .. }
                        | AbilityDefinition::Megamorph { .. }
                        | AbilityDefinition::Disguise { .. }
                );
                assert!(
                    !is_face_down_cast,
                    "PB-DX49 r5d: {} declares a face-down CAST keyword on its {face} face. \
                     That path DOES reach apply_self_etb_from_definition (resolution.rs \
                     restores face_down_as before ETB processing), unlike Effect::Manifest \
                     / Effect::Cloak -- so PB-DX49's site-4 analysis needs re-taking with \
                     this member in scope.",
                    def.name
                );
            }
        }
    }
}

/// **Where the face-down channel does NOT run, recorded as a fact rather than an
/// assumption** (`memory/primitives/pb-DX49-execution-notes.md` §1.2).
///
/// `Effect::Manifest` and `Effect::Cloak` move the card to the battlefield, set `face_down`
/// / `face_down_as` themselves, and emit `PermanentEnteredBattlefield` directly. **Neither
/// calls `apply_self_etb_from_definition`**, so CR 714.3a's ETB lore-counter site is never
/// reached on this path — and neither is CR 306.5b starting loyalty nor CR 716.2d Class
/// level. For a face-down permanent all three are the *correct* outcome under CR 708.2a, so
/// this is right by accident rather than by design; it is `OOS-DX49-2`, and it is asserted
/// here so a later batch that wires self-ETB replacements into those arms learns what it is
/// turning on.
#[test]
fn r5b_manifest_and_cloak_do_not_reach_the_self_etb_site() {
    let src = std::fs::read_to_string(engine_src().join("effects/mod.rs"))
        .expect("effects/mod.rs is readable");
    let stripped = strip_comments(&src);
    let arms: Vec<(&str, usize)> = ["Effect::Manifest {", "Effect::Cloak {"]
        .into_iter()
        .filter_map(|needle| stripped.find(needle).map(|at| (needle, at)))
        .collect();
    assert_eq!(
        arms.len(),
        2,
        "OOS-DX49-2: both Effect::Manifest and Effect::Cloak arms must be findable in \
         effects/mod.rs for this assertion to mean anything; found {arms:?}"
    );
    for (needle, at) in arms {
        // **The arm's own closing brace, not a byte window** (`/review` FINDING 4). The
        // first draft scanned a fixed 4,000 bytes after the pattern, which fails OPEN the
        // day either arm outgrows it -- silently under-scanning past the very call this row
        // exists to catch -- and over-scans into the FOLLOWING arm until then. Measured and
        // PRINTED by `t_census_report`, never transcribed (PB-DX8's rule -- and the first
        // draft of this comment was wrong by 2 bytes because it quoted a figure taken in a
        // different unit): the Manifest arm's body is **3,413** bytes and Cloak's **2,820**,
        // and the superseded window ran **520** bytes past the Manifest arm's own closing
        // brace. So it was reading the next arm, and was 520 bytes of arm growth away from
        // failing open. Both directions are wrong; the bound is now a measurement.
        let (open, end) = match_arm_body_span(&stripped, at).unwrap_or_else(|| {
            panic!(
                "OOS-DX49-2: could not bound the {needle} arm's body by brace matching \
                 (pattern at byte {at}). This row FAILS CLOSED rather than falling back to \
                 a byte window: an unbounded scan is not a measurement of the arm."
            )
        });
        let body = &stripped[open..=end];
        // Non-vacuity floor: a bounding bug that returned an empty or near-empty span would
        // satisfy the `!contains` assertion below while measuring nothing.
        assert!(
            body.len() >= 200,
            "OOS-DX49-2: the {needle} arm's measured body is only {} bytes, which is too \
             small to be the real arm -- the brace bounding has broken and the assertion \
             below would pass vacuously",
            body.len()
        );
        assert!(
            !body.contains("apply_self_etb_from_definition"),
            "OOS-DX49-2: the {needle} arm now calls apply_self_etb_from_definition. That \
             turns on CR 714.3a's lore counter, CR 306.5b starting loyalty and CR 716.2d \
             Class level for a FACE-DOWN permanent, all three of which CR 708.2a says must \
             not happen. PB-DX49's Pair B analysis (live symptom at sites 1/2/3/5, not site \
             4) was measured against the arms NOT calling it."
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// r6 — the site roster
// ─────────────────────────────────────────────────────────────────────────────

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/engine -> crates -> workspace root
    p.pop();
    p.pop();
    p
}

fn engine_src() -> PathBuf {
    workspace_root().join("crates/engine/src")
}

/// Every `<crate>/src` and `<tool>/src` directory in the workspace, **except**
/// `crates/card-defs/src`.
///
/// **This is the fix for the `/review`'s FINDING 2, and the finding is PB-DX48's defeat one
/// crate up.** PB-DX48's `SITE_SRCS` named six `rules/` files while the function it policed
/// was `pub(crate)`; this file's first draft answered that by walking `crates/engine/src` --
/// the whole *crate* -- while `rules::saga::saga_view` is `pub`. The reviewer added a
/// `saga_view` consumer to `crates/simulator/src/lib.rs` and `r6` stayed **green**. A gate's
/// reach must be at least as wide as its subject's visibility.
///
/// `crates/card-defs/src` is excluded deliberately and the exclusion is stated rather than
/// silent: it is ~1,800 generated-shaped declaration files, and a card def naming
/// `LayerModification::RemoveAllAbilities` is a *declaration* (r3's subject, walked from
/// `all_cards()`), never a predicate.
fn workspace_src_roots() -> Vec<PathBuf> {
    let root = workspace_root();
    let mut out = Vec::new();
    for base in ["crates", "tools"] {
        let Ok(entries) = std::fs::read_dir(root.join(base)) else {
            continue;
        };
        let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        dirs.sort();
        for dir in dirs {
            if dir.file_name().is_some_and(|n| n == "card-defs") {
                continue;
            }
            let src = dir.join("src");
            if src.is_dir() {
                out.push(src);
            }
        }
    }
    out
}

/// Every `.rs` file under [`workspace_src_roots`], as `(workspace-relative label, path)`.
fn workspace_src_files() -> Vec<(String, PathBuf)> {
    let root = workspace_root();
    let mut out = Vec::new();
    for src_root in workspace_src_roots() {
        let mut files = Vec::new();
        walk_rs(&src_root, &mut files);
        files.sort();
        for path in files {
            let label = path
                .strip_prefix(&root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            out.push((label, path));
        }
    }
    out.sort();
    out
}

/// [`workspace_src_files`] with its **non-vacuity floors executed**.
///
/// A walk that silently returns `[]` -- a moved directory, a renamed crate, a `read_dir`
/// that errors -- makes every gate built on it pass while measuring nothing, which is the
/// failure mode both `r6` and `r7` exist to prevent in the code they police. Measured at
/// HEAD: **14** roots, **148** files. The floors are set well below both so ordinary churn
/// does not trip them, and `t_census_report` PRINTS the live figures so the gap between
/// floor and reality stays visible rather than being trusted.
fn workspace_src_files_checked() -> Vec<(String, PathBuf)> {
    let roots = workspace_src_roots();
    assert!(
        roots.len() >= 8,
        "PB-DX49: the workspace source walk found only {} `src` roots (measured 14 at HEAD). \
         Every gate built on this walk is vacuous until it is fixed; roots: {:?}",
        roots.len(),
        roots
    );
    assert!(
        roots.iter().any(|r| r.ends_with("crates/engine/src")),
        "PB-DX49: the workspace source walk does not contain crates/engine/src, which is the \
         crate every pinned site lives in; roots: {roots:?}"
    );
    let files = workspace_src_files();
    assert!(
        files.len() >= 100,
        "PB-DX49: the workspace source walk found only {} .rs files (measured 148 at HEAD); \
         the walk has gone vacuous",
        files.len()
    );
    files
}

/// The offset of the `}` matching the `{` at `open`, skipping `"..."` string literals.
///
/// Returns `None` on unbalanced input, so every caller **fails closed** rather than
/// falling back to a byte window.
fn matching_brace(src: &str, open: usize) -> Option<usize> {
    let b = src.as_bytes();
    if b.get(open) != Some(&b'{') {
        return None;
    }
    let mut depth = 0usize;
    let mut i = open;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
        } else {
            match c {
                b'"' => in_str = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// The `(open brace, close brace)` span of the `match` arm whose PATTERN starts at
/// `pattern_at`.
///
/// The needle this file searches with (`Effect::Manifest {`) ends at the **pattern's**
/// brace, so a naive brace match closes the pattern, not the body. This walks
/// pattern-braces -> `=>` -> body-brace -> matching close, and refuses (`None`) when the arm
/// body is not a block, because in that case the next `{` belongs to a LATER arm and
/// bounding on it would silently widen the scan.
fn match_arm_body_span(src: &str, pattern_at: usize) -> Option<(usize, usize)> {
    let pat_open = src[pattern_at..].find('{').map(|r| pattern_at + r)?;
    let pat_end = matching_brace(src, pat_open)?;
    let arrow = src[pat_end..].find("=>").map(|r| pat_end + r)?;
    let body_open = src[arrow..].find('{').map(|r| arrow + r)?;
    // Fail closed on a non-block arm body: `=> foo(),` would otherwise bound on the NEXT
    // arm's brace.
    if !src[arrow + 2..body_open].trim().is_empty() {
        return None;
    }
    let body_end = matching_brace(src, body_open)?;
    Some((body_open, body_end))
}

/// The `(open brace, close brace)` span of the function enclosing byte offset `at`, or
/// `None` when `at` is not inside any function body (a top-level `enum` / `const`).
fn enclosing_fn_span(src: &str, at: usize) -> Option<(usize, usize)> {
    let mut decl: Option<usize> = None;
    let mut off = 0usize;
    for line in src.split_inclusive('\n') {
        if off >= at {
            break;
        }
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub(crate) fn ")
        {
            decl = Some(off + indent);
        }
        off += line.len();
    }
    let decl = decl?;
    let open = src[decl..].find('{').map(|r| decl + r)?;
    let end = matching_brace(src, open)?;
    if end < at {
        // The last `fn` before `at` closed before it: `at` sits between items.
        None
    } else {
        Some((open, end))
    }
}

fn walk_rs(dir: &Path, acc: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk_rs(&p, acc);
        } else if p.extension().is_some_and(|x| x == "rs") {
            acc.push(p);
        }
    }
}

/// Strip `//` line comments **and** `/* */` block comments, replacing each stripped byte
/// with a space so byte offsets are preserved.
///
/// Both halves are load-bearing and both are proven so by
/// [`r6b_comment_stripping_is_load_bearing`] rather than assumed. The line half is what
/// makes `resolution.rs`'s **zero** a real zero — that file mentions `saga::saga_view` twice
/// in `//` comments explaining the CR 113.7a exclusion, and an unstripped scan would count
/// them as call sites and pin the exclusion backwards. The block half is `OOS-DX32-6`'s
/// class: a `/* */`-wrapped roster row left the compiled roster while every gate stayed
/// green, so a stripper that handles only one comment syntax measures only one comment
/// syntax.
///
/// Deliberately naive about string literals containing `//` or `/*` — over-stripping can
/// only DELETE apparent call sites, which makes `r6`'s pinned-set assertion redder (a
/// pinned site would go missing), never greener.
fn strip_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            let mut depth = 1usize;
            out.push_str("  ");
            i += 2;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                } else {
                    out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
        } else {
            let ch = src[i..].chars().next().expect("char boundary");
            out.push_str(&src[i..i + ch.len_utf8()]);
            i += ch.len_utf8();
        }
    }
    out
}

const SAGA_VIEW_NEEDLE: &str = "saga_view(";

/// The enclosing function's name: the last `fn ` (or `pub fn ` / `pub(crate) fn `)
/// line-start before byte offset `at`.
fn enclosing_fn_name(src: &str, at: usize) -> String {
    let head = &src[..at];
    let mut name = "UNKNOWN".to_string();
    for line in head.lines() {
        let t = line.trim_start();
        let rest = t
            .strip_prefix("pub(crate) fn ")
            .or_else(|| t.strip_prefix("pub fn "))
            .or_else(|| t.strip_prefix("fn "));
        if let Some(rest) = rest {
            let n: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !n.is_empty() {
                name = n;
            }
        }
    }
    name
}

/// Byte offsets of every genuine CALL to `saga_view` in `src` (already comment-stripped) —
/// excluding the `fn saga_view(` definition itself.
fn saga_view_call_offsets(src: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = src[from..].find(SAGA_VIEW_NEEDLE) {
        let at = from + rel;
        if !src[..at].trim_end().ends_with("fn") {
            out.push(at);
        }
        from = at + 1;
    }
    out
}

/// Every `saga_view` call site in **workspace source**, as `(file, enclosing fn, OFFSET)`.
///
/// **Keyed on the mechanism, not on a hardcoded file list.** PB-DX48's `/review` defeated
/// exactly that construct: `SITE_SRCS` named six `rules/` files while the function it
/// policed was `pub(crate)`, so a site added anywhere else stayed invisible.
///
/// **This file's first draft answered that defeat one directory too narrowly and lost to it
/// again** (`/review` FINDING 2): it walked `crates/engine/src` while `rules::saga::saga_view`
/// is `pub`, so the reviewer added a consumer to `crates/simulator/src/lib.rs` and `r6`
/// stayed green -- PB-DX48's defeat one *crate* up rather than one *directory* up. The walk
/// is now [`workspace_src_files_checked`]: every `crates/*/src` and `tools/*/src` bar
/// `crates/card-defs/src`, with the walk's own non-vacuity floors executed, and labels
/// workspace-relative so a same-named file in two crates cannot collide.
///
/// The OFFSET is in the tuple for PB-DX48's other defeat: as a `BTreeSet<(file, func)>` a
/// **duplicated** call inside an already-pinned function collapses into one element, and a
/// duplicated CR 714 query is precisely how a Saga would take two lore counters in one
/// precombat main phase.
fn live_saga_view_sites() -> BTreeSet<(String, String, usize)> {
    let mut out = BTreeSet::new();
    for (label, path) in workspace_src_files_checked() {
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        let src = strip_comments(&raw);
        for at in saga_view_call_offsets(&src) {
            out.insert((label.clone(), enclosing_fn_name(&src, at), at));
        }
    }
    out
}

/// One pinned consumer of `rules::saga::saga_view`.
struct PinnedSagaSite {
    file: &'static str,
    func: &'static str,
    cr: &'static str,
    reason: &'static str,
}

/// The five behavioural sites, and only those.
const PINNED_SAGA_SITES: &[PinnedSagaSite] = &[
    PinnedSagaSite {
        file: "crates/engine/src/rules/sba.rs",
        func: "check_saga_sbas",
        cr: "CR 714.4",
        reason: "site 1 -- the final-chapter threshold. `final_chapter()` returns None when \
                 the permanent retains no chapter abilities, which is what exempts a blanked \
                 Saga from the sacrifice entirely rather than giving it a threshold of 0",
    },
    PinnedSagaSite {
        file: "crates/engine/src/rules/sba.rs",
        func: "check_saga_sbas",
        cr: "CR 714.4",
        reason: "site 2 -- the 'chapter ability has triggered but not yet left the stack' \
                 guard, via `is_chapter_index`",
    },
    PinnedSagaSite {
        file: "crates/engine/src/rules/turn_actions.rs",
        func: "precombat_main_actions",
        cr: "CR 714.3b",
        reason: "site 3 -- the precombat lore counter, gated on `has_chapters()` because \
                 714.3b says 'with one or more chapter abilities' explicitly",
    },
    PinnedSagaSite {
        file: "crates/engine/src/rules/replacement.rs",
        func: "apply_self_etb_from_definition",
        cr: "CR 714.3a",
        reason: "site 4 -- the ETB lore counter, gated on `is_saga_permanent` and NOT on \
                 `has_chapters()`: 714.3a carries no chapter-ability clause, so a Layer-6 \
                 blanked Saga still takes the counter (it keeps its subtypes) while a \
                 face-down one does not (CR 708.2a strips subtypes)",
    },
    PinnedSagaSite {
        file: "crates/engine/src/rules/replacement.rs",
        func: "fire_saga_chapter_triggers",
        cr: "CR 714.2b",
        reason: "site 5 -- the chapter triggers, enumerated from the view's retained \
                 `chapters` rather than from a `&CardDefinition` parameter, so producer and \
                 consumer share one index space",
    },
];

/// CR 714: exactly five functions consume the query, and each says which rule it serves.
#[test]
fn r6_saga_view_consumer_roster_is_pinned() {
    let live = live_saga_view_sites();
    let live_classified: BTreeSet<(String, String)> = live
        .iter()
        .map(|(f, n, _)| (f.clone(), n.clone()))
        .collect();
    let pinned: BTreeSet<(String, String)> = PINNED_SAGA_SITES
        .iter()
        .map(|p| (p.file.to_string(), p.func.to_string()))
        .collect();
    assert_eq!(
        live_classified,
        pinned,
        "PB-DX49 r6: the rules::saga::saga_view consumer census moved. A new consumer must \
         be classified in PINNED_SAGA_SITES with its CR rule and a reason before this row \
         can be re-pinned. live only: {:?}; pinned only: {:?}",
        live_classified.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&live_classified).collect::<Vec<_>>()
    );
    // The COUNT is over the offset-carrying set: two calls inside one pinned function are
    // two entries, not one. `check_saga_sbas` legitimately holds two (sites 1 and 2), which
    // is why the classification set above has four elements and this count has five.
    assert_eq!(
        live.len(),
        PINNED_SAGA_SITES.len(),
        "PB-DX49 r6: {} saga_view call sites, {} pinned. A DUPLICATED call inside an \
         already-pinned function is invisible to a (file, func) set and is exactly how a \
         Saga would take two lore counters in one precombat main phase. Sites: {:?}",
        live.len(),
        PINNED_SAGA_SITES.len(),
        live
    );
}

/// **CR 113.7a is a deliberate NON-consumer, and this is the row that says so.**
///
/// *"An ability on the stack exists independently of its source."* `resolution.rs` resolves
/// a chapter ability that has **already triggered**; blanking the Saga afterwards neither
/// counters nor changes it. Those sites keep reading the printed def on purpose, and a
/// later batch must not "finish the job" by wiring them to `saga_view`.
///
/// The zero is only meaningful because `strip_comments` runs first: `resolution.rs` names
/// `rules::saga::saga_view` twice in `//` comments explaining this very exclusion, so an
/// unstripped scan would measure **2** and pin the exclusion backwards.
#[test]
fn r6b_resolution_is_not_a_consumer() {
    let raw = std::fs::read_to_string(engine_src().join("rules/resolution.rs"))
        .expect("rules/resolution.rs is readable");
    assert!(
        raw.contains("saga_view"),
        "PB-DX49 r6b is only a meaningful zero while resolution.rs still documents the \
         CR 113.7a exclusion by name; if that comment is gone, the exclusion has lost its \
         in-source record and this row must be re-adjudicated rather than deleted"
    );
    let calls = saga_view_call_offsets(&strip_comments(&raw));
    assert!(
        calls.is_empty(),
        "CR 113.7a: resolution.rs must have ZERO saga_view call sites -- an ability already \
         on the stack exists independently of its source, so blanking the Saga after the \
         chapter trigger went on the stack does not change it. Found {} call site(s) at \
         offsets {:?}",
        calls.len(),
        calls
    );
}

/// Both halves of `strip_comments` proven load-bearing by execution, on synthetic source
/// rather than on whatever the corpus happens to contain today.
#[test]
fn r6b_comment_stripping_is_load_bearing() {
    let line_commented = "fn f() {\n    // let v = saga_view(state, id);\n}\n";
    assert_eq!(
        saga_view_call_offsets(line_commented).len(),
        1,
        "control: without stripping, a `//`-commented mention counts as a call -- which is \
         what would make resolution.rs's CR 113.7a zero read as a two"
    );
    assert!(
        saga_view_call_offsets(&strip_comments(line_commented)).is_empty(),
        "the `//` half of strip_comments must remove it"
    );

    let block_commented = "fn f() {\n    /* let v = saga_view(state, id); */\n}\n";
    assert_eq!(
        saga_view_call_offsets(block_commented).len(),
        1,
        "control: `OOS-DX32-6`'s class -- a `/* */` block is invisible to a line-only \
         stripper, so this must count as a call BEFORE stripping"
    );
    assert!(
        saga_view_call_offsets(&strip_comments(block_commented)).is_empty(),
        "the `/* */` half of strip_comments must remove it too; a stripper that handles one \
         comment syntax measures one comment syntax"
    );

    // And a real call must survive both.
    let real = "fn f() {\n    let v = saga_view(state, id);\n}\n";
    assert_eq!(
        saga_view_call_offsets(&strip_comments(real)).len(),
        1,
        "over-stripping would make r6's pinned-set assertion vacuous in the safe direction; \
         it must not"
    );
    // The definition site is not a call.
    let definition = "pub fn saga_view(state: &GameState, id: ObjectId) -> SagaView {\n}\n";
    assert!(
        saga_view_call_offsets(&strip_comments(definition)).is_empty(),
        "`fn saga_view(` is the definition, not a consumer"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r7 — the ability-blanking VARIANT-NAMING roster
// ─────────────────────────────────────────────────────────────────────────────

/// The two `LayerModification` variants that `layers::modification_blanks_abilities`
/// classifies as blanking at HEAD (r8 pins that set exhaustively).
const BLANKING_VARIANTS: &[&str] = &["RemoveAllAbilities", "SetLandTypes"];

/// The tokens that turn "names a blanking variant" into "IS a blanking predicate".
///
/// A predicate must do two things: name the variant **and** decide whether the effect
/// carrying it applies to some object. The reviewer's defeat is the canonical shape --
/// `matches!(e.modification, LayerModification::RemoveAllAbilities) && effect_applies_to_object(..)`
/// over `state.continuous_effects` -- so the second conjunct is what
/// [`r7_blanking_variant_naming_sites_are_pinned`] checks inside each allowlisted site.
/// **This is the checkable half of every allowlist reason below**: PB-DX47's rule is that an
/// allowlist whose reason is not checked is a comment.
const PREDICATE_TOKENS: &[&str] = &["effect_applies_to_object", "continuous_effects"];

/// One site that names an ability-blanking `LayerModification` variant itself.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct VariantNamingSite {
    file: String,
    func: String,
    variant: String,
    offset: usize,
}

/// Every ability-blanking variant name in `raw`, comment-stripped, keyed by
/// `(file, enclosing fn, variant)` with its byte offset.
///
/// **Matched on the bare variant name at word boundaries, not on the
/// `LayerModification::` prefix.** A second predicate written as
/// `use LayerModification::RemoveAllAbilities;` + `matches!(m, RemoveAllAbilities)` is the
/// same defect in a different spelling, and PB-DX47's finding was precisely that *a gate
/// written for one syntactic form measures that form*. Over-collection (a variant name
/// inside a string literal, say) can only make `r7` redder -- an unclassified site fails the
/// set assertion -- never greener.
fn variant_naming_sites_in(label: &str, raw: &str) -> Vec<VariantNamingSite> {
    let src = strip_comments(raw);
    let is_ident = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    let mut out = Vec::new();
    for variant in BLANKING_VARIANTS {
        let mut from = 0usize;
        while let Some(rel) = src[from..].find(variant) {
            let at = from + rel;
            from = at + variant.len();
            if is_ident(src[..at].chars().next_back())
                || is_ident(src[at + variant.len()..].chars().next())
            {
                continue;
            }
            out.push(VariantNamingSite {
                file: label.to_string(),
                func: enclosing_fn_name(&src, at),
                variant: (*variant).to_string(),
                offset: at,
            });
        }
    }
    out.sort();
    out
}

/// Every blanking-variant naming site in workspace source.
fn live_variant_naming_sites() -> Vec<VariantNamingSite> {
    let mut out = Vec::new();
    for (label, path) in workspace_src_files_checked() {
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        out.extend(variant_naming_sites_in(&label, &raw));
    }
    out.sort();
    out
}

/// Why an allowlisted site is not a second blanking predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamingKind {
    /// `modification_blanks_abilities` itself -- THE classifier.
    Classifier,
    /// The layer walk's application arms: it performs the modification, it does not decide
    /// whether abilities are blanked.
    Application,
    /// CR 613.8 dependency / representative-modification bookkeeping.
    Dependency,
    /// A declaration of the variant in a registry enumeration.
    Registry,
    /// The state hasher's exhaustive match: it hashes the variant, it decides nothing.
    Hashing,
    /// The `enum LayerModification` declaration itself.
    Declaration,
}

struct NamingSiteRow {
    file: &'static str,
    func: &'static str,
    variants: &'static [&'static str],
    kind: NamingKind,
    reason: &'static str,
}

/// **The allowlist, re-derived at HEAD rather than trusted from the finding's sketch.**
///
/// Seven `(file, fn)` sites carrying **11** `(file, fn, variant)` triples. The finding's
/// suggested list named `layers.rs`'s classifier "and the layer walk's application arms",
/// `state/hash.rs` and `state/ability_definition_registry.rs`; the walk adds `depends_on`
/// and `representative_modifications` (two more `layers.rs` functions) and the **enum
/// declaration itself** in `crates/card-types`, and finds **no** test fixture in `src`.
const BLANKING_NAMING_SITES: &[NamingSiteRow] = &[
    NamingSiteRow {
        file: "crates/card-types/src/state/continuous_effect.rs",
        func: "UNKNOWN",
        variants: &["RemoveAllAbilities", "SetLandTypes"],
        kind: NamingKind::Declaration,
        reason: "The `pub enum LayerModification` declaration. Checkable half: both offsets \
                 must lie INSIDE the enum's own brace span, so a predicate added to an `impl \
                 LayerModification` block further down the same file is a new row, not this \
                 one. (`enclosing_fn_name` reports UNKNOWN because no `fn` precedes the \
                 declaration in that file -- an honest label, not a miss.)",
    },
    NamingSiteRow {
        file: "crates/engine/src/rules/layers.rs",
        func: "apply_layer_modification",
        variants: &["RemoveAllAbilities", "SetLandTypes"],
        kind: NamingKind::Application,
        reason: "The layer walk's application arms: they PERFORM the modification on an \
                 object the caller already selected. They never scan `continuous_effects` \
                 and never call `effect_applies_to_object`, which is what the kind check \
                 below asserts rather than asserts about.",
    },
    NamingSiteRow {
        file: "crates/engine/src/rules/layers.rs",
        func: "depends_on",
        variants: &["SetLandTypes"],
        kind: NamingKind::Dependency,
        reason: "CR 613.8 dependency detection -- 'does applying A change what B does'. It \
                 compares two modifications; it answers no question about any object's \
                 abilities.",
    },
    NamingSiteRow {
        file: "crates/engine/src/rules/layers.rs",
        func: "modification_blanks_abilities",
        variants: &["RemoveAllAbilities", "SetLandTypes"],
        kind: NamingKind::Classifier,
        reason: "THE classifier. It is the one function allowed to name these variants as a \
                 classification, and r3 decides its whole roster by CALLING it. Its own \
                 exhaustiveness (no wildcard arm) is r8's subject.",
    },
    NamingSiteRow {
        file: "crates/engine/src/rules/layers.rs",
        func: "representative_modifications",
        variants: &["SetLandTypes"],
        kind: NamingKind::Dependency,
        reason: "Builds the representative modification set the CR 613.8 dependency check \
                 compares; same non-predicate shape as `depends_on`.",
    },
    NamingSiteRow {
        file: "crates/engine/src/state/ability_definition_registry.rs",
        func: "all_ability_definitions",
        variants: &["RemoveAllAbilities"],
        kind: NamingKind::Registry,
        reason: "A declaration inside the SR-5-adjacent ability-definition enumeration: it \
                 lists the variant so the registry is exhaustive, and decides nothing.",
    },
    NamingSiteRow {
        file: "crates/engine/src/state/hash.rs",
        func: "hash_into",
        variants: &["RemoveAllAbilities", "SetLandTypes"],
        kind: NamingKind::Hashing,
        reason: "The state hasher's exhaustive `LayerModification` match (SR-8). It hashes \
                 the variant; a hasher that answered a rules question would be a far larger \
                 finding than this row.",
    },
];

/// **CR 613.1f / CR 305.7: there is exactly ONE ability-blanking predicate in this tree, and
/// this is the row that makes that an assertion instead of a sentence.**
///
/// `rules/replacement.rs` asserts it in bold and `pb-DX49-execution-notes.md` §2.1 says it
/// was "verified by enumeration" -- a one-time enumeration, i.e. a claim with no gate. The
/// `/review` appended a second hand-rolled predicate to `rules/turn_actions.rs`
/// (`matches!(e.modification, LayerModification::RemoveAllAbilities) && effect_applies_to_object(..)`)
/// and the whole `--test core` target stayed **GREEN (652 passed)**. That shape is not
/// hypothetical: it is verbatim the pre-PB-DX43 IG-1 suppressor whose 26-def regression
/// `layers.rs`'s own doc comment narrates, because it cannot see CR 305.7's `SetLandTypes`
/// channel.
///
/// **Keyed on the MECHANISM**: a second predicate must NAME a blanking variant itself rather
/// than ask `modification_blanks_abilities`. So every naming site in workspace source is
/// collected and set-compared against the allowlist above, and each allowlisted site is then
/// re-checked for the second conjunct a predicate needs (`PREDICATE_TOKENS`) -- which catches
/// a predicate planted INSIDE an already-allowlisted function, where set equality alone
/// would not.
///
/// **Stated recall bound.** Duplication within one `(file, fn, variant)` triple is invisible
/// to the set: `apply_layer_modification` legitimately names each variant once, and a second
/// naming inside it would collapse. That direction is covered by the `PREDICATE_TOKENS`
/// check, not by the count -- unlike `r6`, where a duplicated CALL is itself the defect and
/// the offset therefore rides in the tuple.
#[test]
fn r7_blanking_variant_naming_sites_are_pinned() {
    let live = live_variant_naming_sites();
    let live_triples: BTreeSet<(String, String, String)> = live
        .iter()
        .map(|s| (s.file.clone(), s.func.clone(), s.variant.clone()))
        .collect();
    let pinned: BTreeSet<(String, String, String)> = BLANKING_NAMING_SITES
        .iter()
        .flat_map(|r| {
            r.variants
                .iter()
                .map(|v| (r.file.to_string(), r.func.to_string(), (*v).to_string()))
        })
        .collect();
    assert_eq!(
        live_triples,
        pinned,
        "PB-DX49 r7: a site names LayerModification::RemoveAllAbilities or ::SetLandTypes \
         itself instead of asking layers::modification_blanks_abilities. If it is a second \
         BLANKING PREDICATE, that is the finding, not the fix -- it will not see CR 305.7's \
         channel and reproduces the 26-def regression PB-DX43's /review closed. If it is \
         not, classify it in BLANKING_NAMING_SITES with a kind and a reason. live only: \
         {:?}; pinned only: {:?}",
        live_triples.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&live_triples).collect::<Vec<_>>()
    );
    assert!(
        !live.is_empty(),
        "PB-DX49 r7: zero naming sites found -- the walk or the needles have gone vacuous, \
         and the set assertion above would then be comparing two empty sets"
    );

    // The checkable half of every reason: no allowlisted site is a predicate.
    let enum_span = layer_modification_enum_span();
    for site in &live {
        let row = BLANKING_NAMING_SITES
            .iter()
            .find(|r| r.file == site.file && r.func == site.func)
            .expect("set equality above proves every live site is classified");
        let raw = std::fs::read_to_string(workspace_root().join(&site.file))
            .expect("a file the walk just read is readable");
        let src = strip_comments(&raw);
        if row.kind == NamingKind::Declaration {
            assert!(
                site.offset >= enum_span.0 && site.offset <= enum_span.1,
                "PB-DX49 r7 ({}::{}): classified as the enum DECLARATION, but byte {} lies \
                 outside `pub enum LayerModification`'s own span {:?}. Reason on file: {}",
                site.file,
                site.func,
                site.offset,
                enum_span,
                row.reason
            );
            continue;
        }
        let (open, end) = enclosing_fn_span(&src, site.offset).unwrap_or_else(|| {
            panic!(
                "PB-DX49 r7 ({}::{}): could not bound the enclosing function around byte \
                 {}, so the 'this is not a predicate' check cannot be performed. This row \
                 FAILS CLOSED rather than skipping the check.",
                site.file, site.func, site.offset
            )
        });
        let body = &src[open..=end];
        for token in PREDICATE_TOKENS {
            assert!(
                !body.contains(token),
                "PB-DX49 r7 ({}::{}): this site names LayerModification::{} AND its \
                 enclosing function contains `{}` -- which together is a second \
                 ability-blanking PREDICATE, the exact shape the /review planted in \
                 turn_actions.rs and the exact shape that cannot see CR 305.7's SetLandTypes \
                 channel. Call layers::modification_blanks_abilities instead. Reason \
                 previously on file for this site: {}",
                site.file,
                site.func,
                site.variant,
                token,
                row.reason
            );
        }
    }
}

/// **The reviewer's defeat, reproduced as a unit test on synthetic source.**
///
/// The end-to-end proof (planting the predicate in `rules/turn_actions.rs` and watching `r7`
/// go red) was executed once and is recorded in the execution notes; this row is what keeps
/// it executable forever, since the tree cannot ship with the defect planted.
#[test]
fn r7b_extractor_sees_a_hand_rolled_predicate() {
    let planted = "fn saga_abilities_are_blanked(state: &GameState, id: ObjectId) -> bool {\n\
         \x20   state.continuous_effects.iter().any(|e| {\n\
         \x20       matches!(e.modification, LayerModification::RemoveAllAbilities)\n\
         \x20           && effect_applies_to_object(state, e, id)\n\
         \x20   })\n\
         }\n";
    let sites = variant_naming_sites_in("crates/engine/src/rules/turn_actions.rs", planted);
    assert_eq!(
        sites.len(),
        1,
        "the extractor must see the planted predicate's variant name exactly once; got {sites:?}"
    );
    assert_eq!(sites[0].func, "saga_abilities_are_blanked");
    assert_eq!(sites[0].variant, "RemoveAllAbilities");
    assert!(
        !BLANKING_NAMING_SITES
            .iter()
            .any(|r| r.file == sites[0].file && r.func == sites[0].func),
        "and it must NOT be in the allowlist, which is what makes r7's set assertion fail"
    );
    // Even if someone allowlisted it, the reason check would refuse it: the enclosing
    // function carries both predicate tokens.
    let src = strip_comments(planted);
    let (open, end) =
        enclosing_fn_span(&src, sites[0].offset).expect("the planted fn must be boundable");
    let body = &src[open..=end];
    for token in PREDICATE_TOKENS {
        assert!(
            body.contains(token),
            "the planted predicate must carry `{token}`, or r7's second conjunct proves nothing"
        );
    }

    // The UNQUALIFIED spelling is the same defect and must also be seen: a gate written for
    // one syntactic form measures that form (PB-DX47).
    let bare = "use LayerModification::RemoveAllAbilities;\nfn p(m: &LayerModification) -> bool {\n    matches!(m, RemoveAllAbilities)\n}\n";
    assert_eq!(
        variant_naming_sites_in("x.rs", bare).len(),
        2,
        "BOTH unqualified spellings must be visible -- the `use LayerModification::X;` import \
         and the bare `matches!(m, X)` that the import enables. A needle keyed on the \
         `LayerModification::` prefix would see the import alone and MISS the predicate, \
         which is PB-DX47's finding exactly: a gate written for one syntactic form measures \
         that form"
    );
    // A comment mention is not a site (SR-36's rule, and r6b's `strip_comments`).
    let commented = "fn f() {\n    // LayerModification::RemoveAllAbilities is not used here\n}\n";
    assert!(
        variant_naming_sites_in("x.rs", commented).is_empty(),
        "a prose mention is not a naming site -- OOS-CARDS2-7 / OOS-DX47-2's shape"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r8 — modification_blanks_abilities, classified EXHAUSTIVELY
// ─────────────────────────────────────────────────────────────────────────────

/// The `(open, close)` brace span of `pub enum LayerModification` in comment-stripped
/// `crates/card-types/src/state/continuous_effect.rs`.
fn layer_modification_enum_span() -> (usize, usize) {
    let path = workspace_root().join("crates/card-types/src/state/continuous_effect.rs");
    let raw = std::fs::read_to_string(&path).expect("continuous_effect.rs is readable");
    let src = strip_comments(&raw);
    let decl = src
        .find("pub enum LayerModification")
        .expect("`pub enum LayerModification` is declared in continuous_effect.rs");
    let open = src[decl..]
        .find('{')
        .map(|r| decl + r)
        .expect("the enum has a body");
    let end = matching_brace(&src, open).expect("the enum body is balanced");
    (open, end)
}

/// Every variant name **parsed from the enum's own declaration**.
///
/// Gated against the declaration rather than hand-listed, for `OOS-DX28-1`'s reason and
/// PB-DX43's `TOKEN_SPEC_FIELDS` repair: a hand-listed set is a claim, and the claim this
/// file needs is *"r8 classified all of them"*.
fn declared_layer_modification_variants() -> BTreeSet<String> {
    let path = workspace_root().join("crates/card-types/src/state/continuous_effect.rs");
    let raw = std::fs::read_to_string(&path).expect("continuous_effect.rs is readable");
    let src = strip_comments(&raw);
    let (open, end) = layer_modification_enum_span();
    let body = &src[open + 1..end];
    let mut out = BTreeSet::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = body.as_bytes();
    let push = |seg: &str, out: &mut BTreeSet<String>| {
        let mut t = seg.trim();
        // Strip any leading `#[..]` attributes.
        while let Some(rest) = t.strip_prefix('#') {
            let Some(close) = rest.find(']') else { break };
            t = rest[close + 1..].trim_start();
        }
        let name: String = t
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            out.insert(name);
        }
    };
    for i in 0..bytes.len() {
        match bytes[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
                push(&body[start..i], &mut out);
                start = i + 1;
            }
            _ => {}
        }
    }
    push(&body[start..], &mut out);
    out
}

/// One instance of **every** `LayerModification` variant.
///
/// Payloads are the cheapest legal thing in every case -- r8 asks a classification question,
/// and `modification_blanks_abilities` reads a payload in exactly one arm (`SetLandTypes`,
/// CR 305.7's basic-land-type conjunct), which is exercised in both directions.
fn every_layer_modification() -> Vec<(&'static str, LayerModification)> {
    use LayerModification as M;
    let sub = |s: &str| SubType(s.to_string());
    vec![
        ("CopyOf", M::CopyOf(ObjectId(1))),
        ("SetController", M::SetController(PlayerId(1))),
        (
            "SetTypeLine",
            M::SetTypeLine {
                supertypes: [SuperType::Legendary].into_iter().collect(),
                card_types: [CardType::Land].into_iter().collect(),
                subtypes: [sub("Mountain")].into_iter().collect(),
            },
        ),
        (
            "AddCardTypes",
            M::AddCardTypes([CardType::Creature].into_iter().collect()),
        ),
        (
            "RemoveCardTypes",
            M::RemoveCardTypes([CardType::Creature].into_iter().collect()),
        ),
        (
            "AddSubtypes",
            M::AddSubtypes([sub("Swamp")].into_iter().collect()),
        ),
        ("LoseAllSubtypes", M::LoseAllSubtypes),
        ("RemoveSuperType", M::RemoveSuperType(SuperType::Legendary)),
        ("AddAllCreatureTypes", M::AddAllCreatureTypes),
        (
            "SetCreatureTypes",
            M::SetCreatureTypes([sub("Frog")].into_iter().collect()),
        ),
        (
            "SetCardTypes",
            M::SetCardTypes([CardType::Creature].into_iter().collect()),
        ),
        (
            "SetLandTypes",
            M::SetLandTypes([sub("Mountain")].into_iter().collect()),
        ),
        (
            "SetColors",
            M::SetColors([Color::Blue].into_iter().collect()),
        ),
        (
            "AddColors",
            M::AddColors([Color::Blue].into_iter().collect()),
        ),
        ("BecomeColorless", M::BecomeColorless),
        ("AddKeyword", M::AddKeyword(KeywordAbility::Flying)),
        (
            "AddKeywords",
            M::AddKeywords([KeywordAbility::Flying].into_iter().collect()),
        ),
        ("RemoveAllAbilities", M::RemoveAllAbilities),
        ("RemoveKeyword", M::RemoveKeyword(KeywordAbility::Flying)),
        (
            "AddActivatedAbility",
            M::AddActivatedAbility(Box::<ActivatedAbility>::default()),
        ),
        ("AddManaAbility", M::AddManaAbility(ManaAbility::default())),
        (
            "SetPtViaCda",
            M::SetPtViaCda {
                power: 1,
                toughness: 1,
            },
        ),
        (
            "SetPtDynamic",
            M::SetPtDynamic {
                power: Box::new(EffectAmount::Fixed(1)),
                toughness: Box::new(EffectAmount::Fixed(1)),
            },
        ),
        ("SetPtToManaValue", M::SetPtToManaValue),
        (
            "SetPowerToughness",
            M::SetPowerToughness {
                power: 1,
                toughness: 1,
            },
        ),
        (
            "SetBothDynamic",
            M::SetBothDynamic {
                amount: Box::new(EffectAmount::Fixed(1)),
            },
        ),
        ("ModifyPower", M::ModifyPower(1)),
        ("ModifyToughness", M::ModifyToughness(1)),
        ("ModifyBoth", M::ModifyBoth(1)),
        (
            "ModifyBothDynamic",
            M::ModifyBothDynamic {
                amount: Box::new(EffectAmount::Fixed(1)),
                negate: false,
            },
        ),
        (
            "ModifyPowerDynamic",
            M::ModifyPowerDynamic {
                amount: Box::new(EffectAmount::Fixed(1)),
                negate: false,
            },
        ),
        (
            "ModifyToughnessDynamic",
            M::ModifyToughnessDynamic {
                amount: Box::new(EffectAmount::Fixed(1)),
                negate: false,
            },
        ),
        ("SwitchPowerToughness", M::SwitchPowerToughness),
    ]
}

/// **CR 613.1f / CR 305.7: the blanking classification is exhaustive, and every negative is
/// asserted rather than inferred from the corpus.**
///
/// `/review` FINDING 3: the reviewer moved `LayerModification::SwitchPowerToughness` into
/// `modification_blanks_abilities`' `true` arm and the entire `-p mtg-engine` test set stayed
/// **green**. `SetTypeLine` and `LoseAllSubtypes` redden `r3` only because the corpus happens
/// to declare them, so before this row the classifier was gated **exactly where the corpus
/// reaches** -- and PB-DX43's `f3_..._recognises_both_channels_and_no_others` pins 2
/// positives and 5 hand-picked negatives out of 33 variants, so its "and no others" was an
/// overclaim this file inherited and then made load-bearing at a second site (r3 decides its
/// whole roster by calling this function).
///
/// A variant that flips to `true` here is a **new ability-blanking channel**: it silently
/// joins r3's roster, silently changes `layers::abilities_are_blanked`, and therefore
/// silently changes which permanents CR 714's five sites treat as having no chapter
/// abilities.
#[test]
fn r8_modification_blanks_abilities_is_exhaustively_classified() {
    let built = every_layer_modification();
    let built_names: BTreeSet<String> = built.iter().map(|(n, _)| (*n).to_string()).collect();
    let declared = declared_layer_modification_variants();
    assert_eq!(
        built_names,
        declared,
        "PB-DX49 r8: `every_layer_modification()` is out of sync with the enum's own \
         declaration in crates/card-types/src/state/continuous_effect.rs. A NEW \
         LayerModification variant must be constructed here and classified below before it \
         can arrive unclassified -- which is the whole point of gating the list against the \
         declaration rather than hand-listing it. built only: {:?}; declared only: {:?}",
        built_names.difference(&declared).collect::<Vec<_>>(),
        declared.difference(&built_names).collect::<Vec<_>>()
    );
    assert_eq!(
        built.len(),
        33,
        "PB-DX49 r8: 33 LayerModification variants measured at HEAD, {} built. If the enum \
         genuinely grew, re-pin this count DELIBERATELY -- a silent count change is how a \
         fourth blanking channel arrives.",
        built.len()
    );

    // CardType::Land in the fixture, so CR 305.7's own precondition is satisfiable and the
    // SetLandTypes arm is REACHABLE. Without it that arm answers `false` for fixture reasons
    // and r8 would pin a positive set of one while claiming to have tested both channels.
    let chars = land_characteristics();
    let positives: BTreeSet<&str> = built
        .iter()
        .filter(|(_, m)| modification_blanks_abilities(m, &chars))
        .map(|(n, _)| *n)
        .collect();
    let expected: BTreeSet<&str> = ["RemoveAllAbilities", "SetLandTypes"].into_iter().collect();
    assert_eq!(
        positives,
        expected,
        "PB-DX49 r8: the ability-blanking classification moved. CR 613.1f's \
         RemoveAllAbilities and CR 305.7's SetLandTypes-with-a-basic-payload are the only \
         two channels; anything else classified `true` is a THIRD channel and every CR 714 \
         site, r3's roster and layers::abilities_are_blanked change with it. Anything \
         removed is a channel this engine has stopped honouring. classified only: {:?}; \
         expected only: {:?}",
        positives.difference(&expected).collect::<Vec<_>>(),
        expected.difference(&positives).collect::<Vec<_>>()
    );

    // CR 305.7's own precondition, both conjuncts, on the exhaustive path as well as r3b's:
    // a NONBASIC payload is not a blanker even on a land.
    let nonbasic =
        LayerModification::SetLandTypes([SubType("Gate".to_string())].into_iter().collect());
    assert!(
        !modification_blanks_abilities(&nonbasic, &chars),
        "CR 305.7 applies only when the payload names one or more BASIC land types; a \
         SetLandTypes(Gate) that blanked abilities would blank every Gate-setting effect's \
         target"
    );
    // And the two positives are not fixture artefacts: RemoveAllAbilities blanks regardless
    // of card type, SetLandTypes does not.
    let plain = Characteristics::default();
    assert!(modification_blanks_abilities(
        &LayerModification::RemoveAllAbilities,
        &plain
    ));
    assert!(!modification_blanks_abilities(
        &LayerModification::SetLandTypes([SubType("Mountain".to_string())].into_iter().collect()),
        &plain
    ));
}

/// **The arm bounding is a measurement, not a window** (`/review` FINDING 4), proven on
/// synthetic source in both failure directions.
#[test]
fn r5e_match_arm_bounding_is_a_measurement_not_a_window() {
    let src = "match e {\n    Effect::Manifest { player } => {\n        let x = 1;\n    }\n    \
               Effect::Cloak { player } => {\n        apply_self_etb_from_definition(s, id);\n    \
               }\n}\n";
    let manifest_at = src.find("Effect::Manifest {").expect("fixture");
    let (open, end) = match_arm_body_span(src, manifest_at).expect("the Manifest arm is a block");
    assert!(
        !src[open..=end].contains("apply_self_etb_from_definition"),
        "OVER-SCAN direction: the bound must stop at the arm's OWN closing brace. A fixed \
         byte window runs into the NEXT arm -- `t_census_report` prints how far -- and would \
         redden on a call that is not in the arm at all"
    );
    let cloak_at = src.find("Effect::Cloak {").expect("fixture");
    let (open2, end2) = match_arm_body_span(src, cloak_at).expect("the Cloak arm is a block");
    assert!(
        src[open2..=end2].contains("apply_self_etb_from_definition"),
        "UNDER-SCAN direction: a call anywhere inside the arm's own body must be seen, \
         however far in -- which is what a too-small fixed window silently stops doing"
    );

    // A non-block arm body must FAIL CLOSED: the next `{` belongs to a later arm.
    let non_block = "match e {\n    Effect::Manifest { player } => manifest(player),\n    \
                     Effect::Cloak { player } => {\n        apply_self_etb_from_definition();\n    \
                     }\n}\n";
    assert!(
        match_arm_body_span(
            non_block,
            non_block.find("Effect::Manifest {").expect("fixture")
        )
        .is_none(),
        "a non-block arm body must fail closed rather than silently bounding on a later \
         arm's brace"
    );
    // Unbalanced input fails closed too.
    assert!(
        match_arm_body_span("Effect::Manifest { player } => {\n    oops", 0).is_none(),
        "unbalanced input must fail closed"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t_census_report — every population, PRINTED
// ─────────────────────────────────────────────────────────────────────────────

/// PRINTS every population this file pins, with names.
///
/// PB-DX8's rule and PB-DX28's MEDIUM: a figure that appears in prose but that no test
/// prints is a figure that was transcribed, and three of PB-DX28's published fingerprints
/// had never existed in any source file in this repository. Run with
/// `cargo test -p mtg-engine --test core pb_dx49 -- --nocapture`.
#[test]
fn t_census_report() {
    eprintln!("\n=== PB-DX49 census (walked from all_cards(), never grepped) ===");

    let sagas = saga_defs();
    eprintln!(
        "\nr1 -- AbilityDefinition::SagaChapter declarers: {}",
        sagas.len()
    );
    for d in &sagas {
        eprintln!(
            "  {:<32} complete={:<5} chapters={:?}",
            d.name, d.complete, d.chapters
        );
    }
    eprintln!(
        "r1 -- deck-legal (Completeness::Complete) subset: {}",
        sagas.iter().filter(|d| d.complete).count()
    );

    let oracle = oracle_saga_defs();
    let structural: BTreeSet<String> = sagas.iter().map(|d| d.name.clone()).collect();
    eprintln!("\nr2 -- oracle-text Saga axis: {}", oracle.len());
    for n in &oracle {
        eprintln!(
            "  {:<32} {}",
            n,
            if structural.contains(n) {
                "(also structural)"
            } else {
                "RESIDUAL -- prints a Saga, declares no chapters"
            }
        );
    }
    eprintln!("r2 -- residual classification:");
    for (name, reason) in ORACLE_AXIS_RESIDUAL {
        eprintln!("  {name}\n      {reason}");
    }

    let sites = blanking_sites();
    let complete = complete_names();
    let blanker_cards: BTreeSet<String> = sites.iter().map(|s| s.card.clone()).collect();
    eprintln!(
        "\nr3 -- blanking defs (modification_blanks_abilities == true): {} defs, {} \
         modification sites",
        blanker_cards.len(),
        sites.len()
    );
    for s in &sites {
        eprintln!(
            "  {:<28} {:<8} {:<16} {:<20} filter={:<20} targets={:?} deck-legal={}",
            s.card,
            s.face,
            s.ability,
            s.modification,
            s.filter.clone().unwrap_or_else(|| "-".to_string()),
            s.ability_targets,
            complete.contains(&s.card)
        );
    }
    eprintln!(
        "r3 -- deck-legal Complete blanker defs: {}",
        blanker_cards
            .iter()
            .filter(|c| complete.contains(*c))
            .count()
    );

    eprintln!("\nr4 -- reach classification:");
    for row in REACH_ROWS {
        eprintln!(
            "  {:<28} reaches_enchantment={:<5} enchant={:?} targets={:?} filters={:?}",
            row.card,
            row.can_reach_enchantment,
            row.enchant,
            row.requires_target_variants,
            row.requires_filters
        );
        eprintln!("      {}", row.reason);
    }

    let fd = face_down_makers();
    eprintln!(
        "\nr5 -- Effect::Manifest / Effect::Cloak defs (CR 708.2a channel): {}",
        fd.len()
    );
    for (name, kind) in &fd {
        eprintln!(
            "  {:<32} {:<9} deck-legal={}",
            name,
            kind,
            complete.contains(name)
        );
    }

    let ss = live_saga_view_sites();
    eprintln!("\nr6 -- rules::saga::saga_view call sites: {}", ss.len());
    for (file, func, at) in &ss {
        eprintln!("  {file}::{func} @ byte {at}");
    }
    eprintln!("r6 -- classification:");
    for p in PINNED_SAGA_SITES {
        eprintln!("  {:<28} {:<28} {}", p.file, p.func, p.cr);
        eprintln!("      {}", p.reason);
    }

    // The walk r6 and r7 both rest on: PRINTED, so the gap between the non-vacuity floors
    // and reality is visible rather than trusted (the floors are 8 roots / 100 files).
    let roots = workspace_src_roots();
    let files = workspace_src_files();
    eprintln!(
        "\nworkspace source walk: {} `src` roots, {} .rs files (card-defs excluded)",
        roots.len(),
        files.len()
    );

    let naming = live_variant_naming_sites();
    eprintln!(
        "\nr7 -- blanking-variant naming sites: {} triples across {} (file, fn) sites",
        naming.len(),
        naming
            .iter()
            .map(|s| (s.file.clone(), s.func.clone()))
            .collect::<BTreeSet<_>>()
            .len()
    );
    for site in &naming {
        eprintln!(
            "  {:<52} {:<32} {:<20} @ byte {}",
            site.file, site.func, site.variant, site.offset
        );
    }
    eprintln!("r7 -- classification:");
    for row in BLANKING_NAMING_SITES {
        eprintln!(
            "  {:<52} {:<32} {:?} {:?}",
            row.file, row.func, row.variants, row.kind
        );
        eprintln!("      {}", row.reason);
    }

    let chars = land_characteristics();
    let built = every_layer_modification();
    eprintln!(
        "\nr8 -- LayerModification variants classified: {} ({} declared in the enum)",
        built.len(),
        declared_layer_modification_variants().len()
    );
    for (name, m) in &built {
        if modification_blanks_abilities(m, &chars) {
            eprintln!("  BLANKS  {name}");
        }
    }
    eprintln!(
        "  (all {} others classified false against a Land fixture)",
        built
            .iter()
            .filter(|(_, m)| !modification_blanks_abilities(m, &chars))
            .count()
    );

    // r5b's measured arm spans -- the figures that make "3,413 / 2,820 vs a 4,000-byte
    // window" a measurement rather than a remembered number.
    let effects = strip_comments(
        &std::fs::read_to_string(engine_src().join("effects/mod.rs")).expect("readable"),
    );
    eprintln!("\nr5b -- measured match-arm spans in effects/mod.rs:");
    for needle in ["Effect::Manifest {", "Effect::Cloak {"] {
        if let Some(at) = effects.find(needle) {
            if let Some((open, end)) = match_arm_body_span(&effects, at) {
                // How far the superseded fixed window (4,000 bytes from the PATTERN) ran
                // past the arm's own closing brace. Positive = it over-scanned into the
                // next arm by that much, and was that many bytes of arm growth away from
                // failing open; negative would mean it was ALREADY failing open.
                let overscan = (at + 4_000) as i64 - end as i64;
                eprintln!(
                    "  {:<20} body = {} bytes; the superseded 4,000-byte window ran {} bytes \
                     past this arm's own closing brace",
                    needle,
                    end - open,
                    overscan
                );
            }
        }
    }
    eprintln!();
}
