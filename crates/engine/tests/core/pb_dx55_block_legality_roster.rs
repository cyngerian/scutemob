//! PB-DX55 Half 2 (`OOS-SIM5-3`) — the mechanism gate: exactly ONE per-pair
//! block-legality predicate exists anywhere in workspace source, and it is
//! `combat::check_block_pair`.
//!
//! Before this batch the engine held TWO independent, hand-rolled copies of the
//! per-pair restriction list inside `handle_declare_blockers` — the per-pair loop and
//! the CR 702.39a provoke satisfiability mirror — and they were NOT identical (the
//! mirror omitted phased-out, `CrossPlayerBlock` and the duplicate-blocker check).
//! "Never a second hand-rolled copy" describes HEAD after this batch's extraction,
//! not merely a future risk, so this gate exists to keep it that way.
//!
//! # Why this is keyed on the MECHANISM, not on a hardcoded file/function list
//!
//! This project's own registry (`docs/audits/decision-point-audit.md`) records at
//! least three prior defeats of exactly this SHAPE of gate: `OOS-DX51-6` (a
//! multi-line spelling defeated a re-pin regex), `OOS-DX54-6` (a needle set only
//! covering the spellings its own author had already thought of), and `OOS-DX54-7`
//! (a crate-scoped walk missed a site one crate over). This gate is designed against
//! all three:
//!
//! 1. **Mechanism, not name.** It does not look for the literal text
//!    `check_block_pair` at all — it looks for the CO-OCCURRENCE of several
//!    per-pair-blocking-specific markers ([`MARKERS`]) that a hand-rolled REWRITE of
//!    the restriction list would have to reproduce regardless of what its author
//!    calls the function or how they format it.
//! 2. **Workspace-wide, not crate-scoped.** It walks every `crates/*/src` and
//!    `tools/*/src` directory (bar `crates/card-defs/src`, which is card DATA, not
//!    engine logic) — `check_block_pair` is `pub`, so a second copy could be planted
//!    in `crates/simulator/src` or anywhere else in the workspace.
//! 3. **Self-defeated in this very file, twice, before being trusted.** See `r2` and
//!    `r3` below: both a same-function duplicate call and a genuinely SECOND function
//!    elsewhere in the tree are planted against SYNTHETIC strings and confirmed to
//!    redden the checker, so the checker is proven discriminating rather than merely
//!    plausible.

use std::path::PathBuf;

// ── Path / source-reading plumbing (house idiom: `workspace_root()` + byte-preserving
//    `strip_comments`, matching `pb_dx49_saga_blanking_roster.rs` / `pb_dx52_stack_target_
//    roster.rs`'s convention of a self-contained copy per file rather than a cross-module
//    import, since `pub(super)` items in a sibling `mod` are not the visibility this
//    codebase's test files rely on) ────────────────────────────────────────────────────

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/engine -> crates -> workspace root
    p.pop();
    p.pop();
    p
}

/// Strip `//` line comments **and** `/* */` block comments, replacing each stripped byte
/// with a space so byte offsets are preserved. Deliberately naive about string literals
/// containing `//`/`/*` — over-stripping can only delete apparent matches, which makes
/// every assertion below REDDER, never falsely green.
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

/// Byte offset of the `}` matching the `{` at `open`, string-literal-aware. Used only
/// to extract a full function body for the r2/r3 defeat fixtures below — never to
/// decide anything about the real workspace source, which `r1` never needs a
/// brace-matched span for.
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

/// The FULL, brace-matched body of `combat::check_block_pair` in the real
/// `combat.rs`, with its name replaced by `new_name` — used to build the r2/r3
/// defeat fixtures. Extracted from `real` (not `strip_comments(real)`) so the
/// planted duplicate is byte-identical (comments and all) to the real function,
/// which is the honest shape of "someone copy-pasted the function" rather than a
/// stripped-down approximation of it.
fn renamed_check_block_pair_body(real: &str, new_name: &str) -> String {
    let stripped = strip_comments(real);
    let sig_marker = "pub fn check_block_pair(";
    let sig_start = stripped
        .find(sig_marker)
        .expect("check_block_pair must exist in combat.rs");
    let open = stripped[sig_start..]
        .find('{')
        .map(|r| sig_start + r)
        .expect("check_block_pair must have a body");
    let close = matching_brace(&stripped, open).expect("check_block_pair's body must be balanced");
    real[sig_start..=close].replacen("check_block_pair", new_name, 1)
}

/// Every `.rs` file directly or transitively under `dir`.
fn walk_rs(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every `crates/*/src` and `tools/*/src` root, bar `crates/card-defs/src` (card DATA,
/// never engine logic — the same exclusion `pb_dx49_saga_blanking_roster.rs` uses).
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

/// Every `.rs` file under [`workspace_src_roots`], as `(workspace-relative label, path)`,
/// with **non-vacuity floors executed** — a walk that silently returns `[]` would make
/// this gate pass while checking nothing, which is exactly the failure mode it exists to
/// prevent in the code it polices. Measured at HEAD: >= 8 roots, >= 100 files (matching
/// `pb_dx49_saga_blanking_roster.rs`'s own measured 14 / 148, floored well below both so
/// ordinary churn does not trip it).
fn workspace_src_files_checked() -> Vec<(String, PathBuf)> {
    let root = workspace_root();
    let roots = workspace_src_roots();
    assert!(
        roots.len() >= 8,
        "PB-DX55: the workspace source walk found only {} `src` roots — every gate built \
         on this walk is vacuous until it is fixed; roots: {:?}",
        roots.len(),
        roots
    );
    assert!(
        roots.iter().any(|r| r.ends_with("crates/engine/src")),
        "PB-DX55: the workspace source walk does not contain crates/engine/src, which is \
         the crate `check_block_pair` lives in; roots: {roots:?}"
    );
    let mut out = Vec::new();
    for src_root in &roots {
        let mut files = Vec::new();
        walk_rs(src_root, &mut files);
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
    assert!(
        out.len() >= 100,
        "PB-DX55: the workspace source walk found only {} .rs files — the walk has gone \
         vacuous",
        out.len()
    );
    out
}

/// Best-effort "which `fn` contains byte offset `at`" — a backward scan for the last
/// `fn `/`pub fn `/`pub(crate) fn ` line before `at`. Same idiom as
/// `pb_dx49_saga_blanking_roster.rs::enclosing_fn_name`; not a real parser (a closure
/// containing an inner `fn` item could confuse it), which is an accepted limitation of
/// this house idiom, not a gap unique to this gate.
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

/// Nine markers, each specific to a genuine per-pair blocking DECISION (a method-call
/// / boolean-comparison idiom), not merely a mention of the enum variant.
///
/// **The first draft of this list used the bare variant names** (`KeywordAbility::
/// Fear`, `LandwalkType::BasicType`, `BlockingExceptionFilter::HasKeyword`, ...) and
/// was WRONG, caught by this gate's own `t1` non-vacuity report rather than assumed:
/// `state/hash.rs::hash_into` (an exhaustive `HashInto` derivation over every
/// `KeywordAbility`/`BlockingExceptionFilter`/`LandwalkType` variant) scored 8 of 9,
/// and `view-model/src/lib.rs::format_keyword` (an exhaustive display-name `match`)
/// scored 5 of 9 — both false positives, because a `match` ARM that enumerates a
/// variant for hashing or display is not a per-pair RESTRICTION. The distinguishing
/// shape is the METHOD-CALL idiom (`.contains(&KeywordAbility::X)`, an "does the
/// creature HAVE this keyword" membership test) versus the match-ARM idiom
/// (`KeywordAbility::X => ...`, "here is what this variant hashes/displays as").
/// Re-keyed on that distinction; every marker below occurs EXACTLY ONCE in the whole
/// workspace outside `check_block_pair` (verified: the `protection.rs` test file's
/// own direct call to `protection::can_block(` scores exactly 1 marker, nowhere near
/// [`THRESHOLD`]).
///
/// **Known limitation, stated rather than hidden**: each marker is a literal,
/// SINGLE-LINE substring search (`OOS-DX51-6`'s exact lesson — a re-pin/scan is only
/// as wide as the spelling its regex matches). A future `cargo fmt` reflow that
/// breaks one of these exact idioms across a line boundary (e.g. `.contains(\n
/// &KeywordAbility::Fear)`) would silently drop that ONE marker from the count. This
/// gate does not defend against that; `THRESHOLD` (5 of 9) gives headroom against
/// losing one or two markers to reflow before `r1`'s exact-nine assertion on
/// `check_block_pair` itself would need loosening, but a wholesale reflow of many
/// markers at once would still need re-verifying by hand.
const MARKERS: &[&str] = &[
    ".contains(&KeywordAbility::Horsemanship)",
    ".contains(&KeywordAbility::Skulk)",
    ".contains(&KeywordAbility::Shadow)",
    ".contains(&KeywordAbility::Intimidate)",
    ".contains(&KeywordAbility::Fear)",
    "required_kw.as_ref()",
    ".any(|k| blocker_chars.keywords.contains(k))",
    "LandwalkType::BasicType(st) => chars.subtypes.contains(st)",
    "protection::can_block(",
];

/// **AXIS B — the COMMON-case markers, added by PB-DX55's own coordinator-run revert
/// matrix after row R9 DEFEATED the exotic axis by execution.**
///
/// Every one of [`MARKERS`] is EXOTIC — horsemanship, skulk, shadow, intimidate, fear,
/// the `CantBeBlockedExceptBy` filter internals, landwalk, protection. That is the
/// right axis for catching a wholesale REWRITE of `check_block_pair`, which is what
/// `r2`/`r3` plant. It is the wrong axis for catching the copy a human actually
/// writes. R9 planted a 5-guard hand-rolled predicate — controller, tapped,
/// `CantBlock`, flying/reach, protection — in `combat.rs` itself, i.e. someone
/// answering "can this block that?" for one local purpose and covering only the cases
/// they had in mind. It scored **1 of 9** and left `r1` GREEN.
///
/// **A similarity gate keyed entirely on the rare members of a set is blind to the
/// partial copy, and the partial copy is the likely one** — a person writing a second
/// predicate reaches for the common guards first and never gets to horsemanship. So a
/// SECOND axis keys on those common guards. Measured across the workspace before the
/// threshold was chosen (`src/` only, comments stripped): `check_block_pair` scores
/// **8 of 8**, `handle_declare_attackers` scores **2** (it reads Flying/Reach for the
/// ATTACKER-side evasion question, a different subject), and nothing else scores above
/// one. A threshold of **3** therefore has five points of headroom below the real
/// predicate and one point above its nearest neighbour. (Written "one" rather than
/// "1" deliberately: a doc line opening with `1.` is an ordered-list item and makes
/// the next line its lazy continuation — `clippy::doc_lazy_continuation`, which fired
/// here on the first draft, PB-DX39's own case one punctuation mark over.)
const COMMON_MARKERS: &[&str] = &[
    ".contains(&KeywordAbility::Flying)",
    ".contains(&KeywordAbility::Reach)",
    ".contains(&KeywordAbility::CantBlock)",
    ".contains(&KeywordAbility::Decayed)",
    "Designations::SUSPECTED",
    "GameStateError::CrossPlayerBlock",
    "GameStateError::DuplicateBlocker",
    "GameStateError::PermanentAlreadyTapped",
];

/// See [`COMMON_MARKERS`]. Measured headroom: real predicate 8, nearest other 2.
const COMMON_THRESHOLD: usize = 3;

/// A function qualifies as "a per-pair block-legality predicate" once it contains at
/// least this many of the nine [`MARKERS`] — chosen with real headroom: the genuine
/// predicate contains all nine, and no other function in the corpus at HEAD contains
/// more than one (measured by `t1`'s own report).
const THRESHOLD: usize = 5;

/// For every function in `src` (a byte string), the set of DISTINCT markers found in
/// it, keyed by `(file label, function name)`. A marker occurring twice in one
/// function counts once (co-occurrence, not volume, is the signal).
fn functions_with_markers(files: &[(String, PathBuf)]) -> Vec<(String, String, Vec<&'static str>)> {
    functions_with_marker_set(files, MARKERS)
}

/// The same walk, parameterised by marker list, so AXIS B reuses the identical
/// comment-stripping and enclosing-function attribution rather than a second copy of
/// it — which would be this file's own subject matter committed inside this file.
fn functions_with_marker_set(
    files: &[(String, PathBuf)],
    markers: &[&'static str],
) -> Vec<(String, String, Vec<&'static str>)> {
    use std::collections::BTreeMap;
    let mut hits: BTreeMap<(String, String), std::collections::BTreeSet<&'static str>> =
        BTreeMap::new();
    for (label, path) in files {
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        let src = strip_comments(&raw);
        for marker in markers {
            let mut from = 0usize;
            while let Some(rel) = src[from..].find(marker) {
                let at = from + rel;
                let func = enclosing_fn_name(&src, at);
                hits.entry((label.clone(), func))
                    .or_default()
                    .insert(marker);
                from = at + 1;
            }
        }
    }
    hits.into_iter()
        .map(|((file, func), markers)| (file, func, markers.into_iter().collect()))
        .collect()
}

/// **r1**: exactly one function in the whole workspace qualifies as a per-pair
/// block-legality predicate ([`THRESHOLD`] of nine [`MARKERS`]), and it is
/// `combat::check_block_pair`. If a second hand-rolled copy is ever written anywhere
/// in the workspace, this reddens — it does not need to know the copy's name, its
/// file, or its exact wording.
#[test]
fn r1_exactly_one_per_pair_block_predicate_exists() {
    let files = workspace_src_files_checked();
    let hits = functions_with_markers(&files);

    let qualifying: Vec<&(String, String, Vec<&'static str>)> = hits
        .iter()
        .filter(|(_, _, markers)| markers.len() >= THRESHOLD)
        .collect();

    eprintln!(
        "PB-DX55 r1: functions scored against {} markers:",
        MARKERS.len()
    );
    for (file, func, markers) in &hits {
        if !markers.is_empty() {
            eprintln!("  {file}::{func}: {} marker(s) {markers:?}", markers.len());
        }
    }

    assert_eq!(
        qualifying.len(),
        1,
        "PB-DX55 (`OOS-SIM5-3`): exactly ONE function must qualify as a per-pair \
         block-legality predicate (>= {THRESHOLD} of {} markers); found {}: {:?}. A \
         second hand-rolled copy of the per-pair restriction list has been \
         (re)introduced somewhere in the workspace.",
        MARKERS.len(),
        qualifying.len(),
        qualifying
    );
    let (file, func, markers) = qualifying[0];
    assert_eq!(
        (file.as_str(), func.as_str()),
        ("crates/engine/src/rules/combat.rs", "check_block_pair"),
        "the one qualifying function must be combat::check_block_pair, got {file}::{func} \
         (markers {markers:?})"
    );
    // Non-vacuity: the real predicate carries ALL nine markers, not merely the
    // threshold — so THRESHOLD has real headroom below the true count rather than
    // being tuned to the exact measurement.
    assert_eq!(
        markers.len(),
        MARKERS.len(),
        "combat::check_block_pair should carry every one of the {} markers; got {}: \
         {markers:?} (has this function been split, silently narrowing what it checks?)",
        MARKERS.len(),
        markers.len()
    );
}

/// **r2** (defeat, executed against a SYNTHETIC string — never the real files under
/// `crates/engine/src/`): a duplicated per-pair block predicate INSIDE the same file,
/// under a different function name, is caught.
#[test]
fn r2_defeat_a_second_copy_in_the_same_file_reddens() {
    let real = std::fs::read_to_string(workspace_root().join("crates/engine/src/rules/combat.rs"))
        .expect("combat.rs must exist");

    // The FULL, brace-matched body of `check_block_pair`, renamed -- exactly what a
    // "helpful" refactor that forgot to delete the old copy would leave behind.
    let duplicate_src = renamed_check_block_pair_body(&real, "check_block_pair_v2");

    let mut synthetic = real.clone();
    synthetic.push_str("\n// PB-DX55 r2 planted duplicate:\n");
    synthetic.push_str(&duplicate_src);

    let planted_path = std::env::temp_dir().join("pb_dx55_r2_planted_combat.rs");
    std::fs::write(&planted_path, &synthetic).expect("write synthetic file");

    let files = vec![("synthetic/combat.rs".to_string(), planted_path.clone())];
    let hits = functions_with_markers(&files);
    let qualifying = hits.iter().filter(|(_, _, m)| m.len() >= THRESHOLD).count();

    let _ = std::fs::remove_file(&planted_path);

    assert!(
        qualifying >= 2,
        "PB-DX55 r2 DEFEAT CHECK FAILED: planting a renamed duplicate of \
         check_block_pair in the same file must make at least 2 functions qualify \
         (found {qualifying}) -- the checker would not have caught this shape"
    );
}

/// **r3** (defeat, executed against a SYNTHETIC file — never real workspace source): a
/// second per-pair block predicate planted in an ENTIRELY DIFFERENT crate (not
/// `crates/engine`) is still caught, because the walk is workspace-wide
/// (`OOS-DX54-7`'s exact lesson: a crate-scoped walk misses a site one crate over).
#[test]
fn r3_defeat_a_second_copy_in_a_different_crate_reddens() {
    let real = std::fs::read_to_string(workspace_root().join("crates/engine/src/rules/combat.rs"))
        .expect("combat.rs must exist");
    let duplicate_src = renamed_check_block_pair_body(&real, "sneaky_second_predicate");

    let planted_path = std::env::temp_dir().join("pb_dx55_r3_planted_simulator_helper.rs");
    std::fs::write(&planted_path, &duplicate_src).expect("write synthetic file");

    // Note the label: this pretends to live under `crates/simulator/src/`, a
    // DIFFERENT crate from the real predicate's `crates/engine/src/rules/combat.rs`.
    let files = vec![(
        "crates/simulator/src/sneaky_second_predicate.rs".to_string(),
        planted_path.clone(),
    )];
    let hits = functions_with_markers(&files);
    let qualifying = hits.iter().filter(|(_, _, m)| m.len() >= THRESHOLD).count();

    let _ = std::fs::remove_file(&planted_path);

    assert!(
        qualifying >= 1,
        "PB-DX55 r3 DEFEAT CHECK FAILED: a second per-pair block predicate planted \
         under a DIFFERENT crate's src/ must still be detected -- a workspace-wide \
         walk must not be blind to sites outside crates/engine"
    );
}

/// **r4 (AXIS B)** — exactly one function in the workspace qualifies on the COMMON
/// block-legality guards ([`COMMON_THRESHOLD`] of [`COMMON_MARKERS`]), and it is
/// `combat::check_block_pair`.
///
/// This axis exists because [`MARKERS`] is entirely EXOTIC and `r1` was DEFEATED by
/// execution during PB-DX55's own coordinator-run revert matrix (row R9): a five-guard
/// hand-rolled predicate covering controller, tapped, `CantBlock`, flying/reach and
/// protection scored 1 of 9 and left `r1` green. See [`COMMON_MARKERS`]' doc.
///
/// `handle_declare_attackers` is the one function that scores above 1 (it reads
/// Flying/Reach for the ATTACKER-side evasion question). It is named here rather than
/// allowlisted, so the assertion below reddens if it ever starts answering the
/// per-pair BLOCK question too.
#[test]
fn r4_axis_b_exactly_one_common_case_block_predicate_exists() {
    let files = workspace_src_files_checked();
    let hits = functions_with_marker_set(&files, COMMON_MARKERS);

    // Non-vacuity: the walk must actually have found the real predicate at full score,
    // or a zero-qualifier result below would be "the scan found nothing", not "the
    // invariant holds".
    let real = hits
        .iter()
        .find(|(f, n, _)| f.ends_with("rules/combat.rs") && n == "check_block_pair")
        .unwrap_or_else(|| {
            panic!("AXIS B non-vacuity: combat::check_block_pair was not found by the walk at all")
        });
    assert_eq!(
        real.2.len(),
        COMMON_MARKERS.len(),
        "AXIS B non-vacuity: check_block_pair must carry every common marker (found {:?})",
        real.2
    );

    let qualifying: Vec<_> = hits
        .iter()
        .filter(|(_, _, m)| m.len() >= COMMON_THRESHOLD)
        .map(|(f, n, m)| (f.clone(), n.clone(), m.len()))
        .collect();
    assert_eq!(
        qualifying.len(),
        1,
        "AXIS B: exactly one function may answer the per-pair block question on the \
         COMMON guards; found {}: {:?}. A second one is the shape R9 planted — a \
         partial hand-rolled copy covering the cases a person actually thinks of. \
         Consume `combat::check_block_pair` (or `queries::legal_blocks`) instead.",
        qualifying.len(),
        qualifying
    );
    assert!(
        qualifying[0].0.ends_with("rules/combat.rs") && qualifying[0].1 == "check_block_pair",
        "AXIS B: the one qualifier must be combat::check_block_pair, got {:?}",
        qualifying[0]
    );
}

/// **r5 (AXIS B defeat, executed against a SYNTHETIC file)** — R9's own plant, verbatim
/// in shape: a PARTIAL hand-rolled predicate covering only the common guards. It scored
/// 1 of 9 on [`MARKERS`] and left `r1` green; it must score at or above
/// [`COMMON_THRESHOLD`] here.
///
/// This is the defeat that produced AXIS B, kept as a test so the repair cannot be
/// undone silently.
#[test]
fn r5_axis_b_defeat_a_partial_common_case_copy_reddens() {
    let planted = r#"
fn second_hand_rolled_block_predicate(
    state: &GameState,
    player: PlayerId,
    blocker_id: ObjectId,
    attacker_id: ObjectId,
) -> bool {
    let Ok(obj) = state.object(blocker_id) else { return false };
    if obj.controller != player || obj.status.tapped {
        return false;
    }
    let chars = calculate_characteristics(state, blocker_id).unwrap();
    if chars.keywords.contains(&KeywordAbility::CantBlock) {
        return false;
    }
    let atk = calculate_characteristics(state, attacker_id).unwrap();
    if atk.keywords.contains(&KeywordAbility::Flying)
        && !chars.keywords.contains(&KeywordAbility::Flying)
        && !chars.keywords.contains(&KeywordAbility::Reach)
    {
        return false;
    }
    true
}
"#;
    let planted_path = std::env::temp_dir().join("pb_dx55_r5_planted_partial_copy.rs");
    std::fs::write(&planted_path, planted).expect("write synthetic file");
    let files = vec![(
        "crates/simulator/src/planted_partial.rs".to_string(),
        planted_path.clone(),
    )];

    let exotic = functions_with_marker_set(&files, MARKERS);
    let exotic_qualifying = exotic
        .iter()
        .filter(|(_, _, m)| m.len() >= THRESHOLD)
        .count();
    let common = functions_with_marker_set(&files, COMMON_MARKERS);
    let common_qualifying = common
        .iter()
        .filter(|(_, _, m)| m.len() >= COMMON_THRESHOLD)
        .count();

    let _ = std::fs::remove_file(&planted_path);

    // The historical record, asserted rather than described: the exotic axis really
    // does miss this shape. If a later batch widens MARKERS so that it no longer does,
    // this assertion reddens and the reader is sent to re-read why AXIS B exists.
    assert_eq!(
        exotic_qualifying, 0,
        "the exotic axis is expected to MISS a partial common-case copy (that is why \
         AXIS B exists); if it now catches it, MARKERS was widened and this note needs \
         rewriting rather than the assertion flipping"
    );
    assert!(
        common_qualifying >= 1,
        "AXIS B DEFEAT CHECK FAILED: a partial hand-rolled block predicate covering \
         only the common guards must be caught by AXIS B; it scored below \
         COMMON_THRESHOLD={COMMON_THRESHOLD}: {common:?}"
    );
}
