//! SR-35 gate: the card-definition corpus is actually format-checked.
//!
//! `cargo fmt --all -- --check` reports success while checking NONE of the 1,748
//! files in `crates/card-defs/src/defs/`. rustfmt discovers files by walking `mod`
//! declarations textually — it expands no macros and runs no build scripts — and
//! `defs/mod.rs` is a single `include!(concat!(env!("OUT_DIR"), …))` whose
//! `#[path]` mods are written by `build.rs` into `target/`. Both halves defeat the
//! walk, so a green `cargo fmt --check` says nothing whatsoever about the corpus.
//!
//! `tools/check-defs-fmt.sh` is the real check; its header explains why it needs
//! `format_strings=true` (without it a long `oracle_text` line makes rustfmt give
//! up on the whole file and exit 0 — 1,380 of 1,748 defs were in that state) and
//! `error_on_line_overflow=true` (an unbreakable over-width line does the same).
//! `crates/engine/tests/sr35_adversarial_demo.sh` demonstrates all of it.
//!
//! This test exists so the gate is enforced by the workflow people actually run.
//! CI has its own step for it, but `cargo test --all` is what a dev runs before
//! pushing and what the milestone checklist names — a gate only CI runs is a gate
//! discovered late. It shells out to the same script rather than re-encoding the
//! rustfmt invocation, so the two cannot drift.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The workspace root: `crates/engine/` is two levels down from it.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

#[test]
fn card_defs_are_rustfmt_clean() {
    let root = workspace_root();
    let script = root.join("tools/check-defs-fmt.sh");
    assert!(
        script.is_file(),
        "the SR-35 fmt gate is missing at {}",
        script.display()
    );

    let out = Command::new("bash")
        .arg(&script)
        .current_dir(&root)
        .output()
        .expect("failed to run tools/check-defs-fmt.sh");

    assert!(
        out.status.success(),
        "card defs are not rustfmt-clean.\n\
         Fix with: tools/check-defs-fmt.sh --fix\n\
         Note `cargo fmt --all -- --check` does NOT cover these files and will \
         keep reporting success.\n\n--- stdout ---\n{}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

/// The gate above is worthless if it silently checks nothing — which is the exact
/// failure it was written to fix, so it is the one failure it must not repeat.
/// The script reports the count it checked; require it to be large. A bare
/// "exit 0" from a script that globbed an empty directory would pass the test
/// above and prove nothing.
#[test]
fn the_fmt_gate_is_not_vacuous() {
    let root = workspace_root();
    let out = Command::new("bash")
        .arg(root.join("tools/check-defs-fmt.sh"))
        .current_dir(&root)
        .output()
        .expect("failed to run tools/check-defs-fmt.sh");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let checked: usize = stdout
        .split_whitespace()
        .find_map(|w| w.parse().ok())
        .unwrap_or_else(|| panic!("gate did not report a count; said: {stdout}"));

    // The corpus is ~1,748 and only grows. This is a floor against a glob that
    // silently stops matching, not an assertion about the exact card count.
    assert!(
        checked > 1000,
        "the fmt gate only checked {checked} defs — it has stopped seeing the corpus"
    );

    // And the count it reports must match what is actually on disk, or the number
    // above is theatre.
    let on_disk = std::fs::read_dir(root.join("crates/card-defs/src/defs"))
        .expect("defs dir")
        .filter_map(Result::ok)
        .filter(|e| {
            let p = e.path();
            p.extension().is_some_and(|x| x == "rs") && p.file_stem().is_some_and(|s| s != "mod")
        })
        .count();
    assert_eq!(
        checked, on_disk,
        "the gate checked {checked} defs but {on_disk} exist on disk"
    );
}
