//! Hook 3 from `docs/mtg-engine-game-scripts.md`: automatic script discovery.
//!
//! Discovers all JSON files under `test-data/generated-scripts/`, deserializes them
//! as [`GameScript`] values, and runs every script marked `review_status: approved`
//! through the replay harness.
//!
//! ## Running
//!
//! ```text
//! cargo test -p mtg-engine --test scripts run_all_scripts
//! SCRIPT_FILTER=015_declare_attackers cargo test -p mtg-engine --test scripts run_all_scripts -- --nocapture
//! ```
//!
//! ## Nothing is skipped silently (SR-9c)
//!
//! This file used to end at `run_all_approved_scripts`, and every way a script could
//! fall out of that run was silent:
//!
//! | Escape hatch | Was | Now |
//! |---|---|---|
//! | JSON that doesn't deserialize | `if let Ok(script) = …` — dropped | `every_script_file_deserializes` names the file and the serde error |
//! | `review_status: pending_review` | filtered out, uncounted | `no_script_is_awaiting_triage` fails |
//! | `review_status: retired` | (didn't exist) | counted, reason printed, `retired_scripts_carry_a_reason` gates it |
//! | a script with no `assert_state` | ran, asserted nothing | `every_approved_script_asserts_something` fails |
//! | a `player_action` the harness can't translate | no-op, no record | `ReplayResult::ActionNotTranslated`, gated against a shrinking allowlist |
//! | an assertion path `check_assertions` doesn't implement | returned "no mismatch" | a hard mismatch (`script_replay.rs`) |
//!
//! The discovered set is *partitioned*: `approved + retired + other == discovered`,
//! asserted by `the_corpus_is_fully_accounted_for`. A script cannot leave the run
//! without landing in a bucket that something prints.

use std::collections::{BTreeMap, BTreeSet};
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

/// Path to the corpus, relative to the workspace root (where `cargo test` runs).
const SCRIPTS_DIR: &str = "../../test-data/generated-scripts";

/// `player_action.action` strings that `translate_player_action` deliberately maps to
/// no `Command`, and that an **approved** script may therefore contain.
///
/// This is a **shrinking allowlist**, not a config knob. Each entry is a script action
/// that runs as a no-op, so every assertion after it describes a board the engine never
/// reached. `search_library` is the one legitimate member: the engine resolves
/// `SearchLibrary` deterministically and the action is a documentation marker (M10 will
/// add `Command::SelectLibraryCard`, at which point this list should be empty).
///
/// `approved_scripts_only_use_allowlisted_untranslatable_actions` fails if an approved
/// script uses an action outside this set — and `the_untranslatable_allowlist_has_no_dead_entries`
/// fails if an entry stops being used, so the list cannot rot into a rubber stamp.
const ALLOWED_UNTRANSLATABLE_ACTIONS: &[&str] = &["search_library"];

// ── Script discovery ──────────────────────────────────────────────────────────

/// One discovered `*.json` file: either a parsed script or the serde error that
/// stopped it from parsing.
///
/// Before SR-9c a parse failure was `if let Ok(script) = serde_json::from_str(…)`
/// with the `Err` arm dropped on the floor. Two `commander/` scripts carried
/// `"review_status": "draft"`, which is not a `ReviewStatus` variant, so they had
/// been invisible to this suite since the day they were written.
type Discovered = (String, Result<GameScript, String>);

/// Recursively discover all JSON files under `dir`, parsing each as a [`GameScript`].
fn discover_scripts(dir: &Path) -> Vec<Discovered> {
    let mut scripts = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => panic!("cannot read scripts dir {:?}: {e}", dir),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scripts.extend(discover_scripts(&path));
        } else if path.extension().map(|e| e == "json").unwrap_or(false) {
            let label = path.display().to_string();
            let parsed = fs::read_to_string(&path)
                .map_err(|e| format!("unreadable: {e}"))
                .and_then(|content| {
                    serde_json::from_str::<GameScript>(&content).map_err(|e| e.to_string())
                });
            scripts.push((label, parsed));
        }
    }

    scripts
}

/// The parsed corpus. Panics — loudly, naming files — if anything failed to parse,
/// so no other test in this file has to think about the unparseable case.
fn parsed_corpus() -> Vec<(String, GameScript)> {
    let mut ok = Vec::new();
    let mut bad = Vec::new();
    for (label, parsed) in discover_scripts(Path::new(SCRIPTS_DIR)) {
        match parsed {
            Ok(s) => ok.push((label, s)),
            Err(e) => bad.push(format!("  {label}\n    {e}")),
        }
    }
    assert!(
        bad.is_empty(),
        "{} script file(s) failed to deserialize as GameScript:\n{}",
        bad.len(),
        bad.join("\n")
    );
    assert!(
        !ok.is_empty(),
        "no scripts found under {SCRIPTS_DIR} — the corpus cannot vanish"
    );
    ok
}

fn assert_state_count(script: &GameScript) -> usize {
    script
        .script
        .iter()
        .flat_map(|step| step.actions.iter())
        .filter(|a| {
            matches!(
                a,
                mtg_engine::testing::script_schema::ScriptAction::AssertState { .. }
            )
        })
        .count()
}

// ── Corpus accounting gates ───────────────────────────────────────────────────

#[test]
/// Every `*.json` under the corpus deserializes. A file that does not is not a
/// "different schema version" to be skipped — it is a test that stopped existing.
fn every_script_file_deserializes() {
    parsed_corpus();
}

#[test]
/// SR-9c: `pending_review` is not a resting state. Every script is `approved` (it runs)
/// or `retired` (it does not, for a written-down reason). `disputed` and `corrected`
/// are transitional and must not persist either.
fn no_script_is_awaiting_triage() {
    let untriaged: Vec<_> = parsed_corpus()
        .into_iter()
        .filter(|(_, s)| {
            !matches!(
                s.metadata.review_status,
                ReviewStatus::Approved | ReviewStatus::Retired
            )
        })
        .map(|(label, s)| format!("  {label} ({:?})", s.metadata.review_status))
        .collect();

    assert!(
        untriaged.is_empty(),
        "{} script(s) are neither approved nor retired. A script in this state is \
         silently excluded from `run_all_approved_scripts` — approve it, fix it, or \
         retire it with a `retirement_reason`:\n{}",
        untriaged.len(),
        untriaged.join("\n")
    );
}

#[test]
/// A retired script must say why, and a live script must not pretend to be retired.
/// The reason is the entire difference between "excluded" and "missing".
fn retired_scripts_carry_a_reason() {
    let mut problems = Vec::new();
    for (label, s) in parsed_corpus() {
        let retired = s.metadata.review_status == ReviewStatus::Retired;
        match (retired, s.metadata.retirement_reason.as_deref()) {
            (true, None) | (true, Some("")) => {
                problems.push(format!("  {label}: retired with no `retirement_reason`"))
            }
            (false, Some(r)) => problems.push(format!(
                "  {label}: not retired but carries `retirement_reason` = {r:?}"
            )),
            _ => {}
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
/// An approved script with no `assert_state` checkpoint runs the engine and checks
/// nothing. It cannot fail, so it is not a test.
fn every_approved_script_asserts_something() {
    let vacuous: Vec<_> = parsed_corpus()
        .into_iter()
        .filter(|(_, s)| s.metadata.review_status == ReviewStatus::Approved)
        .filter(|(_, s)| assert_state_count(s) == 0)
        .map(|(label, _)| format!("  {label}"))
        .collect();

    assert!(
        vacuous.is_empty(),
        "{} approved script(s) contain zero `assert_state` actions and therefore \
         cannot fail:\n{}",
        vacuous.len(),
        vacuous.join("\n")
    );
}

#[test]
/// The discovered set is partitioned by review status, with nothing left over. This is
/// the test that makes "N approved scripts passed" mean "and here is where the other
/// M went", rather than "and I did not look".
fn the_corpus_is_fully_accounted_for() {
    let corpus = parsed_corpus();
    let mut buckets: BTreeMap<String, usize> = BTreeMap::new();
    for (_, s) in &corpus {
        *buckets
            .entry(format!("{:?}", s.metadata.review_status))
            .or_default() += 1;
    }

    let approved = *buckets.get("Approved").unwrap_or(&0);
    let retired = *buckets.get("Retired").unwrap_or(&0);

    eprintln!("script corpus: {} file(s) discovered", corpus.len());
    for (status, n) in &buckets {
        eprintln!("  {status:>14}: {n}");
    }
    for (label, s) in &corpus {
        if s.metadata.review_status == ReviewStatus::Retired {
            eprintln!(
                "  RETIRED {}: {}",
                label,
                s.metadata.retirement_reason.as_deref().unwrap_or("<none>")
            );
        }
    }

    assert_eq!(
        approved + retired,
        corpus.len(),
        "corpus is not partitioned into approved + retired: {buckets:?}"
    );
    assert!(approved > 0, "no approved scripts — the suite is vacuous");
}

// ── Untranslatable-action accounting ──────────────────────────────────────────

/// Every distinct `player_action.action` in approved scripts that the harness maps to
/// no `Command`, paired with the scripts that use it.
fn untranslatable_actions_in_approved_scripts() -> BTreeMap<String, BTreeSet<String>> {
    let mut found: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (label, script) in parsed_corpus() {
        if script.metadata.review_status != ReviewStatus::Approved {
            continue;
        }
        for r in replay_script(&script) {
            if let ReplayResult::ActionNotTranslated { action, .. } = r {
                found.entry(action).or_default().insert(label.clone());
            }
        }
    }
    found
}

#[test]
/// An approved script whose action the harness cannot translate runs that action as a
/// **no-op**: the state does not change, and every assertion after it is describing a
/// board the engine never reached. Nine such action names were live in the corpus
/// (`assign_damage`, `choose_option`, `cast_spell_from_command_zone`, `transform`,
/// `activate_craft`, `cast_spell_disturb`, `order_replacements`, `sacrifice`,
/// `mulligan_decision`); SR-9c retired every script that used one.
fn approved_scripts_only_use_allowlisted_untranslatable_actions() {
    let found = untranslatable_actions_in_approved_scripts();
    let offenders: Vec<_> = found
        .iter()
        .filter(|(action, _)| !ALLOWED_UNTRANSLATABLE_ACTIONS.contains(&action.as_str()))
        .map(|(action, labels)| {
            format!(
                "  '{action}' is a no-op in: {}",
                labels.iter().cloned().collect::<Vec<_>>().join(", ")
            )
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "{} untranslatable action(s) used by approved scripts. `translate_player_action` \
         returns `None` for these, so they change nothing and the assertions that follow \
         are meaningless. Implement the translation, or retire the script:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

#[test]
/// Denominator guard, in the shape SR-8 and SR-9b both needed: an allowlist whose
/// entries are no longer exercised is a rubber stamp. If `search_library` stops
/// appearing in an approved script, delete it from the list — don't leave it there
/// blessing a case that no longer occurs.
fn the_untranslatable_allowlist_has_no_dead_entries() {
    let found = untranslatable_actions_in_approved_scripts();
    let dead: Vec<_> = ALLOWED_UNTRANSLATABLE_ACTIONS
        .iter()
        .filter(|a| !found.contains_key(**a))
        .collect();
    assert!(
        dead.is_empty(),
        "ALLOWED_UNTRANSLATABLE_ACTIONS entries no longer used by any approved script \
         (remove them): {dead:?}"
    );
}

// ── The run ───────────────────────────────────────────────────────────────────

#[test]
/// CR-agnostic: runs every approved script found in `test-data/generated-scripts/`.
///
/// Fails if any approved script has assertion mismatches, command rejections, an
/// unknown player, or an untranslatable action outside the allowlist. Reports the
/// full ledger — discovered / approved / retired — so a shrinking corpus is visible
/// in the output rather than inferable from a count that nobody remembers.
fn run_all_approved_scripts() {
    // Optional filter: SCRIPT_FILTER=<substring> runs only scripts whose path or id
    // contains the substring. Used by agents to validate a single new script quickly.
    // When SCRIPT_FILTER is set, non-approved scripts are also included so agents can
    // validate before approving.
    let filter = env::var("SCRIPT_FILTER").ok();

    let all_scripts = parsed_corpus();
    let total = all_scripts.len();
    let mut retired = 0usize;

    let selected: Vec<_> = all_scripts
        .iter()
        .filter(|(label, s)| {
            if let Some(ref f) = filter {
                label.contains(f.as_str()) || s.metadata.id.contains(f.as_str())
            } else {
                if s.metadata.review_status == ReviewStatus::Retired {
                    retired += 1;
                }
                s.metadata.review_status == ReviewStatus::Approved
            }
        })
        .collect();

    if selected.is_empty() {
        let f = filter.expect(
            "0 approved scripts with no SCRIPT_FILTER set — \
             `the_corpus_is_fully_accounted_for` should have caught this first",
        );
        panic!("SCRIPT_FILTER={f:?} matched 0 scripts in {SCRIPTS_DIR}");
    }

    let allowed = |r: &ReplayResult| match r {
        ReplayResult::Ok { .. } => true,
        ReplayResult::ActionNotTranslated { action, .. } => {
            ALLOWED_UNTRANSLATABLE_ACTIONS.contains(&action.as_str())
        }
        _ => false,
    };

    let mut failures: Vec<(String, Vec<ReplayResult>)> = Vec::new();
    for (label, script) in &selected {
        let bad: Vec<_> = replay_script(script)
            .into_iter()
            .filter(|r| !allowed(r))
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
                    } => eprintln!(
                        "  Command rejected at step {step_idx} action {action_idx}: {error}"
                    ),
                    ReplayResult::ActionNotTranslated {
                        action,
                        step_idx,
                        action_idx,
                    } => eprintln!(
                        "  Untranslatable action '{action}' at step {step_idx} action {action_idx} \
                         (ran as a no-op; every later assertion is suspect)"
                    ),
                    ReplayResult::UnknownPlayer {
                        player,
                        step_idx,
                        action_idx,
                    } => eprintln!(
                        "  Unknown player '{player}' at step {step_idx} action {action_idx}"
                    ),
                    ReplayResult::Ok { .. } => {}
                }
            }
        }
        panic!("{} of {} scripts failed", failures.len(), selected.len());
    }

    eprintln!(
        "run_all_approved_scripts: {} of {} discovered scripts ran and passed; \
         {} retired (see `the_corpus_is_fully_accounted_for` for reasons); \
         0 skipped silently",
        selected.len(),
        total,
        retired,
    );
}
