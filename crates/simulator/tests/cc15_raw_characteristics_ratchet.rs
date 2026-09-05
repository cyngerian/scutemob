//! CC-15 (`scutemob-252`, 2026-09-05) — per-file ceiling ratchet over raw `.characteristics.`
//! reads in `crates/simulator/src`. Course-correction addendum A1, accepted with the
//! battlefield qualifier (`docs/course-correction-2026-09.md` §9.1).
//!
//! **What it guards.** The simulator's offer layer is a second legality implementation: every
//! SR-38 defect on record ("clean offer, guaranteed refusal" — PB-DX20, PB-DX29, PB-DX44,
//! PB-DX45, PB-DX50, PB-DX51 `OOS-DX51-3`, PB-DX55, PB-DX20b `OOS-DX20b-1`) has the same
//! mechanism: the simulator re-derives a predicate the engine already owns, and the two drift.
//! A raw `obj.characteristics.<field>` read is the printed value with no layer walk, so a
//! Layer-4 type change or a granted Defender is invisible to the offer while the engine (which
//! reads `calculate_characteristics`) refuses. The correct direction already exists —
//! `rules/queries.rs` — and the **split-on-touch** rule in `memory/conventions.md` says a batch
//! that touches an offer routes it through a query and never through a raw read. This ratchet
//! is what makes that rule cost something: a file's count may only ever go DOWN.
//!
//! **Battlefield qualifier — why the ceilings are not zero and are not all defects.** A raw read
//! on an object in HAND or LIBRARY ("is this a land I can play", a castable-cost read on a card
//! in hand) is CORRECT: no continuous effect applies to an object off the battlefield, so the
//! printed value IS the layer-resolved value there. The ceilings are therefore a residue to walk
//! down as offers are touched, not a defect count; several are legitimate and will
//! stay (several of the 47). Lower on touch, never raise. Splitting a raw read into a query does not require
//! rewriting the file — `queries.rs` gains a function, the offer calls it.
//!
//! **Pairing (CC-17, pair-or-demote).** This is a source-text gate and therefore a PROXY. Its
//! behavioural half is the existing SR-38 channel-probe family in this directory, which drives
//! the offer layer and asserts the engine ACCEPTS what was offered (or that a refused shape is
//! no longer offered):
//! `pb_dx20b_enchant_offer_channel.rs`, `pb_dx51_blocker_offer.rs`,
//! `pb_dx55_blocker_offer_mirrors_the_engine.rs`, `pb_dx55_activation_auto_tap.rs`,
//! `pb_dx55_modal_activated_channel.rs`, `pb_dx50_mutate_legality_channel.rs`,
//! `pb_dx44_split_half_channel.rs`, `pb_dx44_pitch_channel.rs`, `pb_dx45_optional_cost_channel.rs`,
//! `pb_dx29_cost_kind_surface.rs`, `sim5_bot_cast_discipline.rs`, plus the fuzzer's rejection
//! channel pinned in `pb_dx32_fuzz_output.rs` (`total_rejections`). Those probes are the verdict
//! on whether an offer is honest; this ratchet only says whether a batch added a new place where
//! it could stop being honest. Per CC-17 a paired gate is a backstop and needs no bypass matrix.
//!
//! **Counting rule** (SR-25 `bare_lookup_ratchet` technique): strip `//`-to-end-of-line
//! comments, remove ALL whitespace, count the substring `.characteristics.`. Whitespace removal
//! makes the count rustfmt-stable and un-evadable by line-splitting; comment stripping keeps
//! a doc comment that names the field from inflating a ceiling. `calculate_characteristics(`
//! does not contain the needle (no leading `.`), so layer-resolved reads are never counted.
//! Known limitation shared with every source-scan gate here: `/* */` block comments are not
//! stripped. **The addendum's "43 at HEAD" was a grep-LINE count and is wrong in both
//! directions**: it included three comment-only mentions (`local_game.rs` ×1, `mana_solver.rs`
//! ×2) and it MISSED six reads whose method chain rustfmt had wrapped across lines
//! (`obj\n.characteristics\n.field` — one in `heuristic_bot.rs`, five in `legal_actions.rs`),
//! which no per-line grep can see. The whitespace-blind, comment-stripped count pinned here is
//! **47** (2 + 14 + 28 + 1 + 1 + 1).
//!
//! **Executed defeats, recorded per the change-class table row 4** (run 2026-09-05 at HEAD,
//! each restored byte-identically): (1) lowering `legal_actions.rs`'s ceiling 28 → 27 → RED
//! (the "up from the pinned" branch: actual 28 > pinned 27); (2) raising it 28 → 29 → RED
//! (the "down to … tighten" branch: the two-sided pin refuses slack in either direction); (3) planting `obj.characteristics.power`
//! in `local_game.rs`, a file NOT on the roster → RED via the directory walk; (4) planting the
//! same read split across three lines → RED (whitespace-blind); (5) planting it inside a `//`
//! comment → GREEN (a comment is not a read, and that is the intended non-match).

use std::fs;
use std::path::{Path, PathBuf};

/// Per-file ceilings, pinned at the non-comment count measured 2026-09-05. Files may be ADDED
/// to this roster (with their measured count) and ceilings may be LOWERED; neither may be
/// raised. A file absent from the roster must have a count of ZERO — the directory walk below
/// enforces that, so a new file cannot smuggle raw reads past a hand-written list (PB-DX57's
/// lesson about rosters that rot).
const PINNED: &[(&str, usize)] = &[
    ("src/heuristic_bot.rs", 2),
    ("src/invariants.rs", 14),
    ("src/legal_actions.rs", 28),
    ("src/mana_solver.rs", 1),
    ("src/params.rs", 1),
    ("src/random_bot.rs", 1),
];

/// Denominator guard: a broken counter or a mis-pathed roster collapses the total; a real scan
/// finds dozens. Set below the live total (47) with room for the walk-down.
const MIN_TOTAL: usize = 20;

const NEEDLE: &str = ".characteristics.";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: impl AsRef<Path>) -> String {
    let path = crate_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Strip `//` comments and all whitespace, then count the needle.
fn raw_characteristics_count(src: &str) -> usize {
    let decommented: String = src
        .lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let collapsed: String = decommented.chars().filter(|c| !c.is_whitespace()).collect();
    collapsed.matches(NEEDLE).count()
}

fn rs_files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
    {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rs_files_under(&path, out);
        } else if path.extension().is_some_and(|x| x == "rs") {
            out.push(path);
        }
    }
}

/// The ratchet: every rostered file's count equals its ceiling (two-sided), every unrostered
/// file under `crates/simulator/src` counts zero, and the total clears the denominator floor.
#[test]
fn raw_characteristics_counts_are_pinned() {
    let mut total = 0usize;
    for &(rel, ceiling) in PINNED {
        let src = read(rel);
        assert!(
            src.len() > 200,
            "CC-15: {rel} is suspiciously small ({} bytes) — wrong path? A misread file would \
             report 0 reads and pass a 0 ceiling vacuously.",
            src.len()
        );
        let count = raw_characteristics_count(&src);
        total += count;
        if count > ceiling {
            panic!(
                "CC-15 ratchet: {rel} now has {count} raw `.characteristics.` reads, up from the \
                 pinned {ceiling}. A raw read on a BATTLEFIELD object is the printed value with no \
                 layer walk — the SR-38 shape (clean offer, guaranteed refusal). Split on touch: \
                 route the offer through `mtg_engine::rules::queries` (add a query there if none \
                 fits) and read `calculate_characteristics` for anything on the battlefield. If \
                 the new read is on a HAND or LIBRARY object it is correct — say so in a comment \
                 at the site and still prefer a query; raising this ceiling is not an option."
            );
        }
        if count < ceiling {
            panic!(
                "CC-15 ratchet: {rel} is down to {count} raw `.characteristics.` reads from the \
                 pinned {ceiling} — good. Lower its ceiling in PINNED to {count} so the ratchet \
                 keeps the gain (a stale-high ceiling is slack a regression hides in — PB-DX49)."
            );
        }
    }

    // Directory walk: a file not on the roster must contribute nothing. This is what stops a
    // new module from carrying raw reads past a hand-written list.
    let src_dir = crate_root().join("src");
    let mut files = Vec::new();
    rs_files_under(&src_dir, &mut files);
    assert!(
        files.len() > PINNED.len(),
        "CC-15: the walk found only {} files",
        files.len()
    );
    for path in files {
        let rel = path.strip_prefix(crate_root()).expect("under crate root");
        let rel = rel.to_string_lossy().replace('\\', "/");
        if PINNED.iter().any(|(r, _)| *r == rel) {
            continue;
        }
        let count = raw_characteristics_count(&read(&rel));
        assert_eq!(
            count, 0,
            "CC-15 ratchet: {rel} is not on the PINNED roster and has {count} raw \
             `.characteristics.` reads. Either route them through `rules::queries` (preferred) \
             or add the file to PINNED at its measured count with the reason at each site."
        );
    }

    assert!(
        total >= MIN_TOTAL,
        "CC-15 denominator guard: the scan found only {total} raw reads (< {MIN_TOTAL}); the \
         counter or the roster paths are probably broken — a real scan finds dozens."
    );
}

/// Non-vacuity: the counter sees a read, ignores `//` comments, is blind to whitespace, and
/// does NOT count the layer-resolved spelling.
#[test]
fn counter_is_non_vacuous() {
    assert_eq!(
        raw_characteristics_count("let p = obj.characteristics.power;"),
        1
    );
    assert_eq!(
        raw_characteristics_count("// obj.characteristics.power in a comment\nlet x = 1;"),
        0
    );
    assert_eq!(
        raw_characteristics_count("let p = obj\n    .characteristics\n    .power;"),
        1,
        "a line-wrapped chain must count exactly like the inline form"
    );
    assert_eq!(
        raw_characteristics_count(
            "let c = calculate_characteristics(&state, id); let p = c.power;"
        ),
        0,
        "the layer-resolved read has no leading `.` before `characteristics` and must not count"
    );
    assert_eq!(
        raw_characteristics_count(
            "a.characteristics.x; b.characteristics.y; // c.characteristics.z"
        ),
        2
    );
}

/// The destination the ratchet points at must keep existing, or its failure message sends a
/// worker to a module that is gone.
#[test]
fn queries_module_still_exists() {
    let path = crate_root().join("../engine/src/rules/queries.rs");
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(
        src.contains("pub fn "),
        "rules/queries.rs has no public query — the ratchet's remedy vanished"
    );
}
