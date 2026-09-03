//! PB-DX50 half 1 (`OOS-DX25-1`) — the mutate corpus census and the single-predicate gate.
//!
//! Three axes, none of which nests inside another (PB-DX26's / PB-DX43's durable lesson:
//! *a roster derived from one declaration construct measures that construct*):
//!
//! * **`r1`** — the STRUCTURAL population, by walking `all_cards()` and matching
//!   `AbilityDefinition::Keyword(KeywordAbility::Mutate)` on every face. SR-36's rule:
//!   enumerate `all_cards()` for rosters, **never grep source**. Four consecutive batches
//!   in this queue have broken that rule and published a grep count as a census
//!   (`OOS-CARDS2-7` → `OOS-DX47-2` → PB-DX48 → PB-DX49), so `r1` **prints** its members
//!   rather than asking the next reader to trust a bare number.
//! * **`r2`** — the INVERSE, oracle-text axis: defs whose printed text says "Mutate"
//!   while declaring no marker. A structural axis cannot see these by construction.
//! * **`r3` / `r3b`** — the source gate: there is exactly ONE mutate target-legality
//!   predicate in the workspace, and its call sites are exactly the three behavioural
//!   sites PB-DX50 unified.
//!
//! **The gate walks the WHOLE WORKSPACE, and that is not a stylistic choice.** PB-DX48's
//! equivalent gate was defeated because it walked one crate; PB-DX49's `r6` was defeated
//! the same way one batch later, by a consumer planted one crate up. `casting::
//! mutate_target_requirement` is `pub(crate)` and `queries::legal_mutate_hosts` is `pub`,
//! and the fourth hand-rolled copy this batch deleted lived in
//! `crates/simulator/src/legal_actions.rs` — a different crate from every other site. A
//! gate narrower than the workspace would have been blind to the exact defect.
//!
//! **The walk helpers are SHARED, not mirrored.** `workspace_src_files_checked` (with its
//! executing non-vacuity floors) and `strip_comments` are `pub(super)` in
//! `pb_dx49_saga_blanking_roster` and called from here. Two copies of a workspace walk is
//! two things to keep in step, and the one that drifts is the one nobody re-measures.

use std::collections::BTreeSet;

use mtg_engine::{all_cards, AbilityDefinition, CardDefinition, KeywordAbility};

use crate::decision_site_walk::is_effectively_complete;
use crate::pb_dx49_saga_blanking_roster::{strip_comments, workspace_src_files_checked};

// ─────────────────────────────────────────────────────────────────────────────
// r1 — the structural census
// ─────────────────────────────────────────────────────────────────────────────

/// Every ability list a declaration can hide a keyword marker in. A `CardFace` carries its
/// own `abilities`, so reading `def.abilities` alone is `OOS-DX8`'s exact defect.
fn all_ability_lists(def: &CardDefinition) -> Vec<&[AbilityDefinition]> {
    let mut out: Vec<&[AbilityDefinition]> = vec![&def.abilities];
    if let Some(f) = def.back_face.as_ref() {
        out.push(&f.abilities);
    }
    if let Some(f) = def.adventure_face.as_ref() {
        out.push(&f.abilities);
    }
    out
}

fn all_oracle_texts(def: &CardDefinition) -> Vec<String> {
    let mut out = vec![def.oracle_text.to_lowercase()];
    for f in [def.back_face.as_ref(), def.adventure_face.as_ref()]
        .into_iter()
        .flatten()
    {
        out.push(f.oracle_text.to_lowercase());
    }
    out
}

fn declares_mutate(def: &CardDefinition) -> bool {
    all_ability_lists(def)
        .into_iter()
        .flatten()
        .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Mutate)))
}

fn declares_mutate_cost(def: &CardDefinition) -> bool {
    all_ability_lists(def)
        .into_iter()
        .flatten()
        .any(|a| matches!(a, AbilityDefinition::MutateCost { .. }))
}

/// `(name, deck_legal)` for every def declaring the Mutate keyword, from `all_cards()`.
fn mutate_defs() -> Vec<(String, bool, bool)> {
    let mut out: Vec<(String, bool, bool)> = all_cards()
        .iter()
        .filter(|d| declares_mutate(d))
        .map(|d| {
            (
                d.name.clone(),
                is_effectively_complete(d),
                declares_mutate_cost(d),
            )
        })
        .collect();
    out.sort();
    out
}

/// The mutate population, PINNED and PRINTED.
///
/// Measured at HEAD by walking `all_cards()`: **8** defs declare the Mutate keyword, of
/// which **6** are deck-legal `Completeness::Complete` (`Mindleecher` and
/// `Nethroi, Apex of Death` are not) and **8** also declare an
/// `AbilityDefinition::MutateCost` (without which `handle_cast_spell` refuses the cast
/// before target legality is ever reached, so a def in the first set but not the third
/// would be a live hole this roster would surface).
///
/// If any number below moves: re-derive by running this test with `--nocapture` and read
/// the printed member list. Do not adjust a constant against a remembered figure.
#[test]
fn r1_mutate_population_is_pinned_and_printed() {
    let defs = mutate_defs();
    println!("\n=== PB-DX50 r1: mutate population, from all_cards() ===");
    for (name, complete, has_cost) in &defs {
        println!(
            "  {name}  [{}]  [{}]",
            if *complete {
                "Complete"
            } else {
                "not-deck-legal"
            },
            if *has_cost {
                "MutateCost"
            } else {
                "NO MutateCost"
            }
        );
    }
    let complete: Vec<&String> = defs
        .iter()
        .filter(|(_, c, _)| *c)
        .map(|(n, _, _)| n)
        .collect();
    println!(
        "  total {} / deck-legal Complete {} / with MutateCost {}\n",
        defs.len(),
        complete.len(),
        defs.iter().filter(|(_, _, h)| *h).count()
    );

    assert_eq!(
        defs.len(),
        8,
        "PB-DX50: the Mutate-declaring population moved. Members: {defs:#?}"
    );
    assert_eq!(
        complete.len(),
        6,
        "PB-DX50: the deck-legal Complete mutate population moved. Members: {complete:#?}"
    );
    assert_eq!(
        defs.iter().filter(|(_, _, h)| *h).count(),
        8,
        "PB-DX50: every Mutate-declaring def must also declare an \
         AbilityDefinition::MutateCost -- without one `handle_cast_spell` refuses the \
         cast before target legality is ever reached, so a mismatch here is a live \
         uncastable-card hole. Members: {defs:#?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r2 — the inverse, oracle-text axis
// ─────────────────────────────────────────────────────────────────────────────

/// Defs whose PRINTED text carries "mutate" while declaring no `KeywordAbility::Mutate`.
///
/// Measured EXACTLY 0 at HEAD, so it is PINNED at zero rather than ceilinged. Stated
/// precision bound: the needle is a substring, so an ordinary English use of the word in a
/// rules blob would land here without being a defect -- the assertion message says so and
/// tells a future reader to allowlist with a reason rather than raise a number.
#[test]
fn r2_oracle_axis_residual_is_ratcheted() {
    let residual: BTreeSet<String> = all_cards()
        .iter()
        .filter(|d| !declares_mutate(d))
        .filter(|d| all_oracle_texts(d).iter().any(|t| t.contains("mutate")))
        .map(|d| {
            format!(
                "{} [{}]",
                d.name,
                if is_effectively_complete(d) {
                    "Complete"
                } else {
                    "not-deck-legal"
                }
            )
        })
        .collect();
    println!("\n=== PB-DX50 r2: prints \"mutate\", declares no marker ===");
    for r in &residual {
        println!("  {r}");
    }
    println!("  total {}\n", residual.len());

    assert!(
        residual.is_empty(),
        "PB-DX50: {} def(s) print \"mutate\" without declaring KeywordAbility::Mutate. \
         Measured EXACTLY 0 at HEAD, so this is pinned rather than ceilinged. A new member \
         is one of two things and they need different answers: (a) a card whose printed \
         mutate is unreachable by every channel -- r1's structural axis is blind to it by \
         construction, and it is a real gap; or (b) an ordinary English use of the word in \
         a rules blob, in which case allowlist it HERE, with the reason. Members: \
         {residual:#?}",
        residual.len()
    );

    // Non-vacuity: the axis must actually be able to see the word, or its ceiling is
    // meaningless. Every def that DOES declare the marker prints it too.
    let declaring_and_printing = all_cards()
        .iter()
        .filter(|d| declares_mutate(d))
        .filter(|d| all_oracle_texts(d).iter().any(|t| t.contains("mutate")))
        .count();
    assert_eq!(
        declaring_and_printing, 8,
        "PB-DX50: the oracle needle must match the 8 defs that DO declare Mutate; if it \
         does not, r2's ceiling is measuring nothing"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r3 / r3b — the single-predicate source gate
// ─────────────────────────────────────────────────────────────────────────────

/// Count occurrences of `needle` as a whole token (not as a substring of a longer
/// identifier) in `src`.
///
/// **Keyed on the BARE symbol at word boundaries, never on a qualified path.** PB-DX47's
/// defect was a gate whose needle was a qualified `Enum::Variant`, evaded the moment a
/// `use` import let the same code be written unqualified; PB-DX49's `/review` found the
/// same shape again in its own prescribed fix. Every real call site here already spells
/// the symbol three different ways (`mutate_target_requirement()`,
/// `casting::mutate_target_requirement()`, `crate::rules::casting::…`), and the bare-token
/// scan sees all three plus any fourth spelling.
fn token_count(src: &str, needle: &str) -> usize {
    let bytes = src.as_bytes();
    let mut count = 0usize;
    let mut from = 0usize;
    while let Some(rel) = src[from..].find(needle) {
        let start = from + rel;
        let end = start + needle.len();
        let before_ok = start == 0 || !is_ident_byte(bytes[start - 1]);
        let after_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            count += 1;
        }
        from = end;
    }
    count
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// CR 702.140a — there is exactly ONE mutate target-legality predicate in the workspace,
/// and exactly three consumers of it.
///
/// Before PB-DX50 there were FOUR hand-rolled copies of the CR 702.140a conjunct, in three
/// crates: `casting.rs` (cast time), `resolution.rs` (CR 702.140b re-check),
/// `legal_actions.rs` (the offer layer, and the only one reading RAW rather than
/// layer-resolved characteristics) — plus the requirement is now also consumed by
/// `queries.rs`. This gate is what stops a fifth appearing.
///
/// Two independent conjuncts, because either alone is defeatable:
///
/// 1. **The definition is unique and its call sites are exactly the known set.** A new
///    consumer is fine — it is the shared predicate, that is the point — but it must be
///    declared here, so nobody adds one silently while believing they added a copy.
/// 2. **Nothing outside the predicate hand-rolls the non-Human conjunct.** The only way to
///    express CR 702.140a's "non-Human" is a `"Human"` subtype literal, so the census of
///    that literal across the workspace is the mechanism. Comments are stripped first
///    (PB-DX49's `/review` finding: a `//`-only stripper leaves `/* */` blocks visible, and
///    this file's own module doc quotes the deleted predicate verbatim), so a gate that
///    did not strip would be satisfied by its own documentation.
#[test]
fn r3_exactly_one_mutate_target_legality_predicate_in_the_workspace() {
    let files = workspace_src_files_checked();

    // ── Conjunct 1: the predicate's definition and its consumers ────────────────
    let mut definitions: Vec<String> = Vec::new();
    let mut referencing: Vec<(String, usize)> = Vec::new();
    for (label, path) in &files {
        let src = strip_comments(&std::fs::read_to_string(path).expect("read source"));
        let n = token_count(&src, "mutate_target_requirement");
        if n == 0 {
            continue;
        }
        if src.contains("fn mutate_target_requirement") {
            definitions.push(label.clone());
        }
        referencing.push((label.clone(), n));
    }
    println!("\n=== PB-DX50 r3: mutate_target_requirement sites ===");
    for (label, n) in &referencing {
        println!("  {label}  x{n}");
    }

    assert_eq!(
        definitions,
        vec!["crates/engine/src/rules/casting.rs".to_string()],
        "PB-DX50: `mutate_target_requirement` must be defined EXACTLY ONCE, in \
         crates/engine/src/rules/casting.rs. A second definition is a second CR 702.140a \
         predicate, which is the defect this batch removed. Found: {definitions:?}"
    );

    let expected: Vec<(String, usize)> = vec![
        // The definition plus its one cast-path call (`handle_cast_spell`).
        ("crates/engine/src/rules/casting.rs".to_string(), 2),
        // The offer layer's shared query, `queries::legal_mutate_hosts`.
        ("crates/engine/src/rules/queries.rs".to_string(), 1),
        // CR 702.140b re-validation in the `MutatingCreatureSpell` resolution arm.
        ("crates/engine/src/rules/resolution.rs".to_string(), 1),
    ];
    assert_eq!(
        referencing, expected,
        "PB-DX50: the consumers of `mutate_target_requirement` moved. Adding a consumer \
         is CORRECT (it is the shared predicate) -- add it to this list with the CR site \
         it serves. What must never happen is a site deciding mutate host legality \
         WITHOUT calling it."
    );

    // ── Conjunct 2: nobody hand-rolls the non-Human conjunct ────────────────────
    //
    // Allowlist, each entry with the reason it is not a predicate:
    //   * card-types/src/state/types.rs — the printed creature-subtype data table
    //     (`ALL_CREATURE_TYPES`), a declaration, never a decision.
    //   * engine/src/rules/casting.rs — `mutate_target_requirement` itself, checked
    //     below for CONTENT, not merely allowlisted by name.
    let allow: BTreeSet<&str> = [
        "crates/card-types/src/state/types.rs",
        "crates/engine/src/rules/casting.rs",
    ]
    .into_iter()
    .collect();
    let mut human_sites: Vec<String> = Vec::new();
    for (label, path) in &files {
        let src = strip_comments(&std::fs::read_to_string(path).expect("read source"));
        if src.contains("\"Human\"") {
            human_sites.push(label.clone());
        }
    }
    println!("  \"Human\" literal sites: {human_sites:?}\n");
    let unexpected: Vec<&String> = human_sites
        .iter()
        .filter(|l| !allow.contains(l.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "PB-DX50: a `\"Human\"` subtype literal appeared in {unexpected:?}. CR 702.140a's \
         non-Human restriction has exactly one home -- \
         `casting::mutate_target_requirement`'s `exclude_subtypes`. A hand-rolled copy is \
         the defect this batch deleted from three files; route the decision through the \
         shared requirement instead. If the literal is genuinely unrelated to mutate, add \
         it to this allowlist WITH ITS REASON."
    );

    // Non-vacuity for conjunct 2: the census must actually be finding sites, and the
    // predicate's own site must be one of them. A stripper bug or a broken walk would
    // otherwise leave this assertion passing on an empty set.
    assert!(
        human_sites.contains(&"crates/engine/src/rules/casting.rs".to_string()),
        "PB-DX50: the `\"Human\"` census does not contain the predicate's own file; the \
         scan has gone vacuous. Sites: {human_sites:?}"
    );

    // ...and the occurrence in casting.rs must be INSIDE the requirement builder, not
    // merely somewhere in a 6,900-line file. Set equality over FILES cannot catch a
    // predicate added inside an already-allowlisted file -- PB-DX49's `/review` defeated
    // its own `r7` exactly that way.
    let casting = strip_comments(
        &std::fs::read_to_string(
            files
                .iter()
                .find(|(l, _)| l == "crates/engine/src/rules/casting.rs")
                .map(|(_, p)| p)
                .expect("casting.rs is in the walk"),
        )
        .expect("read casting.rs"),
    );
    assert_eq!(
        casting.matches("\"Human\"").count(),
        1,
        "PB-DX50: casting.rs must contain EXACTLY ONE `\"Human\"` literal, the one inside \
         `mutate_target_requirement`. A second is a second predicate hiding inside an \
         allowlisted file."
    );
    let def_at = casting
        .find("fn mutate_target_requirement")
        .expect("the definition is in casting.rs");
    let human_at = casting
        .find("\"Human\"")
        .expect("the literal is in casting.rs");
    assert!(
        human_at > def_at && human_at - def_at < 400,
        "PB-DX50: the `\"Human\"` literal in casting.rs is not inside \
         `mutate_target_requirement`'s body (definition at {def_at}, literal at \
         {human_at}). Some other code in this file is deciding non-Human-ness."
    );
}

/// `r3`'s own discrimination proof, on SYNTHETIC input rather than by trusting the live
/// tree: `token_count` must be a whole-token match and `strip_comments` must actually
/// blind the census to a commented-out copy.
///
/// Without this, `r3` could be passing because its needles never match anything.
#[test]
fn r3b_the_gate_primitives_discriminate() {
    // Whole-token, not substring: a longer identifier must not count.
    assert_eq!(
        token_count("mutate_target_requirement()", "mutate_target_requirement"),
        1
    );
    assert_eq!(
        token_count(
            "casting::mutate_target_requirement();\ncrate::rules::casting::mutate_target_requirement()",
            "mutate_target_requirement"
        ),
        2,
        "both the two- and four-segment qualified spellings must be seen -- keying on a \
         qualified path is PB-DX47's defect"
    );
    assert_eq!(
        token_count(
            "fn my_mutate_target_requirement_v2()",
            "mutate_target_requirement"
        ),
        0,
        "a longer identifier containing the needle must NOT count"
    );

    // Comment stripping: both comment shapes must hide a planted copy.
    for planted in [
        "// let x = SubType(\"Human\".to_string());",
        "/* let x = SubType(\"Human\".to_string()); */",
    ] {
        assert!(
            !strip_comments(planted).contains("\"Human\""),
            "strip_comments must blind the census to {planted:?} -- an unstripped gate is \
             satisfied by its own documentation"
        );
    }
    assert!(
        strip_comments("let x = SubType(\"Human\".to_string());").contains("\"Human\""),
        "strip_comments must NOT hide real code -- otherwise r3's conjunct 2 is vacuous"
    );
}
