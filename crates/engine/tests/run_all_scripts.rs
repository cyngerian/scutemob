//! Hook 3 from `docs/mtg-engine-game-scripts.md`: automatic script discovery.
//!
//! Discovers all JSON files under `test-data/generated-scripts/`, deserializes
//! them as [`GameScript`] values, and runs every script marked
//! `review_status: approved` through the replay harness.
//!
//! ## Running
//!
//! ```
//! cargo test -p mtg-engine --test run_all_scripts
//! ```
//!
//! Scripts that don't deserialize are silently skipped (they may belong to a
//! different schema version or be work-in-progress). Scripts that fail
//! assertions or cause command rejections produce test failures with details.

use std::env;
use std::fs;
use std::path::Path;

use mtg_engine::testing::script_schema::{GameScript, ReviewStatus};

// Import the replay harness via a path include — this avoids duplicating the
// module in two crates or adding replay as a public library module.
// (Hook 2 lives in tests/script_replay.rs, which we reference here.)
mod script_replay_lib {
    // Re-export the public types and functions from script_replay.rs.
    // We do this by compiling the file as an inline module using include!.
    // This is the standard pattern for sharing test helper code in Rust.
    include!("script_replay.rs");
}

use script_replay_lib::{replay_script, ReplayResult};

// ── Script discovery ──────────────────────────────────────────────────────────

/// Recursively discover all JSON files under `dir` and try to parse them as
/// [`GameScript`] values. Returns only those that parse successfully.
fn discover_scripts(dir: &Path) -> Vec<(String, GameScript)> {
    let mut scripts = Vec::new();

    if !dir.exists() {
        return scripts;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return scripts,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Recurse into subdirectories.
            scripts.extend(discover_scripts(&path));
        } else if path.extension().map(|e| e == "json").unwrap_or(false) {
            let label = path.display().to_string();
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<GameScript>(&content) {
                    Ok(script) => scripts.push((label, script)),
                    Err(_) => {
                        // Not a valid GameScript — skip silently.
                    }
                },
                Err(_) => {}
            }
        }
    }

    scripts
}

// ── Test ──────────────────────────────────────────────────────────────────────

#[test]
/// CR-agnostic: runs every approved script found in `test-data/generated-scripts/`.
///
/// Passes vacuously if no approved scripts exist yet (M7 initial state).
/// Fails if any approved script has assertion mismatches or command rejections.
fn run_all_approved_scripts() {
    // Path is relative to the workspace root (where `cargo test` runs).
    let scripts_dir = Path::new("../../test-data/generated-scripts");

    // Optional filter: SCRIPT_FILTER=<substring> runs only scripts whose path or id
    // contains the substring. Used by agents to validate a single new script quickly:
    //   SCRIPT_FILTER=015_declare_attackers cargo test --test run_all_scripts
    // When SCRIPT_FILTER is set, pending_review scripts are also included so agents
    // can validate before approving.
    let filter = env::var("SCRIPT_FILTER").ok();

    let all_scripts = discover_scripts(scripts_dir);
    let approved: Vec<_> = all_scripts
        .iter()
        .filter(|(label, s)| {
            if let Some(ref f) = filter {
                // Filter mode: match path OR script id; include any review_status.
                label.contains(f.as_str()) || s.metadata.id.contains(f.as_str())
            } else {
                // Normal mode: approved scripts only.
                s.metadata.review_status == ReviewStatus::Approved
            }
        })
        .collect();

    if approved.is_empty() {
        if filter.is_some() {
            panic!(
                "SCRIPT_FILTER={:?} matched 0 scripts in {:?}",
                filter.unwrap(),
                scripts_dir
            );
        }
        // No approved scripts yet — that's fine for M7 initial state.
        eprintln!(
            "run_all_approved_scripts: 0 approved scripts found in {:?} (pass vacuously)",
            scripts_dir
        );
        return;
    }

    let mut failures: Vec<(String, Vec<ReplayResult>)> = Vec::new();

    for (label, script) in &approved {
        let results = replay_script(script);
        let bad: Vec<_> = results
            .iter()
            .filter(|r| !matches!(r, ReplayResult::Ok { .. }))
            .cloned()
            .collect();
        if !bad.is_empty() {
            failures.push((label.clone(), bad));
        }
    }

    if !failures.is_empty() {
        for (label, bad_results) in &failures {
            eprintln!("SCRIPT FAILED: {}", label);
            for r in bad_results {
                match r {
                    ReplayResult::Mismatch {
                        description,
                        mismatches,
                        ..
                    } => {
                        eprintln!("  Assertion mismatch at '{}':", description);
                        for m in mismatches {
                            eprintln!(
                                "    path='{}' expected={} actual={}",
                                m.path, m.expected, m.actual
                            );
                        }
                    }
                    ReplayResult::CommandRejected {
                        error,
                        step_idx,
                        action_idx,
                    } => {
                        eprintln!(
                            "  Command rejected at step {} action {}: {}",
                            step_idx, action_idx, error
                        );
                    }
                    ReplayResult::Ok { .. } => {}
                }
            }
        }
        panic!(
            "{} of {} approved scripts failed",
            failures.len(),
            approved.len()
        );
    }

    eprintln!(
        "run_all_approved_scripts: {} approved scripts all passed",
        approved.len()
    );
}
