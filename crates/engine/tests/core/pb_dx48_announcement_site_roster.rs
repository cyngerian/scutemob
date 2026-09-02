//! PB-DX48 (`OOS-ENG2-1` ≡ `OOS-ENG2-2`, + `OOS-ENG2-3`): the CR 702.21a
//! announcement-site and Ward-population roster gates.
//!
//! This file is the SECOND axis over the SAME source region
//! `primitives::pb_eng2_targets_announced::every_announcement_site_is_classified`
//! already gates, and it is deliberately not a duplicate of it. That gate walks
//! every site that pushes a `StackObject` onto the stack (the wider "what puts an
//! object on the stack" census — casts, activations, triggers, copies, cascades,
//! and free-casts alike) and asks whether each one is classified as
//! "announces" / "does not announce". THIS file walks the narrower
//! `push_target_announcement(` CALL-SITE axis specifically — every place that
//! already decided to announce and is now asking "does the CR 702.21a Ward
//! dispatch actually SEE this announcement" — which is a property the wider gate
//! cannot see, because it stops at "an announcement happens", not "something
//! consumes it". Neither gate subsumes the other: the wider one would stay green
//! if a `push_target_announcement` call were silently duplicated at one site (its
//! census is by STACK-PUSH site, not by announcement-call site); this one would
//! stay green if a brand-new stack-push site were added that announces nothing at
//! all (it only sees sites that already call the helper).
//!
//! * **r1** — the inverse-method site census as a gate: every
//!   `push_target_announcement(` call site across the six `rules/` files the
//!   function can be called from, keyed by (file, enclosing function, the
//!   in-source `ENG-2 (<marker>, ...)` comment that already uniquely labels every
//!   one of the 12 real sites), each with a REASON. A 13th site — with or without
//!   a marker — is what reddens this row.
//! * **r2** — exactly ONE `GameEvent::PermanentTargeted` CONSTRUCTION site in
//!   `crates/engine/src` (`rules::events::permanent_targeted_events`), as opposed
//!   to the two MATCH-arm destructures (`state/hash.rs`, `rules/abilities.rs`'s
//!   dispatch arm) and the one `matches!` pattern (`rules/abilities.rs`'s
//!   `TargetsAnnounced`/`PermanentTargeted` pair-count helper) that also mention
//!   the variant. Ceiling pinned AT the measurement (1), not above it —
//!   PB-DX45's lesson that a ratchet's slack is its blind spot.
//! * **r3** — the Ward population, PRINTED, walking `all_cards()` (SR-36).
//!   **Corrects the plan's stated 5-member set to 4**: `vein_ripper.rs` mentions
//!   `KeywordAbility::Ward` only inside a `// TODO` explaining why "Ward—Sacrifice
//!   a creature" (a non-mana Ward cost) cannot use that variant — it declares no
//!   `AbilityDefinition::Keyword(KeywordAbility::Ward(_))` at all. That is
//!   `OOS-CARDS2-7` / `OOS-DX47-2`'s exact shape (a source-text grep counting
//!   prose as usage) reproduced inside the batch whose own r5 exists to catch it;
//!   `vein_ripper` is r5's member, not r3's.
//! * **r4** — the `WhenBecomesTarget` / `WhenBecomesTargetByOpponent` population,
//!   PRINTED. **Corrects the plan's "6 defs (5 partial + 1 inert), 0 deck-legal"
//!   to 1 structural member.** A `grep -rln WhenBecomesTarget` genuinely does
//!   return 6 files, but 5 of them only NAME the condition inside a blocker
//!   comment while declaring nothing (SR-36's failure a second time in this same
//!   file) — `r4` pins the 1 real declaration (`goldspan_dragon`, `partial`, 0
//!   deck-legal), `r4b` pins the other 5 as a separate "mentioned, not declared"
//!   list.
//! * **r5** — the INVERSE axis (PB-DX26/DX43/DX45/DX47's lesson, a fifth time): defs
//!   whose printed oracle text carries the whole word "ward" but which declare no
//!   `KeywordAbility::Ward`. **Six members, not one** (the module's first draft
//!   assumed only `vein_ripper`): two more non-mana-cost Ward defs
//!   (`scavenger_regent`, and `brutal_cathar`'s back face), two Cloak/Disguise
//!   reminder-text false-positives already tracked by r6 (`cryptic_coat`,
//!   `lumbering_laundry`), and `innkeeper's_talent` (grants Ward to OTHER
//!   permanents via a static, `inert`). **`brutal_cathar` is `Complete` and
//!   deck-legal** with its printed "Ward—Pay 3 life" wholly unauthored — a LIVE
//!   finding this row surfaces and does not fix.
//! * **r6** — the Disguise/Cloak population. `rules/layers.rs` grants
//!   `KeywordAbility::Ward(2)` to a face-down Disguise/Cloak permanent (CR
//!   702.168a / 701.58a) but the Ward TRIGGERED ability is synthesized only in
//!   `state/builder.rs`, keyed off `spec.keywords` at OBJECT-CONSTRUCTION time — a
//!   card turned or put face down mid-game never re-runs that synthesis. **Corrects
//!   the plan's "0 deck-legal Complete members, gap is latent" to 1, LIVE**:
//!   `cryptic_coat.rs` (`Complete`, deck-legal, no `completeness` field so it
//!   derives `Complete`) resolves `Effect::Cloak` in its own ETB trigger, which
//!   `effects/mod.rs:5319` sets as `obj.face_down_as = Some(FaceDownKind::Cloak)`
//!   on the manifested object — the plan's `grep`-derived "0 Cloak defs" measured
//!   `KeywordAbility::Cloak`, which does not exist as an enum variant; Cloak is an
//!   `Effect`, not a keyword marker. Filed as `OOS-DX48-4`.
//! * **`t_census_report`** — PRINTS every population above so the numbers are
//!   PUBLISHED, never transcribed (PB-DX8's rule).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mtg_engine::{
    all_cards, AbilityDefinition, CardDefinition, Completeness, KeywordAbility, TriggerCondition,
};

use crate::decision_site_walk::{def_contains_variant, is_effectively_complete};

// ─────────────────────────────────────────────────────────────────────────────
// Shared parsing helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Strip `//` line comments. Its own load-bearing status is checked directly by
/// [`r1_source_strips_no_block_comments`] and [`r2_source_strips_no_block_comments`]
/// rather than assumed (`OOS-DX32-6`'s class, PB-DX8's `/review` finding).
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every face's printed text, joined and lowercased.
fn all_oracle_text(def: &CardDefinition) -> String {
    let mut out = def.oracle_text.to_lowercase();
    for face in [def.back_face.as_ref(), def.adventure_face.as_ref()]
        .into_iter()
        .flatten()
    {
        out.push('\n');
        out.push_str(&face.oracle_text.to_lowercase());
    }
    out
}

/// Every ability list a `KeywordAbility::Ward` declaration can hide in: the
/// front face's, and every alternate face's. Mirrors PB-DX47's
/// `all_ability_lists` for the same reason (PB-DX27's `/review`: a `CardFace`
/// carries its own ability list).
fn all_ability_lists(def: &CardDefinition) -> Vec<&[AbilityDefinition]> {
    let mut out: Vec<&[AbilityDefinition]> = vec![def.abilities.as_slice()];
    if let Some(face) = def.back_face.as_ref() {
        out.push(face.abilities.as_slice());
    }
    if let Some(face) = def.adventure_face.as_ref() {
        out.push(face.abilities.as_slice());
    }
    out
}

/// The Ward cost this def declares via `AbilityDefinition::Keyword(KeywordAbility::Ward(n))`,
/// if any, across every face.
fn declared_ward_cost(def: &CardDefinition) -> Option<u32> {
    for abilities in all_ability_lists(def) {
        for ability in abilities {
            if let AbilityDefinition::Keyword(KeywordAbility::Ward(n)) = ability {
                return Some(*n);
            }
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// r1 — the push_target_announcement( call-site census
// ─────────────────────────────────────────────────────────────────────────────

/// **`SITE_SRCS` was DELETED by the `/review` fix cycle, and the deletion is the
/// point.** It hardcoded six `rules/` files, and `push_target_announcement` is
/// `pub(crate)` — so a 13th call site anywhere else in `crates/engine/src` was
/// invisible to `r1`, proven by the reviewer adding one to `rules/combat.rs` and
/// watching this gate stay green. `live_sites` and `r1b` now both walk the whole
/// crate with `walk_rs`, the traversal `r2` already used. Keeping a narrower list
/// beside a wider one is how the two axes came to disagree about their own search
/// space in the first place.

const CALL_NEEDLE: &str = "push_target_announcement(";

/// Byte offsets in the ORIGINAL (non-comment-stripped) `src` of every genuine
/// CALL to `push_target_announcement` — excludes the function's own `fn`
/// definition and any occurrence a `//` comment earlier on the SAME physical
/// line would have hidden (e.g. a doc comment reading
/// `// ...push_target_announcement... `). Deliberately NOT run against
/// [`strip_line_comments`]'s output: every real call site sits directly below an
/// `// ENG-2 (<marker>, ...)` comment that THIS function's caller
/// ([`nearby_marker`]) must still be able to read, so the marker text cannot be
/// stripped before it is found.
fn call_site_offsets(src: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = src[from..].find(CALL_NEEDLE) {
        let at = from + rel;
        let before = &src[..at];
        let trimmed_before = before.trim_end();
        // Exclude the definition site: `...fn push_target_announcement(`.
        if !trimmed_before.ends_with("fn") {
            // Exclude an occurrence a `//` comment earlier on the SAME physical
            // line would hide (a doc comment mentioning the call by name).
            let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_prefix = &src[line_start..at];
            if !line_prefix.contains("//") {
                out.push(at);
            }
        }
        from = at + 1;
    }
    out
}

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

/// How far back from a call site to look for its `ENG-2 (<marker>, ...)` comment.
/// Measured: the widest gap is `engine.rs`'s A13 site, whose six-line comment
/// runs 501 BYTES above the call (not characters -- the file's em-dashes are
/// multi-byte UTF-8, so a naive `str::len()`-based estimate undercounts). This
/// is comfortably more than double that measurement.
const MARKER_WINDOW: usize = 1_200;

/// The `<marker>` out of the nearest preceding `// ENG-2 (<marker>, ...)` comment
/// within [`MARKER_WINDOW`] bytes before `at`, or `"UNMARKED"` if none is found —
/// deliberately not a panic, because an unmarked call site is exactly the shape a
/// 13th site would take, and it must show up as a roster MISMATCH, not a test
/// crash.
fn nearby_marker(src: &str, at: usize) -> String {
    const PREFIX: &str = "ENG-2 (";
    let start = at.saturating_sub(MARKER_WINDOW);
    let mut window_start = start;
    while window_start < at && !src.is_char_boundary(window_start) {
        window_start += 1;
    }
    let window = &src[window_start..at];
    match window.rfind(PREFIX) {
        Some(i) => {
            let tail = &window[i + PREFIX.len()..];
            let marker: String = tail.chars().take_while(|c| *c != ',').collect();
            if marker.is_empty() {
                "UNMARKED".to_string()
            } else {
                marker
            }
        }
        None => "UNMARKED".to_string(),
    }
}

/// One pinned `push_target_announcement` call site.
struct PinnedSite {
    file: &'static str,
    func: &'static str,
    marker: &'static str,
    reason: &'static str,
}

/// The 12 sites, re-verified at HEAD by the inverse method
/// (`memory/primitives/pb-DX48-execution-notes.md` §1): 3 pre-existing emitters +
/// 5 sites PB-DX48 taught the wave loop to dispatch + 4 structurally target-free
/// `OOS-ENG2-3` free-cast sites.
const PINNED_SITES: &[PinnedSite] = &[
    PinnedSite {
        file: "rules/casting.rs",
        func: "handle_cast_spell",
        marker: "S1",
        reason: "emitter, pre-existing (site 1 of 3); unchanged after Part A folded its \
                 hand-rolled loop into the shared helper",
    },
    PinnedSite {
        file: "rules/abilities.rs",
        func: "handle_activate_ability",
        marker: "A1",
        reason: "emitter, pre-existing (site 2 of 3)",
    },
    PinnedSite {
        file: "rules/abilities.rs",
        func: "handle_activate_bloodrush",
        marker: "A4",
        reason: "emitter, pre-existing (site 3 of 3); Part A made the push conditional on \
                 zone_at_cast == Some(Battlefield), inert because check_triggers's own arm \
                 already required Battlefield",
    },
    PinnedSite {
        file: "rules/abilities.rs",
        func: "handle_activate_forecast",
        marker: "A3",
        reason: "MISSING pre-PB-DX48: the emission reached check_and_flush_triggers's scan, \
                 but nothing swept a flush's OWN produced events before the wave loop existed",
    },
    PinnedSite {
        file: "rules/abilities.rs",
        func: "flush_sorted",
        marker: "T6",
        reason: "MISSING pre-PB-DX48 -- flush_sorted's Modular arm",
    },
    PinnedSite {
        file: "rules/abilities.rs",
        func: "flush_sorted",
        marker: "T7",
        reason: "MISSING pre-PB-DX48 -- flush_sorted's main (non-modular) arm, the batch's \
                 headline site (the Fell Specter class, generalized to an object target)",
    },
    PinnedSite {
        file: "rules/abilities.rs",
        func: "handle_scavenge_card",
        marker: "A12",
        reason: "MISSING pre-PB-DX48",
    },
    PinnedSite {
        file: "rules/engine.rs",
        func: "handle_activate_loyalty_ability",
        marker: "A13",
        reason: "MISSING pre-PB-DX48",
    },
    PinnedSite {
        file: "rules/copy.rs",
        func: "resolve_cascade",
        marker: "S2",
        reason: "structurally target-free today (OOS-ENG2-3): targets: vec![] via \
                 trigger_default",
    },
    PinnedSite {
        file: "rules/copy.rs",
        func: "resolve_discover",
        marker: "S3",
        reason: "structurally target-free today (OOS-ENG2-3)",
    },
    PinnedSite {
        file: "rules/resolution.rs",
        func: "resolve_top_of_stack_inner",
        marker: "S4",
        reason: "structurally target-free today (OOS-ENG2-3) -- cipher-copy",
    },
    PinnedSite {
        file: "rules/resolution.rs",
        func: "resolve_top_of_stack_inner",
        marker: "S5",
        reason: "structurally target-free today (OOS-ENG2-3) -- suspend free-cast",
    },
];

/// Every `push_target_announcement` call site, as `(file, func, marker, OFFSET)`.
///
/// # Two `/review` defeats, both executed, both fixed here
///
/// **(a) The set collapsed duplicates.** This returned
/// `BTreeSet<(file, func, marker)>`, so a SECOND call inside an already-marked site
/// collapsed into the same tuple and the `len() == 12` floor — a *set* length —
/// never noticed. The reviewer added a second `push_target_announcement(...)`
/// directly under the `ENG-2 (A3` one in `handle_activate_forecast` and **`r1`
/// stayed green**. That is not a hypothetical: a duplicated announcement is
/// literally the **Ward-fires-twice** shape this batch rejected by execution, so the
/// gate was blind to its own headline defect. The tuple now carries the byte
/// OFFSET, which no two distinct calls can share.
///
/// **(b) The file list was hardcoded.** `push_target_announcement` is
/// `pub(crate)`, so a 13th site anywhere else in `crates/engine/src` was invisible;
/// the reviewer added one to `rules/combat.rs` and `r1` stayed green. That matters
/// concretely rather than theoretically: `OOS-DX48-6` says the next two dispatch
/// sites belong in `effects/mod.rs`, which the hardcoded list did not contain.
/// The scan now walks the whole crate with `walk_rs`, the same traversal `r2`
/// already used — the two axes disagreeing about their own search space was the
/// real defect.
fn live_sites() -> BTreeSet<(String, String, String, usize)> {
    let root = workspace_root();
    let src_root = root.join("crates/engine/src");
    let mut files = Vec::new();
    walk_rs(&src_root, &mut files);
    files.sort();

    let mut out = BTreeSet::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let label = path
            .strip_prefix(&src_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        for at in call_site_offsets(&src) {
            let func = enclosing_fn_name(&src, at);
            let marker = nearby_marker(&src, at);
            out.insert((label.clone(), func, marker, at));
        }
    }
    out
}

fn pinned_sites() -> BTreeSet<(String, String, String)> {
    PINNED_SITES
        .iter()
        .map(|p| (p.file.to_string(), p.func.to_string(), p.marker.to_string()))
        .collect()
}

/// CR 601.2c / 602.2b / 603.3d / 702.21a: every real `push_target_announcement`
/// call site, classified. A 13th site (a new one, or an existing one that loses
/// its `ENG-2 (<marker>, ...)` comment) reddens this row.
#[test]
fn r1_call_site_census_is_pinned() {
    let live = live_sites();
    // The CLASSIFICATION compares on (file, func, marker); the COUNT below uses the
    // offset-carrying set, which is what makes a duplicated call at an already-marked
    // site visible (the `/review`'s defeat (a) -- see `live_sites`).
    let live_classified: BTreeSet<(String, String, String)> = live
        .iter()
        .map(|(f, n, m, _)| (f.clone(), n.clone(), m.clone()))
        .collect();
    let pinned = pinned_sites();
    assert_eq!(
        live_classified,
        pinned,
        "PB-DX48 r1: the push_target_announcement( call-site census moved. A new \
         site must be classified in PINNED_SITES with a stated reason before this \
         row can be re-pinned. live only: {:?}; pinned only: {:?}",
        live_classified.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&live_classified).collect::<Vec<_>>()
    );
    // The COUNT is taken over the offset-carrying set, so two calls inside one marked
    // site are two entries, not one. `/review` defeat (a): as a
    // `BTreeSet<(file, func, marker)>` this collapsed a duplicated announcement --
    // which IS the Ward-fires-twice defect -- into a single element and stayed green.
    assert_eq!(
        live.len(),
        12,
        "r1: expected 12 real push_target_announcement call sites across the WHOLE of \
         crates/engine/src, found {}: {:?}. A duplicated call inside an already-marked \
         site lands here, not in the classification assert above.",
        live.len(),
        live.iter()
            .map(|(f, n, m, at)| format!("{f}::{n}[{m}]@{at}"))
            .collect::<Vec<_>>()
    );
    // No UNMARKED site should be hiding in the live set today -- every one of the
    // 12 carries its own ENG-2 marker in source.
    assert!(
        !live.iter().any(|(_, _, m, _)| m == "UNMARKED"),
        "r1: an UNMARKED call site exists at HEAD; PINNED_SITES cannot classify a \
         site with no marker, so this indicates a corpus not matching the plan's \
         census"
    );
}

/// [`call_site_offsets`]'s same-line-`//`-prefix check is load-bearing for r1:
/// it is what keeps the doc comments naming `push_target_announcement` (this
/// very file's module doc among them, and several `OOS-ENG2-3` comments in
/// source) from being misread as call sites, WITHOUT stripping the `//` text
/// the marker parser (`nearby_marker`) still needs to read. Its bound is stated
/// rather than silent: a `//` LINE comment on the SAME physical line as the
/// call is excluded; a `/* */` BLOCK comment is not, so every file this row
/// scans is asserted free of one (`OOS-DX32-6`'s class; PB-DX47's `r3b`).
#[test]
fn r1b_source_strips_no_block_comments() {
    // `/review` defeat (b) widened `r1` from six hardcoded files to the whole crate,
    // so this bound has to widen with it: checking only `SITE_SRCS` would have left
    // every other file's block comments unguarded the moment `r1` started reading them.
    let root = workspace_root();
    let src_root = root.join("crates/engine/src");
    let mut files = Vec::new();
    walk_rs(&src_root, &mut files);
    files.sort();
    let mut scanned = 0usize;
    for path in &files {
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        scanned += 1;
        let label = path
            .strip_prefix(&src_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        // A naive `raw.contains("/*")` is WRONG here and was measured to be: three
        // files carry the literal `*/*` inside a `//` or `///` comment (a `*/*`
        // creature's P/T in `state/diagnostics.rs` and `rules/layers.rs`, a `grep`
        // glob in `rules/priority.rs`'s doc). Strip line comments FIRST, then look --
        // which is exactly the distinction the gate exists to police.
        let stripped = strip_line_comments(&raw);
        assert!(
            !stripped.contains("/*"),
            "PB-DX48 r1b: `{label}` grew a `/* */` block comment outside a line \
             comment. `strip_line_comments` does not remove those, and
             `call_site_offsets` reads the ORIGINAL source, so a block comment can \
             hide or fake a push_target_announcement( call site from r1. Widen the \
             stripper (`OOS-DX32-6`'s class; PB-DX47's `r3b`)."
        );
    }
    // Non-vacuity: the walk must actually have read the crate, or this row asserts
    // nothing about an empty list.
    assert!(
        scanned >= 40,
        "r1b non-vacuity: only {scanned} files walked under crates/engine/src -- the \
         traversal is not reaching the crate it claims to guard"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r2 — exactly one GameEvent::PermanentTargeted CONSTRUCTION site
// ─────────────────────────────────────────────────────────────────────────────

const PERMANENT_TARGETED_NEEDLE: &str = "GameEvent::PermanentTargeted {";

/// The three fields of `GameEvent::PermanentTargeted` (`rules/events.rs:767`). A
/// construction must mention all three; the parser asserts it rather than assuming
/// it, so a payload change surfaces here instead of silently reclassifying a site.
const PERMANENT_TARGETED_FIELDS: &[&str] =
    &["target_id", "targeting_stack_id", "targeting_controller"];

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/engine -> crates -> workspace root
    p.pop();
    p.pop();
    p
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

/// `(file, offset)` of every `GameEvent::PermanentTargeted {` CONSTRUCTION —
/// distinguished from a match-arm destructure by the token immediately following
/// the opening brace.
///
/// # The heuristic, and the `/review` defeat that re-keyed it
///
/// **First draft, defeated by EXECUTION.** It classified a hit as a construction
/// only when the token immediately after `{` was literally `target_id:`, on the
/// reasoning that match destructures use shorthand patterns and carry no colon.
/// Rust does not constrain struct-variant field order: the reviewer appended a real
/// second construction written `{ targeting_stack_id, targeting_controller,
/// target_id }` — it compiles, it is a genuine second dispatch payload source, and
/// **`r2` stayed GREEN**. The old docstring named only the explicit-rebind residual
/// and called it "measured rather than merely disclosed"; the likelier form was the
/// one it could not see. That is this batch's own thesis committed inside the gate
/// that states it, for the fifth time in this queue (PB-DX26, PB-DX43, PB-DX45,
/// PB-DX47, now here): *a gate written for one syntactic variant measures that
/// variant.*
///
/// **Re-keyed on the MECHANISM, not on a field name's position.** From the opening
/// brace, scan to the matching `}` (brace-balanced, so a nested initializer cannot
/// end the scan early) and classify by what that region and its tail contain:
///
/// * a `=>` immediately after the closing brace means a MATCH ARM, whatever the
///   field order and whatever rebinding it uses — this is the only form that can
///   legally follow a pattern, so it is a positive discriminator rather than the
///   absence of one;
/// * otherwise it is a construction, and all three field names must appear in the
///   region or the gate fails loudly rather than silently reclassifying.
///
/// Over-collection can only make `r2` REDDER, which is the direction a ratchet is
/// allowed to be wrong in.
fn permanent_targeted_construction_sites() -> Vec<(String, usize)> {
    let root = workspace_root();
    let src_root = root.join("crates/engine/src");
    let mut files = Vec::new();
    walk_rs(&src_root, &mut files);
    files.sort();

    let mut out = Vec::new();
    let mut mentions = Vec::new();
    for path in &files {
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        let stripped = strip_line_comments(&raw);
        let label = path
            .strip_prefix(&src_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        let mut from = 0usize;
        while let Some(rel) = stripped[from..].find(PERMANENT_TARGETED_NEEDLE) {
            let at = from + rel;
            let body_start = at + PERMANENT_TARGETED_NEEDLE.len();
            // Brace-balanced scan to the matching `}` so a nested initializer inside
            // the region cannot end it early.
            let mut depth = 1usize;
            let mut end = body_start;
            for (i, c) in stripped[body_start..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = body_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let region = &stripped[body_start..end];
            let tail = stripped[end + 1..].trim_start();
            // A pattern, by either of two positive discriminators:
            //   * it is followed by `=>`, which only a match arm can be; or
            //   * it does not bind all three fields, which only a pattern may omit
            //     (`{ .. }` in a `matches!`, a partial destructure). A CONSTRUCTION
            //     must name every field -- the type has no `Default`.
            let names_all = PERMANENT_TARGETED_FIELDS.iter().all(|f| region.contains(f));
            let is_pattern = tail.starts_with("=>") || !names_all;
            // The genuinely ambiguous shape is worth failing loudly on rather than
            // silently bucketing: something that is not a match arm and names SOME but
            // not all of the fields is either a payload change or a form this parser
            // does not understand.
            let names_some = PERMANENT_TARGETED_FIELDS.iter().any(|f| region.contains(f));
            assert!(
                !(is_pattern && !tail.starts_with("=>") && names_some && !names_all),
                "ambiguous GameEvent::PermanentTargeted site in {label} at byte {at}: \
                 not a match arm, and it names some but not all of \
                 {PERMANENT_TARGETED_FIELDS:?}. Either the payload changed (update the \
                 const and the wire gates) or this is a form the parser does not \
                 understand -- do NOT relax it without re-running the /review's \
                 field-order experiment, which defeated its first draft."
            );
            if is_pattern {
                mentions.push((label.clone(), at));
            } else {
                out.push((label.clone(), at));
            }
            from = at + 1;
        }
    }
    let _ = &mentions;
    out
}

/// CR 702.21a: there is exactly ONE place in `crates/engine/src` that builds a
/// `GameEvent::PermanentTargeted`, and it is `rules::events::permanent_targeted_events`.
/// This is what makes "one mechanism" a measured property rather than an
/// assertion -- a second construction site would be a second, un-audited path a
/// future dispatch bug could hide behind.
#[test]
fn r2_exactly_one_construction_site() {
    let sites = permanent_targeted_construction_sites();
    assert_eq!(
        sites.len(),
        1,
        "PB-DX48 r2: expected exactly 1 GameEvent::PermanentTargeted CONSTRUCTION \
         site in crates/engine/src, found {}: {:?}. Ceiling is pinned AT the \
         measurement (PB-DX45's lesson: a ratchet's slack is its blind spot) -- a \
         second site is a second, un-audited dispatch path.",
        sites.len(),
        sites
    );
    assert_eq!(
        sites[0].0, "rules/events.rs",
        "r2: the sole construction site moved out of rules/events.rs (found in {})",
        sites[0].0
    );
}

/// Non-vacuity + the heuristic's stated residual, checked: the two KNOWN
/// non-construction mentions (`state/hash.rs`'s hash arm, `rules/abilities.rs`'s
/// dispatch arm, and its `matches!(.., { .. })` count-helper) must all still be
/// present and must all still use the shorthand/`..`-only forms the heuristic
/// relies on -- if any of them switched to an explicit rebind
/// (`target_id: renamed`), r2 would silently misclassify it as a second
/// construction site rather than measuring what actually changed.
#[test]
fn r2b_non_construction_mentions_use_the_shorthand_forms_the_heuristic_relies_on() {
    let hash_src = strip_line_comments(include_str!("../../src/state/hash.rs"));
    let abilities_src = strip_line_comments(include_str!("../../src/rules/abilities.rs"));

    assert!(
        hash_src.contains("GameEvent::PermanentTargeted {\n                target_id,")
            || hash_src.contains("GameEvent::PermanentTargeted {\n            target_id,"),
        "r2b: state/hash.rs's PermanentTargeted match arm no longer uses shorthand \
         field patterns -- re-check whether r2's heuristic still classifies it \
         correctly"
    );
    assert!(
        abilities_src.contains("GameEvent::PermanentTargeted {\n                target_id,"),
        "r2b: rules/abilities.rs's PermanentTargeted dispatch arm no longer uses \
         shorthand field patterns -- re-check whether r2's heuristic still \
         classifies it correctly"
    );
    assert!(
        abilities_src.contains("matches!(e, GameEvent::PermanentTargeted { .. })"),
        "r2b: the TargetsAnnounced/PermanentTargeted pair-count helper's `matches!` \
         form changed; re-verify it still carries no `target_id:` for r2's \
         heuristic to misread"
    );
}

/// [`OOS-DX32-6`]'s class again: `strip_line_comments` only removes `//` line
/// comments. Every file r2 walks is `crates/engine/src/**/*.rs`, which is too
/// wide to `include_str!` file-by-file, so this asserts the invariant on the
/// SAME four files the census identified as mentioning the variant at all
/// (r2's own scan already reads every file; a block comment anywhere in
/// `crates/engine/src` containing the needle text would only ever matter in one
/// of these four, since they are the only files `grep` found the string in).
#[test]
fn r2c_the_four_known_mention_sites_carry_no_block_comments() {
    for (label, src) in [
        ("rules/events.rs", include_str!("../../src/rules/events.rs")),
        ("state/hash.rs", include_str!("../../src/state/hash.rs")),
        (
            "rules/abilities.rs",
            include_str!("../../src/rules/abilities.rs"),
        ),
    ] {
        assert!(
            !src.contains("/*"),
            "PB-DX48 r2c: `{label}` grew a `/* */` block comment -- \
             strip_line_comments cannot remove it, and it could now hide or fake a \
             GameEvent::PermanentTargeted construction from r2. Widen the \
             stripper."
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// r3 — the Ward population
// ─────────────────────────────────────────────────────────────────────────────

/// Every corpus def declaring `AbilityDefinition::Keyword(KeywordAbility::Ward(_))`
/// on any face, with its cost — **4**, not the plan's 5. See this file's module
/// doc: `vein_ripper` mentions the variant name only inside a `// TODO` comment
/// and declares no such ability; it is r5's member, not r3's.
const WARD_MEMBERS: &[(&str, u32)] = &[
    ("Adrix and Nev, Twincasters", 2),
    ("Miirym, Sentinel Wyrm", 2),
    ("Rith, Liberated Primeval", 2),
    ("Tyrranax Rex", 4),
];

/// The deck-legal `Complete` subset — **3**.
const WARD_DECK_LEGAL_MEMBERS: &[&str] = &[
    "Adrix and Nev, Twincasters",
    "Miirym, Sentinel Wyrm",
    "Tyrranax Rex",
];

fn ward_members() -> Vec<CardDefinition> {
    let mut v: Vec<CardDefinition> = all_cards()
        .into_iter()
        .filter(|d| declared_ward_cost(d).is_some())
        .collect();
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

/// CR 702.21a: the exact set of corpus defs declaring the Ward keyword, PRINTED
/// with cost and completeness. Non-vacuity + exact-set pin.
#[test]
fn r3_ward_population_is_pinned_by_name() {
    let members = ward_members();
    let actual: BTreeSet<(String, u32)> = members
        .iter()
        .map(|d| (d.name.clone(), declared_ward_cost(d).unwrap()))
        .collect();
    let expected: BTreeSet<(String, u32)> = WARD_MEMBERS
        .iter()
        .map(|(n, c)| (n.to_string(), *c))
        .collect();
    assert_eq!(
        actual, expected,
        "PB-DX48 r3: the set of defs declaring KeywordAbility::Ward moved. A new \
         member changes the population the CR 702.21a dispatch fix is live on; \
         re-verify against oracle text before re-pinning."
    );
    assert_eq!(actual.len(), 4, "r3 non-vacuity: expected 4 Ward defs");

    let deck_legal: BTreeSet<String> = members
        .iter()
        .filter(|d| is_effectively_complete(d))
        .map(|d| d.name.clone())
        .collect();
    let expected_deck_legal: BTreeSet<String> = WARD_DECK_LEGAL_MEMBERS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        deck_legal, expected_deck_legal,
        "PB-DX48 r3: the deck-legal Complete Ward subset moved"
    );
    assert_eq!(
        deck_legal.len(),
        3,
        "r3 non-vacuity: expected 3 deck-legal Ward defs"
    );

    // The two non-deck-legal members must STAY non-deck-legal, so a promotion
    // reddens this row rather than silently widening the live population.
    let name = "Rith, Liberated Primeval";
    let def = members
        .iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("r3: {name} missing from the Ward roster walk"));
    assert!(
        !is_effectively_complete(def),
        "PB-DX48 r3: {name} was promoted to Complete. It must now be added to \
         WARD_DECK_LEGAL_MEMBERS above -- this is a deliberate stop, not a bug \
         in this test."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r4 — the WhenBecomesTarget / WhenBecomesTargetByOpponent population
// ─────────────────────────────────────────────────────────────────────────────

fn declares_when_becomes_target(def: &CardDefinition) -> bool {
    all_ability_lists(def).into_iter().flatten().any(|a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenBecomesTargetByOpponent,
                ..
            } | AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenBecomesTarget { .. },
                ..
            }
        )
    })
}

/// The one corpus def that structurally DECLARES `WhenBecomesTarget` /
/// `WhenBecomesTargetByOpponent` today, via `all_cards()` (SR-36).
///
/// **Corrects the plan's "6 defs, 0 deck-legal (5 partial + 1 inert)" to 1.** A
/// `grep -rln WhenBecomesTarget crates/card-defs/src/defs/` DOES return 6 files
/// (`bonecrusher_giant`, `flowerfoot_swordmaster`, `goldspan_dragon`,
/// `scalelord_reckoner`, `tectonic_giant`, `venerated_rotpriest`) -- but that is
/// SR-36's exact failure, one more time inside a batch whose own r3 exists to
/// name it: five of the six mention `TriggerCondition::WhenBecomesTarget` ONLY
/// inside an `ENGINE-BLOCKED` / `completeness` prose comment explaining that the
/// EFFECT (not the trigger condition) is what blocks authoring the ability at
/// all, and each of those five has an EMPTY `abilities: vec![]` for the trigger
/// in question. Only `goldspan_dragon` actually constructs the
/// `AbilityDefinition::Triggered { trigger_condition: TriggerCondition::
/// WhenBecomesTarget { .. }, .. }` node. See [`WHEN_BECOMES_TARGET_MENTIONED_
/// MEMBERS`] for the other five, tracked separately and honestly as
/// "mentioned, not declared".
const WHEN_BECOMES_TARGET_DECLARING_MEMBERS: &[&str] = &["Goldspan Dragon"];

/// The five defs whose `completeness` note or an in-source `ENGINE-BLOCKED`
/// comment NAMES `TriggerCondition::WhenBecomesTarget` as the trigger CR 602.2b
/// would need, while declaring NO such ability (the effect side is what blocks
/// them, per each file's own comment) -- an informational roster, not a
/// declaration-construct one. Printed by `t_census_report`, not asserted as a
/// live population (they carry no `AbilityDefinition::Triggered` node for
/// `r4b` to pin against).
const WHEN_BECOMES_TARGET_MENTIONED_MEMBERS: &[&str] = &[
    "Bonecrusher Giant // Stomp",
    "Flowerfoot Swordmaster",
    "Scalelord Reckoner",
    "Tectonic Giant",
    "Venerated Rotpriest",
];

/// CR 601.2c / 602.2b / 603.2: **1** structural member, `Complete`-ineligible
/// (`partial`) -- **0** deck-legal `Complete` -- so a future promotion reddens
/// this row rather than silently widening the class the CR 702.21a dispatch fix
/// does not yet reach for `WhenBecomesTarget` (only Ward's own
/// `WhenBecomesTargetByOpponent` synthesis in `state/builder.rs` reaches a real
/// def today).
#[test]
fn r4_when_becomes_target_population_is_pinned() {
    let mut members: Vec<CardDefinition> = all_cards()
        .into_iter()
        .filter(declares_when_becomes_target)
        .collect();
    members.sort_by(|a, b| a.name.cmp(&b.name));

    let actual: BTreeSet<String> = members.iter().map(|d| d.name.clone()).collect();
    let expected: BTreeSet<String> = WHEN_BECOMES_TARGET_DECLARING_MEMBERS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        actual, expected,
        "PB-DX48 r4: the structural WhenBecomesTarget / WhenBecomesTargetByOpponent \
         population moved."
    );
    assert_eq!(
        actual.len(),
        1,
        "r4 non-vacuity: expected exactly 1 declaring member"
    );

    let deck_legal: Vec<&CardDefinition> = members
        .iter()
        .filter(|d| is_effectively_complete(d))
        .collect();
    assert!(
        deck_legal.is_empty(),
        "PB-DX48 r4: {:?} were promoted to deck-legal Complete. This is a \
         deliberate stop -- CR 602.2b's zone-of-announcement targeting is now \
         wired for these, so re-verify the class before re-pinning at 0.",
        deck_legal.iter().map(|d| &d.name).collect::<Vec<_>>()
    );
    assert!(
        matches!(members[0].completeness, Completeness::Partial(_)),
        "r4: Goldspan Dragon's completeness marker changed shape (expected Partial)"
    );
}

/// The "mentioned, not declared" five, PINNED as a set of NAMES with a stated
/// reason each -- so a def leaving this set gains either an `r4` declaration
/// (repaired) or disappears from the corpus (renamed/removed), and a def
/// ENTERING it is a new blocker comment to account for.
#[test]
fn r4b_mentioned_but_not_declared_members_are_pinned() {
    // No declaring `AbilityDefinition::Triggered` node exists for these, so this
    // is not a structural walk -- it is the exact NAME list, checked for
    // presence in `all_cards()` and for the negative (no declaration) rather
    // than derived from source text (this file is the one place source-text
    // derivation is deliberately used, and it says so).
    let by_name: std::collections::BTreeMap<String, CardDefinition> = all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect();
    for name in WHEN_BECOMES_TARGET_MENTIONED_MEMBERS {
        let def = by_name
            .get(*name)
            .unwrap_or_else(|| panic!("r4b: {name} missing from all_cards()"));
        assert!(
            !declares_when_becomes_target(def),
            "PB-DX48 r4b: {name} now DECLARES WhenBecomesTarget/WhenBecomesTargetByOpponent \
             -- it must move to WHEN_BECOMES_TARGET_DECLARING_MEMBERS in r4, not stay here"
        );
        assert!(
            !is_effectively_complete(def),
            "PB-DX48 r4b: {name} was promoted to Complete while still declaring no \
             WhenBecomesTarget ability -- re-verify the class"
        );
    }
    assert_eq!(
        WHEN_BECOMES_TARGET_MENTIONED_MEMBERS.len(),
        5,
        "r4b non-vacuity: expected 5 mentioned-but-not-declared members"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r5 — the inverse axis over printed oracle text
// ─────────────────────────────────────────────────────────────────────────────

/// Whole-word match for "ward" -- excludes "reward", "steward", "toward",
/// "awards" and similar, which a bare substring search would catch.
fn oracle_prints_ward(def: &CardDefinition) -> bool {
    all_oracle_text(def)
        .split(|c: char| !c.is_alphanumeric())
        .any(|tok| tok == "ward")
}

/// Every corpus def whose printed oracle text carries the whole word "ward" but
/// which declares NO `KeywordAbility::Ward` -- the PB-DX26/DX43/DX45/DX47 lesson
/// a fifth time: a roster derived from one declaration construct (r3) measures
/// that construct, not the printed card. Deliberately NOT filtered to `Complete`
/// only -- two of its six members (`Brutal Cathar`, `Cryptic Coat`) ARE `Complete`,
/// and a Complete-only filter would not have changed that, but the other four
/// would have been hidden exactly the way r3's naive grep hid `vein_ripper`.
///
/// **Six members, each classified by MECHANISM, not just named:**
/// * `Vein Ripper`, `Scavenger Regent // Exude Toxin` — printed "Ward—Sacrifice a
///   creature." / "Ward—Discard a card." — a non-mana Ward cost
///   `KeywordAbility::Ward(u32)` cannot express. Both `partial` for exactly that
///   reason.
/// * `Brutal Cathar` — its back face "Moonrage Brute" prints "Ward—Pay 3 life."
///   with an in-source `// DSL gap` comment, and NO Ward mechanism of any kind is
///   authored for it. **This member is `Completeness::Complete` (deck-legal)** —
///   a genuinely LIVE finding this row exists to surface, not something PB-DX48
///   files or fixes. Distinct from the two reminder-text members below: this one
///   is the card's OWN printed ability, silently missing.
/// * `Cryptic Coat`, `Lumbering Laundry` — print "ward {2}" only as PARENTHESIZED
///   REMINDER TEXT describing what `Effect::Cloak` / Disguise itself grants to
///   the manifested/flipped permanent — correctly tracked by r6, not a gap of
///   either card's own. `Cryptic Coat` is `Complete` (deck-legal); its reminder
///   text is why it appears on this axis and NOT evidence of anything missing.
/// * `Innkeeper's Talent` — Level 2 grants "ward {1}" to OTHER permanents via a
///   static continuous effect, a different mechanism from `KeywordAbility::Ward`
///   entirely (it would need `LayerModification::AddKeyword(KeywordAbility::
///   Ward(1))`, not the card's own keyword list) — `abilities: vec![]`,
///   `inert`, blocked on an unrelated `EffectFilter` gap per its own note.
const OFF_CONSTRUCT_WARD_MEMBERS: &[&str] = &[
    "Brutal Cathar",
    "Cryptic Coat",
    "Innkeeper's Talent",
    "Lumbering Laundry",
    "Scavenger Regent // Exude Toxin",
    "Vein Ripper",
];

/// The subset of [`OFF_CONSTRUCT_WARD_MEMBERS`] that is deck-legal `Complete` —
/// **2**. `Cryptic Coat`'s membership is benign (reminder text, tracked by r6);
/// `Brutal Cathar`'s is the row's own live finding (see the doc above).
const OFF_CONSTRUCT_WARD_DECK_LEGAL_MEMBERS: &[&str] = &["Brutal Cathar", "Cryptic Coat"];

fn off_construct_ward_members() -> Vec<CardDefinition> {
    let mut v: Vec<CardDefinition> = all_cards()
        .into_iter()
        .filter(oracle_prints_ward)
        .filter(|d| declared_ward_cost(d).is_none())
        .collect();
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

/// PB-DX48 r5: the exact inverse-axis set, six members. A member LEAVING this
/// set without gaining an entry in r3 (or, for the two reminder-text members, in
/// r6) would mean its Ward text was silently dropped, not repaired.
#[test]
fn r5_inverse_oracle_axis_is_pinned() {
    let members = off_construct_ward_members();
    let actual: BTreeSet<String> = members.iter().map(|d| d.name.clone()).collect();
    let expected: BTreeSet<String> = OFF_CONSTRUCT_WARD_MEMBERS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        actual, expected,
        "PB-DX48 r5: the inverse Ward-oracle-text axis moved. A new member is a \
         def whose printed Ward either needs KeywordAbility::Ward authored, is a \
         genuinely off-construct case (non-mana cost, granted-to-others, or \
         reminder text for Cloak/Disguise), or is a card whose completeness \
         marker no longer matches its printed text; a member disappearing \
         without an r3 or r6 addition means its Ward text was silently dropped."
    );
    assert_eq!(
        actual.len(),
        6,
        "r5 non-vacuity: expected exactly 6 members"
    );

    let deck_legal: BTreeSet<String> = members
        .iter()
        .filter(|d| is_effectively_complete(d))
        .map(|d| d.name.clone())
        .collect();
    let expected_deck_legal: BTreeSet<String> = OFF_CONSTRUCT_WARD_DECK_LEGAL_MEMBERS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        deck_legal, expected_deck_legal,
        "PB-DX48 r5: the deck-legal Complete subset of the inverse axis moved -- \
         Brutal Cathar's presence is this row's own live finding (a Complete def \
         printing 'Ward—Pay 3 life' with zero Ward mechanism authored) and \
         Cryptic Coat's is benign (Cloak reminder text, tracked by r6). A THIRD \
         member appearing here as deck-legal Complete is a NEW live gap and must \
         be triaged, not silently accepted by widening this set."
    );

    // Non-vacuity + shape checks on the two named cases this row's doc singles
    // out, so a rename or a completeness-marker edit is caught rather than
    // silently absorbed by the set-equality check above.
    let brutal_cathar = members
        .iter()
        .find(|d| d.name == "Brutal Cathar")
        .expect("r5: Brutal Cathar missing from the inverse-axis walk");
    assert_eq!(
        brutal_cathar.completeness,
        Completeness::Complete,
        "r5: Brutal Cathar's completeness marker moved off Complete -- if it is \
         now Partial/Inert citing the Ward gap, that is a genuine repair of the \
         defect this row surfaces and OFF_CONSTRUCT_WARD_DECK_LEGAL_MEMBERS \
         should be updated to reflect it, not left stale"
    );
    let vein_ripper = members
        .iter()
        .find(|d| d.name == "Vein Ripper")
        .expect("r5: Vein Ripper missing from the inverse-axis walk");
    assert!(
        matches!(vein_ripper.completeness, Completeness::Partial(_)),
        "r5: Vein Ripper's completeness marker changed shape (expected Partial)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r6 — the Disguise/Cloak population
// ─────────────────────────────────────────────────────────────────────────────

/// Every corpus def that can produce a face-down permanent tagged
/// `FaceDownKind::Disguise` or `FaceDownKind::Cloak` -- `rules/layers.rs` grants
/// such a permanent `KeywordAbility::Ward(2)` (CR 702.168a / 701.58a) but nothing
/// derives the Ward TRIGGERED ability from it; that synthesis is
/// `state/builder.rs`'s `for spec in self.objects { ... if let
/// KeywordAbility::Ward(cost_n) = kw { ... } }` loop, which runs once at
/// object-CONSTRUCTION time and is never re-run when a permanent turns or enters
/// face down mid-game.
///
/// Two members, not the plan's "1 Disguise, 0 Cloak": `Cryptic Coat`
/// (`Complete`, deck-legal, no `completeness` field so it derives `Complete`)
/// resolves `Effect::Cloak` in its own ETB, and `effects/mod.rs:5319` sets
/// `obj.face_down_as = Some(FaceDownKind::Cloak)` on the manifested object --
/// live, not latent. `Lumbering Laundry` (`partial`) is the Disguise member the
/// plan named.
const DISGUISE_CLOAK_MEMBERS: &[(&str, &str)] = &[
    (
        "Cryptic Coat",
        "Effect::Cloak (cloaks the top card of the controller's library)",
    ),
    (
        "Lumbering Laundry",
        "KeywordAbility::Disguise (may be cast face down for its Disguise cost)",
    ),
];

fn disguise_cloak_members() -> Vec<CardDefinition> {
    let mut v: Vec<CardDefinition> = all_cards()
        .into_iter()
        .filter(|d| {
            def_contains_variant(d, "Cloak")
                || all_ability_lists(d)
                    .into_iter()
                    .flatten()
                    .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Disguise)))
        })
        .collect();
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

/// `OOS-DX48-4`: the exact set of corpus defs reachable to a face-down
/// Disguise/Cloak permanent, PRINTED. A promotion out of the non-deck-legal
/// member, or a new member appearing, changes whether this gap is live.
#[test]
fn r6_disguise_cloak_population_is_pinned() {
    let members = disguise_cloak_members();
    let actual: BTreeSet<String> = members.iter().map(|d| d.name.clone()).collect();
    let expected: BTreeSet<String> = DISGUISE_CLOAK_MEMBERS
        .iter()
        .map(|(n, _)| n.to_string())
        .collect();
    assert_eq!(
        actual, expected,
        "PB-DX48 r6 (OOS-DX48-4): the Disguise/Cloak population moved."
    );
    assert_eq!(actual.len(), 2, "r6 non-vacuity: expected 2 members");

    // Cryptic Coat is the LIVE deck-legal Complete member -- corrects the plan's
    // "0 deck-legal Complete, gap is latent" claim.
    let cryptic_coat = members
        .iter()
        .find(|d| d.name == "Cryptic Coat")
        .expect("r6: Cryptic Coat missing from the walk");
    assert!(
        is_effectively_complete(cryptic_coat),
        "PB-DX48 r6: Cryptic Coat is no longer deck-legal Complete -- the class's \
         live/latent status changed, re-derive before re-pinning"
    );
    assert!(
        def_contains_variant(cryptic_coat, "Cloak"),
        "r6 non-vacuity: Cryptic Coat no longer carries Effect::Cloak"
    );

    let lumbering_laundry = members
        .iter()
        .find(|d| d.name == "Lumbering Laundry")
        .expect("r6: Lumbering Laundry missing from the walk");
    assert!(
        !is_effectively_complete(lumbering_laundry),
        "PB-DX48 r6: Lumbering Laundry was promoted to Complete -- a SECOND \
         deck-legal member of this class, re-verify before re-pinning"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Report
// ─────────────────────────────────────────────────────────────────────────────

/// PRINTS every population above. No figure in this batch's prose is
/// transcribed (PB-DX8's rule).
#[test]
fn t_census_report() {
    println!("── PB-DX48 census (OOS-ENG2-1 / -2 / -3) ──");

    println!("r1 -- push_target_announcement( call sites:");
    let live = live_sites();
    for (file, func, marker, at) in &live {
        println!("  [{marker}] {file} :: {func} @ byte {at}");
    }
    println!("  total: {}", live.len());
    println!("  pinned reasons:");
    for site in PINNED_SITES {
        println!(
            "    [{}] {} :: {} -- {}",
            site.marker, site.file, site.func, site.reason
        );
    }

    println!("r2 -- GameEvent::PermanentTargeted construction sites:");
    for (file, offset) in permanent_targeted_construction_sites() {
        println!("  {file} @ byte {offset}");
    }

    println!("r3 -- Ward population:");
    for d in ward_members() {
        println!(
            "  {} (Ward {{{}}}, {}, complete={})",
            d.name,
            declared_ward_cost(&d).unwrap(),
            if is_effectively_complete(&d) {
                "deck-legal"
            } else {
                "NOT deck-legal"
            },
            is_effectively_complete(&d)
        );
    }

    println!("r4 -- WhenBecomesTarget / WhenBecomesTargetByOpponent population:");
    println!("  declares (structural):");
    for d in all_cards().into_iter().filter(declares_when_becomes_target) {
        println!("    {} ({:?})", d.name, d.completeness);
    }
    println!("  mentions in a blocker comment but declares nothing:");
    for name in WHEN_BECOMES_TARGET_MENTIONED_MEMBERS {
        println!("    {name}");
    }

    println!("r5 -- inverse oracle-text axis (prints \"ward\", no KeywordAbility::Ward):");
    for d in off_construct_ward_members() {
        println!("  {} ({:?})", d.name, d.completeness);
    }

    println!("r6 -- Disguise/Cloak population:");
    for d in disguise_cloak_members() {
        println!(
            "  {} ({:?}, complete={})",
            d.name,
            d.completeness,
            is_effectively_complete(&d)
        );
    }
}
