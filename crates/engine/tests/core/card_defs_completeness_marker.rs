//! SR-39 gate: every card definition NAMES its `completeness:` marker.
//!
//! ## Numbering
//!
//! Filed as "SR-38" in `scutemob-255`'s acceptance criteria. **SR-38 was already
//! taken** — it is the simulator channel-probe family
//! (`crates/simulator/tests/cc15_raw_characteristics_ratchet.rs`) — so this gate
//! ships as **SR-39**. `docs/engine-invariants.md` records the same correction.
//!
//! ## What this guards
//!
//! `CardDefinition::completeness` is `#[serde(default)]` over a `Completeness`
//! whose `Complete` variant is `#[default]` (`card_definition.rs`). A def that
//! never mentions the field is therefore `Complete` — deck-legal, silently, with
//! no author ever having decided that. At the head this gate was written against,
//! **964 of the 1,140 `Complete` defs were `Complete` by that default**, i.e. 53%
//! of the corpus had a deck-legality verdict nobody wrote down.
//!
//! That is not a theoretical hole. `beast_within.rs` and `generous_gift.rs` were
//! both in the 964: deck-legal, and at a four-player table they hand the 3/3 to
//! the caster instead of the destroyed permanent's controller
//! (`docs/mtg-engine-landscape-assessment.md` §2). Neither carried a TODO, so the
//! `authoring-report.py` TODO scan could not see them either — the corpus's
//! marker discipline is good *where a TODO exists*, and this is the class where
//! one does not.
//!
//! So the marker becomes mandatory. `Completeness::Complete` written out is a
//! claim an author made; an absent marker is a claim nobody made, and the two
//! must stop being indistinguishable. CLAUDE.md already names the guessed marker
//! as "the single most prohibited pattern"; this is the machine half of it.
//!
//! ## Why a source scan rather than a runtime check
//!
//! There is nothing to check at runtime. An unmarked def and a def that spells
//! `completeness: Completeness::Complete` compile to the identical
//! `CardDefinition` — same bytes, same `validate_deck` verdict, same state hash.
//! The difference exists only in the source text, which is where the gate reads
//! it. Same technique as SR-5's keyword registry, SR-8's fingerprints and the
//! SR-12 deviation scan next door in `completeness_deviation_scan.rs`.
//!
//! `deliberateness_is_invisible_at_runtime` below PINS that claim rather than
//! asserting it in prose: it builds both decks and watches `validate_deck` return
//! the same verdict. It is the reason this gate has no behavioural probe to pair
//! with — under the pair-or-demote rule (`memory/conventions.md`) this gate's
//! subject is a property of the source itself, like SR-5 and SR-36, not a proxy
//! for a behaviour. What it *does* ship with is the change-class row-4 executed
//! defeat, twice over: see the two `gate_catches_*` tests.
//!
//! ## The executed defeats (change-class row 4, "one executed defeat")
//!
//! Both run the SAME scanner this file's live gate runs, against a throwaway
//! corpus, so a canary cannot drift from the gate it guards:
//!
//! | fixture | outcome |
//! |---|---|
//! | a def with no `completeness:` line at all | RED — reported by name |
//! | a def whose only `completeness:` is inside a `//` comment | RED — reported by name |
//! | a def that names the marker | GREEN |
//!
//! ### The defeat executed against the LIVE corpus (2026-09-05, `scutemob-255`)
//!
//! The canaries above run the scanner over a throwaway corpus. The gate was also
//! defeated once against the real one, because a canary proves the scanner works
//! and only this proves the gate is WIRED to the corpus people edit. Deleting
//! line 27 of `crates/card-defs/src/defs/sol_ring.rs` (its
//! `completeness: Completeness::Complete,`) and running
//! `cargo test -p mtg-engine --test core card_defs_completeness_marker::every_card_def`
//! produced:
//!
//! ```text
//! test card_defs_completeness_marker::every_card_def_names_its_completeness_marker ... FAILED
//! SR-39: 1 card def(s) do not name `completeness:`. …
//! Unmarked:
//!   sol_ring.rs
//! test result: FAILED. 0 passed; 1 failed; 0 ignored; 838 filtered out
//! ```
//!
//! The line was then restored and the gate went green again. That is the
//! change-class row-4 "one executed defeat", recorded where the rule says to
//! record it — in the gate's own doc.
//!
//! The second fixture is not hypothetical. `misdirection.rs` line 36 reads
//! "*Neither ever affected this def's completeness: no card was …*" — prose, in a
//! comment, containing the exact substring. A gate written as
//! `text.contains("completeness:")` passes that file while it is unmarked. The
//! scanner therefore keys on the FIELD ASSIGNMENT (`completeness:` followed by
//! `Completeness::<Variant>`), byte-for-byte the `MARKER_RE` that
//! `tools/authoring-report.py` uses, so the gate, the report and the campaign
//! headline cannot disagree about what "has a marker" means.
//!
//! ## Recognized spellings
//!
//! Exactly four, mirroring `authoring-report.py`'s `RECOGNIZED_MARKERS`:
//! `Complete` (the variant, capitalized) and the three lowercase note-carrying
//! constructors `inert` / `partial` / `known_wrong`. Any OTHER
//! `Completeness::<X>` is a hard failure here, not a silent pass — the report
//! learned that one the expensive way (a def spelling `Completeness::Partial(…)`
//! directly captured `Partial`, which is absent from its bucket map, and got
//! filed as CLEAN, inflating the headline with a card that declares itself
//! incomplete).

use std::path::{Path, PathBuf};

/// The workspace root: `crates/engine/` is two levels down from it.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

/// The four spellings `tools/authoring-report.py` recognizes. Kept in sync by
/// `the_recognized_set_matches_the_authoring_report` below.
const RECOGNIZED: [&str; 4] = ["Complete", "inert", "partial", "known_wrong"];

/// Minimum defs the scan must see. Well below the real corpus (1,803); this
/// catches a scan pointed at an empty or wrong directory, not a corpus that
/// shrank a little. Same floor and same reasoning as `card_defs_fmt.rs`.
const MIN_DEF_FILES: usize = 1000;

/// The marker as a FIELD ASSIGNMENT, whitespace-blind between the colon and the
/// path. Hand-rolled rather than pulled in as a regex dependency: the engine's
/// dev-dependencies carry no regex crate and this pattern is three tokens.
///
/// Returns every variant name assigned to `completeness` in `text`. A `//`
/// comment mentioning the word in prose yields nothing, because prose does not
/// continue into `Completeness::`.
fn markers_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    for (start, _) in text.match_indices("completeness:") {
        // `\b` on the left: `some_completeness:` is not our field.
        if start > 0 {
            let prev = bytes[start - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' {
                continue;
            }
        }
        let rest = &text[start + "completeness:".len()..];
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix("Completeness::") else {
            continue;
        };
        let variant: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !variant.is_empty() {
            out.push(variant);
        }
    }
    out
}

/// Every `*.rs` under a defs directory except `mod.rs` — the same population
/// `tools/authoring-report.py` scans (`DEFS.glob("*.rs")` minus `mod.rs`) and the
/// same one `tools/check-defs-fmt.sh` formats.
fn def_files(defs_dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(defs_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", defs_dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|x| x == "rs") && p.file_stem().is_some_and(|s| s != "mod")
        })
        .collect();
    files.sort();
    files
}

/// The scan itself, factored out so the live corpus and the canary corpora go
/// through identical code. Returns `(unmarked, unrecognized, scanned)`.
fn scan(defs_dir: &Path) -> (Vec<String>, Vec<(String, String)>, usize) {
    let mut unmarked = Vec::new();
    let mut unrecognized = Vec::new();
    let files = def_files(defs_dir);
    for path in &files {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<non-utf8>")
            .to_string();
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let markers = markers_in(&text);
        if markers.is_empty() {
            unmarked.push(name);
            continue;
        }
        for m in markers {
            if !RECOGNIZED.contains(&m.as_str()) {
                unrecognized.push((name.clone(), m));
            }
        }
    }
    (unmarked, unrecognized, files.len())
}

// ── The live gate ────────────────────────────────────────────────────────────

#[test]
fn every_card_def_names_its_completeness_marker() {
    let defs = workspace_root().join("crates/card-defs/src/defs");
    let (unmarked, unrecognized, scanned) = scan(&defs);

    assert!(
        unmarked.is_empty(),
        "SR-39: {} card def(s) do not name `completeness:`. An unnamed marker is \
         `Completeness::Complete` by Default — deck-legal, with nobody having decided \
         that. Add an explicit marker to each: `Completeness::Complete` if you have \
         checked every printed clause is modelled, otherwise \
         `Completeness::partial(\"<the clause that is not>\")`, \
         `Completeness::inert(\"…\")` or `Completeness::known_wrong(\"…\")`.\n\
         Do NOT guess: CLAUDE.md calls a guessed marker the single most prohibited \
         pattern in this repo, because a def that claims Complete while a clause is \
         unmodelled is a deck-legal wrong card no gate can see.\n\n\
         Unmarked:\n{}",
        unmarked.len(),
        unmarked
            .iter()
            .map(|n| format!("  {n}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    assert!(
        unrecognized.is_empty(),
        "SR-39: {} card def(s) spell `completeness:` with something other than the four \
         recognized forms {RECOGNIZED:?}. `tools/authoring-report.py` buckets by the same \
         four and files anything else under CLEAN, so an unrecognized spelling silently \
         inflates the coverage headline with a card that declares itself incomplete. \
         Use the lowercase constructors (`Completeness::partial(\"…\")`), not the \
         capitalized variants.\n\n{}",
        unrecognized.len(),
        unrecognized
            .iter()
            .map(|(n, m)| format!("  {n}: Completeness::{m}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    assert!(
        scanned >= MIN_DEF_FILES,
        "SR-39 scanned only {scanned} defs — it has stopped seeing the corpus. \
         A gate that checks nothing passes everything."
    );
}

/// The gate above is worthless if it silently scans an empty set, which is the
/// exact failure mode `card_defs_fmt.rs` was written to fix. Assert the
/// denominator: the file count the scan walked must equal what is on disk.
#[test]
fn the_completeness_gate_is_not_vacuous() {
    let defs = workspace_root().join("crates/card-defs/src/defs");
    let (_, _, scanned) = scan(&defs);
    let on_disk = std::fs::read_dir(&defs)
        .expect("defs dir")
        .filter_map(Result::ok)
        .filter(|e| {
            let p = e.path();
            p.extension().is_some_and(|x| x == "rs") && p.file_stem().is_some_and(|s| s != "mod")
        })
        .count();
    assert_eq!(
        scanned, on_disk,
        "SR-39 scanned {scanned} defs but {on_disk} exist on disk"
    );
}

/// The four recognized spellings are a second copy of a fact owned by
/// `tools/authoring-report.py`, and copies drift. If the report starts
/// recognizing a fifth spelling and this gate does not, a def using it fails here
/// for no reason; if this gate gains one the report does not, the headline
/// silently miscounts. Compare them directly.
#[test]
fn the_recognized_set_matches_the_authoring_report() {
    let script = workspace_root().join("tools/authoring-report.py");
    let text = std::fs::read_to_string(&script).expect("tools/authoring-report.py");
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("RECOGNIZED_MARKERS"))
        .expect("authoring-report.py declares RECOGNIZED_MARKERS");
    for marker in RECOGNIZED {
        assert!(
            line.contains(&format!("\"{marker}\"")),
            "SR-39 recognizes `{marker}` but tools/authoring-report.py's \
             RECOGNIZED_MARKERS does not: {line}"
        );
    }
    // And the other direction: no spelling the report knows that this gate does not.
    let reported: Vec<&str> = line.split('"').skip(1).step_by(2).collect();
    for marker in &reported {
        assert!(
            RECOGNIZED.contains(marker),
            "tools/authoring-report.py recognizes `{marker}` but SR-39 does not: {line}"
        );
    }
    assert_eq!(
        reported.len(),
        RECOGNIZED.len(),
        "the two recognized sets have different sizes: report={reported:?} gate={RECOGNIZED:?}"
    );
}

// ── The executed defeats (change-class row 4) ────────────────────────────────

/// A def as an author writes one when they never thought about completeness: no
/// marker anywhere. `Completeness::Complete` by Default, deck-legal, undecided.
const FIXTURE_NO_MARKER: &str = r#"use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zz-canary"),
        name: "ZZ Canary".to_string(),
        ..Default::default()
    }
}
"#;

/// `misdirection.rs`'s shape: the substring `completeness:` appears, in prose, in
/// a `//` comment. A `text.contains("completeness:")` gate passes this file.
const FIXTURE_PROSE_ONLY: &str = r#"// Neither ever affected this def's completeness: no card was announceable as a
// copy target before OR after PB-DX25c.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zz-canary"),
        name: "ZZ Canary".to_string(),
        ..Default::default()
    }
}
"#;

/// The same def, marked. The green half of the matrix — without it the two RED
/// canaries would also pass a gate that simply fails everything.
const FIXTURE_MARKED: &str = r#"use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zz-canary"),
        name: "ZZ Canary".to_string(),
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
"#;

/// A def that spells a variant the report does not bucket. Must be reported as
/// unrecognized rather than passing as marked.
const FIXTURE_UNRECOGNIZED: &str = r#"use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zz-canary"),
        name: "ZZ Canary".to_string(),
        completeness: Completeness::Partial("capitalized variant".to_string()),
        ..Default::default()
    }
}
"#;

/// Stand up a throwaway corpus containing exactly `fixture` and run the SHIPPED
/// scanner over it. Returns `(unmarked, unrecognized)` counts.
fn shipped_scan_of(label: &str, fixture: &str) -> (usize, usize) {
    let tmp = std::env::temp_dir().join(format!("sr39_canary_{}_{}", label, std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("create temp defs dir");
    std::fs::write(tmp.join("zz_canary.rs"), fixture).expect("write fixture");
    // A `mod.rs` alongside it, so the canary also proves the exclusion rule is
    // applied: `mod.rs` carries no marker and must not be reported.
    std::fs::write(tmp.join("mod.rs"), "// generated\n").expect("write mod.rs");

    let (unmarked, unrecognized, scanned) = scan(&tmp);
    let _ = std::fs::remove_dir_all(&tmp);

    // Guard against a vacuous canary: the scan must have seen exactly the fixture.
    assert_eq!(
        scanned, 1,
        "the temp corpus was not picked up as one def — this canary proves nothing \
         (scanned {scanned}; `mod.rs` should have been excluded)"
    );
    (unmarked.len(), unrecognized.len())
}

#[test]
fn gate_catches_a_def_with_no_completeness_marker() {
    assert_eq!(
        shipped_scan_of("nomarker", FIXTURE_NO_MARKER),
        (1, 0),
        "SR-39 PASSED a def that never names `completeness:`. That def is \
         `Completeness::Complete` by Default — deck-legal with nobody having decided \
         it — which is the entire reason this gate exists (964 defs were in that state \
         when it was written, including two that hand a token to the wrong player)."
    );
}

#[test]
fn gate_catches_a_def_whose_only_marker_is_prose_in_a_comment() {
    assert_eq!(
        shipped_scan_of("prose", FIXTURE_PROSE_ONLY),
        (1, 0),
        "SR-39 PASSED a def whose only `completeness:` is inside a `//` comment. This \
         almost certainly means the scanner was loosened to a substring test — \
         `misdirection.rs` line 36 has exactly this shape in the live corpus, so a \
         substring gate reports the corpus clean while that def is unmarked. The \
         scanner must key on the FIELD ASSIGNMENT (`completeness:` then \
         `Completeness::<Variant>`), matching `tools/authoring-report.py`'s MARKER_RE."
    );
}

#[test]
fn gate_passes_a_def_that_names_its_marker() {
    assert_eq!(
        shipped_scan_of("marked", FIXTURE_MARKED),
        (0, 0),
        "SR-39 FAILED a def that correctly names `completeness: Completeness::Complete`. \
         A gate that fails everything is not a gate; the two RED canaries above prove \
         nothing without this one."
    );
}

#[test]
fn gate_catches_an_unrecognized_marker_spelling() {
    assert_eq!(
        shipped_scan_of("unrecognized", FIXTURE_UNRECOGNIZED),
        (0, 1),
        "SR-39 PASSED `Completeness::Partial(…)` — the capitalized variant rather than \
         the lowercase constructor. `tools/authoring-report.py` buckets only the four \
         recognized spellings and files anything else under CLEAN, so this spelling \
         counts a self-declared-incomplete card toward the coverage headline."
    );
}

// ── Why the gate has to be a source scan ─────────────────────────────────────

/// PINS the claim the header makes: an unmarked def and a def that writes
/// `completeness: Completeness::Complete` are indistinguishable AFTER
/// compilation — identical serialized `CardDefinition`, identical `validate_deck`
/// verdict. There is no runtime state that differs, so there is nothing a
/// behavioural probe could observe and a source scan is the only instrument
/// available. Under the pair-or-demote rule (`memory/conventions.md`) that puts
/// SR-39 in the same class as SR-5 and SR-36: its subject IS the source text.
///
/// The third deck is the control — it shows the marker is not inert machinery.
/// A `partial` marker DOES change the verdict (`DeckViolation::IncompleteCard`,
/// Architecture Invariant 9), which is exactly why leaving it to a default is a
/// decision and not a formality.
#[test]
fn deliberateness_is_invisible_at_runtime() {
    use mtg_engine::cards::{CardDefinition, CardRegistry, Completeness, TypeLine};
    use mtg_engine::state::{CardId, CardType, ManaCost, SuperType};
    use mtg_engine::{validate_deck, DeckViolation};

    fn filler(id: &str, completeness: Option<Completeness>) -> CardDefinition {
        let mut def = CardDefinition {
            card_id: CardId(id.to_string()),
            name: format!("Filler {id}"),
            mana_cost: Some(ManaCost {
                generic: 1,
                ..Default::default()
            }),
            types: TypeLine {
                supertypes: Default::default(),
                card_types: [CardType::Artifact].into_iter().collect(),
                subtypes: Default::default(),
            },
            oracle_text: String::new(),
            abilities: vec![],
            // NOTE: no `completeness` here — this arm is the unmarked def.
            ..Default::default()
        };
        if let Some(c) = completeness {
            def.completeness = c;
        }
        def
    }

    fn commander() -> CardDefinition {
        CardDefinition {
            card_id: CardId("cmd".to_string()),
            name: "Test Commander".to_string(),
            mana_cost: Some(ManaCost {
                generic: 2,
                white: 1,
                ..Default::default()
            }),
            types: TypeLine {
                supertypes: [SuperType::Legendary].into_iter().collect(),
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: Default::default(),
            },
            oracle_text: String::new(),
            abilities: vec![],
            power: Some(2),
            toughness: Some(2),
            completeness: Completeness::Complete,
            ..Default::default()
        }
    }

    fn plains() -> CardDefinition {
        CardDefinition {
            card_id: CardId("plains".to_string()),
            name: "Plains".to_string(),
            mana_cost: None,
            types: TypeLine {
                supertypes: [SuperType::Basic].into_iter().collect(),
                card_types: [CardType::Land].into_iter().collect(),
                subtypes: Default::default(),
            },
            oracle_text: String::new(),
            abilities: vec![],
            completeness: Completeness::Complete,
            ..Default::default()
        }
    }

    /// 1 commander + 39 unique fillers + 60 Plains = a legal 100-card deck.
    fn verdict(marker: Option<Completeness>) -> Vec<DeckViolation> {
        let mut defs = vec![commander(), plains()];
        for i in 1..=39 {
            defs.push(filler(&format!("f{i}"), marker.clone()));
        }
        let registry = CardRegistry::new(defs);
        let mut ids = vec![CardId("cmd".to_string())];
        for i in 1..=39 {
            ids.push(CardId(format!("f{i}")));
        }
        for _ in 0..60 {
            ids.push(CardId("plains".to_string()));
        }
        assert_eq!(ids.len(), 100);
        validate_deck(&[CardId("cmd".to_string())], &ids, &registry, &[]).violations
    }

    // Same bytes: the two defs serialize identically, field for field.
    let unmarked = serde_json::to_value(filler("f1", None)).expect("serialize unmarked");
    let marked =
        serde_json::to_value(filler("f1", Some(Completeness::Complete))).expect("serialize marked");
    assert_eq!(
        unmarked, marked,
        "an unmarked def and an explicitly-Complete def must serialize identically — \
         if they ever stop doing so, SR-39 could be a runtime check instead of a source scan"
    );

    // Same verdict: the deck gate cannot tell them apart either.
    assert!(
        verdict(None).is_empty(),
        "a deck of unmarked defs must validate clean — that is the hole SR-39 exists for"
    );
    assert!(
        verdict(Some(Completeness::Complete)).is_empty(),
        "a deck of explicitly-Complete defs must validate clean"
    );

    // Control: the marker is not decoration. A non-Complete one is fail-closed.
    let partial = verdict(Some(Completeness::partial("a clause is not modelled")));
    assert!(
        partial
            .iter()
            .any(|v| matches!(v, DeckViolation::IncompleteCard { .. })),
        "Architecture Invariant 9: a `partial` def must be rejected at deck-build time, \
         got {partial:?}"
    );
}
