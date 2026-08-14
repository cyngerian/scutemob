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
//! marker or appear in the reviewed [`ALLOWLIST`] (or the mechanically-frozen
//! [`RECORDED_BASELINE`]) below. `tools/authoring-report.py`
//! reports the same drift, but it is advisory and not in CI; this is the machine
//! gate.
//!
//! ## Why a source scan rather than a runtime check
//!
//! The deviation is documented in a *comment*, which does not survive into the
//! compiled `CardDefinition`. The only place the intent is legible is the source
//! text, so the gate reads the source — the same technique SR-5's keyword
//! registry and SR-8's protocol fingerprint use.
//!
//! ## OOS-CARDS2-7 (fixed here, PB-DX8): the needles were invented, not derived
//!
//! [`DEVIATION_NEEDLES`] used to be five phrases the *gate author* thought of
//! ("simplif", "modeled as"/"modelled as", "deviation", "approximat"). Measured
//! at the time of the fix: those five needles reached only **13** unmarked
//! (`Complete`-by-default) defs, all thirteen already in [`ALLOWLIST`] — i.e.
//! the gate had never once found a real, un-reviewed offender since it shipped.
//! The corpus's actual deviation vocabulary is different: defs carrying "DSL
//! gap" / "deferred" / "TODO" / "not expressible" / "cannot be expressed" /
//! "unsupported" in their own prose shipped `Complete` and reddened nothing.
//! The binding lesson (seed-rerank memo §2.6): **derive the category from the
//! thing being checked, not from the checker.**
//!
//! ### The derivation rule (re-run it with the corpus, not with this comment)
//!
//! Candidate needles are well-formed 1-3-word n-grams (every token >= 2 letters,
//! alphabetic-or-hyphen, apostrophes split away) drawn from the corpus's own
//! declared-deviation vocabulary and filtered in three steps:
//!
//! - **D1.** A candidate must occur inside at least 10 *distinct*
//!   `Completeness::{partial,known_wrong,inert}("...")` note strings — the only
//!   construct in the corpus whose entire purpose is to declare a departure from
//!   the printed card, so its vocabulary IS the corpus's deviation vocabulary.
//! - **D2.** Matched against ALL author prose (defined below as the union of
//!   every `//` comment body and every completeness note string — see "why
//!   prose, not raw source" below) across the corpus, a candidate survives iff
//!   it appears in >= 8 distinct defs and >= 95% of those defs are *already*
//!   marked non-`Complete`. This is the precision floor: a needle whose hits are
//!   mostly on defs that already declare themselves incomplete is a real
//!   deviation-vocabulary word: one that hits mostly-`Complete` defs is noise.
//! - **D3.** Drop any survivor subsumed by a shorter surviving needle (keep the
//!   shortest generative form — e.g. "not expressible" subsumes nothing shorter
//!   in this corpus, but a longer phrase containing an already-kept shorter one
//!   is dropped).
//!
//! This produced **34** needles ("TIER A" below), measured 2026-08-12 against
//! the 1,803-def corpus (670 marked / 1,133 unmarked at the time).
//!
//! ### D2's precision floor has a known confound, and it is stated rather than implied
//!
//! **D2 measures concentration, and concentration is partly base rate.** Marked
//! defs carry far more comment prose than unmarked ones (median ~121 words vs
//! ~49), so an ordinary English word that authors happen to write while
//! explaining a blocker clears 95% largely by *volume*, not by meaning. Six
//! TIER A needles are of that kind — `should`, `needs`, `complex`, `expression`,
//! `executes`, `tracking` — and the `/review` cycle proved the consequence by
//! execution: the innocuous comment
//! `// Straightforward: this should be a plain damage spell, no special handling
//! needed.` on `lightning_bolt.rs` reddens both the gate and the ratchet.
//!
//! **The consequence is a real cost, not a theoretical one**: routine authoring
//! will push benign defs toward the mechanical [`RECORDED_BASELINE`] exit rather
//! than toward a marker, diluting the signal over time. It is left in rather than
//! tuned away because dropping a needle for being inconvenient is precisely the
//! defect this batch exists to remove (zero silent needle tuning), and because a
//! minimum-specificity criterion in D3 would itself be an author's judgement
//! smuggled back into a derived rule. **The honest disposition is: this is a
//! measured precision bound of D2, the next batch may add a specificity criterion
//! with its own stated rule, and until then a `RECORDED_BASELINE` entry whose
//! reason names one of the six should be read as "ordinary English tripped the
//! scan", not as "this def declares a gap".** One baseline entry (`tyrranax_rex`)
//! already says exactly that on its own row.
//!
//! ### The headline finding: the derivation does NOT rediscover the seed's own six
//!
//! The seed that reported this defect (OOS-CARDS2-7) named six phrases by hand:
//! "dsl gap", "deferred", "todo", "not expressible", "cannot be expressed",
//! "unsupported" — reaching **35** unmarked defs (dispatch hygiene 6: a filed
//! scope is a floor, not a census). The D1-D3 derivation above reaches **31**
//! unmarked defs via 34 needles, and **the two sets are not nested**: 14 defs
//! are reachable only through the seed's own six, and 10 only through the
//! derived 34. Their union is **45** unmarked defs.
//!
//! Why the derivation misses "todo" and "deferred": D1 keys on `Completeness`
//! **note strings** — the one construct whose entire purpose is declaring a
//! departure. "TODO" and "deferred" overwhelmingly live in `// TODO:` **source
//! comments**, not in compiled `Completeness` notes, so D1's 10-distinct-notes
//! floor never sees them (`todo` clears 568 comment-prose hits but essentially
//! zero note-string hits; `deferred` similarly). **A derivation keyed on one
//! declaration construct is short by exactly the failure mode OOS-CARDS2-7
//! names, reproduced inside the fix for it.** The shipped needle set is
//! therefore the measured UNION of two tiers, each recorded with its own
//! derivation and its own measured `(prose_defs, marked, unmarked)` triple —
//! see the per-needle comments on [`DEVIATION_NEEDLES`] below.
//!
//! - **TIER A**: the 34 D1-D3 needles (note-side vocabulary).
//! - **TIER B**: the seed's own six (comment-side vocabulary). Two of the six —
//!   `"dsl gap"` and `"not expressible"` — are *already* TIER A members
//!   (measured identical, not merely asserted: same needle string, same hits).
//!   A third, `"cannot be expressed"`, is a genuinely distinct phrase that never
//!   independently reaches an unmarked def not already reached by `"dsl gap"`
//!   (measured: its one unmarked hit, `slickshot_show_off`, also matches `"dsl
//!   gap"`), so it contributes zero to the reachable population and is *not*
//!   duplicated into the literal needle array below — but it IS part of the
//!   seed's original six and is recorded here for that reason. The three
//!   genuinely net-new members are `"deferred"`, `"todo"`, `"unsupported"`.
//!   `"deferred"` is the low-precision member of the six (0.836 concentration,
//!   12 unmarked hits) — stated rather than implied uniform, because the other
//!   five all clear 0.93+.
//!
//! Measured triples (`prose_defs`, `marked`, `unmarked`), all six, for the
//! record even where a needle does not appear a second time in the literal
//! array below:
//!
//! | needle | prose_defs | marked | unmarked | tier |
//! |---|---|---|---|---|
//! | `dsl gap` | 246 | 234 | 12 | A (also seed) |
//! | `deferred` | 73 | 61 | 12 | B (net new) |
//! | `todo` | 568 | 557 | 11 | B (net new) |
//! | `not expressible` | 124 | 121 | 3 | A (also seed) |
//! | `cannot be expressed` | 16 | 15 | 1 | seed only, subsumed by `dsl gap` |
//! | `unsupported` | 3 | 2 | 1 | B (net new) |
//!
//! ### Why the scan reads *prose*, not the whole raw file (a second bug caught deriving this fix)
//!
//! The original `has_deviation_language` scanned the ENTIRE lowercased file
//! source, not just comments. That was invisible with the original five
//! needles (none of them ever collide with a Rust identifier), but two of the
//! newly-derived TIER A needles do: `"drawcards"` is also the literal DSL variant
//! name `Effect::DrawCards`, and `"partial"` is also the literal marker
//! constructor `Completeness::partial(`. MEASURED: scanning the whole file
//! instead of prose, `"drawcards"` alone reaches **127** unmarked defs and its
//! precision crashes from the derivation's own 95% concentration floor down to
//! **37%** (76 marked / 203 total) — it would not have survived D2 under a
//! fair, apples-to-apples full-source measurement, and shipping it unchanged
//! would have silently blown the whole 45-def population this file's
//! [`RECORDED_BASELINE`] freezes. The fix: `has_deviation_language` now scans
//! *author prose* — every `//` comment body plus every completeness note string
//! — which is also this file's own original rationale ("the deviation is
//! documented in a *comment*", above) finally made literal. See
//! [`author_prose`] and `scanner_ignores_needles_that_appear_only_in_code_not_prose`
//! for the executed proof. The five legacy needles are unaffected: MEASURED
//! identical hit sets under full-source and prose-only scanning (18/32/2/16/79
//! files respectively, byte-for-byte the same defs both ways).
//!
//! No engine or wire change: this file lives under `crates/engine/tests/`,
//! outside every `SCAN_ROOTS` PROTOCOL/HASH gate.

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

/// Deviation-language needles, matched against [`author_prose`] (lower-cased).
/// A card-def whose author PROSE contains any of these is claiming (or denying)
/// a departure from the printed card and must account for it — marker,
/// [`ALLOWLIST`], or [`RECORDED_BASELINE`].
///
/// Three tiers, concatenated into one flat array because `has_deviation_language`
/// does not care which tier fired — see the module doc for how each tier was
/// derived and its measured `(prose_defs, marked, unmarked)` triple.
const DEVIATION_NEEDLES: &[&str] = &[
    // ── Legacy (SR-12, 2026-07-10). Reviewed, five items, unchanged. ──
    "simplif",     // "Simplified", "simplification"
    "modeled as",  // US spelling
    "modelled as", // UK spelling
    "deviation",   // "deviation from the oracle text"
    "approximat",  // "approximate", "approximation"
    // ── TIER A (derived 2026-08-12, OOS-CARDS2-7 fix, PB-DX8, rule D1-D3) ──
    // Sorted by measured prose_defs descending, matching the derivation's own
    // print order, so this array is a faithful transcript of the derivation run.
    "dsl gap",             // prose_defs=246 marked=234 unmarked=12 (== TIER B seed member)
    "in dsl",              // prose_defs=150 marked=146 unmarked=4
    "needs",               // prose_defs=146 marked=145 unmarked=1
    "not expressible",     // prose_defs=124 marked=121 unmarked=3 (== TIER B seed member)
    "blocked on",          // prose_defs=99  marked=99  unmarked=0
    "in the dsl",          // prose_defs=86  marked=82  unmarked=4
    "no effect",           // prose_defs=64  marked=64  unmarked=0
    "expressible in",      // prose_defs=53  marked=51  unmarked=2
    "interactive",         // prose_defs=53  marked=51  unmarked=2
    "partial", // prose_defs=52  marked=50  unmarked=2 (prose-only; full-source is 441/439/2 — see module doc)
    "should",  // prose_defs=46  marked=44  unmarked=2
    "does not exist", // prose_defs=45  marked=43  unmarked=2
    "is expressible", // prose_defs=45  marked=43  unmarked=2
    "gaps",    // prose_defs=42  marked=42  unmarked=0
    "there is no", // prose_defs=38  marked=37  unmarked=1
    "lacks",   // prose_defs=35  marked=35  unmarked=0
    "tracking", // prose_defs=32  marked=31  unmarked=1
    "complex", // prose_defs=29  marked=29  unmarked=0
    "exists but", // prose_defs=25  marked=25  unmarked=0
    "cost variant", // prose_defs=21  marked=21  unmarked=0
    "drawcards", // prose_defs=20  marked=19  unmarked=1 (prose-only; full-source is 203/76/127 — see module doc)
    "no triggercondition", // prose_defs=20 marked=20  unmarked=0
    "targetfilter has", // prose_defs=19  marked=19  unmarked=0
    "choose is", // prose_defs=18  marked=18  unmarked=0
    "expression", // prose_defs=18  marked=18  unmarked=0
    "remaining blocker", // prose_defs=17  marked=17  unmarked=0
    "maypayorelse", // prose_defs=15  marked=15  unmarked=0
    "no condition", // prose_defs=15  marked=15  unmarked=0
    "rewire",    // prose_defs=15  marked=15  unmarked=0
    "old note",  // prose_defs=13  marked=13  unmarked=0
    "trigger not", // prose_defs=13  marked=13  unmarked=0
    "inexpressible", // prose_defs=12  marked=12  unmarked=0
    "executes",  // prose_defs=11  marked=11  unmarked=0
    "exists and", // prose_defs=10  marked=10  unmarked=0
    // ── TIER B net-new (the seed's own six, minus the three already present
    // above as TIER A members or as a subsumed duplicate — see module doc). ──
    "deferred", // prose_defs=73 marked=61 unmarked=12 (low-precision member, 0.836 concentration)
    "todo",     // prose_defs=568 marked=557 unmarked=11
    "unsupported", // prose_defs=3  marked=2   unmarked=1
];

/// Extract every string-literal body passed to `Completeness::partial("…")`,
/// `Completeness::known_wrong("…")` or `Completeness::inert("…")`. No `regex`
/// dependency in this crate's test target — a manual, char-boundary-safe scan
/// (uses `str::find`/`char_indices`, never raw byte-to-char casts, so a
/// multi-byte character in a note, e.g. an em dash, cannot corrupt a later
/// slice).
fn completeness_note_bodies(src: &str) -> Vec<String> {
    const CTORS: &[&str] = &[
        "Completeness::partial(",
        "Completeness::known_wrong(",
        "Completeness::inert(",
    ];
    let mut out = Vec::new();
    for ctor in CTORS {
        let mut tail = src;
        while let Some(rel) = tail.find(ctor) {
            let after_ctor = &tail[rel + ctor.len()..];
            let Some(quote_rel) = after_ctor.find('"') else {
                break;
            };
            let after_quote = &after_ctor[quote_rel + 1..];
            let mut body = String::new();
            let mut consumed_end = after_quote.len();
            let mut escape = false;
            let mut closed = false;
            for (i, c) in after_quote.char_indices() {
                if escape {
                    body.push(c);
                    escape = false;
                    continue;
                }
                if c == '\\' {
                    escape = true;
                    continue;
                }
                if c == '"' {
                    consumed_end = i + c.len_utf8();
                    closed = true;
                    break;
                }
                body.push(c);
            }
            out.push(body);
            tail = if closed {
                &after_quote[consumed_end..]
            } else {
                ""
            };
        }
    }
    out
}

/// "Author prose": the body of every `//` line comment, every `/* … */` block
/// comment, and every [`completeness_note_bodies`] string, lower-cased. This is
/// the corpus's own deviation vocabulary — deliberately narrower than the whole
/// file. See the module doc, "Why the scan reads prose, not the whole raw file",
/// for the measured reason a full-source scan is wrong (it collides with DSL
/// identifiers like `Effect::DrawCards` and the `Completeness::partial(`
/// constructor itself).
///
/// **Block comments are included because the `/review` cycle proved their omission
/// by execution.** Narrowing from whole-source to `//`-only was a coverage
/// *regression* nobody had measured: the identical sentence
/// `known dsl gap: …` reddens the gate as `// known dsl gap: …` and leaves all
/// tests green as `/* known dsl gap: … */`. That is `OOS-DX32-6`'s class exactly —
/// a `/* */` wrapper leaving a gate green — and it was latent rather than live only
/// because the corpus happens to carry **zero** deviation-language block comments
/// today (all 12 `/*` occurrences under `defs/` are `*/*` power/toughness notation
/// **inside** `//` comments). Latent is not the same as absent, and a fix that
/// depends on a coincidence is not a fix. Pinned by
/// [`block_comments_are_prose_too`].
fn author_prose(src: &str) -> String {
    let mut out = String::new();
    for line in src.lines() {
        if let Some(idx) = line.find("//") {
            out.push_str(&line[idx + 2..]);
            out.push('\n');
        }
    }
    for body in block_comment_bodies(src) {
        out.push_str(&body);
        out.push('\n');
    }
    for note in completeness_note_bodies(src) {
        out.push_str(&note);
        out.push('\n');
    }
    out.to_lowercase()
}

/// The body of every `/* … */` block comment in `src`.
///
/// Deliberately simple and deliberately over-inclusive at the margin: it does not
/// track string literals, so a `/*` inside a string would open a span. That errs
/// toward *more* prose reaching the scan, which is the safe direction for a gate
/// whose failure mode is missing a declared gap — and the corpus contains no such
/// literal (checked: every `/*` under `defs/` is `*/*` P/T notation inside a `//`
/// comment, which this function's own `//`-stripping pass has already consumed).
/// Nested block comments are not tracked either; Rust allows them, the corpus has
/// none, and the outer span still yields the inner text.
fn block_comment_bodies(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i + 2;
            let mut j = start;
            while j + 1 < bytes.len() && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
                j += 1;
            }
            let end = if j + 1 < bytes.len() { j } else { bytes.len() };
            // Char-boundary safe: slice on the nearest boundaries at or inside the span.
            let (mut s0, mut e0) = (start.min(src.len()), end.min(src.len()));
            while s0 < src.len() && !src.is_char_boundary(s0) {
                s0 += 1;
            }
            while e0 > s0 && !src.is_char_boundary(e0) {
                e0 -= 1;
            }
            if s0 < e0 {
                out.push(src[s0..e0].to_string());
            }
            i = end + 2;
        } else {
            i += 1;
        }
    }
    out
}

/// Non-`Complete` marker fragments. Presence of any means the def already
/// declares itself incomplete, so its deviation language is accounted for.
///
/// Both the constructor form (`Completeness::partial("…")`, the form the whole
/// corpus uses) and the bare variant form (`Completeness::Partial`) are matched,
/// so the gate does not depend on authoring style. This deliberately scans the
/// raw (non-lowercased, non-prose-restricted) source: a marker is CODE, not
/// prose, and must be found wherever it appears, including inside a `#[derive]`
/// or a struct-literal field — matching `read_def_sources`'s original behavior.
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
    // ── PB-DX27 (`scutemob-209`, 2026-08-13), OOS-CARDS2-8 / OOS-RR3-2 ──
    //
    // Five defs whose prose describes a blocker claim that PB-DX27 REFUTED and
    // REPAIRED. The matched needles are all inside a historical sentence recording
    // that a previous note was wrong — the `hazorets_monument` / `reforge_the_soul`
    // shape, one step further: those describe a superseded *modeling*, these describe
    // a superseded *gap claim*.
    //
    // The record is deliberately kept rather than deleted, and that is the whole
    // point of the batch: the reason these notes went stale is that nobody revisits a
    // blocker claim when the primitive lands, so the file now says in-place that the
    // claim was checked and found false. `greater_good.rs` is the corpus's existing
    // model for this and sits in RECORDED_BASELINE for the same reason.
    (
        "chord_of_calling",
        "\"claimed a DSL gap that had already closed\" — historical record of the \
         refuted `// TODO: max_cmc should be XValue`. TargetFilter.max_cmc_amount \
         shipped with PB-EF10; the clause is now authored and the def is Complete. \
         Describes a repaired claim, not a live deviation.",
    ),
    // `green_suns_zenith` was listed here by the implement phase and is REMOVED by the
    // /review fix cycle: it is no longer `Complete`, so the deviation scan (which only
    // examines unmarked defs) never reaches it, and `every_allowlist_entry_is_live_and_
    // necessary` correctly fails on a row that has stopped doing anything. Its second
    // printed clause turned out to be unauthored — `self_shuffle_on_resolution` places
    // deterministically on top of the library rather than shuffling — so it carries a
    // `partial` marker now and needs no exemption.
    (
        "wight_of_the_reliquary",
        "\"Cost::SacrificeAnother does not exist\" survives only as the record of a \
         refuted claim: TargetFilter.exclude_self is lowered onto the activation cost \
         (replay_harness.rs) and CR 109.1-enforced (rules/abilities.rs), so the \
         ability is authored and the def is Complete.",
    ),
    (
        "chandra_flamecaller",
        "\"EffectAmount::HandSize not in DSL\" survives as a record with its own \
         correction: the identifier DOES exist and was still the WRONG primitive — a \
         naive DiscardCards{HandSize}+DrawCards{HandSize} reads 0 (effects/mod.rs \
         says so in-source), which is why Effect::WheelHand exists. Authored with \
         WheelHand; Complete. Also removed an activatable Effect::Nothing loyalty \
         ability (W5 wrong game state).",
    ),
    (
        "voldaren_epicure",
        "the prose records that this def shipped Complete and deck-legal while \
         SILENTLY DROPPING its first printed sentence (\"it deals 1 damage to each \
         opponent\"), and that the damage half was always expressible via \
         EffectTarget::EachOpponent. The clause is now authored; the def stays \
         Complete. OOS-CARDS2-10.",
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

/// Mechanically-frozen acknowledgement of the widened needle set (OOS-CARDS2-7,
/// PB-DX8, 2026-08-12). File stem plus a reason quoting the needle(s) that
/// matched and the substantive fragment of the def's own prose that matched it.
///
/// **This roster was populated MECHANICALLY from measurement, not adjudicated
/// against oracle text def-by-def** — the same honesty correction PB-DX4's own
/// review forced on `decision_gate.rs`'s `BASELINE` (see that file's `T4`
/// module doc, "PB-DX4 ... performed that triage"): an entry here asserts only
/// that this def's prose matched a needle and the def still ships `Complete`.
/// It asserts NOTHING about whether the deviation is real, faithful, or already
/// fixed. A handful of entries below were read closely enough while writing
/// this freeze to flag as likely candidates for the reviewed [`ALLOWLIST`]
/// instead (their own reason text says so) — they are frozen here anyway,
/// deliberately, rather than silently reclassified without the review that
/// class of decision needs.
const RECORDED_BASELINE: &[(&str, &str)] = &[
    (
        "akroma_angel_of_fury",
        "Matched \"dsl gap\": \"was left false behind a // stale 'DSL gap' comment even though \
         the field exists (see tyrranax_rex.rs)\" — the def's own comment says the DSL gap it \
         quotes is stale, but the phrase itself still trips the needle.",
    ),
    (
        "alela_cunning_conqueror",
        "Matched \"tracking\", \"deferred\": \"'First spell per turn' tracking deferred \
         (requires per-turn state counter)\" — a real, live gap: the engine has no per-turn \
         first-spell tracker, so the goad-on-first-spell clause cannot fire selectively.",
    ),
    (
        "arcanis_the_omnipotent",
        "Matched \"dsl gap\": \"Wrong under Bribery/steal effects (systemic DSL gap, not \
         Arcanis-specific)\" — an ownership-vs-control gap in the tap ability's return \
         destination, corpus-wide class, not unique to this card.",
    ),
    (
        "archetype_of_endurance",
        "Matched \"not expressible\", \"expressible in\", \"todo\": \"The 'can't have or gain \
         hexproof' prevention is not expressible in the current DSL ... the prevention \
         sub-clause is left as a TODO\" — the RemoveKeyword half ships; the keyword-lock half \
         does not.",
    ),
    (
        "archetype_of_imagination",
        "Matched \"not expressible\", \"expressible in\", \"todo\": \"The 'can't have or gain \
         flying' prevention is not expressible in the current DSL ... the prevention \
         sub-clause is left as a TODO\" — identical shape to archetype_of_endurance.rs, \
         different keyword.",
    ),
    (
        "archon_of_emeria",
        "Matched \"deferred\": \"DEFERRED (PB-18 review Finding 5): 'Nonbasic lands your \
         opponents control enter tapped.'\" — the max-one-spell-per-turn clause ships; the \
         land-tapped clause is an unimplemented printed ability, not merely a modeling choice.",
    ),
    (
        "awakening_zone",
        "Matched \"in dsl\": \"Note: 'you may' optional not in DSL — always creates (bot \
         always opts in)\" — the printed token creation is optional; the engine creates it \
         unconditionally every upkeep.",
    ),
    (
        "birthing_pod",
        "Matched \"is expressible\", \"unsupported\": the file's own comment narrates BOTH \
         blockers as CLOSED (\"Both blockers are now closed; this card flips to Complete\") — \
         this reads as a HISTORICAL fix-log, not a live deviation, and is flagged here as a \
         likely-faithful case worth a future ALLOWLIST review rather than adjudicated now.",
    ),
    (
        "blight_mound",
        "Matched \"dsl gap\": \"the death-trigger lifegain on the Pest token is a known DSL \
         gap (token_triggered_abilities)\" — a token created by this permanent is missing its \
         own printed triggered ability.",
    ),
    (
        "bootleggers_stash",
        "Matched \"todo\": \"Authored 2026-04-12 (PB-N stale-TODO sweep): unblocked by PB-S \
         (LayerModification::AddActivatedAbility + EffectFi...)\" — a TODO sweep note; check \
         whether the referenced primitive fully closed the gap before demoting.",
    ),
    (
        "brutal_cathar",
        "Matched \"dsl gap\": \"DSL gap: Ward—Pay 3 life (Ward(u32) only supports mana costs)\" \
         — the printed life-cost Ward variant has no DSL expression; only mana-cost Ward exists.",
    ),
    (
        "complete_the_circuit",
        "Matched \"in the dsl\", \"partial\", \"deferred\": \"PARTIAL (PB-J): \
         Effect::CopySpellOnStack is now available in the DSL. The remaining gap is the 'When \
         you NEXT cast an instant or sorcery spell THIS TURN' delayed trigger ... deferred \
         until the delayed-spell-cast-trigger primitive is added\" — the flash grant ships; the \
         copy-next-spell delayed trigger does not.",
    ),
    (
        "cruel_celebrant",
        "Matched \"dsl gap\": \"fires on creature deaths, not planeswalker deaths. Known DSL \
         gap\" — CR-correct oracle text also drains on a planeswalker dying; the trigger \
         condition is creature-only.",
    ),
    (
        "cryptic_command",
        "Matched \"in the dsl\": \"all four sub-effects exist in the DSL but per-mode targeting \
         did not\" — describes the historical stub this def has since replaced; retained by \
         the needle because the sentence still narrates the DSL's shape using the phrase.",
    ),
    (
        "deflecting_swat",
        "Matched \"interactive\", \"deferred\": \"Interactive choice deferred to M10\" (PB-DX25b \
         review Finding C3) — the object-target redirect branch falls back to an unchanged \
         target rather than offering the controller a live choice.",
    ),
    (
        "den_of_the_bugbear",
        "Matched \"in dsl\": \"granting triggered abilities via layers is not in DSL \
         (AddTriggeredAbility missing)\" — the animated creature's printed \"gets +1/+0 and \
         has haste\" ships; a menace-granting layer effect referenced elsewhere in the comment \
         does not.",
    ),
    (
        "dreadhound",
        "Matched \"dsl gap\", \"partial\": \"Partial: only creature deaths fire \
         (WheneverCreatureDies). 'Creature card put into GY from library' (mill trigger) is a \
         known DSL gap\" — the printed trigger has two clauses (dies OR milled); only the \
         first is expressible.",
    ),
    (
        "general_kreat_the_boltbringer",
        "Matched \"todo\": \"PB-OS11 (forced add, self-identified TODO): 'Whenever one or more \
         Goblins you control attack' is a BATCH trigger\" — a batch-attack trigger condition \
         the def's own note flags as a forced add pending review.",
    ),
    (
        "ghave_guru_of_spores",
        "Matched \"deferred\": \"the source permanent (Ghave itself). This is a limitation \
         deferred to PB-37\" — a counter-removal-source restriction the def does not enforce.",
    ),
    (
        "gingerbrute",
        "Matched \"dsl gap\": \"{1}: can't be blocked this turn except by haste creatures — DSL \
         gap (filtered evasion)\" — the sacrifice-for-life ability ships; the conditional \
         evasion activated ability does not.",
    ),
    (
        "greater_good",
        "Matched \"in the dsl\", \"todo\": \"Note: Effect::DiscardCards exists in the DSL; the \
         prior TODO claiming it was missing was stale\" — describes a stale TODO the def has \
         since corrected; the correcting sentence itself still trips both needles.",
    ),
    (
        "growing_rites_of_itlimoc",
        "Matched \"deferred\": \"PB-OS8 (closes PB-OS6 deferred sub-primitive (d)): the ETB \
         'look at top four cards, may reveal a creature...'\" — describes a since-closed \
         deferral; the historical phrasing still trips the needle.",
    ),
    (
        "hands_of_binding",
        "Matched \"deferred\": \"Note: TargetRequirement::TargetCreature does not restrict to \
         'an opponent controls' — that is a pre-existing card-def gap unrelated to this batch; \
         scope deferred\" — a live, reachable overbroad-target gap (can target your own \
         creature, not just an opponent's).",
    ),
    (
        "hermes_overseer_of_elpis",
        "Matched \"todo\": \"PB-OS11 (forced add, self-identified TODO): 'Whenever you attack \
         with one or more Birds' is a BATCH trigger\" — same batch-attack-trigger shape as \
         general_kreat_the_boltbringer.rs.",
    ),
    (
        "jadar_ghoulcaller_of_nephalia",
        "Matched \"is expressible\": \"That filter is expressible today: \
         `Condition::YouControlNOrMoreWithFilter` with a Creature + Decayed-keyword \
         TargetFilter, negated. ... See the fixed intervening_if below\" — this reads as a \
         HISTORICAL note describing an ALREADY-CLOSED gap (PB-DX3b), flagged here as a \
         likely-faithful case worth a future ALLOWLIST review rather than adjudicated now.",
    ),
    (
        "kolaghan_the_storms_fury",
        "Matched \"in dsl\": \"PB-N: Dragon subtype filter now in DSL via filter: \
         Some(TargetFilter { has_subtype })\" — a historical unblocking note; the past-tense \
         'now in DSL' phrasing still trips the needle.",
    ),
    (
        "korvold_fae_cursed_king",
        "Matched \"there is no\": \"Split into two separate triggers (there is no combined \
         enters-or-attacks TriggerCondition); each is an exact translation...\" — reads like the \
         faithful-decomposition pattern the reviewed ALLOWLIST already covers for \
         tainted_field.rs and siblings, flagged here as a likely-faithful case worth a future \
         ALLOWLIST review rather than adjudicated now.",
    ),
    (
        "land_tax",
        "Matched \"interactive\": \"'up to three' follows the established engine convention ... \
         of a deterministic auto-search ... naturally implementing 'up to three' without a real \
         interactive choice model\" — describes a deliberate, corpus-standard modeling choice; \
         flagged here as a likely-faithful case worth a future ALLOWLIST review rather than \
         adjudicated now.",
    ),
    (
        "mistblade_shinobi",
        "Matched \"deferred\", \"todo\": \"TODO(MayEffect): 'you may' optionality deferred to a \
         future MayEffect primitive; authoring as mandatory is correct...\" — the printed \
         optional bounce is authored as mandatory.",
    ),
    (
        "obelisk_of_urd",
        "Matched \"deferred\": \"below is active immediately — with a Triggered form, the \
         choice would be deferred to trigger-resolution, leaving chosen_creature_type=None \
         during...\" — describes a timing choice made to avoid a worse gap; not obviously a \
         live deviation on its own, frozen mechanically per this file's stated policy.",
    ),
    (
        "ophiomancer",
        "Matched \"dsl gap\": \"PB-DX3b: the def's own former note was right that the DSL gap \
         was stale, but wrong about which variant to use\" — a historical correction note; the \
         quoted phrase 'DSL gap' still trips the needle even though the note narrates its own \
         staleness.",
    ),
    (
        "reanimate",
        "Matched \"dsl gap\", \"does not exist\": \"DSL GAP (PB-10 Finding 6): 'You lose life \
         equal to its mana value' requires ... This variant does not exist yet\" — the printed \
         life-loss clause tied to the reanimated card's mana value is unimplemented.",
    ),
    (
        "scavenger_grounds",
        "Matched \"not expressible\": \"{2},{T}, Sacrifice a Desert: Exile all graveyards (not \
         expressible)\" — the def's own top-of-file summary flags the activated ability as \
         unimplemented in the DSL shape it describes.",
    ),
    (
        "sea_gate_restoration",
        "Matched \"drawcards\": \"EffectAmount::Sum(HandSize, Fixed(1)) is resolved once by \
         Effect::DrawCards (resolve_amount is called before the draw loop), so this is NOT the \
         'count cards, then draw, recount' self-referential trap\" — a CR 608.2h correctness \
         note, not a deviation; it names the `DrawCards` effect variant in prose while \
         explaining why the implementation IS faithful. Frozen mechanically per this file's \
         stated policy rather than reclassified without review.",
    ),
    (
        "signal_pest",
        "Matched \"deferred\": \"Blocking restriction (flying/reach only) deferred — no DSL \
         variant\" — the printed \"can only be blocked by creatures with flying or reach\" \
         restriction is unimplemented.",
    ),
    (
        "slayers_stronghold",
        "Matched \"todo\": \"{R}{W},{T}: Target creature +2/+0 vigilance haste (TODO)\" — the \
         land's activated pump ability is entirely unimplemented.",
    ),
    (
        "slickshot_show_off",
        "Matched \"dsl gap\": \"DSL gap: WheneverYouCastSpell has no spell-type filter field, so \
         the 'noncreature spell' condition on the pump trigger cannot be expressed\" — the \
         Plot cost and evasion ship; the noncreature-spell pump trigger is entirely omitted.",
    ),
    (
        "steel_guardian",
        "Matched \"deferred\": this is a SYNTHETIC TEST CARD (no real printed card), whose own \
         header states so: \"All real Living Metal cards ... are Transformers double-faced \
         cards, which require the blocked DFC subsystem (deferred). This synthetic 3/3 Artifact \
         Vehicle ... stands in for testing.\" There is no oracle text to deviate FROM — the \
         needle fired on the design rationale for why the fixture exists, not a claim about \
         this (fictional) card's own fidelity. Frozen mechanically rather than special-cased.",
    ),
    (
        "teneb_the_harvester",
        "Matched \"dsl gap\", \"in the dsl\", \"should\", \"does not exist\": \"DSL GAP (PB-10 \
         Finding 5): 'you may pay {2}{B}. If you do, ...' requires an optional mana payment on \
         triggered abilities ... which does not exist in the DSL yet ... Teneb should only \
         reanimate when the controller pays the additional cost\" — the trigger fires and \
         reanimates unconditionally; the optional-payment gate is unimplemented.",
    ),
    (
        "tyrranax_rex",
        "Matched \"should\": \"The rule the heuristic should have been: **a wrong printed field \
         is reason to re-read the whole oracle, not to fix the field.**\" — this is META-PROSE \
         about the AUTHORING PROCESS that fixed a prior mis-cost, not a claim about this card's \
         OWN fidelity; the file's own body states every primitive it needs \"already existed\" \
         and the def \"stays Complete\" as a full repair. A needle over-match on process prose, \
         not a live per-card deviation — frozen here per this file's stated mechanical policy \
         rather than special-cased, since \"should\" fires on ordinary engineering-lesson \
         English as readily as on a real gap.",
    ),
    (
        "vindictive_vampire",
        "Matched \"dsl gap\": \"WheneverCreatureDies is overbroad (fires on all creature \
         deaths, not just 'another creature you control'). DSL gap — no controller/exclusion \
         filter\" — the trigger fires on ANY creature's death, including this creature's own \
         and opponents' creatures, not just \"another creature you control.\"",
    ),
    (
        "vishgraz_the_doomhive",
        "Matched \"deferred\", \"todo\": \"Gets +1/+1 for each poison counter opponents have \
         (CDA — deferred, see TODO below)\" — the characteristic-defining-ability power/toughness \
         boost keyed on opponents' poison counters is unimplemented.",
    ),
    (
        "windbrisk_heights",
        "Matched \"needs\": \"It is also still not deduplicated by creature. Filed as \
         `OOS-DX21-1`; closing it needs the field to become a per-turn accumulation with \
         per-creature dedup\" — a stated, filed, live residual (the extra-combat half of the \
         raid-count tracking gap PB-DX21 only partially closed).",
    ),
    (
        "xenagos_the_reveler",
        "Matched \"in dsl\", \"todo\": \"+1: TODO — count-based mana production (X = creatures \
         you control) not in DSL\" — the +1 loyalty ability's variable mana production is \
         unimplemented.",
    ),
    (
        "yavimaya_hollow",
        "Matched \"todo\": \"{G},{T}: Regenerate target creature (TODO)\" — the land's \
         regenerate-granting activated ability is entirely unimplemented.",
    ),
];

/// The population [`RECORDED_BASELINE`] must equal, EXACTLY — the T6-style
/// two-direction ratchet (`decision_gate.rs::auto_chosen_complete_union_is_ratcheted`).
/// Measured 2026-08-12 against the same corpus snapshot the module doc's table
/// above records (670 marked / 1,133 unmarked of 1,803 total defs). A grower
/// means a new `Complete` def started using deviation language and needs its
/// own entry (or a demotion) in the SAME commit; a shrinker means a frozen
/// entry's def was demoted or fixed and the roster should be pruned to match,
/// in the SAME commit, so the gate keeps the gain rather than silently
/// widening its own slack.
const RECORDED_BASELINE_POPULATION: usize = 45;

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

/// Whether `src`'s author prose ([`author_prose`]) contains any [`DEVIATION_NEEDLES`]
/// entry. Takes raw (non-lowercased) source — the lower-casing happens inside
/// `author_prose`, once, alongside the comment/note extraction, rather than at
/// each call site (OOS-CARDS2-7 fix: the previous call sites lower-cased and
/// passed the WHOLE file, which is exactly the bug this function now prevents).
fn has_deviation_language(src: &str) -> bool {
    let prose = author_prose(src);
    DEVIATION_NEEDLES.iter().any(|n| prose.contains(n))
}

fn has_incomplete_marker(src: &str) -> bool {
    MARKER_FRAGMENTS.iter().any(|m| src.contains(m))
}

// ── The gate ──────────────────────────────────────────────────────────────────

/// The gate's own offender-detection logic, extracted so both the gate test and
/// its non-vacuity probe (`gate_logic_reddens_on_an_unrecorded_deviation`) drive
/// the IDENTICAL code path — the same lesson `decision_gate.rs`'s `offenders()`
/// extraction encodes (review finding PB-DP10 #3: a probe that re-checks
/// something else and never executes the real loop is not a probe of the gate).
fn offenders(sources: &[(String, String)]) -> Vec<String> {
    let allow: std::collections::HashSet<&str> = ALLOWLIST.iter().map(|(f, _)| *f).collect();
    let baseline: std::collections::HashSet<&str> =
        RECORDED_BASELINE.iter().map(|(f, _)| *f).collect();

    sources
        .iter()
        .filter(|(stem, src)| {
            has_deviation_language(src)
                && !has_incomplete_marker(src)
                && !allow.contains(stem.as_str())
                && !baseline.contains(stem.as_str())
        })
        .map(|(stem, _)| stem.clone())
        .collect()
}

#[test]
/// `author_prose` must reach `/* … */` block comments, not only `//` lines.
///
/// Written because the `/review` cycle DEFEATED the first draft with this exact input: the
/// sentence `known dsl gap: …` reddened the gate as a `//` comment and left every test green
/// wrapped in `/* */`. Both halves are asserted here — the positive so the extractor is not
/// vacuously returning nothing, and the negative control so a future "simplification" that drops
/// block comments fails by name instead of silently shrinking the gate's reach.
fn block_comments_are_prose_too() {
    let line = "// known dsl gap: the trigger has no TriggerCondition variant\npub fn card() {}";
    let block =
        "/* known dsl gap: the trigger has no TriggerCondition variant */\npub fn card() {}";
    let neither = "pub fn card() { /* nothing to declare here */ }";

    assert!(
        has_deviation_language(line),
        "a `//` comment carrying deviation language must reach the scan"
    );
    assert!(
        has_deviation_language(block),
        "the IDENTICAL text in a `/* */` block comment must reach the scan too — a `/* */` \
         wrapper leaving a gate green is OOS-DX32-6's class, and this is the input the review \
         used to defeat the first draft of `author_prose`"
    );
    assert!(
        !has_deviation_language(neither),
        "negative control: a block comment with no deviation language must NOT match, so the \
         positive above is about the needles and not about the extractor matching everything"
    );

    // Multi-line and unterminated spans, the two shapes a naive extractor gets wrong.
    assert!(
        has_deviation_language("/* line one\n   known dsl gap here\n*/"),
        "a multi-line block comment's interior lines must reach the scan"
    );
    assert!(
        has_deviation_language("/* known dsl gap and no closing delimiter"),
        "an unterminated block comment must not swallow its own body"
    );
}

#[test]
/// A card def that documents a deviation from its oracle text must not ship as
/// `Complete`. Either it carries a `Partial` / `KnownWrong` (/ `Inert`) marker,
/// or it is a reviewed false positive in [`ALLOWLIST`], or it is a mechanically
/// frozen acknowledgement in [`RECORDED_BASELINE`].
///
/// This is the anti-rot guard for the two marker classes the Inert gate does not
/// cover. A future def that adds a `// Simplified: we ignore the second clause`
/// comment and forgets the marker fails here by name.
fn deviation_language_requires_a_marker_or_allowlist() {
    let offenders = offenders(&read_def_sources());

    assert!(
        offenders.is_empty(),
        "these card defs use deviation language (one of DEVIATION_NEEDLES) but ship as \
         Complete with no marker. Three legal exits, and only three:\n\
         1. Mark the def non-Complete (`completeness: Completeness::partial(\"…\")` / \
            `known_wrong(\"…\")`).\n\
         2. If the language describes faithful modeling rather than a real deviation, add it \
            to ALLOWLIST in this file with a REVIEWED reason.\n\
         3. Otherwise, add it to RECORDED_BASELINE in this file with a reason quoting the \
            matched needle(s) and the substantive fragment that matched — a mechanical \
            acknowledgement, not a review verdict — and raise RECORDED_BASELINE_POPULATION in \
            the SAME commit.\n\nOffenders: {offenders:?}"
    );
}

#[test]
/// [`offenders`] drives the real gate; this probe drives it against a synthetic
/// three-source corpus, never touching the real `defs/` directory, exercising
/// all three "why is this NOT an offender" exits plus the one "this IS an
/// offender" case. Mirrors `decision_gate.rs`'s
/// `t4_gate_logic_reddens_on_a_new_unbaselined_auto_chosen_complete_def`.
fn gate_logic_reddens_on_an_unrecorded_deviation() {
    let sources = vec![
        (
            "synthetic_unmarked_offender".to_string(),
            "// this ability has a known dsl gap in its implementation\npub fn card() {}"
                .to_string(),
        ),
        (
            "synthetic_marked_ok".to_string(),
            "// dsl gap noted here\ncompleteness: Completeness::partial(\"noted\"),".to_string(),
        ),
        (
            ALLOWLIST[0].0.to_string(),
            "// modeled as two things, dsl gap".to_string(),
        ),
        (
            RECORDED_BASELINE[0].0.to_string(),
            "// dsl gap, frozen".to_string(),
        ),
        (
            "synthetic_clean".to_string(),
            "// nothing to see here\npub fn card() {}".to_string(),
        ),
    ];

    let found = offenders(&sources);
    assert_eq!(
        found,
        vec!["synthetic_unmarked_offender".to_string()],
        "exactly one of the five synthetic sources is an offender -- unmarked, unallowlisted, \
         unbaselined, and carrying deviation language: {found:?}"
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
/// The deviation detector must actually fire on the corpus, AND each tier must
/// independently fire — a tier that stopped matching (a typo, a lower-casing
/// bug, a needle silently dropped from the array) would shrink the gate without
/// making it fail outright, since the OTHER tiers would still produce hits.
fn the_deviation_detector_is_not_vacuous() {
    let sources = read_def_sources();
    let hits = sources
        .iter()
        .filter(|(_, src)| has_deviation_language(src))
        .count();
    assert!(
        hits >= 50,
        "deviation detector matched only {hits} files; the corpus is known to contain well \
         over 100. The needle set or the matcher is broken and the marker gate is vacuous"
    );

    let hits_with = |needles: &[&str]| -> usize {
        sources
            .iter()
            .filter(|(_, src)| {
                let prose = author_prose(src);
                needles.iter().any(|n| prose.contains(n))
            })
            .count()
    };

    let legacy = hits_with(&[
        "simplif",
        "modeled as",
        "modelled as",
        "deviation",
        "approximat",
    ]);
    assert!(
        legacy >= 50,
        "the LEGACY tier (5 SR-12 needles) matched only {legacy} files; it should still match \
         well over 100 -- a tier that stopped firing would silently shrink this gate"
    );

    let tier_a = hits_with(&[
        "dsl gap",
        "in dsl",
        "needs",
        "not expressible",
        "blocked on",
        "in the dsl",
        "no effect",
        "expressible in",
        "interactive",
        "partial",
        "should",
        "does not exist",
        "is expressible",
        "gaps",
        "there is no",
        "lacks",
        "tracking",
        "complex",
        "exists but",
        "cost variant",
        "drawcards",
        "no triggercondition",
        "targetfilter has",
        "choose is",
        "expression",
        "remaining blocker",
        "maypayorelse",
        "no condition",
        "rewire",
        "old note",
        "trigger not",
        "inexpressible",
        "executes",
        "exists and",
    ]);
    assert!(
        tier_a >= 300,
        "the TIER A tier (34 derived needles) matched only {tier_a} files; measured at \
         derivation time it matched several hundred -- a tier that stopped firing would \
         silently shrink this gate"
    );

    let tier_b_new = hits_with(&["deferred", "todo", "unsupported"]);
    assert!(
        tier_b_new >= 50,
        "the TIER B net-new tier (\"deferred\"/\"todo\"/\"unsupported\") matched only \
         {tier_b_new} files; \"todo\" alone is measured at 568 -- a tier that stopped firing \
         would silently shrink this gate"
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
    // CARDS-2 THIRD pass (2026-08-02, `scutemob-181`, review fix cycles): 668 -> 670. Two more
    // honest demotions, both from a review that found `Complete` defs implementing text their
    // card does not print: `braided_net` (a repair pass had just *implemented* three invented
    // abilities into it, briefed from the file's own stale comment rather than the oracle) and
    // `birchlore_rangers` (printed "Tap two untapped Elves you control: Add one mana of any
    // colour" has no `Cost` variant; its Morph cost was also `{0}` for a printed `{G}`).
    // RE-MEASURED DIRECTLY, not derived: `all_cards()` reports 1,133 Complete / 670 non-Complete
    // of 1,803, and an independent MARKER_FRAGMENTS grep also reports 670.
    // PB-DX26 (2026-08-11, `scutemob-206`): threshold UNCHANGED at 670 — but not because
    // nothing moved. TWO markers changed in opposite directions and cancelled:
    //   * `sword_of_body_and_mind` `partial` -> `Complete` (a flip UP, the direction this
    //     comment's earlier passes never went): its note named the unimplemented "Equip {2}"
    //     as its ONLY remaining blocker and PB-DX26 authored that ability (`OOS-CARDS1-3`).
    //   * `the_reaver_cleaver` derive-`Complete` -> `partial` (an honest demotion, review
    //     Finding 7): the trigger it grants fires only on damage to a PLAYER while the
    //     printed card says "player or planeswalker", and no exact `TriggerCondition`
    //     variant exists. It had no `completeness` field at all, so nobody had ever ruled
    //     on it — the `aurelia_the_warleader` trap, a fifth time in this table.
    // RE-MEASURED DIRECTLY, not derived: `all_cards()` reports 1,133 Complete / 670
    // non-Complete of 1,803 — the same totals as before the batch, over a different set.
    // A stable count is not evidence that nothing changed.
    // PB-DX8 (2026-08-12, OOS-CARDS2-7 fix): threshold UNCHANGED at 670. This batch is a
    // TEST-ONLY change (`crates/engine/tests/core/completeness_deviation_scan.rs` alone) --
    // it widens which defs the DEVIATION scan flags as needing a marker/ALLOWLIST/
    // RECORDED_BASELINE entry, but touches zero card-def files, so MARKER_FRAGMENTS'
    // own count cannot move from this batch. RE-MEASURED DIRECTLY, not assumed: `all_cards()`
    // reports 1,133 Complete / 670 non-Complete of 1,803, matching PB-DX26's figure exactly.
    // PB-DX27 (2026-08-13, `scutemob-209`): threshold 670 -> **666**, and this is a
    // LOWERING, which every previous entry in this comment block avoided having to make.
    // Cause is reason (2) in the assertion message below, not a detector bug: the batch
    // refuted stale blocker notes and authored the clauses they had been blocking, so five
    // defs were promoted to `Complete` (`chord_of_calling`, `green_suns_zenith`,
    // `reconnaissance`, `wight_of_the_reliquary`, `chandra_flamecaller`) and one was
    // honestly demoted (`qarsi_sadist`, whose second printed clause needs a
    // `TriggerCondition::WhenThisExploitsACreature` that does not exist) -- net -4.
    // RE-MEASURED DIRECTLY, not derived, as this comment block's own rule requires:
    // `tools/authoring-report.py` regenerated against the live corpus reports
    // **1,137 Complete / 666 non-Complete of 1,803 (63.1%)**, up from 1,133 / 670 (62.8%).
    // Note for the next reader: a FALLING marked-count is the healthy direction here only
    // because the promotions are repairs. PB-DX26's entry above records the opposite lesson
    // -- a stable count is not evidence that nothing changed -- and both are true.
    assert!(
        marked >= 666,
        "marker detector matched {marked} files; expected >= 666. This assertion has NO \
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
            has_deviation_language(src),
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

#[test]
/// Every `RECORDED_BASELINE` entry must still (a) name a real file, (b) still
/// match a deviation needle, (c) still be Complete/unmarked (a demoted def
/// passes on the marker and the entry becomes redundant), and (d) carry a real
/// reason, not a stub. Mirrors `every_allowlist_entry_is_live_and_necessary` and
/// `decision_gate.rs`'s `every_baseline_entry_is_live_and_necessary` /
/// `T5`'s reason-length check.
fn every_recorded_baseline_entry_is_live_and_necessary() {
    let sources: std::collections::HashMap<String, String> =
        read_def_sources().into_iter().collect();

    const MIN_REASON_LEN: usize = 60;

    for (stem, reason) in RECORDED_BASELINE {
        let src = sources.get(*stem).unwrap_or_else(|| {
            panic!(
                "RECORDED_BASELINE names {stem:?}, which is not a file under defs/ (reason: \
                 {reason})"
            )
        });
        assert!(
            has_deviation_language(src),
            "RECORDED_BASELINE entry {stem:?} no longer matches any deviation needle — the \
             freeze is dead weight; remove it (reason on file: {reason})"
        );
        assert!(
            !has_incomplete_marker(src),
            "RECORDED_BASELINE entry {stem:?} now carries a non-Complete marker, so it passes \
             the gate on the marker and does not need the freeze entry — remove the redundant \
             entry"
        );
        assert!(
            reason.len() >= MIN_REASON_LEN,
            "RECORDED_BASELINE entry {stem:?}'s reason is too short ({} chars, need >= \
             {MIN_REASON_LEN}); a mechanical acknowledgement still needs to name the matched \
             needle(s) and quote the substantive matching fragment, not be a stub",
            reason.len()
        );
    }
}

#[test]
/// The `RECORDED_BASELINE` population must equal [`RECORDED_BASELINE_POPULATION`]
/// EXACTLY, two directions — mirrors `decision_gate.rs`'s
/// `auto_chosen_complete_union_is_ratcheted` (`T6`). A grower means a new
/// `Complete` def started using deviation language and this gate alone (not
/// `every_recorded_baseline_entry_is_live_and_necessary`, which only checks
/// entries ALREADY present) is what catches it. A shrinker is good news --
/// prune the stale entries `every_recorded_baseline_entry_is_live_and_necessary`
/// will name.
fn recorded_baseline_population_is_ratcheted() {
    assert_eq!(
        RECORDED_BASELINE.len(),
        RECORDED_BASELINE_POPULATION,
        "RECORDED_BASELINE has {} entries but RECORDED_BASELINE_POPULATION says {} -- these \
         two must be updated together in the same commit (mirrors decision_gate.rs's BASELINE \
         ratchet discipline)",
        RECORDED_BASELINE.len(),
        RECORDED_BASELINE_POPULATION
    );

    // Deliberately the RAW corpus population -- unmarked, unallowlisted, needle-matching --
    // with NO further filter down to `baseline`. Filtering by `baseline.contains(..)` here
    // would make this count vacuously equal `RECORDED_BASELINE.len()` on every run regardless
    // of what the corpus actually contains (a real bug this file shipped with and caught by
    // its own revert matrix, V4b below): a NEW offender outside the current baseline would
    // never be counted, so this ratchet would never redden no matter how large the true
    // population grew. The raw count below is exactly what `offenders()` would return if
    // `RECORDED_BASELINE` were empty, which is the population this constant must track.
    let sources = read_def_sources();
    let allow: std::collections::HashSet<&str> = ALLOWLIST.iter().map(|(f, _)| *f).collect();

    let live_population: usize = sources
        .iter()
        .filter(|(stem, src)| {
            has_deviation_language(src)
                && !has_incomplete_marker(src)
                && !allow.contains(stem.as_str())
        })
        .count();

    if live_population > RECORDED_BASELINE.len() {
        panic!(
            "the corpus now has MORE unmarked-and-unallowlisted deviation-language defs \
             ({live_population}) than RECORDED_BASELINE covers ({}) -- deviation_language_\
             requires_a_marker_or_allowlist will already name the new def(s); add entries and \
             raise RECORDED_BASELINE_POPULATION to {live_population} in the SAME commit.",
            RECORDED_BASELINE.len()
        );
    }
    if live_population < RECORDED_BASELINE.len() {
        panic!(
            "the corpus now has FEWER unmarked-and-unallowlisted deviation-language defs \
             ({live_population}) than RECORDED_BASELINE covers ({}) -- good, some frozen defs \
             were demoted or fixed. every_recorded_baseline_entry_is_live_and_necessary will \
             name the stale entries; prune them and lower RECORDED_BASELINE_POPULATION to \
             {live_population} in the SAME commit so the gate keeps the gain.",
            RECORDED_BASELINE.len()
        );
    }
}

#[test]
/// **The reason `has_deviation_language` scans comment/note PROSE rather than
/// the whole file** (OOS-CARDS2-7 fix, PB-DX8, see the module doc "Why the scan
/// reads prose"). Executed proof: a needle that only appears in real Rust CODE
/// (not a `//` comment or a completeness note) must not fire, and the identical
/// text inside a comment must still fire -- the fix narrows what the scanner
/// reads, it does not disable any needle.
fn scanner_ignores_needles_that_appear_only_in_code_not_prose() {
    let code_only =
        "abilities: vec![AbilityDefinition::Activated {\n    effect: Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },\n}],";
    assert!(
        !has_deviation_language(code_only),
        "a DSL variant name matching a needle (\"drawcards\") appearing only in CODE, not a \
         comment, must not trigger the scan -- this is the exact class that made \"drawcards\" \
         reach 127 unmarked defs (95% -> 37% precision) under a full-source scan"
    );

    let same_text_in_comment =
        "// deferred: DrawCards handling needs more work, no real dsl gap yet\nlet x = 1;";
    assert!(
        has_deviation_language(same_text_in_comment),
        "the SAME needle-bearing text inside a `//` comment must still fire -- the fix narrows \
         the scan to prose, it does not disable the needle set"
    );
}

#[test]
/// [`completeness_note_bodies`] must find a needle living ONLY inside a
/// `Completeness::partial("…")` note string, with no `//` comment anywhere in
/// the source -- D1's whole premise is that the note-string vocabulary IS
/// (part of) the corpus's declared-deviation vocabulary, so a needle hiding
/// only there must still be found.
fn author_prose_includes_completeness_note_bodies() {
    let src = "completeness: Completeness::partial(\"this card has a known dsl gap in its \
               targeting\"),";
    assert!(
        has_deviation_language(src),
        "a needle living only inside a Completeness note string (no // comment anywhere in the \
         source) must still be found by author_prose"
    );
}

#[test]
/// [`completeness_note_bodies`] must not run past the end of the source (an
/// unterminated string) and must not confuse an escaped quote (`\"`) inside a
/// note for the note's closing delimiter.
fn completeness_note_extraction_handles_escapes_and_missing_close() {
    let escaped =
        "completeness: Completeness::partial(\"a \\\"quoted\\\" dsl gap inside the note\"),";
    let bodies = completeness_note_bodies(escaped);
    assert_eq!(bodies.len(), 1, "exactly one note body: {bodies:?}");
    assert!(
        bodies[0].contains("dsl gap"),
        "the escaped-quote note body must be extracted whole, past the embedded \\\" pair: \
         {bodies:?}"
    );

    let unterminated = "completeness: Completeness::partial(\"never closes";
    // Must not panic (byte-slice past the end) and must not infinite-loop.
    let _ = completeness_note_bodies(unterminated);
}
