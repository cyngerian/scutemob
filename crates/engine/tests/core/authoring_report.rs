//! SR-26: cross-check `tools/authoring-report.py` against the compiled registry.
//!
//! `authoring-report.py` owns the card-authoring campaign headline number
//! (clean / todo / empty), and it computes those buckets by *text-scanning* the
//! def files for a `completeness:` marker. That scanner was untested and
//! case-fragile: `MARKER_BUCKETS` only knew the lowercase helper spellings
//! (`Completeness::partial(...)`), so a def written `Completeness::Partial(...)`
//! directly captured `Partial`, fell through `MARKER_BUCKETS.get(marker, "clean")`,
//! and silently counted as **done**. Nothing tied the number the tool emits to the
//! thing it is supposed to measure — the actual `CardDefinition.completeness`
//! values in the compiled registry.
//!
//! This test closes that loop. It recomputes the three buckets from `all_cards()`
//! (the compiled registry — every def's `completeness` field, which is also what
//! the deck-build gate reads, per invariant #9) and diffs them against the tool's
//! own parse, obtained from its read-only `--check` mode. If the tool's regex ever
//! drifts from the registry — a new marker spelling, a bucketing bug, a scan that
//! silently finds fewer files — the two disagree and this fails.
//!
//! Per the SR track rule ("assert the denominator"), the registry side has a
//! non-vacuity floor so an empty registry cannot vacuously match empty tool output.

use std::process::Command;

use mtg_engine::{all_cards, Completeness};

/// Floor for the compiled registry. Well below the real count (~1,748); it catches
/// a registry that failed to build its card list, not a codebase that shrank.
const MIN_REGISTRY_CARDS: usize = 1000;

/// Bucket a `Completeness` exactly as `authoring-report.py`'s `MARKER_BUCKETS`
/// does: `Inert` → empty, `Partial`/`KnownWrong` → todo, `Complete` → clean.
#[derive(Default, Debug, PartialEq, Eq)]
struct Buckets {
    total: u64,
    clean: u64,
    todo: u64,
    empty: u64,
}

fn registry_buckets() -> Buckets {
    let mut b = Buckets::default();
    for def in all_cards() {
        b.total += 1;
        match def.completeness {
            Completeness::Complete => b.clean += 1,
            Completeness::Inert(_) => b.empty += 1,
            Completeness::Partial(_) | Completeness::KnownWrong(_) => b.todo += 1,
        }
    }
    b
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

/// Read the integer value of `"key"` out of the tool's one-line JSON. Deliberately
/// tiny (no serde_json dev-dep for one flat object): find `"key"`, skip to the
/// first digit, take the digit run.
fn json_u64(json: &str, key: &str) -> u64 {
    let needle = format!("\"{key}\"");
    let start = json
        .find(&needle)
        .unwrap_or_else(|| panic!("tool JSON missing key {key:?}: {json}"));
    let after = &json[start + needle.len()..];
    let digits: String = after
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits
        .parse()
        .unwrap_or_else(|_| panic!("no integer after key {key:?} in tool JSON: {json}"))
}

/// **The gate.** The tool's parse must equal the compiled registry.
#[test]
fn authoring_report_buckets_match_the_compiled_registry() {
    let registry = registry_buckets();

    // Denominator guard: the registry actually loaded a real card list.
    assert!(
        registry.total as usize >= MIN_REGISTRY_CARDS,
        "all_cards() returned only {} cards (< {}); the compiled registry is broken, \
         so a match against the tool would be vacuous",
        registry.total,
        MIN_REGISTRY_CARDS
    );

    let script = workspace_root().join("tools/authoring-report.py");
    let output = Command::new("python3")
        .arg(&script)
        .arg("--check")
        .current_dir(workspace_root())
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "failed to run `python3 {} --check`: {e}. python3 is required to run this \
                 cross-check gate (it is present in CI and on the dev box).",
                script.display()
            )
        });

    // A non-zero exit is the tool's own guard firing (e.g. an unrecognized marker
    // spelling, or a denominator guard). Surface its stderr rather than a bare
    // parse failure — that stderr IS the SR-26 anti-rot message.
    assert!(
        output.status.success(),
        "`authoring-report.py --check` exited non-zero — its own guards rejected the \
         current tree. stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tool = Buckets {
        total: json_u64(&stdout, "total_files"),
        clean: json_u64(&stdout, "clean"),
        todo: json_u64(&stdout, "todo"),
        empty: json_u64(&stdout, "empty"),
    };

    assert_eq!(
        tool, registry,
        "\n\nthe authoring report's buckets disagree with the compiled registry.\n\
         tool (text-scan of def files):   {tool:?}\n\
         registry (all_cards() markers):  {registry:?}\n\n\
         Every def file maps 1:1 to one `all_cards()` entry (build.rs), so these must \
         be equal. A mismatch means the tool's completeness-marker scan drifted from \
         the actual `CardDefinition.completeness` values — most likely a new marker \
         spelling `MARKER_BUCKETS` does not know, or a def file that stopped exposing \
         `pub fn card()`. Fix the tool's parse (or the def), do not adjust this test.\n"
    );
}
