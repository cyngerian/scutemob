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

fn strip_line_comments(src: &str) -> String {
    src.lines()
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
        let count = stripped.matches(".attackers.insert(").count();
        if count > 0 {
            offenders.push((f.display().to_string(), count));
        }
    }

    assert!(
        offenders.is_empty(),
        "PB-DX51 r1: found the raw mutation `.attackers.insert(` outside \
         CombatState::add_attacker's own implementation in: {offenders:#?}. Every CR \
         508.1/508.4 entry site must route through CombatState::add_attacker so \
         `had_attackers` (CR 508.8) cannot be silently forgotten at a new site -- see \
         plan §1.2."
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
