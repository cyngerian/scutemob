//! PB-DX51 (`scutemob-226`) — CR 508.1/508.4: the `r1` source gate.
//!
//! `crates/card-types/src/state/combat.rs::CombatState::add_attacker` is the ONLY
//! production path that puts a creature into `combat.attackers`, and it is the only
//! place that sets `had_attackers` (CR 508.8). Written as one mutator so a sixth entry
//! site cannot silently bypass the marker (plan `memory/primitives/pb-plan-DX51.md`
//! §1.2). This file walks the whole workspace's PRODUCTION source (never a hardcoded
//! file list -- `OOS-DX48`'s `SITE_SRCS` defeat was exactly a hardcoded list, and
//! `OOS-DX49`'s `r6` defeat was walking one crate when the policed item was `pub`) and
//! asserts two things by MECHANISM:
//!
//! - `r1`: no production `.rs` file other than `crates/card-types/src/state/combat.rs`
//!   spells the raw map mutation `.attackers.insert(`.
//! - `r1b`: `add_attacker(` has exactly 5 production CALL sites (1 CR 508.1 declaration
//!   loop + 4 CR 508.4 entrants, per the plan's §0.2 re-derived census), listed by
//!   `file:line` so a site appearing or disappearing silently is a red test rather than
//!   a number nobody re-checks.
//!
//! Both scans strip `//` line comments before counting (the same known-limitation
//! technique `bare_lookup_ratchet.rs` documents: block comments and string interiors
//! are not handled, which is not a realistic regression path here).
//!
//! `crates/view-model/src/tests.rs` is the one file in the whole workspace whose
//! filename stem is `tests` under a `src/` directory -- it is `#[cfg(test)] mod tests;`
//! in `crates/view-model/src/lib.rs`, i.e. compiled ONLY under `cfg(test)`, and it DOES
//! contain a hand-built `combat.attackers.insert(..)` fixture. Excluded by filename stem
//! rather than by a hardcoded path, so a second such file anywhere in the workspace is
//! excluded the same way without anyone having to remember to list it.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // crates/engine -> crates -> workspace root
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p
}

/// The one legitimate file: `CombatState::add_attacker`'s own implementation.
fn combat_state_file() -> PathBuf {
    workspace_root().join("crates/card-types/src/state/combat.rs")
}

/// A `#[cfg(test)]`-only file, identified by filename stem rather than by a hardcoded
/// path -- the only such file in the workspace today is `crates/view-model/src/tests.rs`
/// (`#[cfg(test)] mod tests;` in that crate's `lib.rs`), but the rule is general: any
/// file whose stem is exactly `tests`/`test` or ends `_tests`/`_test` is excluded.
fn is_test_only_file(path: &Path) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|stem| {
            let lower = stem.to_ascii_lowercase();
            lower == "tests"
                || lower == "test"
                || lower.ends_with("_tests")
                || lower.ends_with("_test")
        })
        .unwrap_or(false)
}

fn walk_rs(dir: &Path, acc: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            walk_rs(&path, acc);
        } else if path.extension().is_some_and(|x| x == "rs") && !is_test_only_file(&path) {
            acc.push(path);
        }
    }
}

/// Every `<crate>/src` and `<tool>/src` directory in the workspace.
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
            let src = dir.join("src");
            if src.is_dir() {
                out.push(src);
            }
        }
    }
    out
}

/// Every `.rs` file under [`workspace_src_roots`], excluding `#[cfg(test)]`-only files
/// by the [`is_test_only_file`] heuristic. Non-vacuity floors are executed by the
/// caller, not here -- this function must be able to return `[]` observably so the
/// floor can fail on it.
fn workspace_src_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in workspace_src_roots() {
        walk_rs(&root, &mut out);
    }
    out.sort();
    out
}

/// Strips `//` line comments AND `/* … */` block comments.
///
/// The block-comment half was added by the `/review` (Issue 7): `r1c` re-checks each
/// allowlist entry's stated reason by asserting the code still exists in source, and a
/// line-comment-only stripper would let a needle sitting inside a `/* … */` block satisfy
/// that check without the code existing. That is `OOS-DX32-6`'s class — a scan narrowed to
/// `//` silently dropping `/* */` — applied to a REASON check rather than to a count.
///
/// **Known residual, stated rather than left to be discovered**: string and char literals
/// are not handled, so a needle inside a string literal still satisfies `r1c`. Zero
/// exposure today (all four allowlisted sites are live code, re-checked by `r1c` itself).
fn strip_line_comments(src: &str) -> String {
    // Block comments first, so a `//` inside one cannot truncate the wrong thing.
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut depth = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"/*") {
            // Nested block comments are legal in Rust, so this counts depth.
            depth += 1;
            i += 2;
        } else if depth > 0 && bytes[i..].starts_with(b"*/") {
            depth -= 1;
            i += 2;
        } else {
            if depth == 0 {
                out.push(bytes[i] as char);
            } else if bytes[i] == b'\n' {
                // Keep line structure so the per-line scans stay line-accurate.
                out.push('\n');
            }
            i += 1;
        }
    }
    out.lines()
        .map(|line| line.find("//").map(|i| &line[..i]).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// [`workspace_src_files`] with non-vacuity floors executed -- measured at HEAD:
/// **15** roots, **1953** files (`t_census_report` prints the live figures). Floors
/// are set well below both so ordinary churn does not trip them.
fn workspace_src_files_checked() -> Vec<PathBuf> {
    let roots = workspace_src_roots();
    assert!(
        roots.len() >= 8,
        "PB-DX51 r1: the workspace source walk found only {} `src` roots (measured 15 at \
         HEAD). Every assertion built on this walk is vacuous until this is fixed; \
         roots: {:?}",
        roots.len(),
        roots
    );
    assert!(
        roots.iter().any(|r| r.ends_with("crates/engine/src")),
        "PB-DX51 r1: the workspace source walk does not contain crates/engine/src, which \
         is where 4 of the 5 pinned call sites live; roots: {roots:?}"
    );
    assert!(
        roots.iter().any(|r| r.ends_with("crates/card-types/src")),
        "PB-DX51 r1: the workspace source walk does not contain crates/card-types/src, \
         which is where CombatState::add_attacker itself lives; roots: {roots:?}"
    );
    let files = workspace_src_files();
    assert!(
        files.len() >= 500,
        "PB-DX51 r1: the workspace source walk found only {} .rs files (measured 1953 at \
         HEAD) -- the walk has gone vacuous",
        files.len()
    );
    let engine_combat = workspace_root().join("crates/engine/src/rules/combat.rs");
    let effects_mod = workspace_root().join("crates/engine/src/effects/mod.rs");
    let resolution = workspace_root().join("crates/engine/src/rules/resolution.rs");
    assert!(
        files.contains(&engine_combat),
        "PB-DX51 r1: the walk did not read crates/engine/src/rules/combat.rs (the CR \
         508.1 declaration-loop call site) -- non-vacuity check failed"
    );
    assert!(
        files.contains(&effects_mod),
        "PB-DX51 r1: the walk did not read crates/engine/src/effects/mod.rs (two of the \
         four CR 508.4 entrant call sites) -- non-vacuity check failed"
    );
    assert!(
        files.contains(&resolution),
        "PB-DX51 r1: the walk did not read crates/engine/src/rules/resolution.rs \
         (Myriad + Ninjutsu, the other two CR 508.4 entrant call sites) -- non-vacuity \
         check failed"
    );
    assert!(
        files.contains(&combat_state_file()),
        "PB-DX51 r1: the walk did not read the one legitimate file, \
         crates/card-types/src/state/combat.rs -- non-vacuity check failed"
    );
    files
}

// ─────────────────────────────────────────────────────────────────────────────
// r1 — no production site bypasses CombatState::add_attacker
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// The MECHANISM r1 polices, and why it is not the literal `.attackers.insert(`
// ─────────────────────────────────────────────────────────────────────────────
//
// **The first draft of `r1` matched the single literal `.attackers.insert(`, and the
// coordinator DEFEATED it by execution**: a sixth entry site written with an
// intermediate binding --
//
// ```ignore
// if let Some(combat) = state.combat.as_mut() {
//     let map = &mut combat.attackers;
//     map.insert(id, target.clone());
// }
// ```
//
// -- contains no `.attackers.insert(` at all, so `r1` stayed GREEN, and because it ADDS
// a site rather than replacing one, `r1b`'s exact-5 call-site count stayed GREEN too.
// That is *a gate written for one spelling measures that spelling* -- the lesson this
// queue has now recorded for PB-DX26, PB-DX43, PB-DX45 and PB-DX47 -- committed inside
// the gate whose own module doc cites two of those defeats. Reproduced before fixing.
//
// The axis is therefore the MECHANISM: a mutable path to the map. **This list is
// ENUMERATED, NOT EXHAUSTIVE, and its first draft claimed otherwise — that claim was
// itself defeated twice by the `/review`, which is why the wording changed.** A textual
// gate over a `pub` field can always be out-spelled; the only construct that cannot is
// making the field private, which is filed with its measured cost as `OOS-DX51-7`.
// Six forms are checked, the last two added after the review's executed defeats:
//
//   1. a mutating method called directly on the field  (`.attackers.insert(` / `.extend(`
//      / `.entry(` / `.append(` / `.iter_mut(`),
//   2. a mutable borrow bound to a name             (`&mut …attackers`),
//   3. whole-FIELD assignment                        (`.attackers =`, not `==`),
//   4. `std::mem::{replace,swap,take}` over the field,
//   5. whole-STRUCT write-back                       (`CombatState {` outside this type's
//      own file) -- `*combat = CombatState { attackers, ..combat.clone() }` names the
//      field with NO leading dot, so forms 1-4 are all blind to it (`/review` Issue 2,
//      EXECUTED: all five gates were green with a real sixth entry site in the tree),
//   6. a SECOND `&mut self` mutator on `CombatState` itself. `r1` skips
//      `combat.rs` wholesale as the one legitimate file, and `r1b` counts only the
//      literal `add_attacker(`, so a differently-named sibling mutator was invisible to
//      both halves at once (`/review` Issue 3, EXECUTED). Policed by `r1e`, which is a
//      bounded check on that ONE file rather than a workspace scan.
//
// Over-collection can only make `r1` REDDER (the PB-DX47 principle), so the forms are
// deliberately wide: `.attackers` is matched on ANY receiver, which also catches
// `ActionParams::attackers` and `CastSpellData::attackers` -- different fields on
// different types that merely share a name. Each such false positive is named in
// `ALLOWLIST` **with the mechanism that separates it**, and `r1c` re-checks that
// mechanism in source, because an allowlist whose reason is never checked is a comment
// (`OOS-DX47`).

/// The four mutable-path forms, as `(label, regex-free matcher)` pairs.
const MUTATING_FORMS: [&str; 5] = ["method", "borrow", "assign", "mem", "struct"];

/// Every mutable path to a field named `attackers` in `stripped`, as
/// `(form-label, matched-line-text)` pairs. Line text is carried so `ALLOWLIST` can
/// discriminate on the actual code rather than on a bare line number that drifts.
fn mutable_paths_to_attackers(stripped: &str) -> Vec<(&'static str, Vec<String>)> {
    const MUTATING_METHODS: [&str; 8] = [
        ".insert(",
        ".extend(",
        ".entry(",
        ".append(",
        ".iter_mut(",
        ".remove(",
        ".retain(",
        ".clear(",
    ];
    let mut by_form: Vec<(&'static str, Vec<String>)> =
        MUTATING_FORMS.iter().map(|f| (*f, Vec::new())).collect();

    for raw in stripped.lines() {
        let line = raw.trim();
        // Form 1: a mutating method called on the field, on this line or the next
        // (rustfmt breaks long chains). Joining the whole file and searching a window
        // is what makes this multi-line-aware.
        for m in MUTATING_METHODS {
            let needle = format!(".attackers{m}");
            if line.contains(&needle) {
                by_form[0].1.push(line.to_string());
            }
        }
        // Form 2: a mutable borrow of the field bound to a name.
        if line.contains("&mut") && line.contains(".attackers") {
            by_form[1].1.push(line.to_string());
        }
        // Form 3: whole-map assignment (never `==`, never `!=`, never `>=`/`<=`).
        if let Some(i) = line.find(".attackers") {
            let rest = line[i + ".attackers".len()..].trim_start();
            if rest.starts_with('=') && !rest.starts_with("==") {
                by_form[2].1.push(line.to_string());
            }
        }
        // Form 4: mem::replace / swap / take over the field.
        if line.contains(".attackers")
            && (line.contains("mem::replace")
                || line.contains("mem::swap")
                || line.contains("mem::take"))
        {
            by_form[3].1.push(line.to_string());
        }
    }

    // Form 5 (`/review` Issue 2): a whole-STRUCT write-back. `*combat = CombatState {
    // attackers, ..combat.clone() }` mentions the field as a bare `attackers,` shorthand
    // with no leading dot, so every per-line form above is blind to it. Keyed on the TYPE
    // NAME instead, over the whole file rather than per line, because rustfmt splits a
    // struct literal across lines by default. Outside `CombatState`'s own file there is no
    // legitimate reason for production code to construct or overwrite one -- the engine
    // builds them through `CombatState::new`.
    let joined: String = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
    for needle in ["CombatState {", "CombatState{"] {
        let mut from = 0usize;
        while let Some(i) = joined[from..].find(needle) {
            let at = from + i;
            // Only a CONSTRUCTION counts. `impl HashInto for CombatState {` and
            // `pub struct CombatState {` are declarations, not write-backs -- and the
            // first draft of this form flagged `state/hash.rs`'s impl header, which is
            // how the discriminator got written rather than assumed.
            let before = &joined[at.saturating_sub(24)..at];
            let is_declaration =
                before.contains("impl ") || before.contains("struct ") || before.contains("for ");
            if !is_declaration {
                by_form[4]
                    .1
                    .push(joined[at..(at + 160).min(joined.len())].to_string());
            }
            from = at + needle.len();
        }
    }

    by_form
}

/// Multi-line spellings, with a matcher of a DIFFERENT SHAPE from the per-line scan
/// above: the whole file is whitespace-joined and searched in a window, which is the
/// "a survivor check written with the same regex is not a check" rule PB-DX50 recorded.
///
/// **The `/review` (Issue 4) refuted this function's first docstring, which called itself
/// "the multi-line half of `r1`".** It covered only the seven METHOD-call needles, so
/// forms 2 (`&mut` borrow), 3 (assignment) and 4 (`mem::*`) had NO multi-line coverage at
/// all — demonstrated by execution: a borrow split as `let map = &mut combat\n
/// .attackers;` left all five gates GREEN, and only reddened after `cargo fmt` rejoined
/// it. **For those three forms the multi-line robustness was supplied by rustfmt, not by
/// this gate, and nothing said so.** All four are now covered here, and the rustfmt
/// dependency is stated rather than relied on silently.
fn multiline_mutating_paths(stripped: &str) -> Vec<String> {
    let joined: String = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = Vec::new();
    let needles = [
        // form 1, split before the method call
        ".attackers .insert(",
        ".attackers .extend(",
        ".attackers .entry(",
        ".attackers .append(",
        ".attackers .iter_mut(",
        ".attackers .retain(",
        ".attackers .clear(",
        // form 2, split between the receiver and the field
        "&mut combat .attackers",
        "&mut c .attackers",
        "&mut cs .attackers",
        "&mut self .attackers",
        "&mut state.combat .attackers",
        // form 3, split before the `=`
        ".attackers =",
        // form 4, split inside the mem:: call
        "mem::replace(&mut combat .attackers",
        "mem::swap(&mut combat .attackers",
        "mem::take(&mut combat .attackers",
    ];
    for m in needles {
        let mut from = 0usize;
        while let Some(i) = joined[from..].find(m) {
            let at = from + i;
            out.push(
                joined[at.saturating_sub(60)..(at + m.len() + 40).min(joined.len())].to_string(),
            );
            from = at + m.len();
        }
    }
    out
}

/// `(file-suffix, form, line-substring, reason)`.
///
/// Every entry is a field named `attackers` that is **not** `CombatState::attackers`,
/// or a mutation that provably cannot ADD an attacker. `r1c` re-checks each reason.
const ALLOWLIST: [(&str, &str, &str, &str); 4] = [
    (
        "crates/engine/src/rules/combat.rs",
        "method",
        "combat.attackers.remove(&object_id)",
        "CR 506.4 `remove_from_combat`: a REMOVAL cannot add an attacker, and \
         `had_attackers` is deliberately monotone (it must survive CR 506.4 -- that is \
         the whole fix). See `t5`.",
    ),
    (
        "crates/engine/src/rules/abilities.rs",
        "method",
        "combat.attackers.remove(&attacker_to_return)",
        "CR 702.49a Ninjutsu bounce: a REMOVAL, same reason as above. Note this site \
         bypasses `remove_from_combat` entirely -- filed as part of the CR 506.4 \
         cleanup seed, not fixed here.",
    ),
    (
        "crates/simulator/src/random_bot.rs",
        "assign",
        "params.attackers = attackers",
        "`ActionParams::attackers` (a Vec on the command being built), NOT \
         `CombatState::attackers`. Different type, different field.",
    ),
    (
        "crates/view-model/src/redact.rs",
        "method",
        ".attackers.iter_mut()",
        "`CombatView::attackers` (a redaction view DTO), NOT `CombatState::attackers`.",
    ),
];

fn is_allowlisted(rel: &str, form: &'static str, line: &str) -> bool {
    ALLOWLIST
        .iter()
        .any(|(file, f, needle, _)| rel.ends_with(file) && *f == form && line.contains(needle))
}

/// `r1c` — an allowlist whose reason is never checked is a comment (`OOS-DX47`).
///
/// For each `ALLOWLIST` entry this re-reads the named file and asserts the separating
/// mechanism is still true in source, so an entry cannot outlive its justification.
#[test]
fn r1c_every_allowlist_entry_still_has_its_stated_reason() {
    let root = workspace_root();
    for (file, form, needle, reason) in ALLOWLIST {
        let path = root.join(file);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("PB-DX51 r1c: cannot read allowlisted {file}: {e}"));
        let stripped = strip_line_comments(&src);
        assert!(
            stripped.contains(needle),
            "PB-DX51 r1c: allowlist entry {file} / {form} / {needle:?} no longer \
             matches anything in that file. Either the site moved (re-key the entry) or \
             it is gone (DELETE the entry -- a dead allowlist entry is slack a real \
             offender hides in). Stated reason was: {reason}"
        );
    }

    // The two "different type, same field name" reasons are checked by TYPE, not by
    // trust: neither file may mention `CombatState` at all, which is what makes
    // "different type" a fact rather than an assertion.
    for file in [
        "crates/simulator/src/random_bot.rs",
        "crates/view-model/src/redact.rs",
    ] {
        let src = std::fs::read_to_string(root.join(file)).expect("allowlisted file readable");
        let stripped = strip_line_comments(&src);
        assert!(
            !stripped.contains("CombatState"),
            "PB-DX51 r1c: {file} is allowlisted on the grounds that its `attackers` \
             field belongs to a DIFFERENT type, but the file now mentions \
             `CombatState` -- re-adjudicate the entry before trusting it."
        );
    }
}

/// `r1e` — the file `r1` exempts is not exempt from having ONE mutator.
///
/// **`/review` Issue 3, EXECUTED and this is the defeat that mattered most.** `r1` skips
/// `crates/card-types/src/state/combat.rs` wholesale as "the one legitimate file", and
/// `r1b` counts only occurrences of the literal `add_attacker(`. So a SECOND `&mut self`
/// mutator added beside `add_attacker` —
///
/// ```ignore
/// pub fn set_attacking(&mut self, id: ObjectId, target: AttackTarget) {
///     self.attackers.insert(id, target);   // never sets `had_attackers`
/// }
/// ```
///
/// — plus a production caller was invisible to BOTH halves at once, and all five gates
/// stayed GREEN. The reviewer's own judgement is worth keeping: this is *"arguably the
/// most likely real-world regression — the natural thing a future batch does is add a
/// helper next to `add_attacker`."*
///
/// This is a BOUNDED check on that one known file rather than another workspace scan:
/// every mutating path to `self.attackers` in it must occur exactly once, inside
/// `fn add_attacker`. `remove_from_combat` does not live here, so there is no legitimate
/// second mutator to allowlist.
#[test]
fn r1e_combat_state_has_exactly_one_attacker_mutator() {
    let src = std::fs::read_to_string(combat_state_file()).expect("combat.rs readable");
    let stripped = strip_line_comments(&src);

    // Non-vacuity: the file really is the one that defines the mutator.
    assert!(
        stripped.contains("pub fn add_attacker"),
        "PB-DX51 r1e: {} no longer defines `add_attacker` -- this check is vacuous until \
         it is re-pointed",
        combat_state_file().display()
    );

    const SELF_MUTATIONS: [&str; 8] = [
        "self.attackers.insert(",
        "self.attackers.extend(",
        "self.attackers.entry(",
        "self.attackers.append(",
        "self.attackers.iter_mut(",
        "self.attackers.retain(",
        "self.attackers.clear(",
        "self.attackers =",
    ];

    // 1. Exactly one mutating path in the whole file.
    let total: usize = SELF_MUTATIONS
        .iter()
        .map(|m| stripped.matches(m).count())
        .sum();
    assert_eq!(
        total,
        1,
        "PB-DX51 r1e: expected EXACTLY ONE mutating path to `self.attackers` in {}, found \
         {}. A second mutator beside `add_attacker` is invisible to `r1` (which exempts \
         this file) and to `r1b` (which counts only the literal `add_attacker(`), so it \
         would silently re-create the CR 508.8 defect this batch closed. Per-form counts: \
         {:?}",
        combat_state_file().display(),
        total,
        SELF_MUTATIONS
            .iter()
            .map(|m| (*m, stripped.matches(m).count()))
            .collect::<Vec<_>>()
    );

    // 2. …and it is inside `add_attacker`, not merely somewhere in the file. Bounded by
    //    the NEXT `pub fn` after `add_attacker`'s signature, so the window cannot run on
    //    into the rest of the impl (`OOS-DX49`'s over-scanning window lesson).
    let start = stripped
        .find("pub fn add_attacker")
        .expect("checked non-vacuous above");
    let rest = &stripped[start + "pub fn add_attacker".len()..];
    let end = rest
        .find("pub fn")
        .map(|i| start + i)
        .unwrap_or(stripped.len());
    let body = &stripped[start..end.max(start)];
    assert!(
        body.contains("self.attackers.insert("),
        "PB-DX51 r1e: the single mutating path to `self.attackers` is NOT inside \
         `add_attacker`'s own body. Either the mutator moved (re-point this check) or a \
         different function now owns the mutation, which is the defeat this test exists \
         for."
    );
    // 3. …and that body still sets the CR 508.8 marker, which is the whole point.
    assert!(
        body.contains("had_attackers = true"),
        "PB-DX51 r1e: `add_attacker` no longer sets `had_attackers`. Every entry site \
         routes through it precisely so CR 508.8's marker cannot be forgotten; without \
         this line the routing is decoration."
    );
}

/// `r1d` — the multi-line half of `r1`, with a differently shaped matcher.
#[test]
fn r1d_no_multiline_mutating_path_to_attackers() {
    let files = workspace_src_files_checked();
    let legit = combat_state_file();
    let mut offenders = Vec::new();
    for f in &files {
        if *f == legit {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        let stripped = strip_line_comments(&src);
        let rel = f
            .strip_prefix(workspace_root())
            .unwrap_or(f)
            .display()
            .to_string();
        for hit in multiline_mutating_paths(&stripped) {
            // Allowlist by the matched TEXT, never by the file.
            //
            // The first draft skipped any file appearing in `ALLOWLIST` wholesale, and
            // that is how the `/review`'s Issue-4 defeat survived the fix written FOR it:
            // a multi-line `&mut combat` / `.attackers` planted in
            // `crates/engine/src/rules/combat.rs` -- allowlisted there only for a
            // `remove(` call -- was skipped along with the whole file, so `r1d` stayed
            // GREEN even after its needle set had been widened to cover borrows. **A
            // file-scoped allowlist grants an exemption far wider than the reason that
            // earned it**, the same shape as `OOS-DX48`'s hardcoded `SITE_SRCS`. Matching
            // on content keeps the exemption exactly as wide as its stated reason.
            if ALLOWLIST
                .iter()
                .any(|(file, _, needle, _)| rel.ends_with(file) && hit.contains(needle))
            {
                continue;
            }
            offenders.push((rel.clone(), hit));
        }
    }
    assert!(
        offenders.is_empty(),
        "PB-DX51 r1d: a mutating call on `.attackers` spelled across a line break: \
         {offenders:#?}"
    );
}

#[test]
fn r1_no_production_file_bypasses_add_attacker() {
    let files = workspace_src_files_checked();
    let legit = combat_state_file();

    let mut offenders = Vec::new();
    for f in &files {
        if *f == legit {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        let stripped = strip_line_comments(&src);
        let rel = f
            .strip_prefix(workspace_root())
            .unwrap_or(f)
            .display()
            .to_string();
        for (form, hits) in mutable_paths_to_attackers(&stripped) {
            for hit in hits {
                if is_allowlisted(&rel, form, &hit) {
                    continue;
                }
                offenders.push((rel.clone(), form, hit));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "PB-DX51 r1: found a MUTABLE path to `combat.attackers` outside \
         CombatState::add_attacker's own implementation: {offenders:#?}. Every CR \
         508.1/508.4 entry site must route through CombatState::add_attacker so \
         `had_attackers` (CR 508.8) cannot be silently forgotten at a new site -- see \
         plan `memory/primitives/pb-plan-DX51.md` §1.2. If this is a legitimate \
         non-adding mutation, add it to ALLOWLIST with its reason AND a companion \
         assertion in `r1c` that the reason still holds."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r1b — add_attacker has exactly 5 production call sites
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn r1b_add_attacker_has_exactly_five_production_call_sites() {
    let files = workspace_src_files_checked();
    let legit = combat_state_file();

    let mut sites: Vec<String> = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        let stripped = strip_line_comments(&src);
        for (i, line) in stripped.lines().enumerate() {
            if !line.contains("add_attacker(") {
                continue;
            }
            // Exclude the method's own declaration in combat_state_file(); every
            // other occurrence (in that file or any other) is a CALL site.
            if *f == legit && line.contains("fn add_attacker(") {
                continue;
            }
            sites.push(format!("{}:{}", f.display(), i + 1));
        }
    }
    sites.sort();

    assert_eq!(
        sites.len(),
        5,
        "PB-DX51 r1b: expected exactly 5 production call sites of \
         CombatState::add_attacker (1 CR 508.1 declaration loop + 4 CR 508.4 entrants, \
         per plan §0.2's re-derived census), found {}: {:#?}. A count that changed means \
         a site appeared or disappeared -- re-derive the census against plan §0.2 before \
         updating this pin, never just bump the number.",
        sites.len(),
        sites
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t_census_report — prints the live figures (non-assertion, always green)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t_census_report() {
    let roots = workspace_src_roots();
    let files = workspace_src_files();
    println!(
        "PB-DX51 r1 census: {} src roots, {} .rs files walked (test-only files excluded \
         by filename stem)",
        roots.len(),
        files.len()
    );
}
