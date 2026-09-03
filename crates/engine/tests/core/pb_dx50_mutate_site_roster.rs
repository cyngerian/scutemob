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
//!   predicate in the workspace, its call sites are exactly the three behavioural sites
//!   PB-DX50 unified, **and the offer layer's host list is derived from
//!   `queries::legal_mutate_hosts` rather than merely agreeing with it** (conjunct 3, the
//!   `/review`'s finding — see below).
//!
//! # `r3`'s first draft policed the DEFINITION and the copies live in the CONSUMER
//!
//! The `/review` defeated `r3` **twice**, with all four tests in this file GREEN both
//! times, by planting a second host predicate in `crates/simulator/src/legal_actions.rs`
//! — the offer layer, which contains **zero** occurrences of `mutate_target_requirement`
//! because it calls `queries::legal_mutate_hosts`, so conjunct 1's set equality over
//! files that name the predicate could never see it. Both defeats are reproduced,
//! executed and now RED; the fixes are conjunct 3 here plus the behavioural
//! `pb_dx50_mutate_legality_channel::c6`. The durable half: **all four of the hand-rolled
//! copies this batch deleted lived in consumers, and a gate on the definition is blind to
//! every one of them by construction.**
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

/// How many times `src` **constructs** a `LegalAction::CastWithMutate`, as opposed to
/// pattern-matching one.
///
/// **Discriminated by what FOLLOWS the brace-matched block, never by its fields.** The
/// obvious keys are both wrong here and both were tried: "contains `mutate_target:`"
/// misclassifies `view.rs`'s renaming pattern `{ card: c, mutate_target }`, and any
/// field-name key is PB-DX48's `r2` defect verbatim (*it fell to FIELD ORDER, because Rust
/// does not constrain it*). What is invariant is the grammar: a match pattern is followed
/// by `=>` or by `|` (an or-pattern's next alternative); an expression is followed by
/// anything else. The live tree has 5 patterns and 1 construction and this separates them.
fn constructions_of_cast_with_mutate(src: &str) -> usize {
    let needle = "LegalAction::CastWithMutate";
    let bytes = src.as_bytes();
    let mut count = 0usize;
    let mut from = 0usize;
    while let Some(rel) = src[from..].find(needle) {
        let at = from + rel;
        from = at + needle.len();
        // Skip to the `{` that opens the pattern-or-struct body.
        let Some(open_rel) = src[from..].find('{') else {
            continue;
        };
        // Anything other than whitespace between the path and the brace means this is
        // not a `Path { .. }` form at all (e.g. a doc reference); skip it.
        if !src[from..from + open_rel].trim().is_empty() {
            continue;
        }
        let mut i = from + open_rel + 1;
        let mut depth = 1usize;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        if depth != 0 {
            panic!("unbalanced braces after {needle} -- fail closed");
        }
        let tail = src[i..].trim_start();
        let is_pattern = tail.starts_with("=>") || tail.starts_with('|');
        if !is_pattern {
            count += 1;
        }
        from = i;
    }
    count
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
/// 2. **Nothing outside the predicate hand-rolls the non-Human conjunct**, in the obvious
///    spelling. **Its recall bound is now MEASURED rather than claimed, and the claim it
///    replaces was wrong**: this used to say *"the only way to express CR 702.140a's
///    non-Human is a `"Human"` subtype literal"*. The `/review` refuted it by execution
///    with `SubType(String::from("Hum") + "an")`, which is the same predicate and carries
///    no such literal. **A string-literal census cannot be made concatenation-proof**, so
///    conjunct 2 is a cheap tripwire for the copy written the obvious way, and the
///    load-bearing checks for the consumer are conjunct 3 (structural, keyed on the
///    binding) and `simulator::pb_dx50_mutate_legality_channel::c6` (behavioural). The
///    census of that literal across the workspace is the mechanism. Comments are stripped first
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

    // ── Conjunct 3: the CONSUMER, which is where all four copies actually lived ──
    //
    // **The `/review` defeated conjuncts 1 and 2 twice, and both defeats are the same
    // structural fact: this gate polices the DEFINITION and the consumer never names
    // it.** `crates/simulator/src/legal_actions.rs` — the offer layer, the single most
    // likely place for a fifth hand-rolled predicate, and the crate three of the four
    // historical copies were NOT in — contains zero occurrences of
    // `mutate_target_requirement`, because it calls `queries::legal_mutate_hosts`. So
    // conjunct 1's set equality could never see it. Planted there, with all four roster
    // tests GREEN both times:
    //
    //  1. a second host predicate omitting the non-Human conjunct (the literal SR-38
    //     defect: a Human host offered, then refused by the cast path);
    //  2. one spelling the subtype `SubType(String::from("Hum") + "an")`, invisible to
    //     conjunct 2's `"Human"` census by construction.
    //
    // Defeat 2 is the one that decides the design here: **a string-literal scan cannot be
    // made concatenation-proof**, so conjunct 2's real recall bound is "a hand-rolled
    // copy written in the obvious way", and it is stated as that below rather than
    // claimed as a census. The load-bearing assertion for the consumer is BEHAVIOURAL —
    // `simulator::pb_dx50_mutate_legality_channel::c6` asserts the offered host set IS
    // `legal_mutate_hosts`' live return value on a board carrying a legal host, a Human,
    // a shroud host and an opponent-owned one. Both defeats redden it.
    //
    // What this conjunct adds on top is the structural half a behavioural probe cannot
    // give: that the offer's host list is *derived from* the query rather than merely
    // agreeing with it today. Keyed on the MECHANISM — the identifier the
    // `LegalAction::CastWithMutate` construction iterates must be the one bound from
    // `legal_mutate_hosts` — not on a file list, because a file list is what conjunct 1
    // was.
    let mut construction_sites: Vec<String> = Vec::new();
    for (label, path) in &files {
        let src = strip_comments(&std::fs::read_to_string(path).expect("read source"));
        if constructions_of_cast_with_mutate(&src) == 0 {
            continue;
        }
        construction_sites.push(label.clone());

        let n_query = token_count(&src, "legal_mutate_hosts");
        assert_eq!(
            n_query, 1,
            "PB-DX50: {label} constructs `LegalAction::CastWithMutate` and calls \
             `queries::legal_mutate_hosts` {n_query} time(s). It must call it EXACTLY \
             once, and use that answer. Zero means a hand-rolled CR 702.140a predicate \
             has reappeared in the offer layer -- the SR-38 shape this batch deleted, \
             where a host is offered and then refused by the cast path. More than one \
             means two host sets, and nothing here says which one reaches the offer."
        );

        // The binding the query produces, by name...
        let q_at = src
            .find("legal_mutate_hosts")
            .expect("checked non-zero above");
        let stmt_start = src[..q_at]
            .rfind("let ")
            .expect("the query's result must be bound with `let`");
        let bound: String = src[stmt_start + 4..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        assert!(
            !bound.is_empty(),
            "{label}: could not read the identifier bound from `legal_mutate_hosts`"
        );

        // ...bound EXACTLY ONCE, which is the conjunct that catches the `/review`'s
        // second defeat. That one did not replace the query call; it SHADOWED its
        // result --
        //
        //     let non_human_own = queries::legal_mutate_hosts(..);
        //     let human = SubType(String::from("Hum") + "an");   // the concatenation
        //     let non_human_own = non_human_own.into_iter().filter(..).collect();
        //
        // -- so the query is still called, the loop still iterates a variable of that
        // name, and every other conjunct here stays green. It is also invisible to the
        // BEHAVIOURAL probe (`simulator::pb_dx50_mutate_legality_channel::c6`), and that
        // is not a gap in the probe: the filter is a no-op TODAY, because
        // `legal_mutate_hosts` already excludes Humans. **A redundant second predicate
        // is not wrong until the first one changes, and then it is wrong silently** --
        // which is the entire thesis of `OOS-DX24-4` and of this batch. So the honest
        // instrument is structural, and it keys on the rebinding rather than on the
        // spelling of "Human", because a string-literal census is exactly what the
        // concatenation defeated.
        let mut bindings = 0usize;
        let needle_let = format!("let {bound}");
        let mut from = 0usize;
        while let Some(rel) = src[from..].find(&needle_let) {
            let at = from + rel;
            let end = at + needle_let.len();
            if src.as_bytes().get(end).is_none_or(|b| !is_ident_byte(*b)) {
                bindings += 1;
            }
            from = end;
        }
        assert_eq!(
            bindings, 1,
            "PB-DX50: `{bound}` -- the host list the mutate offer iterates -- is bound \
             {bindings} times in {label}. It must be bound EXACTLY once, from \
             `queries::legal_mutate_hosts`. A second `let {bound}` is a SHADOWING \
             rebind: the query is still called, the loop still reads a variable of that \
             name, and a hand-rolled CR 702.140a predicate has been spliced in between. \
             That copy is a no-op only for as long as it happens to agree with \
             `legal_mutate_hosts`, and nothing makes it agree."
        );

        // ...must be the one the construction iterates. Brace-matched from the `for`
        // header, never a byte window (PB-DX49's `/review` caught a fixed-width scan
        // over-running into the next arm by a kilobyte).
        let header = format!("for &target in &{bound} {{");
        let for_at = src.find(&header).unwrap_or_else(|| {
            panic!(
                "PB-DX50: {label} binds `{bound}` from `legal_mutate_hosts`, but the \
                 `LegalAction::CastWithMutate` construction does not sit in a \
                 `{header}` loop. The offer's host list must BE the engine's answer, \
                 not a set filtered or replaced afterwards -- see \
                 `simulator::pb_dx50_mutate_legality_channel::c6`."
            )
        });
        let bytes = src.as_bytes();
        let mut i = for_at + header.len();
        let mut depth = 1usize;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        assert!(
            depth == 0,
            "{label}: unbalanced braces in the offer loop -- fail closed"
        );
        let loop_body = &src[for_at..i];
        assert!(
            constructions_of_cast_with_mutate(loop_body) > 0,
            "PB-DX50: {label}'s `LegalAction::CastWithMutate` is constructed OUTSIDE the \
             loop over `{bound}` (the identifier bound from `legal_mutate_hosts`), so \
             its `mutate_target` comes from somewhere this gate cannot vouch for."
        );
        assert_eq!(
            constructions_of_cast_with_mutate(loop_body),
            constructions_of_cast_with_mutate(&src),
            "PB-DX50: {label} constructs `LegalAction::CastWithMutate` somewhere outside \
             the loop over the engine's own answer. Every offered mutate host must come \
             from `legal_mutate_hosts`."
        );
    }
    assert_eq!(
        construction_sites,
        vec!["crates/simulator/src/legal_actions.rs".to_string()],
        "PB-DX50: `LegalAction::CastWithMutate` must be constructed in exactly one place \
         in the workspace. A second offer site is a second place for a CR 702.140a \
         predicate to grow. Found: {construction_sites:?}"
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

    // Conjunct 3's construction-vs-pattern discriminator, on synthetic input. Every
    // shape below is copied from a REAL site in this workspace, so the separation is
    // proven against the forms that exist rather than against invented ones.
    assert_eq!(
        constructions_of_cast_with_mutate(
            "actions.push(LegalAction::CastWithMutate { card: obj.id, mutate_target: t });"
        ),
        1,
        "a construction must count"
    );
    for pattern in [
        // heuristic_bot.rs:304
        "LegalAction::CastWithMutate { .. } => 50,",
        // params.rs:577 -- shorthand bindings, no colons
        "LegalAction::CastWithMutate {\n card,\n mutate_target,\n } => Ok(x),",
        // view.rs:1585 -- a RENAMING pattern, which carries `card:` and would be
        // misclassified by any field-name key.
        "LegalAction::CastWithMutate {\n card: c,\n mutate_target,\n } => format!(\"x\"),",
        // view.rs:1494 -- an or-pattern alternative, followed by `|` rather than `=>`.
        "| LegalAction::CastWithMutate { card, .. }\n | LegalAction::CastMorphFaceDown { card, .. } => Some(*card),",
    ] {
        assert_eq!(
            constructions_of_cast_with_mutate(pattern),
            0,
            "a match PATTERN must not count as a construction: {pattern:?}"
        );
    }
}
