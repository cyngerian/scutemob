//! PB-DX20b (`OOS-DX20-10`, HIGH; `OOS-DX20-5`) — the printed Enchant line **census**.
//!
//! CR 702.5a: an Aura's *"Enchant [object or player]"* line restricts its target (CR 303.4a)
//! and its attachment (CR 303.4c / 704.5m). `crates/engine/tests/primitives/
//! pb_dx20b_enchant_card_type_or.rs` measures the ENGINE — that the repaired filters behave.
//! This file measures the **corpus**: does every def's *declared* `EnchantTarget` say what its
//! *printed* Enchant line says?
//!
//! Every population below is built by walking `all_cards()`, or by parsing engine/type source.
//! **Never by grepping card source.** That is SR-36's rule, and this queue has broken it four
//! batches running (`OOS-CARDS2-7` → `OOS-DX47-2` → PB-DX48 → PB-DX49), every time by counting
//! a variant named inside a `// TODO` as a usage. The exploration for this file reproduced the
//! trap in miniature: `grep -l 'KeywordAbility::Enchant('` over `crates/card-defs/src/defs`
//! returns **23** files and the printed-line axis returns **25**, and the two extra are not a
//! grep artefact but a real, load-bearing gap — see `r1`'s allowlist.
//!
//! ## The two axes DO NOT NEST, and that is the point
//!
//! * **Axis (a), structural** — every def declaring `AbilityDefinition::Keyword(
//!   KeywordAbility::Enchant(_))` on **any** face.
//! * **Axis (b), printed** — every def whose `oracle_text` on **any** face carries a line
//!   beginning `"Enchant "`. A `CardFace` carries its **own** `oracle_text`, and reading only
//!   `def.oracle_text` is `OOS-DX8`'s exact defect (PB-DX8's oracle axis was blind to every
//!   transformed face and Adventure half until it was widened), repeated by `OOS-DX47`.
//!
//! Neither contains the other. A def can declare the keyword with no printed line (a keyword
//! synthesized for a mechanic, or a fixture-shaped def), and a def can print the line with no
//! declaration — which is exactly where this corpus's two residuals live, and both of them are
//! *blockers*, not oversights: `animate_dead` prints a **zone** restriction `EnchantFilter`
//! cannot express at all, and `curse_of_opulence` prints `"Enchant player"`, whose attachment
//! path does not exist (`OOS-DX20-2`).
//!
//! ## Rows
//!
//! * **r1** — the census, **PRINTED**. Both axes, the parsed printed spec beside the declared
//!   one, the mismatch set asserted EXACTLY empty, and every unclassifiable printed line
//!   allowlisted with a reason that is itself checked. PB-DX27's rule: print the population,
//!   never transcribe it.
//! * **r2** — the INVERSE axis: every def whose printed line needs an OR over classes or a
//!   controller clause. Keyed on the **parsed mechanism** (`>1` class, or a controller
//!   constraint), not on a `" or "` substring, because a substring axis measures the substring.
//! * **r3** — the variant sweep: declared-vs-printed agreement for every `EnchantTarget`
//!   variant the corpus reaches, with the deck-legal `Complete` members separated from the
//!   rest, and every unreached variant pinned at **zero**.
//! * **r4** — non-vacuity floors, each a named count, so a corpus move is a finding rather than
//!   a silent re-tune.
//! * **r5** — the STRUCTURAL gate: `EnchantFilter`'s field list, parsed from its own
//!   declaration, against the field list `casting::enchant_filter_to_target_filter` reads. See
//!   that row's own doc for why the **compiler cannot** catch an unlowered field here.
//! * **`t_census_report`** — prints every table above under `--nocapture`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use mtg_engine::{
    all_cards, AbilityDefinition, CardDefinition, CardType, EnchantControllerConstraint,
    EnchantFilter, EnchantTarget, KeywordAbility,
};

use crate::decision_site_walk::is_effectively_complete;
use crate::pb_dx49_saga_blanking_roster::strip_comments;

// ─────────────────────────────────────────────────────────────────────────────
// Shared walk helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Every ability list a declaration can hide in: the front face's, and every alternate face's.
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

/// Every face's printed text, as `(face label, text)`.
fn all_oracle_texts(def: &CardDefinition) -> Vec<(&'static str, &str)> {
    let mut out: Vec<(&'static str, &str)> = vec![("front", def.oracle_text.as_str())];
    if let Some(face) = def.back_face.as_ref() {
        out.push(("back", face.oracle_text.as_str()));
    }
    if let Some(face) = def.adventure_face.as_ref() {
        out.push(("adventure", face.oracle_text.as_str()));
    }
    out
}

/// Axis (a): every `KeywordAbility::Enchant(_)` declared on any face, in declaration order.
fn declared_enchant_targets(def: &CardDefinition) -> Vec<(&'static str, EnchantTarget)> {
    let mut out = Vec::new();
    for (label, abilities) in all_ability_lists(def) {
        for ability in abilities {
            if let AbilityDefinition::Keyword(KeywordAbility::Enchant(et)) = ability {
                out.push((label, et.clone()));
            }
        }
    }
    out
}

/// Axis (b): every printed line beginning `"Enchant "`, on any face, with the leading keyword
/// stripped. The match is on the LINE, not on the text — `"Enchant"` also appears inside
/// reminder text and inside `animate_dead`'s own quoted ability text, and a substring axis
/// would count both.
fn printed_enchant_lines(def: &CardDefinition) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    for (label, text) in all_oracle_texts(def) {
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("Enchant ") {
                out.push((label, rest.trim().to_string()));
            }
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// The normalized Enchant spec — one shape both axes reduce to
// ─────────────────────────────────────────────────────────────────────────────

/// A printed Enchant line, or a declared `EnchantTarget`, reduced to one comparable shape.
///
/// `types` is an **OR** set (empty = no card-type restriction); `subtypes` likewise.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct EnchantSpec {
    types: BTreeSet<String>,
    subtypes: BTreeSet<String>,
    basic: bool,
    nonbasic: bool,
    controller: &'static str,
    /// CR 702.5d — `"Enchant player"`. A different attachment axis entirely.
    player: bool,
}

impl EnchantSpec {
    fn render(&self) -> String {
        let types = if self.types.is_empty() {
            "-".to_string()
        } else {
            self.types.iter().cloned().collect::<Vec<_>>().join("|")
        };
        let subs = if self.subtypes.is_empty() {
            "-".to_string()
        } else {
            self.subtypes.iter().cloned().collect::<Vec<_>>().join("|")
        };
        format!(
            "types={} subtypes={} basic={} nonbasic={} controller={}{}",
            types,
            subs,
            self.basic,
            self.nonbasic,
            self.controller,
            if self.player { " PLAYER" } else { "" }
        )
    }
}

fn controller_label(c: &EnchantControllerConstraint) -> &'static str {
    match c {
        EnchantControllerConstraint::Any => "Any",
        EnchantControllerConstraint::You => "You",
        EnchantControllerConstraint::Opponent => "Opponent",
    }
}

fn card_type_word(t: &CardType) -> String {
    format!("{t:?}")
}

/// CR 205.3i / CR 305.6 — the five basic land types. A printed Enchant line naming one of them
/// (`"Enchant Mountain"`) restricts to a **land**, and the declaration says so explicitly with
/// `has_card_type: Some(Land)`. Deriving that implication here is what lets the two axes be
/// compared with plain equality instead of a bespoke per-def rule.
const BASIC_LAND_TYPES: &[&str] = &["Plains", "Island", "Swamp", "Mountain", "Forest"];

/// The lowercase word for each card type an Enchant line can print.
///
/// Deliberately a fixed list rather than a `CardType` variant sweep: an Enchant line prints the
/// **English** class word, and a card type with no printed Enchant usage (`Instant`, `Sorcery`,
/// `Kindred`, …) must fall to the UNCLASSIFIED branch rather than be silently accepted.
fn card_type_from_word(w: &str) -> Option<CardType> {
    Some(match w {
        "creature" => CardType::Creature,
        "land" => CardType::Land,
        "artifact" => CardType::Artifact,
        "enchantment" => CardType::Enchantment,
        "planeswalker" => CardType::Planeswalker,
        "battle" => CardType::Battle,
        _ => return None,
    })
}

/// Parse a printed Enchant line (the text AFTER `"Enchant "`) into an [`EnchantSpec`].
///
/// The grammar, and why each step is where it is:
///
/// 1. **Strip the controller clause** — a trailing `" you control"` / `" an opponent controls"`
///    / `" you don't control"`. It must come off before splitting, because it qualifies the
///    whole disjunction (*"creature or planeswalker you control"* is `(C|PW) & You`, never
///    `C | (PW & You)`).
/// 2. **Normalize the separators** — `", or "` and `" or "` and `", "` all become `,`. Order
///    matters: `", or "` first, or `" or "` leaves a dangling comma.
/// 3. **Per token**: strip a leading `basic ` / `nonbasic ` supertype word (CR 205.4a), then
///    classify the remainder.
/// 4. **Classification is keyed on CASE, which is how Magic prints the distinction**: a card
///    type is printed lowercase (*"Enchant creature"*), a subtype is printed capitalized
///    (*"Enchant Mountain"*). A lowercase token that is not a known class word is
///    **UNCLASSIFIED** — it is returned as an error rather than silently treated as a subtype,
///    which is what makes `animate_dead`'s *"creature card in a graveyard"* a reported residual
///    instead of a phantom `SubType("creature card in a graveyard")` that would quietly match
///    nothing.
///
/// Returns `Err(token)` naming the first token it cannot classify.
fn parse_printed_enchant_line(rest: &str) -> Result<EnchantSpec, String> {
    let mut spec = EnchantSpec {
        controller: "Any",
        ..Default::default()
    };

    let mut body = rest.trim().to_string();
    for (suffix, label) in [
        (" you control", "You"),
        (" an opponent controls", "Opponent"),
        (" you don't control", "Opponent"),
    ] {
        if let Some(head) = body.strip_suffix(suffix) {
            spec.controller = label;
            body = head.trim().to_string();
            break;
        }
    }

    let normalized = body
        .replace(", or ", ",")
        .replace(" or ", ",")
        .replace(", ", ",");

    for raw in normalized.split(',') {
        let mut token = raw.trim();
        if token.is_empty() {
            continue;
        }
        if let Some(t) = token.strip_prefix("basic ") {
            spec.basic = true;
            token = t.trim();
        } else if let Some(t) = token.strip_prefix("nonbasic ") {
            spec.nonbasic = true;
            token = t.trim();
        }

        if token == "player" {
            spec.player = true;
            continue;
        }
        if token == "permanent" {
            // No card-type restriction at all (CR 702.5a's widest object form).
            continue;
        }
        let starts_upper = token.chars().next().is_some_and(char::is_uppercase);
        if starts_upper {
            spec.subtypes.insert(token.to_string());
            continue;
        }
        match card_type_from_word(token) {
            Some(ct) => {
                spec.types.insert(card_type_word(&ct));
            }
            None => return Err(token.to_string()),
        }
    }

    // CR 205.3i / 305.6: a basic land type implies the Land card type. Applied only when NO
    // card type was printed, so it can never widen an explicit one.
    if spec.types.is_empty()
        && !spec.subtypes.is_empty()
        && spec
            .subtypes
            .iter()
            .all(|s| BASIC_LAND_TYPES.contains(&s.as_str()))
    {
        spec.types.insert(card_type_word(&CardType::Land));
    }

    Ok(spec)
}

/// Reduce a declared `EnchantFilter` to the comparable shape.
///
/// `has_card_type` (a single AND conjunct) and `has_card_types` (an OR set) are independent
/// fields — the struct's own doc says so — so "at most one is set" is an assumption, not a
/// fact, and it is **asserted** by `r1` rather than assumed here. When only one is set the
/// union below is exact; `r1`'s assertion is what keeps it exact.
fn filter_to_spec(f: &EnchantFilter) -> EnchantSpec {
    let mut types: BTreeSet<String> = f.has_card_types.iter().map(card_type_word).collect();
    if let Some(t) = f.has_card_type.as_ref() {
        types.insert(card_type_word(t));
    }
    let mut subtypes: BTreeSet<String> = f.has_subtypes.iter().map(|s| s.0.clone()).collect();
    if let Some(s) = f.has_subtype.as_ref() {
        subtypes.insert(s.0.clone());
    }
    EnchantSpec {
        types,
        subtypes,
        basic: f.basic,
        nonbasic: f.nonbasic,
        controller: controller_label(&f.controller),
        player: false,
    }
}

fn one_type_spec(t: CardType) -> EnchantSpec {
    EnchantSpec {
        types: [card_type_word(&t)].into_iter().collect(),
        controller: "Any",
        ..Default::default()
    }
}

fn declared_to_spec(et: &EnchantTarget) -> EnchantSpec {
    match et {
        EnchantTarget::Creature => one_type_spec(CardType::Creature),
        EnchantTarget::Artifact => one_type_spec(CardType::Artifact),
        EnchantTarget::Enchantment => one_type_spec(CardType::Enchantment),
        EnchantTarget::Land => one_type_spec(CardType::Land),
        EnchantTarget::Planeswalker => one_type_spec(CardType::Planeswalker),
        EnchantTarget::Permanent => EnchantSpec {
            controller: "Any",
            ..Default::default()
        },
        EnchantTarget::CreatureOrPlaneswalker => EnchantSpec {
            types: [
                card_type_word(&CardType::Creature),
                card_type_word(&CardType::Planeswalker),
            ]
            .into_iter()
            .collect(),
            controller: "Any",
            ..Default::default()
        },
        EnchantTarget::Player => EnchantSpec {
            controller: "Any",
            player: true,
            ..Default::default()
        },
        EnchantTarget::Filtered(f) => filter_to_spec(f),
    }
}

/// The variant NAME of a declared `EnchantTarget`, for r3's per-variant table.
fn variant_name(et: &EnchantTarget) -> &'static str {
    match et {
        EnchantTarget::Creature => "Creature",
        EnchantTarget::Permanent => "Permanent",
        EnchantTarget::Artifact => "Artifact",
        EnchantTarget::Enchantment => "Enchantment",
        EnchantTarget::Land => "Land",
        EnchantTarget::Planeswalker => "Planeswalker",
        EnchantTarget::Player => "Player",
        EnchantTarget::CreatureOrPlaneswalker => "CreatureOrPlaneswalker",
        EnchantTarget::Filtered(_) => "Filtered",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The walk
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Row {
    name: String,
    complete: bool,
    /// `None` = the def declares no Enchant keyword.
    declared: Option<EnchantTarget>,
    /// `None` = no face prints an `"Enchant "` line.
    printed_line: Option<String>,
    printed_face: &'static str,
    /// `Some(Ok(spec))` / `Some(Err(unclassifiable token))` / `None` (nothing printed).
    parsed: Option<Result<EnchantSpec, String>>,
}

impl Row {
    fn declared_spec(&self) -> Option<EnchantSpec> {
        self.declared.as_ref().map(declared_to_spec)
    }
    /// A genuine declared-vs-printed disagreement: both sides exist, the line parses, and the
    /// two normalized specs differ.
    fn is_mismatch(&self) -> bool {
        match (self.declared_spec(), self.parsed.as_ref()) {
            (Some(d), Some(Ok(p))) => d != *p,
            _ => false,
        }
    }
}

fn enchant_rows() -> Vec<Row> {
    let mut out: Vec<Row> = Vec::new();
    for def in all_cards().iter() {
        let declared = declared_enchant_targets(def);
        let printed = printed_enchant_lines(def);
        if declared.is_empty() && printed.is_empty() {
            continue;
        }
        let (printed_face, printed_line) = match printed.first() {
            Some((face, line)) => (*face, Some(line.clone())),
            None => ("-", None),
        };
        out.push(Row {
            name: def.name.clone(),
            complete: is_effectively_complete(def),
            // CR 702.5c: a permanent with multiple Enchant abilities is not modelled — the
            // engine reads only the FIRST (`sba::get_enchant_target`), and this walk mirrors
            // that deliberately rather than inventing a semantics the engine does not have.
            // `r4` pins that no def declares more than one.
            declared: declared.first().map(|(_, et)| et.clone()),
            printed_line: printed_line.clone(),
            printed_face,
            parsed: printed_line.as_deref().map(parse_printed_enchant_line),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// r1 — the census, PRINTED
// ─────────────────────────────────────────────────────────────────────────────

/// The printed Enchant lines this parser deliberately does NOT classify, each with the reason
/// it cannot be expressed as an `EnchantFilter` — and each reason is **checked** by `r1`, not
/// merely written down. An allowlist entry whose reason is not itself asserted is a comment.
///
/// `(card name, printed line after "Enchant ", short reason)`.
const UNPARSEABLE_ALLOWLIST: &[(&str, &str, &str)] = &[
    (
        "Animate Dead",
        "creature card in a graveyard",
        "CR 702.5a with a ZONE restriction. `EnchantFilter` has no zone field at all, and the \
         Aura attaches to a card in a GRAVEYARD (CR 303.4a's battlefield conjunct is wrong for \
         it). Nothing in this batch's scope can express it.",
    ),
    (
        "Curse of Opulence",
        "player",
        "CR 702.5d — `EnchantTarget::Player` EXISTS on the enum; what does not exist is the \
         attachment path (`GameObject.attached_to` has no player variant) and `sba.rs` rejects \
         it. `OOS-DX20-2`.",
    ),
];

#[test]
/// CR 702.5a / 205.4a — **the census.** Every def on either axis, its printed Enchant line
/// parsed, and its declared `EnchantTarget` normalized; the mismatch set must be EXACTLY empty.
///
/// This is the row `OOS-DX20-10` and `OOS-DX20-5` close on. Before PB-DX20b it had three
/// members — `imprisoned_in_the_moon` (printed three classes, declared `Permanent`),
/// `kayas_ghostform` (printed two classes plus "you control", declared `Creature`) and
/// `breath_of_fury` (printed "you control", declared `Creature`) — the third of which no seed
/// row and no v4 memo cell names.
fn r1_printed_and_declared_enchant_lines_agree() {
    let rows = enchant_rows();

    // ── The normalization's own precondition, asserted rather than assumed.
    for row in &rows {
        if let Some(EnchantTarget::Filtered(f)) = row.declared.as_ref() {
            assert!(
                f.has_card_type.is_none() || f.has_card_types.is_empty(),
                "PB-DX20b r1: `{}` sets BOTH `has_card_type` (an AND conjunct) and \
                 `has_card_types` (an OR set). They are independent fields and both must then \
                 hold, so `filter_to_spec`'s union stops being an exact normalization and this \
                 census stops measuring what it claims. Express the line with exactly one of \
                 the two, or teach `filter_to_spec` the AND.",
                row.name
            );
            assert!(
                f.has_subtype.is_none() || f.has_subtypes.is_empty(),
                "PB-DX20b r1: `{}` sets BOTH `has_subtype` and `has_subtypes`; same reason as \
                 the card-type conjuncts above.",
                row.name
            );
        }
    }

    // ── The residuals: a printed line the parser cannot classify must be allowlisted.
    let allow_names: BTreeSet<&str> = UNPARSEABLE_ALLOWLIST.iter().map(|(n, _, _)| *n).collect();
    let unparsed: Vec<(String, String, String)> = rows
        .iter()
        .filter_map(|r| match r.parsed.as_ref() {
            Some(Err(tok)) => Some((
                r.name.clone(),
                r.printed_line.clone().unwrap_or_default(),
                tok.clone(),
            )),
            _ => None,
        })
        .collect();
    let unparsed_names: BTreeSet<&str> = unparsed.iter().map(|(n, _, _)| n.as_str()).collect();
    // `Curse of Opulence` parses (it is the `player` branch), so the allowlist is a SUPERSET of
    // the unparseable set; the assertion is containment, and the extra member is re-checked
    // below on the property that actually justifies it.
    let unlisted: Vec<&&str> = unparsed_names.difference(&allow_names).collect();
    assert!(
        unlisted.is_empty(),
        "PB-DX20b r1: printed Enchant line(s) this parser cannot classify and that are not \
         allowlisted: {:?}. Full detail (card, line, offending token): {:?}. Either teach the \
         parser the construct, or add an allowlist row STATING WHY `EnchantFilter` cannot \
         express it.",
        unlisted,
        unparsed
    );

    // ── Every allowlist row's REASON is checked, not merely written. Both residuals are
    //    residuals *because the def cannot be authored*, and the observable consequence of
    //    that is the same in both cases: the def declares no Enchant keyword at all, and is
    //    therefore not deck-legal.
    for (name, line, reason) in UNPARSEABLE_ALLOWLIST {
        let row = rows.iter().find(|r| r.name == *name).unwrap_or_else(|| {
            panic!(
                "PB-DX20b r1: allowlisted def `{}` is not in the corpus; \
                 an allowlist entry for a def that no longer exists is dead weight",
                name
            )
        });
        assert_eq!(
            row.printed_line.as_deref(),
            Some(*line),
            "PB-DX20b r1: allowlisted def `{}` no longer prints the line the allowlist is \
             about. Reason on file: {}",
            name,
            reason
        );
        assert!(
            row.declared.is_none(),
            "PB-DX20b r1: `{}` is allowlisted as INEXPRESSIBLE ({}), yet it now DECLARES an \
             EnchantTarget ({:?}). If the construct became expressible, delete the allowlist \
             row and let the census compare the two sides.",
            name,
            reason,
            row.declared
        );
        assert!(
            !row.complete,
            "PB-DX20b r1: `{}` is allowlisted as inexpressible and declares no Enchant \
             restriction, so it must NOT be deck-legal `Complete` — a Complete Aura with no \
             enforceable Enchant line is `OOS-DX20-10`'s shape one level worse. Reason on \
             file: {}",
            name, reason
        );
    }

    // ── The census itself.
    let mismatches: Vec<String> = rows
        .iter()
        .filter(|r| r.is_mismatch())
        .map(|r| {
            format!(
                "{} [{}]: printed \"Enchant {}\" -> {} BUT declared {:?} -> {}",
                r.name,
                if r.complete { "Complete" } else { "not-legal" },
                r.printed_line.clone().unwrap_or_default(),
                r.parsed
                    .as_ref()
                    .and_then(|p| p.as_ref().ok())
                    .map(EnchantSpec::render)
                    .unwrap_or_else(|| "?".into()),
                r.declared,
                r.declared_spec()
                    .map(|s| s.render())
                    .unwrap_or_else(|| "?".into()),
            )
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "CR 702.5a (`OOS-DX20-10` / `OOS-DX20-5`): {} def(s) declare an EnchantTarget that \
         does not say what their printed Enchant line says:\n  {}",
        mismatches.len(),
        mismatches.join("\n  ")
    );
}

#[test]
/// Non-vacuity floor for `r1`'s parser. A parser that returned `Err` for everything, or
/// `EnchantSpec::default()` for everything, would make `r1`'s mismatch set empty by
/// construction — the whole census would be green and measure nothing.
///
/// The four cases below are the four grammar branches (`basic ` supertype, the `", or "`
/// disjunction, the trailing controller clause, and the CR 205.3i basic-land-type implication),
/// each asserted against a literal expectation rather than against another parse.
fn r1b_the_printed_line_parser_discriminates() {
    let spec = |s: &str| parse_printed_enchant_line(s).expect("parses");

    assert_eq!(
        spec("creature").render(),
        "types=Creature subtypes=- basic=false nonbasic=false controller=Any"
    );
    assert_eq!(
        spec("creature, land, or planeswalker").render(),
        "types=Creature|Land|Planeswalker subtypes=- basic=false nonbasic=false controller=Any"
    );
    assert_eq!(
        spec("creature or planeswalker you control").render(),
        "types=Creature|Planeswalker subtypes=- basic=false nonbasic=false controller=You"
    );
    assert_eq!(
        spec("basic land you control").render(),
        "types=Land subtypes=- basic=true nonbasic=false controller=You"
    );
    assert_eq!(
        spec("nonbasic land").render(),
        "types=Land subtypes=- basic=false nonbasic=true controller=Any"
    );
    // CR 205.3i / 305.6 — a printed basic land type implies the Land card type.
    assert_eq!(
        spec("Mountain you control").render(),
        "types=Land subtypes=Mountain basic=false nonbasic=false controller=You"
    );
    // …and the implication does not fire for a non-land subtype, which would be a silent
    // widening if it did.
    assert_eq!(
        spec("Aura").render(),
        "types=- subtypes=Aura basic=false nonbasic=false controller=Any"
    );
    assert_eq!(
        spec("permanent").render(),
        "types=- subtypes=- basic=false nonbasic=false controller=Any"
    );
    assert!(
        spec("player").player,
        "CR 702.5d: \"Enchant player\" is the player branch"
    );

    // The UNCLASSIFIED branch really refuses: a lowercase token that is not a class word must
    // NOT be silently accepted as a subtype.
    assert_eq!(
        parse_printed_enchant_line("creature card in a graveyard"),
        Err("creature card in a graveyard".to_string()),
        "the parser must REFUSE an unclassifiable lowercase token rather than mint a phantom \
         SubType — that refusal is what makes `animate_dead` a reported residual"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r2 — the INVERSE axis: lines needing an OR over classes, or a controller clause
// ─────────────────────────────────────────────────────────────────────────────

/// Every def whose printed Enchant line needs expressiveness beyond a bare
/// `EnchantTarget::<Type>` variant — more than one class, a controller clause, or a supertype
/// conjunct.
///
/// **Pinned by name. SEVEN, not the six the brief for this row predicted**, and the extra one
/// is the reason this axis is keyed on the parse rather than on a `" or "` / `" you control"`
/// substring: `awaken_the_ancient` prints *"Enchant Mountain"*, which has neither, and still
/// cannot be declared as any bare variant — it needs
/// `EnchantFilter { has_card_type: Land, has_subtype: Mountain }`. A substring axis would have
/// pinned six and called the population measured.
const NEEDS_FILTER_DEFS: &[&str] = &[
    "Awaken the Ancient",
    "Breath of Fury",
    "Chained to the Rocks",
    "Dimensional Exile",
    "Imprisoned in the Moon",
    "Kaya's Ghostform",
    "Ossification",
];

/// The narrower axis the seed rows are actually about: printed lines carrying an **OR over
/// classes** or a **controller clause**. Six members — [`NEEDS_FILTER_DEFS`] minus the
/// subtype-only `awaken_the_ancient`.
///
/// Both are pinned because they are different claims. This one is the population `OOS-DX20-10`
/// and `OOS-DX20-5` live in; the wider one is the population `EnchantFilter` exists for.
const NEEDS_OR_OR_CONTROLLER_DEFS: &[&str] = &[
    "Breath of Fury",
    "Chained to the Rocks",
    "Dimensional Exile",
    "Imprisoned in the Moon",
    "Kaya's Ghostform",
    "Ossification",
];

/// The subset of [`NEEDS_FILTER_DEFS`] whose printed line needs the **OR over card types** that
/// PB-DX20b added. Two members; both were previously mis-declared.
const NEEDS_CARD_TYPE_OR_DEFS: &[&str] = &["Imprisoned in the Moon", "Kaya's Ghostform"];

#[test]
/// CR 702.5a — **the inverse axis.** Membership is decided by the PARSED MECHANISM (more than
/// one class, a controller constraint, or a supertype conjunct), never by a `" or "` substring.
///
/// A substring axis measures the substring: `" or "` also appears in *"target creature or
/// player"* and in every reminder-text disjunction, and it would miss a line that spelled its
/// disjunction with commas alone. Keying on the parse is `OOS-DX47`'s repair applied here
/// before the defeat rather than after it.
fn r2_lines_needing_a_filter_are_pinned_and_declare_one() {
    let rows = enchant_rows();

    let needs_filter: BTreeSet<String> = rows
        .iter()
        .filter(|r| {
            r.parsed
                .as_ref()
                .and_then(|p| p.as_ref().ok())
                .is_some_and(|s| {
                    s.types.len() + s.subtypes.len() > 1
                        || s.controller != "Any"
                        || s.basic
                        || s.nonbasic
                })
        })
        .map(|r| r.name.clone())
        .collect();
    let pinned: BTreeSet<String> = NEEDS_FILTER_DEFS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        needs_filter,
        pinned,
        "PB-DX20b r2: the population of printed Enchant lines needing more than a bare \
         EnchantTarget variant moved. live only: {:?}; pinned only: {:?}",
        needs_filter.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&needs_filter).collect::<Vec<_>>()
    );

    // The narrower axis the two seed rows are about, pinned separately so the wide axis's
    // extra member cannot mask a move in the one the seeds name.
    let needs_or_or_controller: BTreeSet<String> = rows
        .iter()
        .filter(|r| {
            r.parsed
                .as_ref()
                .and_then(|p| p.as_ref().ok())
                .is_some_and(|s| s.types.len() > 1 || s.controller != "Any")
        })
        .map(|r| r.name.clone())
        .collect();
    let pinned_narrow: BTreeSet<String> = NEEDS_OR_OR_CONTROLLER_DEFS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        needs_or_or_controller,
        pinned_narrow,
        "PB-DX20b r2: the population of printed Enchant lines carrying an OR over classes or a \
         controller clause moved. live only: {:?}; pinned only: {:?}",
        needs_or_or_controller
            .difference(&pinned_narrow)
            .collect::<Vec<_>>(),
        pinned_narrow
            .difference(&needs_or_or_controller)
            .collect::<Vec<_>>()
    );
    assert!(
        needs_or_or_controller.is_subset(&needs_filter),
        "PB-DX20b r2: the narrow axis must nest inside the wide one; if it does not, one of \
         the two predicates is wrong"
    );

    // Every member must DECLARE a Filtered filter that expresses its printed line exactly.
    // `r1` already asserts the two specs are equal; this asserts the VARIANT, because a bare
    // variant that happened to normalize equal (there is no such pair today, and there could
    // be one tomorrow) would still be an inexpressible declaration.
    for name in NEEDS_FILTER_DEFS {
        let row = rows
            .iter()
            .find(|r| r.name == *name)
            .unwrap_or_else(|| panic!("PB-DX20b r2: `{}` left the corpus", name));
        assert!(
            matches!(row.declared, Some(EnchantTarget::Filtered(_))),
            "CR 702.5a: `{}` prints \"Enchant {}\", which needs an EnchantFilter; it declares \
             {:?}",
            name,
            row.printed_line.clone().unwrap_or_default(),
            row.declared
        );
    }

    // The `has_card_types` half — PB-DX20b's own new field — pinned separately, so a repair
    // that expressed the disjunction some other way (or dropped it) is a finding.
    let uses_card_type_or: BTreeSet<String> = rows
        .iter()
        .filter(|r| match r.declared.as_ref() {
            Some(EnchantTarget::Filtered(f)) => !f.has_card_types.is_empty(),
            _ => false,
        })
        .map(|r| r.name.clone())
        .collect();
    let pinned_or: BTreeSet<String> = NEEDS_CARD_TYPE_OR_DEFS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        uses_card_type_or,
        pinned_or,
        "PB-DX20b r2: the `EnchantFilter::has_card_types` user set moved. This field is the \
         whole of PB-DX20b's engine change; a member leaving it means a printed OR over card \
         types was expressed some other way, or dropped. live only: {:?}; pinned only: {:?}",
        uses_card_type_or.difference(&pinned_or).collect::<Vec<_>>(),
        pinned_or.difference(&uses_card_type_or).collect::<Vec<_>>()
    );

    // The exact card-type sets, so "declares SOME has_card_types" cannot pass for "declares the
    // printed classes". `r1` compares normalized specs; this reads the field itself.
    let expected: BTreeMap<&str, Vec<CardType>> = [
        (
            "Imprisoned in the Moon",
            vec![CardType::Creature, CardType::Land, CardType::Planeswalker],
        ),
        (
            "Kaya's Ghostform",
            vec![CardType::Creature, CardType::Planeswalker],
        ),
    ]
    .into_iter()
    .collect();
    for (name, want) in expected {
        let row = rows.iter().find(|r| r.name == name).expect("in corpus");
        let Some(EnchantTarget::Filtered(f)) = row.declared.as_ref() else {
            panic!("PB-DX20b r2: `{}` must declare Filtered", name);
        };
        assert_eq!(
            f.has_card_types,
            want,
            "CR 702.5a: `{}` prints \"Enchant {}\"",
            name,
            row.printed_line.clone().unwrap_or_default()
        );
        assert_eq!(
            f.controller,
            if name == "Kaya's Ghostform" {
                EnchantControllerConstraint::You
            } else {
                EnchantControllerConstraint::Any
            },
            "CR 702.5a: `{}`'s printed controller clause",
            name
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// r3 — the variant sweep
// ─────────────────────────────────────────────────────────────────────────────

/// Every `EnchantTarget` variant, with the corpus population it reaches.
///
/// `Permanent`'s **zero** is `OOS-DX20-10`'s closure restated on the variant axis: PB-DX20
/// pinned it wrong-way-round at `{"Imprisoned in the Moon"}` and named itself the assertion to
/// invert. The variant is deliberately not deleted from the enum — a card really can print
/// *"Enchant permanent"* — so a **zero** here, rather than a missing row, is the honest shape.
/// `(variant, every declaring def, the DECK-LEGAL `Complete` subset)`.
///
/// The deck-legal column is pinned separately rather than derived, because it is the column
/// every behavioural claim in this batch rests on: a `partial` def cannot enter a validated
/// deck, so a mis-declared Enchant line on one is latent, and a mis-declared line on a
/// `Complete` one is live in the shipped browser game. `imprisoned_in_the_moon` was the second
/// kind, which is why `OOS-DX20-10` was filed HIGH.
///
/// The ROW KEYS (not the populations) are pinned to `pub enum EnchantTarget`'s own
/// declaration at the end of `r3` — `OOS-DX28-1`. Before PB-DX57 the only check was
/// *"every variant reached by the corpus has a row"*, which six of the nine variants satisfy
/// vacuously.
const VARIANT_POPULATION: &[(&str, &[&str], &[&str])] = &[
    (
        "Creature",
        &[
            "Aqueous Form",
            "Bear Umbra",
            "Crown of Skemfar",
            "Curiosity",
            "Darksteel Mutation",
            "Eaten by Piranhas",
            "Hyena Umbra",
            "Kasmina's Transmutation",
            "Kenrith's Transformation",
            "Ophidian Eye",
            "Rancor",
            "Shiny Impetus",
            "Sigil of Sleep",
            "Smoke Shroud",
        ],
        &[
            "Darksteel Mutation",
            "Eaten by Piranhas",
            "Hyena Umbra",
            "Kasmina's Transmutation",
            "Kenrith's Transformation",
            "Rancor",
            "Sigil of Sleep",
        ],
    ),
    (
        "Land",
        &["Elvish Guidance", "Wild Growth"],
        &["Wild Growth"],
    ),
    (
        "Filtered",
        &[
            "Awaken the Ancient",
            "Breath of Fury",
            "Chained to the Rocks",
            "Dimensional Exile",
            "Imprisoned in the Moon",
            "Kaya's Ghostform",
            "Ossification",
        ],
        &[
            "Awaken the Ancient",
            "Chained to the Rocks",
            "Dimensional Exile",
            "Imprisoned in the Moon",
            "Ossification",
        ],
    ),
    // `Permanent`'s zero is `OOS-DX20-10`'s closure on the variant axis. The other five zeros
    // are variants the corpus has simply never reached; they are rows rather than omissions so
    // a first member arrives as a red test rather than as a silent pass.
    ("Permanent", &[], &[]),
    ("Artifact", &[], &[]),
    ("Enchantment", &[], &[]),
    ("Planeswalker", &[], &[]),
    ("Player", &[], &[]),
    ("CreatureOrPlaneswalker", &[], &[]),
];

#[test]
/// CR 702.5a — **the variant sweep.** For every `EnchantTarget` variant: which defs declare it,
/// which of those are deck-legal `Complete`, and — for every one of them — that the declared
/// variant agrees with the printed line.
///
/// The agreement half is `r1`'s assertion re-run per variant, and it is not redundant: `r1`
/// reports a flat set, so a whole variant regressing at once reads as "N mismatches" there and
/// as "this variant is broken" here. The failure message is the deliverable.
fn r3_every_enchant_target_variant_agrees_with_its_printed_lines() {
    let rows = enchant_rows();

    let mut live: BTreeMap<&'static str, BTreeSet<String>> = BTreeMap::new();
    for row in &rows {
        if let Some(et) = row.declared.as_ref() {
            live.entry(variant_name(et))
                .or_default()
                .insert(row.name.clone());
        }
    }

    let mut live_legal: BTreeMap<&'static str, BTreeSet<String>> = BTreeMap::new();
    for row in rows.iter().filter(|r| r.complete) {
        if let Some(et) = row.declared.as_ref() {
            live_legal
                .entry(variant_name(et))
                .or_default()
                .insert(row.name.clone());
        }
    }

    for (variant, pinned_members, pinned_legal_members) in VARIANT_POPULATION {
        let live_members = live.get(variant).cloned().unwrap_or_default();
        let pinned: BTreeSet<String> = pinned_members.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            live_members,
            pinned,
            "PB-DX20b r3: the `EnchantTarget::{}` population moved. live only: {:?}; pinned \
             only: {:?}",
            variant,
            live_members.difference(&pinned).collect::<Vec<_>>(),
            pinned.difference(&live_members).collect::<Vec<_>>()
        );

        let live_legal_members = live_legal.get(variant).cloned().unwrap_or_default();
        let pinned_legal: BTreeSet<String> =
            pinned_legal_members.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            live_legal_members,
            pinned_legal,
            "PB-DX20b r3: the DECK-LEGAL `Complete` subset of `EnchantTarget::{}` moved. This \
             is the column every behavioural claim rests on — a marker flip here changes what \
             a validated deck can contain. live only: {:?}; pinned only: {:?}",
            variant,
            live_legal_members
                .difference(&pinned_legal)
                .collect::<Vec<_>>(),
            pinned_legal
                .difference(&live_legal_members)
                .collect::<Vec<_>>()
        );
        assert!(
            pinned_legal.is_subset(&pinned),
            "PB-DX20b r3: the pinned deck-legal subset for `EnchantTarget::{}` is not a subset \
             of the pinned population — one of the two lists is wrong",
            variant
        );

        // Per-variant declared-vs-printed agreement, with the deck-legal members named
        // separately in the message so a regression's blast radius is legible at the failure.
        let broken: Vec<String> = rows
            .iter()
            .filter(|r| r.declared.as_ref().map(variant_name) == Some(*variant) && r.is_mismatch())
            .map(|r| {
                format!(
                    "{} ({})",
                    r.name,
                    if r.complete {
                        "DECK-LEGAL"
                    } else {
                        "not deck-legal"
                    }
                )
            })
            .collect();
        assert!(
            broken.is_empty(),
            "CR 702.5a: EnchantTarget::{} is declared by def(s) whose printed line says \
             something else: {:?}",
            variant,
            broken
        );
    }

    // No variant may go unclassified — a tenth variant added to the enum must be given a row
    // here (with an empty population if the corpus does not reach it) rather than silently
    // escaping the sweep.
    //
    // `OOS-DX28-1`: that sentence is what this file has always CLAIMED, and until PB-DX57 the
    // check below only asserted the weaker *"every variant REACHED BY THE CORPUS has a row"* —
    // so a tenth variant with zero corpus reach got no row and nothing fired. The two are the
    // same assertion only while every variant is reached, and six of the nine are reached by
    // nothing. `VARIANT_POPULATION` is now compared to `pub enum EnchantTarget`'s own
    // declaration, which is what makes the doc's claim true rather than aspirational.
    //
    // **The pre-existing mitigation is real and is preserved, not replaced.** `variant_name`
    // above is an exhaustive `match` over `EnchantTarget` with no wildcard arm, so a tenth
    // variant is already a COMPILE error there. What was missing is not detection of the
    // enum's growth — it is detection by THIS list, which is what decides whether the new
    // variant's corpus population is swept at all. A future edit that gives `variant_name` a
    // `_ =>` arm removes the compile-time half and leaves this assertion as the only one.
    //
    // Measured by planting a 10th variant (with the arms needed to make the workspace
    // compile, so the result is a verdict and not a build failure): this assertion goes RED
    // naming the variant, the corpus-reach check below stays GREEN, and the only other reds
    // are the two wire fingerprint gates -- which say "the wire moved", not "the sweep is
    // one variant short".
    let classified: BTreeSet<&str> = VARIANT_POPULATION.iter().map(|(v, _, _)| *v).collect();
    let declared: BTreeSet<String> = crate::pb_dx57_declared_source::declared_enum_variants(
        crate::pb_dx57_declared_source::STATE_TYPES_RS,
        "EnchantTarget",
    );
    let classified_owned: BTreeSet<String> = classified.iter().map(|v| (*v).to_string()).collect();
    assert_eq!(
        classified_owned,
        declared,
        "`OOS-DX28-1` / PB-DX20b r3: VARIANT_POPULATION is no longer exactly \
         `pub enum EnchantTarget`'s declared variant set.\n  DECLARED but unrowed: {:?} — \
         give it a row (with an empty population if the corpus does not reach it) rather \
         than letting it escape the sweep.\n  ROWED but undeclared: {:?} — a renamed or \
         deleted variant, whose row now pins a population that can never be reached and \
         therefore reports a permanent, silent zero.",
        declared.difference(&classified_owned).collect::<Vec<_>>(),
        classified_owned.difference(&declared).collect::<Vec<_>>()
    );

    // Kept beside the equality above rather than deleted: it is the assertion whose FAILURE
    // MESSAGE names the offending variant in terms of the corpus, which is what a reader
    // debugging a real regression wants. It is now implied by the equality, and it costs one
    // line to say so directly.
    let unclassified: Vec<&&str> = live.keys().filter(|v| !classified.contains(*v)).collect();
    assert!(
        unclassified.is_empty(),
        "PB-DX20b r3: EnchantTarget variant(s) reached by the corpus with no row in \
         VARIANT_POPULATION: {:?}",
        unclassified
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r4 — non-vacuity floors
// ─────────────────────────────────────────────────────────────────────────────

/// Every def declaring an Enchant keyword (axis a). **23.**
const DECLARING_COUNT: usize = 23;
/// Every def printing an `"Enchant "` line (axis b). **25.** The two extras are `r1`'s
/// allowlisted residuals.
const PRINTING_COUNT: usize = 25;
/// Defs declaring `EnchantTarget::Filtered`. **7.**
const FILTERED_COUNT: usize = 7;
/// Deck-legal `Complete` defs declaring an Enchant keyword. **13** of 23.
const DECLARING_DECK_LEGAL_COUNT: usize = 13;
/// Deck-legal `Complete` defs declaring `EnchantTarget::Filtered`. **5.**
const FILTERED_DECK_LEGAL_COUNT: usize = 5;

#[test]
/// CR 702.5a — **non-vacuity floors.** Every population above is pinned as a named count, so a
/// corpus move surfaces as a finding rather than as a silently re-tuned constant.
///
/// The exact-equality form is deliberate. A `>=` floor would let the census shrink to nothing
/// while every set assertion in `r1`/`r2`/`r3` stayed vacuously green — which is the shape
/// PB-DX8's population ratchet failed on, and PB-DX45's `MOVED_MSG` failed on again.
fn r4_populations_are_pinned_by_count() {
    let rows = enchant_rows();

    let declaring: Vec<&Row> = rows.iter().filter(|r| r.declared.is_some()).collect();
    let printing: Vec<&Row> = rows.iter().filter(|r| r.printed_line.is_some()).collect();
    let filtered: Vec<&Row> = rows
        .iter()
        .filter(|r| matches!(r.declared, Some(EnchantTarget::Filtered(_))))
        .collect();

    assert_eq!(
        declaring.len(),
        DECLARING_COUNT,
        "PB-DX20b r4: the Enchant-declaring population moved: {:?}",
        declaring.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    assert_eq!(
        printing.len(),
        PRINTING_COUNT,
        "PB-DX20b r4: the printed-Enchant-line population moved: {:?}",
        printing.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    assert_eq!(
        filtered.len(),
        FILTERED_COUNT,
        "PB-DX20b r4: the EnchantTarget::Filtered population moved: {:?}",
        filtered.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    assert_eq!(
        declaring.iter().filter(|r| r.complete).count(),
        DECLARING_DECK_LEGAL_COUNT,
        "PB-DX20b r4: the DECK-LEGAL Enchant-declaring subset moved: {:?}",
        declaring
            .iter()
            .filter(|r| r.complete)
            .map(|r| &r.name)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        filtered.iter().filter(|r| r.complete).count(),
        FILTERED_DECK_LEGAL_COUNT,
        "PB-DX20b r4: the DECK-LEGAL EnchantTarget::Filtered subset moved: {:?}",
        filtered
            .iter()
            .filter(|r| r.complete)
            .map(|r| &r.name)
            .collect::<Vec<_>>()
    );

    // The two axes DO NOT NEST, stated as an executed fact rather than as prose in the module
    // doc. `declared \ printed` is empty today and `printed \ declared` is the two residuals;
    // if the first ever becomes non-empty the module doc's claim needs its own evidence.
    let declared_only: Vec<&str> = rows
        .iter()
        .filter(|r| r.declared.is_some() && r.printed_line.is_none())
        .map(|r| r.name.as_str())
        .collect();
    let printed_only: Vec<&str> = rows
        .iter()
        .filter(|r| r.declared.is_none() && r.printed_line.is_some())
        .map(|r| r.name.as_str())
        .collect();
    assert!(
        declared_only.is_empty(),
        "PB-DX20b r4: def(s) declare an Enchant keyword while printing no \"Enchant \" line: \
         {:?}. That is a legal shape (the axes do not nest) but it means the printed axis can \
         no longer audit them, so it must be recorded here rather than discovered later.",
        declared_only
    );
    assert_eq!(
        printed_only,
        vec!["Animate Dead", "Curse of Opulence"],
        "PB-DX20b r4: the printed-only population is exactly `r1`'s two allowlisted residuals"
    );

    // CR 702.5c: `sba::get_enchant_target` reads only the FIRST Enchant keyword, and this
    // file's walk mirrors that. The mirror is only faithful while no def declares two.
    for def in all_cards().iter() {
        let n = declared_enchant_targets(def).len();
        assert!(
            n <= 1,
            "CR 702.5c: `{}` declares {} Enchant keywords. The engine reads only the first \
             (`sba::get_enchant_target`) and so does this census — a second one is silently \
             unenforced, and this row is where that must be noticed.",
            def.name,
            n
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// r5 — the STRUCTURAL gate: every EnchantFilter field must be lowered
// ─────────────────────────────────────────────────────────────────────────────

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/engine -> crates -> workspace root
    p.pop();
    p.pop();
    p
}

/// The matching `}` for the `{` at `open`.
fn matching_brace(src: &str, open: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    for (i, b) in bytes.iter().enumerate().skip(open) {
        match b {
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
    None
}

/// Every field name parsed out of `pub struct EnchantFilter`'s own declaration in
/// `crates/card-types/src/state/types.rs`.
///
/// ## Every `pub ` in the body, not one per LINE — and the first draft took one per line
///
/// `/review` finding 3, proved by planting `pub nonbasic: bool, pub sneaky_zone: bool,` on a
/// single line: a line-oriented parser takes the FIRST `pub ` and stops, so the second field is
/// declared, serialized (`#[serde(default)]` is inherited from the line's attribute), hashed and
/// stored, is never lowered — and `r5` stays GREEN, because the field it cannot see is absent
/// from BOTH sides of its subset check. That is `r5`'s own subject matter committed inside
/// `r5`'s own parser.
///
/// **The mitigation is stated rather than the hole being overclaimed**: `cargo fmt --check`
/// rejects two struct fields on one line, so the defect is only reachable in an unformatted
/// tree — a state this repository's gates already refuse. It is repaired anyway, because a
/// gate that is correct only because a *different* gate is green is a gate whose reach nobody
/// has measured, and because the repair costs four lines.
///
/// The scan mirrors [`lowered_enchant_filter_fields`]'s needle discipline: a hit only counts
/// when the byte before it is not an identifier character, so `xpub ` / `_pub ` cannot mint a
/// phantom field.
fn declared_enchant_filter_fields() -> BTreeSet<String> {
    let path = workspace_root().join("crates/card-types/src/state/types.rs");
    let raw = std::fs::read_to_string(&path).expect("types.rs is readable");
    let src = strip_comments(&raw);
    let decl = src
        .find("pub struct EnchantFilter {")
        .expect("`pub struct EnchantFilter` is declared in card-types/src/state/types.rs");
    let open = src[decl..]
        .find('{')
        .map(|r| decl + r)
        .expect("the struct has a body");
    let end = matching_brace(&src, open).expect("the struct body is balanced");
    let body = &src[open + 1..end];

    let mut out = BTreeSet::new();
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = body[i..].find("pub ") {
        let at = i + rel;
        // Reject `xpub ` / `_pub ` — the keyword must stand on its own.
        let standalone = at == 0 || {
            let prev = bytes[at - 1];
            !(prev.is_ascii_alphanumeric() || prev == b'_')
        };
        if standalone {
            let name: String = body[at + 4..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.insert(name);
            }
        }
        i = at + 4;
    }
    out
}

/// Every `f.<field>` read inside `casting::enchant_filter_to_target_filter`'s body.
fn lowered_enchant_filter_fields() -> BTreeSet<String> {
    let path = workspace_root().join("crates/engine/src/rules/casting.rs");
    let raw = std::fs::read_to_string(&path).expect("casting.rs is readable");
    let src = strip_comments(&raw);
    let sig = "fn enchant_filter_to_target_filter(";
    let decl = src.find(sig).unwrap_or_else(|| {
        panic!(
            "PB-DX20b r5: `{}` is not in crates/engine/src/rules/casting.rs. It is the ONE \
             lowering of an EnchantFilter onto a TargetFilter; if it was renamed, this gate \
             must be repointed rather than deleted.",
            sig
        )
    });
    let open = src[decl..]
        .find('{')
        .map(|r| decl + r)
        .expect("the function has a body");
    let end = matching_brace(&src, open).expect("the function body is balanced");
    let body = &src[open..=end];

    let mut out = BTreeSet::new();
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = body[i..].find("f.") {
        let at = i + rel;
        // Reject `self.f.` / `xf.` / `_f.` — the read must be of the binding `f` itself, which
        // means the byte before it is not an identifier char.
        let ok = at == 0 || {
            let prev = bytes[at - 1];
            !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.')
        };
        if ok {
            let name: String = body[at + 2..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.insert(name);
            }
        }
        i = at + 2;
    }
    out
}

/// The seven fields `EnchantFilter` carries at HEAD. A non-vacuity floor for BOTH parsers: a
/// parse that silently returned `{}` would make the equality below trivially true.
const KNOWN_ENCHANT_FILTER_FIELDS: &[&str] = &[
    "basic",
    "controller",
    "has_card_type",
    "has_card_types",
    "has_subtype",
    "has_subtypes",
    "nonbasic",
];

#[test]
/// CR 702.5a — **the structural gate.** Every field declared on `EnchantFilter` must be READ by
/// `casting::enchant_filter_to_target_filter`, which PB-DX20b made the single place in the
/// engine that knows what an `EnchantFilter` field means.
///
/// ## Why this cannot be left to the compiler — measured, not assumed
///
/// The stage-1 runner reported that adding `has_card_types` to `EnchantFilter` produced **ZERO**
/// compile errors anywhere in the workspace, because every construction site in the corpus and
/// in the test tree spells the struct with `..Default::default()`. So an eighth field can be
/// added, serialized, hashed and stored, and simply never reach a `TargetFilter` — the cast
/// path, the offer layer and the CR 704.5m SBA would all ignore it, in silence, on a green
/// build. `OOS-DX28-1` recommends exactly this repair and PB-DX43 applied its shape to
/// `TOKEN_SPEC_FIELDS`.
///
/// The gate is one-directional on purpose: it asserts *declared ⊆ lowered*. A `TargetFilter`
/// field the lowering sets from a constant (there are none today) is not an `EnchantFilter`
/// field and is none of this row's business; `..Default::default()` is PB-DX20 §3.4's
/// deliberate choice and must not be gated into a compile error.
fn r5_every_enchant_filter_field_is_lowered() {
    let declared = declared_enchant_filter_fields();
    let lowered = lowered_enchant_filter_fields();

    // Non-vacuity floors, executed BEFORE the comparison, so an empty parse fails here with a
    // message about the parse rather than there with a message about the engine.
    assert!(
        !declared.is_empty(),
        "PB-DX20b r5: the EnchantFilter declaration parse returned NOTHING — the struct moved, \
         was renamed, or changed shape. An empty declared set makes the subset check below \
         trivially true."
    );
    assert!(
        !lowered.is_empty(),
        "PB-DX20b r5: the enchant_filter_to_target_filter body parse returned NOTHING."
    );
    let known: BTreeSet<String> = KNOWN_ENCHANT_FILTER_FIELDS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        declared,
        known,
        "PB-DX20b r5: EnchantFilter's field list moved. live only: {:?}; pinned only: {:?}. If \
         a field was ADDED, lower it in casting::enchant_filter_to_target_filter (nothing else \
         will tell you — see this test's doc) and add it here.",
        declared.difference(&known).collect::<Vec<_>>(),
        known.difference(&declared).collect::<Vec<_>>()
    );

    let unlowered: Vec<&String> = declared.difference(&lowered).collect();
    assert!(
        unlowered.is_empty(),
        "CR 702.5a / `OOS-DX28-1`: EnchantFilter field(s) declared but never read by \
         `casting::enchant_filter_to_target_filter`: {:?}. That function is the ONLY place in \
         the engine that knows what an EnchantFilter field means — the cast path, \
         `queries::spell_target_requirements` and `sba::enchant_filter_matches` all consume its \
         output — so an unlowered field is silently unenforced on every one of them, and the \
         compiler will NOT catch it (every construction site uses `..Default::default()`). \
         Declared: {:?}; lowered: {:?}",
        unlowered,
        declared,
        lowered
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// The report — PB-DX27's rule: print the population, never transcribe it
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// Prints every population this file pins, under `--nocapture`. Asserts nothing beyond the
/// walk being non-empty: a figure that no test prints is a figure that was transcribed.
fn t_census_report() {
    let rows = enchant_rows();
    assert!(!rows.is_empty(), "the Enchant walk must find something");

    println!(
        "\n=== PB-DX20b — printed Enchant line census ({} defs) ===",
        rows.len()
    );
    println!(
        "{:<32} {:<11} {:<8} {:<42} declared / parsed",
        "def", "legal", "face", "printed line"
    );
    for r in &rows {
        let printed = r
            .printed_line
            .clone()
            .map(|l| format!("Enchant {l}"))
            .unwrap_or_else(|| "(none printed)".into());
        let parsed = match r.parsed.as_ref() {
            Some(Ok(s)) => s.render(),
            Some(Err(tok)) => format!("UNCLASSIFIED token `{tok}`"),
            None => "-".into(),
        };
        let declared = match r.declared_spec() {
            Some(s) => s.render(),
            None => "(no Enchant keyword declared)".into(),
        };
        let flag = if r.is_mismatch() { " <<< MISMATCH" } else { "" };
        println!(
            "{:<32} {:<11} {:<8} {:<42}\n{:>62}declared: {}\n{:>62}parsed:   {}{}",
            r.name,
            if r.complete { "deck-legal" } else { "-" },
            r.printed_face,
            printed,
            "",
            declared,
            "",
            parsed,
            flag
        );
    }

    println!("\n--- r3: EnchantTarget variant populations ---");
    let mut by_variant: BTreeMap<&'static str, Vec<(&str, bool)>> = BTreeMap::new();
    for r in &rows {
        if let Some(et) = r.declared.as_ref() {
            by_variant
                .entry(variant_name(et))
                .or_default()
                .push((r.name.as_str(), r.complete));
        }
    }
    for (variant, _, _) in VARIANT_POPULATION {
        let members = by_variant.get(variant).cloned().unwrap_or_default();
        let legal = members.iter().filter(|(_, c)| *c).count();
        println!(
            "  {:<24} {:>2} defs ({} deck-legal): {:?}",
            variant,
            members.len(),
            legal,
            members.iter().map(|(n, _)| *n).collect::<Vec<_>>()
        );
    }

    println!("\n--- r2: lines needing an EnchantFilter (wide axis) ---");
    for name in NEEDS_FILTER_DEFS {
        let r = rows.iter().find(|r| r.name == *name).expect("in corpus");
        println!(
            "  {:<26} Enchant {}",
            name,
            r.printed_line.clone().unwrap_or_default()
        );
    }

    println!("\n--- r1: allowlisted residuals (printed, inexpressible) ---");
    for (name, line, reason) in UNPARSEABLE_ALLOWLIST {
        println!("  {name}: \"Enchant {line}\"\n      reason: {reason}");
    }

    println!("\n--- r5: EnchantFilter fields ---");
    println!("  declared: {:?}", declared_enchant_filter_fields());
    println!("  lowered:  {:?}", lowered_enchant_filter_fields());
    println!();
}
