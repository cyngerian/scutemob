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

// ---------------------------------------------------------------------------
// The gate's two --config flags are load-bearing, and nothing above notices if
// one is deleted.
//
// Both flags exist to stop rustfmt from silently doing nothing (see the script's
// header). Neither is guarded by the tests above, and the corpus cannot guard
// them either: the defs are already formatted, so with a flag removed rustfmt
// leaves them alone and everything stays green — while every newly authored def
// goes back to being invisible. Deleting `format_strings=true` was measured to do
// exactly that: gate green, both tests above passing, 79% of new cards unchecked.
//
// `sr35_adversarial_demo.sh` proves the flags matter, but nothing executes it, and
// this repo already learned that lesson the expensive way (SR-9b): "a documented
// hazard that nothing executes is a hazard, not a note." So the demo's two
// load-bearing fixtures are re-run here, in `cargo test`, against a throwaway
// corpus.
//
// Each fixture isolates exactly one flag — verified across the full matrix:
//
//     fixture                  both   fs-only   eolo-only   neither
//     long one-line string     RED    RED       GREEN       GREEN
//     unbreakable long line    RED    GREEN     RED         GREEN
//
// so a canary asserting RED reddens if, and only if, its flag goes missing.
// ---------------------------------------------------------------------------

/// A def written the way an author writes one: `oracle_text` as a single long
/// line. rustfmt cannot fit it, falls back to verbatim source for the enclosing
/// expression — the whole `CardDefinition` literal — and exits 0, hiding the
/// misindented `card_id`. `format_strings=true` is what makes it visible.
const FIXTURE_LONG_ORACLE_TEXT: &str = r#"use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
                card_id: cid("zz-canary"),
        name: "ZZ Canary".to_string(),
        oracle_text: "Whenever a creature you control deals combat damage to a player, draw a card. Then if creatures you control have total toughness 20 or greater, untap each creature you control.".to_string(),
        ..Default::default()
    }
}
"#;

/// A line rustfmt can neither fit nor break (one over-long path). Same silent
/// bail, same hidden `card_id`. `error_on_line_overflow=true` is what makes it
/// visible. The path is synthetic — no real def has a token that wide today —
/// but the mechanism is the one that left 51 files unformatted during SR-33.
/// rustfmt only parses here, so the fake type names are irrelevant.
const FIXTURE_UNBREAKABLE_LINE: &str = r#"use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
                card_id: cid("zz-canary"),
        name: SomeEnum::AVeryLongVariantNameThatCannotBeBrokenAnywhereAtAllBecauseItIsOneSingleIdentifierToken::Nested,
        ..Default::default()
    }
}
"#;

/// Stand up a throwaway corpus containing exactly `fixture` and run the SHIPPED
/// script against it. The script derives its defs dir from its own location, so
/// copying it into a temp tree is all that is needed — which means these canaries
/// exercise the real argument set and cannot drift from it. Returns true if the
/// gate rejected the fixture.
fn shipped_gate_rejects(label: &str, fixture: &str) -> bool {
    let root = workspace_root();
    let tmp = std::env::temp_dir().join(format!("sr35_canary_{}_{}", label, std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let defs = tmp.join("crates/card-defs/src/defs");
    std::fs::create_dir_all(&defs).expect("create temp defs dir");
    std::fs::create_dir_all(tmp.join("tools")).expect("create temp tools dir");
    std::fs::copy(
        root.join("tools/check-defs-fmt.sh"),
        tmp.join("tools/check-defs-fmt.sh"),
    )
    .expect("copy gate script");
    std::fs::write(defs.join("zz_canary.rs"), fixture).expect("write fixture");

    let out = Command::new("bash")
        .arg(tmp.join("tools/check-defs-fmt.sh"))
        .current_dir(&tmp)
        .output()
        .expect("run gate on temp corpus");

    // Clean up before asserting, so a failed guard does not leak the temp tree.
    let _ = std::fs::remove_dir_all(&tmp);

    // Guard against a vacuous pass: the gate must have actually seen the fixture.
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("1 defs checked"),
        "the temp corpus was not picked up by the gate — this canary proves nothing. \
         stdout: {stdout}"
    );

    !out.status.success()
}

#[test]
fn gate_catches_a_def_whose_oracle_text_is_one_long_line() {
    assert!(
        shipped_gate_rejects("fs", FIXTURE_LONG_ORACLE_TEXT),
        "The gate PASSED a def with a blatantly misindented `card_id`, because a long \
         one-line `oracle_text` makes rustfmt give up on the whole file and exit 0.\n\n\
         This almost certainly means `--config format_strings=true` was dropped from \
         tools/check-defs-fmt.sh. Without it the gate still reports every existing def \
         clean — they already carry `\\` continuations — while every newly authored card \
         is unchecked. That was 1,380 of 1,748 defs before SR-35. Put the flag back."
    );
}

#[test]
fn gate_catches_an_unbreakable_over_width_line() {
    assert!(
        shipped_gate_rejects("eolo", FIXTURE_UNBREAKABLE_LINE),
        "The gate PASSED a def with a misindented `card_id`, because an over-width line \
         rustfmt cannot break makes it give up on the file and exit 0.\n\n\
         This almost certainly means `--config error_on_line_overflow=true` was dropped \
         from tools/check-defs-fmt.sh. The corpus has no such lines today, so nothing \
         else would notice. Put the flag back."
    );
}

/// The script passes `--edition` explicitly (a bare file list carries no Cargo
/// manifest, so rustfmt cannot infer it) — which means it is a second copy of a
/// fact owned by `Cargo.toml`, and copies drift. An edition bump that misses the
/// script would silently format the corpus under the old edition.
#[test]
fn the_gate_edition_matches_the_workspace() {
    let root = workspace_root();
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("workspace Cargo.toml");

    // Scoped to the `[workspace.package]` table rather than grepping the whole
    // file: `edition` is a legal key in other tables, and a first-match search
    // would silently start comparing against the wrong one if a future edit adds
    // any table above this one. (`edition.workspace = true` lines in member
    // manifests never parse as a number, but that is luck, not a guarantee.)
    let declared = manifest
        .lines()
        .skip_while(|l| l.trim() != "[workspace.package]")
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .find_map(|l| {
            let l = l.trim();
            l.strip_prefix("edition")?
                .trim_start()
                .strip_prefix('=')?
                .trim()
                .trim_matches('"')
                .parse::<u32>()
                .ok()
        })
        .expect("workspace Cargo.toml has [workspace.package] edition = \"<year>\"");

    let script =
        std::fs::read_to_string(root.join("tools/check-defs-fmt.sh")).expect("gate script");
    let used = script
        .lines()
        .find_map(|l| {
            l.trim()
                .strip_prefix("--edition ")?
                .trim()
                .parse::<u32>()
                .ok()
        })
        .expect("gate script passes --edition");

    assert_eq!(
        used, declared,
        "tools/check-defs-fmt.sh formats the card defs under edition {used}, but the \
         workspace is edition {declared}. Update the script's --edition."
    );
}
