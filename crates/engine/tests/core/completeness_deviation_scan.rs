//! SR-12 anti-rot gate for the `Partial` / `KnownWrong` completeness markers.
//!
//! `card_registry_gate::test_inert_definitions_are_marked_incomplete` guards the
//! **Inert** class: a def with printed rules text and zero abilities must carry a
//! marker. Nothing guarded the other two classes. A def that *does* register
//! abilities but deliberately deviates from the oracle text — a simplification,
//! an approximation, "modeled as X" where the card actually does Y — is
//! `Partial` or `KnownWrong` by definition, but shipping it as `Complete` (the
//! `Default`) is invisible to every compile gate. That is exactly how SR-2's
//! first pass missed 28 defs.
//!
//! This test closes the hole textually: it scans every card-def source file for
//! deviation language and requires each hit to either carry a non-`Complete`
//! marker or appear in the reviewed [`ALLOWLIST`] below. `tools/authoring-report.py`
//! reports the same drift, but it is advisory and not in CI; this is the machine
//! gate.
//!
//! ## Why a source scan rather than a runtime check
//!
//! The deviation is documented in a *comment*, which does not survive into the
//! compiled `CardDefinition`. The only place the intent is legible is the source
//! text, so the gate reads the source — the same technique SR-5's keyword
//! registry and SR-8's protocol fingerprint use.

use std::fs;
use std::path::{Path, PathBuf};

/// The workspace root: `crates/engine/` is two levels down from it.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

fn defs_dir() -> PathBuf {
    workspace_root().join("crates/card-defs/src/defs")
}

/// Deviation-language needles, lower-cased. A card-def source that contains any
/// of these is claiming (or denying) a departure from the printed card and must
/// account for it — marker or allowlist.
///
/// This is the reviewed, documented needle set the acceptance criterion calls
/// for. `model+ed as` in the brief is spelled out as both the one-`l` and
/// two-`l` forms because that is how the corpus spells it.
const DEVIATION_NEEDLES: &[&str] = &[
    "simplif",     // "Simplified", "simplification"
    "modeled as",  // US spelling
    "modelled as", // UK spelling
    "deviation",   // "deviation from the oracle text"
    "approximat",  // "approximate", "approximation"
];

/// Non-`Complete` marker fragments. Presence of any means the def already
/// declares itself incomplete, so its deviation language is accounted for.
///
/// Both the constructor form (`Completeness::partial("…")`, the form the whole
/// corpus uses) and the bare variant form (`Completeness::Partial`) are matched,
/// so the gate does not depend on authoring style.
const MARKER_FRAGMENTS: &[&str] = &[
    "Completeness::inert",
    "Completeness::partial",
    "Completeness::known_wrong",
    "Completeness::Inert",
    "Completeness::Partial",
    "Completeness::KnownWrong",
];

/// Reviewed exceptions: files whose deviation-language match is a **description
/// of faithful modeling** (or of a since-fixed approximation), not a live
/// deviation. Each is the card's file stem plus the reason it is exempt.
///
/// Reviewed 2026-07-10 (SR-12). An entry is only valid while the file still
/// matches a deviation needle and is still `Complete` — the test asserts both,
/// so a stale entry fails rather than silently masking a future real deviation.
/// See `docs/sr-remediation-plan.md` (SR-12) for the review record.
const ALLOWLIST: &[(&str, &str)] = &[
    (
        "overlord_of_the_hauntwoods",
        "\"Modeled as two separate triggers\" — a faithful decomposition of one \
         ability into two TriggeredAbilityDefs, not a deviation.",
    ),
    (
        "tainted_field",
        "\"the 'or' is modeled as two separate activated abilities, one per color\" \
         — faithful decomposition of a hybrid mana ability, fully implemented.",
    ),
    // SR-33 removed `path_to_exile` from this list. Its justification — "a faithful
    // encoding of the optional search, not a simplification of it" — was false:
    // `Effect::MayPayOrElse` discards `cost`/`payer` and unconditionally executes
    // `or_else`, so the search always fires and the "may" was never encoded at all. The
    // entry was reasoned from the *intent* of the DSL shape without tracing into the
    // effect's implementation — inside the gate that exists to catch exactly that. The
    // def is now `known_wrong`, which is what removes it from this scan's scope.
    (
        "elvish_warmaster",
        "\"not an overbroad generic-creature approximation\" — the comment \
         explicitly asserts the filter is precise, i.e. the opposite of a deviation.",
    ),
    (
        "hazorets_monument",
        "\"was previously modeled as an …\" — describes a superseded modeling; the \
         current implementation is faithful.",
    ),
    (
        "reforge_the_soul",
        "\"Effect::WheelHand fixes the previous approximation\" — describes a \
         now-corrected approximation; the current implementation is faithful.",
    ),
    (
        "fiery_islet",
        "\"the 'or' is modeled as two separate activated abilities, one per color\" \
         — same tainted_field.rs pattern, faithful decomposition, fully implemented \
         (SR-34 un-demoted from known_wrong: the cost is now a real mana ability, \
         CR 605.1a).",
    ),
    (
        "nurturing_peatland",
        "\"the 'or' is modeled as two separate activated abilities, one per color\" \
         — same tainted_field.rs pattern, faithful decomposition, fully implemented \
         (SR-34 un-demoted from known_wrong).",
    ),
    (
        "silent_clearing",
        "\"the 'or' is modeled as two separate activated abilities, one per color\" \
         — same tainted_field.rs pattern, faithful decomposition, fully implemented \
         (SR-34 un-demoted from known_wrong).",
    ),
    (
        "nether_traitor",
        "\"best available approximation\" for oracle \"put into YOUR graveyard\" (ownership, \
         CR 404.3): the DSL has no owner-scoped death trigger, so this keys on controller = You, \
         the corpus-standard expression (athreos, fecundity). Faithful in all play without \
         gain-control of your own creatures; W-PB2 engine finding notes the residual. Not a real \
         deviation — it is the only expression the DSL offers. (scutemob-95)",
    ),
];

/// Read every `*.rs` file directly under `defs/`. Returns `(file_stem, source)`.
fn read_def_sources() -> Vec<(String, String)> {
    let dir = defs_dir();
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).expect("card-defs/src/defs must be readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("utf-8 file stem")
            .to_string();
        // The build-generated `mod.rs` aggregator is not a card def.
        if stem == "mod" {
            continue;
        }
        let src = fs::read_to_string(&path).expect("def source must be readable");
        out.push((stem, src));
    }
    out.sort();
    out
}

fn has_deviation_language(src_lower: &str) -> bool {
    DEVIATION_NEEDLES.iter().any(|n| src_lower.contains(n))
}

fn has_incomplete_marker(src: &str) -> bool {
    MARKER_FRAGMENTS.iter().any(|m| src.contains(m))
}

// ── The gate ──────────────────────────────────────────────────────────────────

#[test]
/// A card def that documents a deviation from its oracle text must not ship as
/// `Complete`. Either it carries a `Partial` / `KnownWrong` (/ `Inert`) marker,
/// or it is a reviewed false positive in [`ALLOWLIST`].
///
/// This is the anti-rot guard for the two marker classes the Inert gate does not
/// cover. A future def that adds a `// Simplified: we ignore the second clause`
/// comment and forgets the marker fails here by name.
fn deviation_language_requires_a_marker_or_allowlist() {
    let allow: std::collections::HashSet<&str> = ALLOWLIST.iter().map(|(f, _)| *f).collect();

    let offenders: Vec<String> = read_def_sources()
        .into_iter()
        .filter(|(stem, src)| {
            has_deviation_language(&src.to_lowercase())
                && !has_incomplete_marker(src)
                && !allow.contains(stem.as_str())
        })
        .map(|(stem, _)| stem)
        .collect();

    assert!(
        offenders.is_empty(),
        "these card defs use deviation language (one of {DEVIATION_NEEDLES:?}) but ship as \
         Complete with no marker. Either mark them non-Complete \
         (`completeness: Completeness::partial(\"…\")` / `known_wrong(\"…\")`) or, if the \
         language describes faithful modeling rather than a real deviation, add them to \
         ALLOWLIST in this file with a reason: {offenders:?}"
    );
}

// ── Non-vacuity guards (SR track policy: assert the denominator) ───────────────

#[test]
/// The scan must actually see the corpus. If the path is wrong or the dir is
/// empty, every other assertion here passes vacuously.
fn the_scan_reaches_the_corpus() {
    let n = read_def_sources().len();
    assert!(
        n > 1500,
        "expected the full card-def corpus (~1748 files), scanned only {n} — the scan is \
         not reaching defs/ and every gate in this file is vacuous"
    );
}

#[test]
/// The deviation detector must actually fire on the corpus. If `DEVIATION_NEEDLES`
/// stopped matching (a typo, a lower-casing bug), the gate above would pass by
/// finding zero hits — the classic absence-shaped vacuity.
fn the_deviation_detector_is_not_vacuous() {
    let hits = read_def_sources()
        .into_iter()
        .filter(|(_, src)| has_deviation_language(&src.to_lowercase()))
        .count();
    assert!(
        hits >= 50,
        "deviation detector matched only {hits} files; the corpus is known to contain well \
         over 100. The needle set or the matcher is broken and the marker gate is vacuous"
    );
}

#[test]
/// The marker detector must actually fire on the corpus. If `MARKER_FRAGMENTS`
/// stopped matching, the gate above would flag every marked def as an offender —
/// but this guard makes the failure legible as a detector bug, not 742 "offenders".
fn the_marker_detector_is_not_vacuous() {
    let marked = read_def_sources()
        .into_iter()
        .filter(|(_, src)| has_incomplete_marker(src))
        .count();
    // PB-EF9 (2026-07-18): threshold lowered 700 -> 690. olivia_voldaren and
    // dragonlord_silumgar both flipped partial/known_wrong -> Complete (the
    // EffectDuration::WhileYouControlSource primitive shipped), dropping the
    // corpus's non-Complete count from 701 to 699. This is a genuine headline-number
    // decrease from authoring work, not detector drift -- lower the floor with the
    // same margin the previous threshold kept, rather than papering over it.
    assert!(
        marked >= 690,
        "marker detector matched only {marked} files; the corpus has 699 non-Complete defs. \
         MARKER_FRAGMENTS is broken and the gate would spuriously flag marked defs"
    );
}

#[test]
/// Every allowlist entry must still (a) name a real file, (b) still match a
/// deviation needle, and (c) still be `Complete`. This keeps the allowlist
/// honest: an entry that no longer matches is dead weight, and one that has since
/// been marked non-Complete is redundant (it would pass on the marker) — either
/// way the entry should be removed, and this fails until it is.
fn every_allowlist_entry_is_live_and_necessary() {
    let sources: std::collections::HashMap<String, String> =
        read_def_sources().into_iter().collect();

    for (stem, reason) in ALLOWLIST {
        let src = sources.get(*stem).unwrap_or_else(|| {
            panic!("ALLOWLIST names {stem:?}, which is not a file under defs/ (reason: {reason})")
        });
        assert!(
            has_deviation_language(&src.to_lowercase()),
            "ALLOWLIST entry {stem:?} no longer matches any deviation needle — the exemption \
             is dead weight; remove it (reason on file: {reason})"
        );
        assert!(
            !has_incomplete_marker(src),
            "ALLOWLIST entry {stem:?} now carries a non-Complete marker, so it passes the gate \
             on the marker and does not need allowlisting — remove the redundant entry"
        );
    }
}
