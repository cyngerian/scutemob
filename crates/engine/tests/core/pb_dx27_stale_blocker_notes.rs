//! PB-DX27 anti-rot gate for **stale blocker notes** (`OOS-CARDS2-8` / `OOS-RR3-2`).
//!
//! ## The class
//!
//! A card def that cannot express a printed clause records why, in prose, naming
//! the DSL primitive it wants:
//!
//! ```text
//! // TODO: "mana value X or less" — max_cmc should be XValue.
//! ```
//!
//! That note is written **once**. When the primitive later ships, nothing revisits
//! it. The card stays `partial` forever, and the next author who reads the file is
//! told a lie by the file itself. `OOS-CARDS2-8` found four such notes by hand in a
//! single batch (`wake_the_dead`, `boon_satyr`, `braided_net`, `windbrisk_heights`),
//! all false by the time they were read, and `OOS-DX3-1`'s closure called the
//! corpus-wide re-check "a cheap standing sweep" and then closed without filing it.
//! `OOS-RR3-2` is that missing filing. **This file is the sweep made permanent.**
//!
//! Its sibling [`completeness_deviation_scan`] answers a different question — "does
//! a def that *declares* a deviation carry a marker?". This one answers "is the
//! declared blocker still **true**?". A def can pass that gate and fail this one:
//! `chord_of_calling` was correctly `partial` with a correctly-worded note, and the
//! note was simply wrong about the world.
//!
//! ## How existence is decided, and why this way
//!
//! For a mention like `Effect::RemoveFromCombat`, the gate asks whether the token
//! `Effect::RemoveFromCombat` occurs anywhere in non-comment source under
//! `crates/card-types/src` or `crates/engine/src`. That is a **usage** test, not a
//! declaration parse, and it is deliberate: an enum body declares a variant as bare
//! `RemoveFromCombat { .. }`, so a declaration parser must reconstruct which enum it
//! is inside — fragile, and the fragility fails *open* (a parser that stops matching
//! reports everything as absent, i.e. every note as correct, i.e. green).
//! Path-qualified usage is unambiguous and every DSL variant in this engine has at
//! least one executor or hash arm naming it that way.
//!
//! **Measured, not assumed** (PB-DX27, 2026-08-13): the heuristic was checked against
//! 15 hand-adjudicated identifiers drawn from the three adjudication chunks — 8
//! expected-present (`Effect::RemoveFromCombat`, `Effect::UntapPermanent`,
//! `TriggerCondition::WheneverOpponentDiscards`, `AltCostKind::Bestow`,
//! `EffectAmount::HandSize`, `KeywordAbility::CantBlock`,
//! `Effect::AdditionalCombatPhase`, `TargetController::DamagedPlayer`) and 7
//! expected-absent (`Effect::LookAtTopN`, `Cost::TapAnotherCreature`,
//! `Cost::SacrificeAnother`, `KeywordAbility::CantAttack`, `Condition::MainPhase`,
//! `TargetController::DefendingPlayer`,
//! `TriggerCondition::WhenThisExploitsACreature`). **15/15 agreed.** That sample is
//! pinned as [`existence_oracle_agrees_with_the_hand_adjudication`] so the oracle
//! cannot drift silently — the check that decides every other row is itself checked.
//!
//! ## Why a frozen roster rather than a bare "no stale notes" assertion
//!
//! Naming a live identifier is **not** by itself a defect, and this is the whole
//! precision problem. The commonest correct shape in this corpus is a contrast:
//!
//! ```text
//! // Cost::TapAnotherCreature (no such Cost variant; only Cost::Tap taps this permanent).
//! ```
//!
//! `Cost::Tap` exists, the note is right, and a gate that reddened on it would be
//! wrong four times over in this corpus alone (`glare_of_subdual`, `opposition`,
//! `springleaf_drum`, `azami_lady_of_scrolls`). There is no textual rule that
//! separates "names the live thing it is contrasting against" from "asserts the
//! live thing is missing" — that is a judgement, and judgements belong in a
//! reviewed list, not in a regex.
//!
//! So the gate freezes the population and ratchets it: every def whose gap prose
//! names a live identifier is in [`REVIEWED_LIVE_IDENTIFIER_MENTIONS`] with a
//! one-line verdict from the PB-DX27 adjudication. A **new** entrant fails, and the
//! failure message says what to do. This is the `KNOWN_DIVERGENT_ORACLE_TEXT`
//! pattern from `cards2_printed_field_fidelity`, chosen because that register's own
//! staleness assertion is what forced PB-DX27's six oracle repairs to be finished
//! rather than half-done.
//!
//! ## What this gate structurally cannot see
//!
//! Stated as a bound rather than discovered later (the `OOS-DX8` lesson):
//!
//! - **A note that names no identifier at all.** "Not expressible in the DSL" with
//!   nothing checkable in it is invisible here. That is precisely why
//!   `OOS-CARDS2-8`'s recommendation was *"have the DSL-gap notes name the primitive
//!   they want, so a grep can check whether it now exists"* — this gate is the
//!   grep, and it only works on notes that took that advice. The corpus is **not**
//!   fully compliant: [`gap_prose_without_a_named_identifier_is_measured`] measures
//!   and pins how many such notes exist, so the blind spot has a number instead of
//!   being a sentence in a comment.
//! - **A note that names an identifier which exists but does not do what the note
//!   assumes.** `EffectAmount::HandSize` exists and is still the wrong primitive for
//!   Chandra's wheel (`Effect::WheelHand` is the right one); `Effect::CreateEmblem`
//!   exists while no combat-damage site dispatches emblem triggers. Existence is
//!   necessary, never sufficient — every roster verdict below was reached by reading
//!   the code, not by reading this gate's output.
//! - **Scope is `crates/card-defs/src/defs` only.** Blocker notes in engine source
//!   are not swept.
//!
//! No engine or wire change: this file lives under `crates/engine/tests/`, outside
//! every `SCAN_ROOTS` PROTOCOL/HASH gate.

use std::collections::BTreeSet;
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

/// Roots whose non-comment source defines "the DSL exists at HEAD".
const DSL_SOURCE_ROOTS: &[&str] = &["crates/card-types/src", "crates/engine/src"];

/// Type prefixes a blocker note can meaningfully name.
///
/// Restricting to a known set keeps `std::fs`-shaped paths and CR citations out of
/// the mention extractor. Derived from the union of every `Type::Member` mention
/// found in the corpus's gap prose during the PB-DX27 census, then filtered to
/// those that name a real DSL type.
const DSL_TYPE_PREFIXES: &[&str] = &[
    "AbilityDefinition",
    "AltCostKind",
    "Condition",
    "Cost",
    "CounterType",
    "EffectAmount",
    "EffectDuration",
    "EffectFilter",
    "EffectLayer",
    "EffectTarget",
    "Effect",
    "ForEachTarget",
    "GameRestriction",
    "KeywordAbility",
    "LayerModification",
    "LoyaltyCost",
    "ManaRestriction",
    "ObjectFilter",
    "PlayerTarget",
    "ProtectionQuality",
    "ReplacementModification",
    "ReplacementTrigger",
    "SacrificeFilter",
    "TargetController",
    "TargetFilter",
    "TargetRequirement",
    "TimingRestriction",
    "TokenSpec",
    "TriggerCondition",
    "TriggerEvent",
    "WheelDisposal",
    "WheelDraw",
    "ZoneTarget",
];

/// Prose that ASSERTS A PRIMITIVE IS MISSING.
///
/// This is not the same category as [`completeness_deviation_scan`]'s
/// `DEVIATION_NEEDLES`, and the difference is the whole point: that gate asks "did
/// the author declare a departure from the printed card", this one asks "did the
/// author claim a primitive does not exist". A note can do the first without the
/// second ("simplified: we round down") and the second without the first (a note on
/// an `inert` def that never claims to match anything).
///
/// **Calibrated by measurement, not by taste** (PB-DX27, 2026-08-13). Three candidate
/// sets were run over the corpus and scored by the size of the population they hand a
/// human to review; set **C** — assertive negation plus `lacks` / `has no` — ships.
///
/// **The scores are NOT transcribed here, and that is deliberate.** The first draft of
/// this doc published a table of per-set figures which did not reproduce against the
/// shipped code — they came from a scratch Python approximation of this module written
/// during calibration, exactly the mistake [`t_derivation_report`] exists to prevent and
/// which PB-DX8 had already made once. A reviewer caught it. Rather than publish a
/// better-transcribed number, the numbers now come from the code: run
/// `cargo test --test core pb_dx27_stale_blocker_notes -- --nocapture` and read
/// `t_derivation_report`, which prints every population this file reasons about.
///
/// A is too narrow — `blocker` is a *label* ("the surviving blocker is…") rather than an
/// assertion, so it drags in prose that names live identifiers while missing the two
/// phrasings the corpus actually uses for a capability gap:
/// `ashaya_soul_of_the_wild` writes "EffectFilter **has no** nontoken-exclusion
/// variant" and `skrevls_hive` writes "EffectFilter::CreaturesYouControl **lacks** a
/// keyword filter". Neither is reachable from set A or B.
///
/// Rejected deliberately, each with its reason, so the next editor does not re-add
/// them: `blocker` / `blocked on` (labels, not claims — they name the live thing
/// being worked around); `would need` / `needs a new` (forward-looking design prose,
/// true of primitives that exist); `not supported` / `unsupported` (used in this
/// corpus about *combinations* that exist individually, e.g. Escalate with
/// `ModeSelection.mode_targets`); bare `not exist` (a substring of `does not exist`
/// that also catches "must not exist").
const GAP_NEEDLES: &[&str] = &[
    "dsl gap",
    "does not exist",
    "doesn't exist",
    "not expressible",
    "inexpressible",
    "not in dsl",
    "not in the dsl",
    "no such",
    "cannot be expressed",
    "can't be expressed",
    "no way to express",
    "not representable",
    "no variant",
    "lacks",
    "has no",
];

/// Gap phrasings deliberately EXCLUDED from [`GAP_NEEDLES`], kept here so the second
/// recall bound has a measurement instead of a sentence.
///
/// **This is the gate's real blind spot, and it was found by a reviewer rather than by
/// the author.** The module doc originally stated exactly one bound — "a note that names
/// no identifier at all" ([`gap_prose_without_a_named_identifier_is_measured`]). But a
/// note can name an identifier, assert it is missing, and still be invisible to both
/// ratchets, simply by phrasing the assertion outside the shipped needle set: `blocked
/// on`, `blocker`, `unimplemented`, `not implemented`, `is missing from the DSL`, `would
/// need`. Two defs in this population literally record their own note as stale in prose
/// (`baron_bertram_graywater`, `demolition_field`) and neither ratchet can see them.
///
/// These phrasings are excluded from the primary set for a stated precision reason — they
/// are labels and design prose rather than assertions, and including them costs far more
/// noise than signal. That trade-off is defensible. Leaving its cost unmeasured was not,
/// so the population gets its own downward-only ratchet below.
const OUT_OF_SET_GAP_NEEDLES: &[&str] = &[
    "blocked on",
    "blocker",
    "unimplemented",
    "not implemented",
    "no primitive",
    "not supported",
    "unsupported",
    "would need",
    "is missing",
];

// ── the frozen population ─────────────────────────────────────────────────────

/// How many defs name a live identifier inside gap prose. Downward-only.
///
/// **Measured at PB-DX27 (2026-08-13): 107.**
///
/// This is a **count** ratchet and not a reviewed roster, and the choice is worth
/// stating because the alternative was tried first. A 107-row hand-verdict list is
/// not a review — it is a rubber stamp with 107 signatures, and the next author
/// would append to it exactly as thoughtlessly as the stale notes it exists to
/// catch. The number is honest about what it is: *this many defs mention a live
/// primitive while asserting a gap, and most of them are legitimately contrasting
/// the live thing against a missing one.* What it forbids is the number **growing**
/// — a newly-written stale note pushes it over and fails.
///
/// It certifies nothing about any individual def in the count. It is a population
/// bound, the same construct and the same limitation as `decision_gate.rs`'s
/// `MAX_AUTO_CHOSEN_COMPLETE_UNION`.
const LIVE_IDENTIFIER_MENTION_CEILING: usize = 107;

/// Defs whose stale blocker note PB-DX27 refuted and REPAIRED.
///
/// Each of these carried a note asserting a primitive was missing when it was not.
/// The clause was authored (or the note rewritten to name the identifier that is
/// genuinely absent), and the def consequently **left** the live-naming set.
///
/// This list is the sweep's closure proof, and it is the non-vacuous half of this
/// file: [`the_repaired_defs_no_longer_claim_a_live_primitive_is_missing`] fails if
/// any of them regains a gap note naming a live identifier. A closure asserted in a
/// commit message is a claim; this is a check.
const REPAIRED_BY_PB_DX27: &[(&str, &str)] = &[
    (
        "chord_of_calling.rs",
        "'max_cmc should be XValue' — TargetFilter.max_cmc_amount shipped with PB-EF10. \
         Authored; promoted partial -> Complete.",
    ),
    (
        "green_suns_zenith.rs",
        "identical false max_cmc claim, plus a phantom trailing oracle clause. Authored; \
         promoted partial -> Complete.",
    ),
    (
        "reconnaissance.rs",
        "'Effect::RemoveFromCombat does not exist in the current DSL' — it exists, and the \
         DSL doc at its declaration prescribes this exact card's Sequence. Authored; \
         inert -> Complete.",
    ),
    (
        "wight_of_the_reliquary.rs",
        "'Cost::Sacrifice has no another/exclude-self variant' — TargetFilter.exclude_self is \
         lowered onto the activation cost and CR 109.1-enforced. Authored; partial -> Complete.",
    ),
    (
        "chandra_flamecaller.rs",
        "'EffectAmount::HandSize not in DSL' — it exists, and is the WRONG primitive; \
         Effect::WheelHand is the right one. Authored; partial -> Complete. Also removed an \
         activatable Effect::Nothing loyalty ability (W5 wrong game state).",
    ),
    (
        "blackblade_reforged.rs",
        "'Equip legendary creature {3} has no DSL representation' — CR 702.6c makes it a \
         SEPARATE activated ability and TargetFilter.legendary exists. Authored; stays \
         partial, because the dynamic land-count static has a real CR 108.5/611.2c \
         controller-resolution problem (shared with crown_of_skemfar, empyrial_plate).",
    ),
    (
        "marisi_breaker_of_the_coil.rs",
        "inline TODO denied TargetController::DamagedPlayer while the file's own note said \
         STALE and six corpus defs already used it. Goad clause authored; inert -> partial.",
    ),
    (
        "ruthless_technomancer.rs",
        "'Cost::Sacrifice threads only a PlayerId, not a source ObjectId' — false since \
         PB-EF1. ETB clause authored; inert -> partial.",
    ),
    (
        "vampire_gourmand.rs",
        "same false mechanism claim as ruthless_technomancer, and the two notes cited each \
         other in a loop. Attack trigger authored; inert -> partial.",
    ),
    (
        "kaito_shizuki.rs",
        "'unblockable is a static ability, not a keyword' — KeywordAbility::CantBeBlocked \
         exists and TokenSpec carries keywords. -2 authored; stays partial. The -7 was \
         deliberately NOT authored: Effect::CreateEmblem exists but no combat-damage site \
         dispatches emblem triggers, so authoring it would ship a 7-loyalty no-op.",
    ),
];

/// Defs that remain in the live-naming set for a REVIEWED reason.
///
/// The commonest correct shape in this corpus is a contrast — naming the live thing
/// in order to say what is missing *about* it. Both rows below name a live TYPE
/// while asserting a missing FIELD on it, which no identifier-level check can
/// distinguish and which is why the ceiling above is a count rather than a verdict.
const REVIEWED_CONTRAST_MENTIONS: &[(&str, &str)] = &[
    (
        "the_world_tree.rs",
        "names Effect::SearchLibrary, which exists, while asserting it has no COUNT FIELD — \
         true (player/filter/reveal/destination/shuffle_before_placing/also_search_graveyard, \
         no count), so 'any number of God cards' stays inexpressible. PB-DX27 authored this \
         def's OTHER blocked clause (the six-lands static grant) and left this one.",
    ),
    (
        "encroaching_dragonstorm.rs",
        "same shape: Effect::SearchLibrary exists, its missing count field is what blocks \
         'up to two basic land cards'. PB-DX27 authored this def's second trigger via \
         Effect::MoveZone and left the search clause blocked.",
    ),
];

// ── helpers ───────────────────────────────────────────────────────────────────

/// Non-comment source across [`DSL_SOURCE_ROOTS`], concatenated.
fn dsl_source_blob() -> String {
    let root = workspace_root();
    let mut out = String::new();
    for r in DSL_SOURCE_ROOTS {
        collect_rs(&root.join(r), &mut out);
    }
    out
}

fn collect_rs(dir: &Path, out: &mut String) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // `tests/` under a src root would let a test's own mention of an
            // absent variant vouch for its existence.
            if path.file_name().is_some_and(|n| n == "tests") {
                continue;
            }
            collect_rs(&path, out);
            continue;
        }
        if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(src) = fs::read_to_string(&path) {
                out.push_str(&strip_comments(&src));
                out.push('\n');
            }
        }
    }
}

/// Remove `/* */` blocks and `//` line comments.
///
/// A comment naming a variant must not count as that variant existing — the
/// engine is full of comments discussing primitives it does not have.
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let b = src.as_bytes();
    let mut i = 0usize;
    let mut in_block = false;
    while i < b.len() {
        if in_block {
            if i + 1 < b.len() && b[i] == b'*' && b[i + 1] == b'/' {
                in_block = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
            in_block = true;
            i += 2;
            continue;
        }
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Char-boundary safe push.
        let start = i;
        i += 1;
        while i < b.len() && !src.is_char_boundary(i) {
            i += 1;
        }
        out.push_str(&src[start..i]);
    }
    out
}

/// One gap-asserting prose unit: a `//` comment body, a `/* */` body, or a
/// `Completeness::{partial,inert,known_wrong}("…")` note body.
struct ProseUnit {
    text: String,
}

fn prose_units(src: &str) -> Vec<ProseUnit> {
    let mut out = Vec::new();
    for line in src.lines() {
        if let Some(idx) = line.find("//") {
            out.push(ProseUnit {
                text: line[idx + 2..].to_string(),
            });
        }
    }
    // Block comments, and completeness-note string bodies, are each one unit.
    for body in block_bodies(src, "/*", "*/") {
        out.push(ProseUnit { text: body });
    }
    for body in completeness_notes(src) {
        out.push(ProseUnit { text: body });
    }
    out
}

fn block_bodies(src: &str, open: &str, close: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(s) = rest.find(open) {
        let after = &rest[s + open.len()..];
        let Some(e) = after.find(close) else { break };
        out.push(after[..e].to_string());
        rest = &after[e + close.len()..];
    }
    out
}

/// The concatenated string-literal body of every completeness note.
fn completeness_notes(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for ctor in [
        "Completeness::partial(",
        "Completeness::inert(",
        "Completeness::known_wrong(",
    ] {
        let mut rest = src;
        while let Some(s) = rest.find(ctor) {
            let after = &rest[s + ctor.len()..];
            let mut depth = 1i32;
            let mut end = after.len();
            for (i, ch) in after.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let body = &after[..end];
            // Concatenate the string-literal pieces (the notes are multi-line
            // `"…" \ "…"` continuations).
            let mut text = String::new();
            let mut in_str = false;
            let mut prev_backslash = false;
            for ch in body.chars() {
                if in_str {
                    if prev_backslash {
                        prev_backslash = false;
                        text.push(ch);
                        continue;
                    }
                    match ch {
                        '\\' => prev_backslash = true,
                        '"' => in_str = false,
                        _ => text.push(ch),
                    }
                } else if ch == '"' {
                    in_str = true;
                }
            }
            if !text.is_empty() {
                out.push(text);
            }
            rest = &after[end.min(after.len())..];
        }
    }
    out
}

fn asserts_a_gap(text: &str) -> bool {
    let low = text.to_lowercase();
    GAP_NEEDLES.iter().any(|n| low.contains(n))
}

fn asserts_a_gap_out_of_set(text: &str) -> bool {
    let low = text.to_lowercase();
    OUT_OF_SET_GAP_NEEDLES.iter().any(|n| low.contains(n))
}

/// Defs reachable ONLY by [`OUT_OF_SET_GAP_NEEDLES`] — they name a live identifier inside
/// a gap assertion the primary needle set does not phrase-match, and they are therefore
/// invisible to both R1 and R3.
fn defs_naming_a_live_identifier_out_of_set_only() -> Vec<(String, BTreeSet<String>)> {
    let blob = dsl_source_blob();
    let primary: BTreeSet<String> = defs_naming_a_live_identifier()
        .into_iter()
        .map(|(f, _)| f)
        .collect();
    let mut out = Vec::new();
    for path in def_paths() {
        let name = path
            .file_name()
            .expect("path has a file name")
            .to_string_lossy()
            .to_string();
        if primary.contains(&name) {
            continue;
        }
        let src = fs::read_to_string(&path).expect("def source must be readable");
        let mut live: BTreeSet<String> = BTreeSet::new();
        for unit in prose_units(&src) {
            if !asserts_a_gap_out_of_set(&unit.text) {
                continue;
            }
            for m in dsl_mentions(&unit.text) {
                if blob.contains(&m) {
                    live.insert(m);
                }
            }
        }
        if !live.is_empty() {
            out.push((name, live));
        }
    }
    out
}

/// Defs that assert a gap in [`GAP_NEEDLES`]'s vocabulary while naming NO DSL identifier
/// at all — the population this gate structurally cannot speak about.
fn opaque_gap_note_defs() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for path in def_paths() {
        let src = fs::read_to_string(&path).expect("def source must be readable");
        let mut asserts = false;
        let mut names_any = false;
        for unit in prose_units(&src) {
            if asserts_a_gap(&unit.text) {
                asserts = true;
                if !dsl_mentions(&unit.text).is_empty() {
                    names_any = true;
                }
            }
        }
        if asserts && !names_any {
            out.insert(
                path.file_name()
                    .expect("path has a file name")
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }
    out
}

/// Every card-def path, sorted. Extracted so the walks in this file cannot drift.
fn def_paths() -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(defs_dir())
        .expect("card-defs/src/defs must be readable")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .filter(|p| p.file_name().is_some_and(|n| n != "mod.rs"))
        .collect();
    entries.sort();
    entries
}

/// Every `Type::Member` mention in `text` whose `Type` is a DSL type.
fn dsl_mentions(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i].is_ascii_uppercase() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_alphanumeric() || bytes[i] == '_') {
                i += 1;
            }
            let ty: String = bytes[start..i].iter().collect();
            if i + 1 < bytes.len() && bytes[i] == ':' && bytes[i + 1] == ':' {
                let ms = i + 2;
                let mut j = ms;
                while j < bytes.len() && (bytes[j].is_alphanumeric() || bytes[j] == '_') {
                    j += 1;
                }
                if j > ms && DSL_TYPE_PREFIXES.contains(&ty.as_str()) {
                    let member: String = bytes[ms..j].iter().collect();
                    // A member must start uppercase to be a variant; `Effect::from`
                    // and friends are method calls, not variants.
                    if member.starts_with(|c: char| c.is_ascii_uppercase()) {
                        out.insert(format!("{ty}::{member}"));
                    }
                }
                i = j;
                continue;
            }
            continue;
        }
        i += 1;
    }
    out
}

/// `(def file name, gap prose, the live identifiers it names)` for every def whose
/// gap prose names at least one identifier that exists at HEAD.
fn defs_naming_a_live_identifier() -> Vec<(String, BTreeSet<String>)> {
    let blob = dsl_source_blob();
    let mut out = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(defs_dir())
        .expect("card-defs/src/defs must be readable")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .filter(|p| p.file_name().is_some_and(|n| n != "mod.rs"))
        .collect();
    entries.sort();
    for path in entries {
        let src = fs::read_to_string(&path).expect("def source must be readable");
        let mut live: BTreeSet<String> = BTreeSet::new();
        for unit in prose_units(&src) {
            if !asserts_a_gap(&unit.text) {
                continue;
            }
            for m in dsl_mentions(&unit.text) {
                if blob.contains(&m) {
                    live.insert(m);
                }
            }
        }
        if !live.is_empty() {
            let name = path
                .file_name()
                .expect("path has a file name")
                .to_string_lossy()
                .to_string();
            out.push((name, live));
        }
    }
    out
}

// ── the gates ─────────────────────────────────────────────────────────────────

/// R0 — the existence oracle itself is checked.
///
/// Every other assertion in this file rests on `blob.contains("Type::Member")`
/// deciding existence correctly. A checker whose reference set is derived from the
/// thing it checks can never disagree with it (`OOS-DX7`/`OOS-DX8`, three batches
/// running), so the oracle is pinned against a sample adjudicated **by hand, from
/// the code**, not by this function.
#[test]
fn existence_oracle_agrees_with_the_hand_adjudication() {
    let blob = dsl_source_blob();
    // (identifier, expected-present) — PB-DX27 adjudication, 2026-08-13.
    let sample: &[(&str, bool)] = &[
        ("Effect::RemoveFromCombat", true),
        ("Effect::UntapPermanent", true),
        ("TriggerCondition::WheneverOpponentDiscards", true),
        ("AltCostKind::Bestow", true),
        ("EffectAmount::HandSize", true),
        ("KeywordAbility::CantBlock", true),
        ("Effect::AdditionalCombatPhase", true),
        ("TargetController::DamagedPlayer", true),
        ("Effect::LookAtTopN", false),
        ("Cost::TapAnotherCreature", false),
        ("Cost::SacrificeAnother", false),
        ("KeywordAbility::CantAttack", false),
        ("Condition::MainPhase", false),
        ("TargetController::DefendingPlayer", false),
        ("TriggerCondition::WhenThisExploitsACreature", false),
    ];
    let mut wrong = Vec::new();
    for (ident, expected) in sample {
        let got = blob.contains(ident);
        if got != *expected {
            wrong.push(format!(
                "{ident}: oracle says present={got}, hand adjudication says present={expected}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "the existence oracle disagrees with the PB-DX27 hand adjudication on {} of {} sampled \
         identifiers. Either the DSL changed (update the sample AND re-run the sweep — a \
         primitive landing is exactly the event this whole file exists to catch) or \
         `dsl_source_blob`/`strip_comments` has stopped matching, in which case every other \
         test in this file is now vacuous:\n  {}",
        wrong.len(),
        sample.len(),
        wrong.join("\n  ")
    );
    // Non-vacuity: a blob that shrank to nothing would report every identifier
    // absent, i.e. every note correct, i.e. silence.
    assert!(
        blob.len() > 1_000_000,
        "DSL source blob is only {} bytes — the walk has stopped finding source",
        blob.len()
    );
}

/// R1 — the population of gap notes naming a live primitive may not GROW.
#[test]
fn live_identifier_mentions_are_ratcheted() {
    let found = defs_naming_a_live_identifier();
    assert!(
        found.len() <= LIVE_IDENTIFIER_MENTION_CEILING,
        "{} card defs assert a DSL gap while naming a primitive that EXISTS at HEAD, above the \
         ratchet of {LIVE_IDENTIFIER_MENTION_CEILING}. At least one note is new.\n\nThis is the \
         `OOS-CARDS2-8` class: a note written once and never revisited after the primitive \
         landed, which then tells the next author a lie. For the new one, do ONE of:\n  (a) the \
         note is stale -> author the clause and repair the note (this is the good outcome, and \
         it is what PB-DX27 did for the 10 defs in REPAIRED_BY_PB_DX27);\n  (b) the note names \
         the live identifier only to CONTRAST it with a genuinely missing one -> reword it to \
         name the absent identifier, and if the mention must stay, add a REVIEWED_CONTRAST_\
         MENTIONS row and raise this ceiling with a stated reason.\n\nDo NOT pick (b) without \
         reading the printed card: existence is necessary, never sufficient. \
         `EffectAmount::HandSize` exists and is still the wrong primitive for a wheel; \
         `Effect::CreateEmblem` exists while no combat-damage site dispatches emblem \
         triggers.\n\nCurrent population:\n  {}",
        found.len(),
        found
            .iter()
            .map(|(f, live)| format!(
                "{f} — {}",
                live.iter().cloned().collect::<Vec<_>>().join(", ")
            ))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    // Non-vacuity: a derivation that stopped matching would report 0 and pass.
    assert!(
        found.len() > 50,
        "only {} defs name a live identifier in gap prose — the derivation has stopped \
         matching, and this ratchet is now vacuous",
        found.len()
    );
}

/// R2 — PB-DX27's repairs stay repaired.
///
/// This is the sweep's closure proof and the non-vacuous half of the file. Each def
/// in [`REPAIRED_BY_PB_DX27`] carried a note asserting a primitive was missing when
/// it was not; the repair made it leave the live-naming set. If one comes back, a
/// stale note has been reintroduced on a def that was explicitly cleaned.
///
/// A closure asserted in a commit message is a claim. This is a check.
#[test]
fn the_repaired_defs_no_longer_claim_a_live_primitive_is_missing() {
    let found: BTreeSet<String> = defs_naming_a_live_identifier()
        .into_iter()
        .map(|(f, _)| f)
        .collect();
    let regressed: Vec<&str> = REPAIRED_BY_PB_DX27
        .iter()
        .map(|(f, _)| *f)
        .filter(|f| found.contains(*f))
        .collect();
    assert!(
        regressed.is_empty(),
        "{} def(s) that PB-DX27 repaired have regained a gap note naming a live primitive:\n  \
         {}\n\nEither a stale note was reintroduced, or a primitive these defs legitimately \
         name as missing has since shipped — in which case author the clause rather than \
         relisting the def.",
        regressed.len(),
        regressed.join("\n  ")
    );
    // Every listed def must still exist, and carry a substantive verdict.
    let dir = defs_dir();
    let missing: Vec<&str> = REPAIRED_BY_PB_DX27
        .iter()
        .chain(REVIEWED_CONTRAST_MENTIONS.iter())
        .map(|(f, _)| *f)
        .filter(|f| !dir.join(f).exists())
        .collect();
    assert!(
        missing.is_empty(),
        "{} listed def file(s) do not exist:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
    let empty: Vec<&str> = REPAIRED_BY_PB_DX27
        .iter()
        .chain(REVIEWED_CONTRAST_MENTIONS.iter())
        .filter(|(_, why)| why.trim().len() < 40)
        .map(|(f, _)| *f)
        .collect();
    assert!(
        empty.is_empty(),
        "row(s) carry no substantive verdict:\n  {}",
        empty.join("\n  ")
    );
}

/// R2b — the reviewed-contrast rows are still real.
///
/// A contrast row that stops being found means its note was repaired or deleted; a
/// register that outlives its subject is an exemption, not a record. This is the
/// same staleness assertion that forced PB-DX27's six `KNOWN_DIVERGENT_ORACLE_TEXT`
/// repairs to be finished rather than half-done.
#[test]
fn every_reviewed_contrast_row_is_still_found() {
    let found: BTreeSet<String> = defs_naming_a_live_identifier()
        .into_iter()
        .map(|(f, _)| f)
        .collect();
    let stale: Vec<&str> = REVIEWED_CONTRAST_MENTIONS
        .iter()
        .map(|(f, _)| *f)
        .filter(|f| !found.contains(*f))
        .collect();
    assert!(
        stale.is_empty(),
        "{} REVIEWED_CONTRAST_MENTIONS row(s) are no longer found by the derivation — the note \
         was repaired or the def deleted. Remove the row(s) and lower \
         LIVE_IDENTIFIER_MENTION_CEILING accordingly:\n  {}",
        stale.len(),
        stale.join("\n  ")
    );
}

/// R3 — the blind spot has a number.
///
/// `OOS-CARDS2-8`'s recommendation was "have the DSL-gap notes name the primitive
/// they want, so a grep can check whether it now exists". This gate IS that grep,
/// and it is worth nothing on a note that names nothing. Rather than describing
/// that limitation in prose and leaving it unmeasured, pin the size of the
/// unreachable population, ratcheted downward-only.
///
/// The ceiling is a **population** bound, not a per-def promise: it says how many
/// defs this gate cannot speak about, and it goes down as notes are rewritten to
/// name their primitive. It does not certify anything about the defs it counts.
#[test]
fn gap_prose_without_a_named_identifier_is_measured() {
    let opaque = opaque_gap_note_defs().len();
    assert!(
        opaque <= OPAQUE_GAP_NOTE_CEILING,
        "{opaque} card defs assert a DSL gap without naming any DSL identifier, above the \
         ratchet of {OPAQUE_GAP_NOTE_CEILING}. A note that names no primitive cannot be \
         machine-rechecked and will go stale in silence — that is the entire `OOS-CARDS2-8` \
         failure mode. Rewrite the new note(s) to name the primitive they want, or raise this \
         ceiling with a stated reason."
    );
    assert!(
        opaque > 0,
        "0 opaque gap notes — either the corpus became fully compliant (excellent: lower the \
         ceiling to 0 and say so) or `asserts_a_gap`/`dsl_mentions` has stopped matching and \
         this ratchet is now vacuous"
    );
}

/// Measured at PB-DX27 (2026-08-13): **357**. Downward-only.
///
/// 357 of 1,803 defs assert a gap in prose while naming no DSL identifier at all,
/// so this gate can say nothing about them. That is the honest size of the blind
/// spot, and it is the number `OOS-CARDS2-8`'s recommendation is aimed at: every
/// note rewritten to name its primitive moves one def out of this count and into
/// the machine-checkable population.
const OPAQUE_GAP_NOTE_CEILING: usize = 357;

/// Ceiling for R4, the SECOND recall bound. Downward-only.
///
/// Set from the gate's own output at the PB-DX27 `/review` fix cycle (2026-08-13) — see
/// [`t_derivation_report`], which prints it rather than leaving it to be transcribed.
///
/// **74**, and worth recording how that number was arrived at. The reviewer measured 74
/// from an independent replica; the author's own scratch replica said 10; this Rust
/// implementation, run as the gate, says **74**. The reviewer's figure was right and the
/// author's replica was the faulty one — which is the second time in this batch that a
/// number computed outside the shipped code disagreed with it, and the reason both this
/// ceiling and every other population here are now printed by [`t_derivation_report`]
/// instead of being transcribed into a comment.
const OUT_OF_SET_LIVE_MENTION_CEILING: usize = 74;

/// A non-test reporter, run as a test so the numbers are PRINTED rather than
/// transcribed into a comment that will rot.
///
/// PB-DX8 published two figures measured against a pre-fix axis and never re-run;
/// the correction was to make the code print them. Same here.
#[test]
fn t_derivation_report() {
    let found = defs_naming_a_live_identifier();
    println!("PB-DX27 stale-blocker-note derivation");
    println!(
        "  defs naming a LIVE identifier in gap prose: {}",
        found.len()
    );
    for (f, live) in &found {
        println!(
            "    {f}: {}",
            live.iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }
    println!("  ratchet ceiling: {LIVE_IDENTIFIER_MENTION_CEILING}");
    let out_of_set = defs_naming_a_live_identifier_out_of_set_only();
    println!(
        "  out-of-set-phrasing live mentions (R4, ceiling {OUT_OF_SET_LIVE_MENTION_CEILING}): {}",
        out_of_set.len()
    );
    for (f, live) in &out_of_set {
        println!(
            "    {f}: {}",
            live.iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }
    // The share of the corpus NO ratchet in this file can speak about. R1/R3/R4 are all
    // POPULATION counts, so adding a stale note to a def already inside one of them moves
    // no number at all — the per-def blind spot the /review asked to be quantified.
    let total = def_paths().len();
    let mut covered: BTreeSet<String> = found.iter().map(|(f, _)| f.clone()).collect();
    covered.extend(out_of_set.iter().map(|(f, _)| f.clone()));
    covered.extend(opaque_gap_note_defs());
    println!(
        "  corpus {total} defs: {} counted by some ratchet here, {} ({:.1}%) counted by \
         NONE — a def in neither set can gain a stale note without moving any number",
        covered.len(),
        total - covered.len(),
        100.0 * (total - covered.len()) as f64 / total as f64
    );
    println!(
        "  REPAIRED_BY_PB_DX27 rows: {} (each proven ABSENT from the primary set above)",
        REPAIRED_BY_PB_DX27.len()
    );
    println!(
        "  REVIEWED_CONTRAST_MENTIONS rows: {}",
        REVIEWED_CONTRAST_MENTIONS.len()
    );
}

/// R4 — the SECOND recall bound, measured rather than described.
///
/// A gap note that names an identifier can still evade R1 and R3 by phrasing the
/// assertion outside [`GAP_NEEDLES`]. Found by the PB-DX27 `/review`, which planted 11
/// stale-note shapes into a clean def and established that `/* */` block comments and
/// every identifier-shape variation it tried ARE caught, while phrasing is the one real
/// escape. This ratchet bounds that escape instead of leaving it unstated.
///
/// Like R1 this is a **population** bound and certifies nothing about any individual def.
#[test]
fn out_of_set_phrasings_are_bounded() {
    let found = defs_naming_a_live_identifier_out_of_set_only();
    assert!(
        found.len() <= OUT_OF_SET_LIVE_MENTION_CEILING,
        "{} card defs name a LIVE identifier inside a gap assertion phrased OUTSIDE \
         GAP_NEEDLES, above the ratchet of {OUT_OF_SET_LIVE_MENTION_CEILING}. These are \
         invisible to R1 and R3 — this gate's second recall bound. Prefer rewording the \
         new note into the primary vocabulary (`does not exist`, `not expressible`, `no \
         such`, `lacks`, `has no`) so R1 can see it; raise this ceiling only with a stated \
         reason.\n\n  {}",
        found.len(),
        found
            .iter()
            .map(|(f, live)| format!(
                "{f} — {}",
                live.iter().cloned().collect::<Vec<_>>().join(", ")
            ))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    assert!(
        !found.is_empty(),
        "0 out-of-set live mentions — either the corpus became fully compliant (lower the \
         ceiling and say so) or `asserts_a_gap_out_of_set`/`dsl_mentions` has stopped \
         matching and this ratchet is vacuous"
    );
}
