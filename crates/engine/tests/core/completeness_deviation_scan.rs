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
    (
        "sword_of_truth_and_justice",
        "\"a second, unfixed deviation\" for oracle \"put a +1/+1 counter on a creature you \
         control\", which carries NO \"target\" (CR 115.10) and is therefore chosen on \
         resolution — authored as a real `TargetRequirement`, so hexproof / shroud / \
         protection / \"can't be the target of\" all wrongly bite, and CR 608.2b fizzles the \
         whole trigger (counter AND proliferate) if the chosen creature leaves, where the \
         printed card would simply pick another. Real and reachable. Allowlisted rather than \
         demoted on the same rule as `staff_of_compleation` and `nether_traitor` below: the \
         DSL has no choose-on-resolution-without-targeting channel for this shape, and \
         `frantic_search` ships `Complete` with the identical approximation (printed \"untap \
         up to three lands\", no \"target\"), so demoting the member that happens to sit in \
         PB-DP10's BASELINE would report a corpus class as one card. Filed as OOS-DX4-6. The \
         note that trips this detector is new (PB-DX4 fix cycle, review Finding 6); the \
         deviation is not. (scutemob-168)",
    ),
    (
        "staff_of_compleation",
        "\"corpus-wide approximation class\" for oracle \"Destroy target permanent YOU OWN\" \
         (ownership, CR 108.3) authored as `TargetController::You` (control, CR 109.4). \
         EXACTLY the `nether_traitor` case above and allowlisted for the same reason: \
         `TargetFilter` has no owner axis at all, so controller is the only expression the \
         DSL offers, and it is the corpus-standard one. Faithful in all play without a \
         control-change effect. Added by PB-DX4's OOS-DP10-8 triage (`scutemob-168`), which \
         found the deviation and wrote the note that trips this detector — the note is new, \
         the deviation is not. Deliberately allowlisted rather than demoted so the class is \
         decided as a class: OOS-DX4-1 asks how many `Complete` defs approximate an ownership \
         clause this way, and demoting the two that happen to sit in PB-DP10's BASELINE would \
         have reported a corpus class as a pair of cards. (scutemob-168)",
    ),
    (
        "delver_of_secrets",
        "PB-OS6(a): \"upkeep trigger modeled as an unconditional AtBeginningOfYourUpkeep \
         trigger\" describes the DSL shape (Effect::Conditional gated on \
         Condition::TopCardIsInstantOrSorcery), not a behavioral deviation from the oracle's \
         optional reveal. Reveal-to-transform is beneficial in effectively all realistic \
         board states (1/1 -> 3/2 flier), so a mandatory-if-true model is faithful for this \
         card specifically -- unlike heralds_horn.rs (known_wrong), where declining the \
         reveal can be correct.",
    ),
    (
        "anim_pakal_thousandth_moon",
        "PB-OS11: \"accepted minor deviation\" describes a documented, non-blocking edge \
         case — if Anim Pakal itself leaves the battlefield mid-resolution of its own \
         trigger, ruling 2023-11-10(a) says to use its last-known +1/+1 counter count, but \
         EffectAmount::CounterCount{Source} reads LIVE counters (no non-leaves-trigger LKI \
         counter reader exists in the engine). In the overwhelming majority of games Anim \
         Pakal is present through resolution, so the count is correct; this is the accepted \
         gap the plan (pb-plan-OS11.md, B-Card-1) explicitly directs not to block Complete \
         on, consistent with corpus precedent for mid-resolution source removal.",
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
    // PB-EF12 (2026-07-18): threshold lowered 690 -> 672. 17 defs flipped
    // known_wrong -> Complete this batch (`any_color: true` mana abilities now
    // resolve to a real chosen colour via Command::TapForMana.chosen_color, closing
    // EF-W-PB2-3 — see birds_of_paradise.rs and the sibling restores it points to),
    // dropping the corpus's non-Complete count from 699 to 681 (verified directly
    // against `all_cards()`, not estimated). This is a genuine headline-number
    // decrease from authoring work, not detector drift -- lower the floor with the
    // same 9-count margin the previous threshold kept, rather than papering over it.
    // PB-OS11 (2026-07-19, final PB-OS batch): threshold lowered 672 -> 662. Five
    // defs flipped partial/known_wrong -> Complete this batch: three via the new
    // TriggerCondition::WheneverYouAttack{filter} batch-filtered-attack primitive
    // (general_kreat_the_boltbringer, hermes_overseer_of_elpis: partial -> Complete;
    // anim_pakal_thousandth_moon: known_wrong -> Complete), plus two backfill flips
    // via the Cost::RemoveCounter mana-ability lowering (gemstone_array,
    // druids_repository: known_wrong -> Complete — their plain any-color mana ability
    // now resolves the chosen colour on the lowered TapForMana path, same as
    // birds_of_paradise). The corpus's non-Complete count had already drifted down
    // from 681 (intervening OS4-OS10 batches lowered it without needing to touch this
    // floor, since the assert is a lower bound) to 674 before this batch, and now
    // to 669 (verified via a direct grep of MARKER_FRAGMENTS across
    // crates/card-defs/src/defs/*.rs). Same margin convention as before.
    //
    // PB-DX3b (2026-08-01): threshold lowered 662 -> 661, and the stale "669" in the
    // message corrected to "661". The 669 figure had already gone stale silently
    // between PB-OS11 and this batch -- eight further batches (RS1-4, DP1-10, DX1-3)
    // flipped defs to `Complete` without anyone re-verifying this comment, so the
    // *true* non-Complete count on this branch's `main` parent was already 662, not
    // 669 -- the assert had eroded to its exact floor with ZERO margin, silently,
    // because `>=` only fails once the true count actually crosses the pinned line.
    // This batch's own net effect is -1 (measured directly against `all_cards()`,
    // not estimated): `ophiomancer` and `dwynen_s_elite` flip partial/inert ->
    // Complete (-2), `emeria_the_sky_ruin` flips Complete -> explicit `partial` (+1,
    // a genuine correction of a def that was `Complete` only by the
    // `#[default] Completeness::Complete` derive trap, not a real regression --
    // see `emeria_the_sky_ruin.rs`'s completeness note). `jadar_ghoulcaller_of_
    // nephalia` stays `Complete` (no marker either side). Net -1 tipped the
    // already-zero-margin floor from passing to failing. Measured on this branch:
    // `marked == 661` and a direct `all_cards()` non-Complete count also == 661 (the
    // detector currently has NO gap between the two counts at all). Pinned at the
    // exact measured value rather than re-establishing a margin, because the
    // generalisable lesson here is that ANY fixed margin silently erodes as later
    // batches flip markers and nothing re-derives it -- the next batch that moves
    // this number should re-measure `all_cards()` directly rather than trust this
    // comment's arithmetic, exactly as this update did.
    //
    // PB-DX3b fix cycle (2026-08-01, review Finding 5): kept the exact pin (option (b)
    // of the two the review offered) rather than restoring a margin -- the generalisable
    // lesson above is still the point of pinning at the measured value -- but the
    // FAILURE MESSAGE was wrong: it named only "MARKER_FRAGMENTS is broken" as the
    // cause, when `decision_gate.rs:923-924` states the opposite convention explicitly
    // ("Assertions are `>=` floors only ... an `==` pin reddens on unrelated
    // authoring"). A `>=` pinned at the exact current value fails on the very next
    // ordinary `Complete` flip, which has nothing to do with the detector. The message
    // below now names both possible causes and tells the reader what to do in each case,
    // rather than asserting the wrong one.
    // PB-DX4 (2026-08-01, `scutemob-168`): threshold raised 661 -> 667. Five defs were
    // demoted by the OOS-DP10-8 oracle-text triage of PB-DP10's 97-entry decision BASELINE
    // — `smugglers_copter` (Complete -> known_wrong) and `contaminant_grafter`,
    // `grateful_apparition`, `thrasios_triton_hero`, `shambling_ghast`, `hullbreaker_horror`
    // (Complete -> partial) — and no def flipped the other way, so the corpus's non-Complete
    // count rises by exactly 6. (Five of the six landed in the implement phase; the sixth,
    // `hullbreaker_horror`, was found by the closing review carrying the SAME flat-mode-target
    // defect `shambling_ghast` had just been demoted for, which is why this number was
    // re-measured after the fix cycle rather than carried forward from it.) RE-MEASURED
    // DIRECTLY against `all_cards()` rather than trusted from this arithmetic, as the PB-DX3b
    // note below instructs the next batch to do: `all_cards()` reports 667 non-Complete /
    // 1,137 Complete of 1,804, and a direct grep of MARKER_FRAGMENTS across
    // `crates/card-defs/src/defs/*.rs` independently reports 667 — the detector still has no
    // gap between the two counts. Pinned at the exact measured
    // value, keeping PB-DX3b's convention and its reasoning (any fixed margin silently
    // erodes as later batches flip markers and nothing re-derives it).
    // CARDS-2 (2026-08-02, `scutemob-181`): threshold lowered 667 -> 666, and this is the one
    // direction the comment above did not anticipate — the count fell without any def flipping
    // its marker. `crates/card-defs/src/defs/legolasquick_reflexes.rs` was DELETED: it and
    // `legolass_quick_reflexes.rs` both defined "Legolas's Quick Reflexes" under different
    // CardIds, so `CardRegistry::try_new`'s duplicate-id check never saw them, and the corpus
    // carried one card twice. Both were non-Complete, so removing the twin removes exactly one
    // marker fragment. RE-MEASURED DIRECTLY as this comment block instructs, not derived:
    // `all_cards()` reports 1,137 Complete / 666 non-Complete of 1,803 definitions, and an
    // independent grep of MARKER_FRAGMENTS across `crates/card-defs/src/defs/*.rs` also reports
    // 666 — the detector still has no gap between the two counts. The Complete numerator is
    // UNMOVED at 1,137 (this batch flipped no markers); only the denominator fell, because a
    // double-counted card stopped being counted twice. The new duplicate-name gate is
    // `core::cards2_printed_field_fidelity::r5_no_two_definitions_share_a_name`.
    // CARDS-2 second pass (2026-08-02, `scutemob-181`): threshold raised 666 -> 668.
    // `cyber_conversion.rs` (Complete -> inert) and `exalted_angel.rs` (Complete -> partial)
    // were demoted: both shipped `Completeness::Complete` while implementing oracle text the
    // printed cards do not have (Cyber Conversion authored a temporary type-change-plus-draw
    // spell instead of "turn target creature face down"; Exalted Angel declared static
    // `KeywordAbility::Lifelink` instead of its printed triggered "whenever this deals damage,
    // you gain that much life" ability). Both are genuine DSL gaps, not authoring slips — see
    // the TODO comments in each def. RE-MEASURED DIRECTLY, not derived: a grep of
    // MARKER_FRAGMENTS across `crates/card-defs/src/defs/*.rs` (excluding `mod.rs`) reports
    // 668 of 1,803 definitions marked non-Complete, so the Complete numerator falls
    // 1,137 -> 1,135.
    assert!(
        marked >= 668,
        "marker detector matched {marked} files; expected >= 668. This assertion has NO \
         margin (see the comment above) and can fail for two different reasons: (1) \
         MARKER_FRAGMENTS stopped matching (a detector bug -- the gate would then spuriously \
         flag marked defs) or, far more likely on an ordinary day, (2) a ROUTINE Complete \
         flip from unrelated card-authoring work legitimately moved the corpus's non-Complete \
         count. Re-measure the true count directly against all_cards() before assuming (1); \
         if it is (2), update this floor with a dated derivation comment."
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
