//! PB-DX18 — the structural gates the batch's own source comments promise.
//!
//! Three claims in production source say "see this file". Each is here, and each is
//! keyed on a MECHANISM rather than on a spelling, because this queue keeps recording
//! gates that measured the one syntactic form their author happened to write
//! (`OOS-DX47`'s `r3`, PB-DX26, PB-DX43, PB-DX45).
//!
//! * **r1** — every `ZoneChangeAction::Redirect` arm in the engine discharges the CR
//!   701.20 shuffle obligation. `Redirect` is destructured with `..` at every consumer,
//!   so **the compiler cannot enforce this** and a 22nd consumer would silently drop a
//!   real shuffle back to a phantom event.
//! * **r2** — the CR 702.47a splice target-index precondition: the spliced effect's own
//!   `DeclaredTarget` indices are relative to the spliced card's text while resolution
//!   hands the splice context the spell's WHOLE target list, so a host that declares
//!   targets of its own would need an offset that PB-DX18 deliberately did not build.
//!   The offset is provably **zero** for every combination the corpus can reach; this
//!   pins that precondition so the day it stops holding is a red test.
//! * **r3** — the CR 702.94a just-drawn record is written UNCONDITIONALLY at the draw
//!   site, so a non-eligible draw CLEARS it. `mechanics_m_z/miracle.rs`'s `t5` covers the
//!   turn boundary behaviourally; the same-turn half needs `perform_one_draw`, which is
//!   `pub(crate)`, so it is pinned structurally here and said so there.
//! * **r4** — the census behind `OOS-DX18-2`: `mod`-declared test modules that contain
//!   no tests at all. `mechanics_m_z/miracle.rs` was one of them until this batch, which
//!   is a large part of why `OOS-DX2-1` survived — a `mod` line naming an empty file
//!   reads exactly like coverage.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mtg_engine::{all_cards, AbilityDefinition, SubType};

fn repo_root() -> PathBuf {
    // `CARGO_MANIFEST_DIR` is `crates/engine`.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

/// Every `.rs` file under `crates/engine/src`, walked rather than listed —
/// `OOS-DX49-6`'s lesson that a hardcoded file list is a claim, and PB-DX48's that a
/// one-crate walk misses a consumer one crate up (the redirect consumers are all
/// engine-internal because `ZoneChangeAction` is only constructed there, which `r1b`
/// checks rather than assumes).
fn engine_src_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).expect("readable dir") {
            let p = e.expect("dir entry").path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(&repo_root().join("crates/engine/src"), &mut out);
    out.sort();
    out
}

// ── r1: every Redirect consumer discharges the CR 701.20 obligation ───────────

#[test]
/// CR 701.20 (`OOS-DP2-7`) — a `ZoneChangeAction::Redirect` match arm that moves the
/// object must also call `GameState::finish_redirect_shuffle`.
///
/// **Why a gate and not the compiler.** Every consumer destructures with `..`, so adding
/// the `shuffle_destination_after` field broke exactly ZERO of them. A 22nd consumer
/// written tomorrow would compile, move the card to the library, and never shuffle —
/// which is the pre-PB-DX18 defect exactly, reintroduced silently.
///
/// The arm is delimited by brace matching from its own `=> {`, not by a fixed byte
/// window: `OOS-DX49`'s `r5b` was found to be over-scanning a fixed 4,000-byte window
/// into the NEXT arm, and an under-scanning window would miss a discharge and fail
/// closed noisily instead. Brace matching has neither failure.
fn r1_every_redirect_consumer_discharges_the_shuffle_obligation() {
    const MOVE_HELPERS: [&str; 3] = [
        "expect_move_object_to_zone",
        "fizzle_move_object_to_zone",
        "move_object_to_zone",
    ];
    let mut arms = 0usize;
    let mut non_moving = 0usize;
    let mut missing: Vec<String> = Vec::new();
    for path in engine_src_files() {
        let src = read(
            path.strip_prefix(repo_root())
                .expect("under root")
                .to_str()
                .expect("utf8"),
        );
        // The DEFINITION site and the two construction sites in `replacement.rs` are not
        // consumers; a consumer is a `match` ARM, which is what `=> {` after the pattern
        // marks.
        let mut idx = 0usize;
        while let Some(found) = src[idx..].find("ZoneChangeAction::Redirect {") {
            let start = idx + found;
            idx = start + 1;
            // Find the arm's `=> {`; if the pattern is a struct LITERAL (a construction)
            // there is none before the closing brace, and it is skipped.
            let after = &src[start..];
            let Some(arrow) = after.find("} => {") else {
                continue;
            };
            // A construction site's `}` closes the literal; require the `=> {` to be
            // close (within the pattern's own field list) rather than anywhere later.
            if arrow > 400 {
                continue;
            }
            arms += 1;
            // Brace-match the arm body from the `{` of `=> {`.
            let body_start = start + arrow + "} => {".len();
            let bytes = src.as_bytes();
            let mut depth = 1i32;
            let mut i = body_start;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            let body = &src[body_start..i.min(src.len())];
            // THE MECHANISM: an arm only owes a discharge if it MOVES the object. The
            // `resolve_pending_zone_change` arm destructures a chained redirect purely to
            // read its destination and its own obligation, and the move happens after the
            // match — flagging it would be a false positive of the gate's SHAPE, and the
            // fix for a shape false-positive is to key on the mechanism, not to allowlist
            // the site (`OOS-DX47`'s `r3`). The three move helpers are enumerated rather
            // than matched loosely, and `r1c` gates that enumeration against the engine's
            // own set so a fourth cannot appear behind this gate's back.
            let moves = MOVE_HELPERS.iter().any(|h| body.contains(h));
            // THE CALL IS NOT THE PROPERTY — the BOUND FIELD reaching it is.
            //
            // This gate's first draft looked for the call by name and was DEFEATED by its
            // own revert row R2: `state.finish_redirect_shuffle(false, to, &mut events)`
            // contains the name, drops the obligation entirely, and left the gate GREEN.
            // That is `OOS-DX47`'s `r3` shape — a gate keyed on a spelling measures the
            // spelling — committed inside the batch whose roster file says so. Found by
            // executing the revert rather than by argument, and fixed by requiring the
            // field the arm binds to appear inside the call's own argument list.
            let discharged = body
                .match_indices("finish_redirect_shuffle(")
                .any(|(at, _)| {
                    let rest = &body[at..];
                    let end = rest.find(')').unwrap_or(rest.len());
                    rest[..end].contains("shuffle_destination_after")
                });
            if moves && !discharged {
                let line = src[..start].matches('\n').count() + 1;
                missing.push(format!("{}:{}", path.display(), line));
            }
            if !moves {
                non_moving += 1;
            }
        }
    }
    // Non-vacuity, both halves. The walk must find the consumers, AND the moving/
    // non-moving split must be the measured one — if every arm suddenly looked
    // non-moving the gate would pass while checking nothing.
    eprintln!("r1: {arms} Redirect arms, {non_moving} of them non-moving");
    assert_eq!(
        non_moving, 1,
        "r1 measured {non_moving} non-moving `Redirect` arms; exactly ONE is expected \
         (`resolve_pending_zone_change`'s chained-redirect read, which moves after the \
         match). A second one is either a new consumer that forgot to move or a \
         `MOVE_HELPERS` entry that has gone stale — both are findings."
    );
    assert!(
        arms >= 20,
        "r1 found only {arms} `ZoneChangeAction::Redirect` match arms in crates/engine/src; \
         the scanner is broken (PB-DX18 converted 21 of them)"
    );
    assert!(
        missing.is_empty(),
        "CR 701.20 (`OOS-DP2-7`): {} `ZoneChangeAction::Redirect` arm(s) move the object \
         and never call `GameState::finish_redirect_shuffle`, so a \
         `ReplacementModification::ShuffleIntoOwnerLibrary` redirect reaching them emits \
         a PHANTOM `LibraryShuffled` and leaves the card on the library TOP — the exact \
         pre-PB-DX18 defect. Sites: {:?}",
        missing.len(),
        missing
    );
}

#[test]
/// `r1`'s scope claim, checked rather than asserted: `ZoneChangeAction` is constructed
/// only inside `crates/engine/src`, so walking that tree is the whole consumer set.
fn r1b_zone_change_action_is_engine_internal() {
    let mut outside: Vec<String> = Vec::new();
    for rel in [
        "crates/simulator/src",
        "crates/view-model/src",
        "crates/card-types/src",
        "tools/play-server/src",
        "tools/tui/src",
        "tools/replay-viewer/src",
    ] {
        let dir = repo_root().join(rel);
        if !dir.exists() {
            continue;
        }
        let mut files = Vec::new();
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            for e in std::fs::read_dir(dir).expect("readable dir") {
                let p = e.expect("entry").path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                    out.push(p);
                }
            }
        }
        walk(&dir, &mut files);
        for f in files {
            let s = std::fs::read_to_string(&f).expect("readable");
            if s.contains("ZoneChangeAction::Redirect") {
                outside.push(f.display().to_string());
            }
        }
    }
    assert!(
        outside.is_empty(),
        "r1 walks `crates/engine/src` only, and that is sound ONLY while every \
         `ZoneChangeAction::Redirect` consumer lives there. These do not: {:?}",
        outside
    );
}

// ── r2: the CR 702.47a splice index-offset precondition ──────────────────────

#[test]
/// CR 702.47a (`OOS-M11-5`) — every reachable (host, splice card) pair has a host that
/// declares **zero** targets, so the spliced effect's own `DeclaredTarget` indices need
/// no offset.
///
/// PB-DX18 appends the spliced card's requirements to the host's (the PB-DX50 append
/// rule, so host indices never move) and does **not** offset the SPLICED effect's
/// indices, because `resolution.rs` hands the splice context the spell's whole target
/// list. That is correct exactly while every host contributes 0 targets. This pins the
/// precondition; PB-DX44's fuse offset is the machinery the day it fails.
fn r2_no_reachable_splice_host_declares_targets_of_its_own() {
    let defs = all_cards();
    // The splice cards, and the subtypes they splice onto.
    let mut splice_onto: Vec<(String, SubType, usize)> = Vec::new();
    for d in &defs {
        for a in &d.abilities {
            if let AbilityDefinition::Splice {
                onto_subtype,
                targets,
                ..
            } = a
            {
                splice_onto.push((d.name.clone(), onto_subtype.clone(), targets.len()));
            }
        }
    }
    assert!(
        !splice_onto.is_empty(),
        "r2 is vacuous: no def in the corpus declares AbilityDefinition::Splice"
    );

    // Every def that could be a HOST for one of them, and how many targets it declares.
    let mut offenders: Vec<(String, String, usize)> = Vec::new();
    for (splice_name, onto, _) in &splice_onto {
        for d in &defs {
            if !d.types.subtypes.contains(onto) {
                continue;
            }
            // CR 702.47a ruling: a card cannot be spliced onto itself.
            if &d.name == splice_name {
                continue;
            }
            let host_targets: usize = d
                .abilities
                .iter()
                .filter_map(|a| match a {
                    AbilityDefinition::Spell { targets, .. } => Some(targets.len()),
                    _ => None,
                })
                .sum();
            if host_targets > 0 {
                offenders.push((splice_name.clone(), d.name.clone(), host_targets));
            }
        }
    }
    eprintln!("r2 splice cards: {splice_onto:?}");
    assert!(
        offenders.is_empty(),
        "CR 702.47a (`OOS-M11-5`): a splice card can now be spliced onto a host that \
         declares targets of its OWN, so the spliced effect's `DeclaredTarget` indices \
         need an offset by the host's target count and PB-DX18 built none. Offending \
         (splice card, host, host target count): {:?}",
        offenders
    );
}

// ── r3: the just-drawn miracle record is written unconditionally ─────────────

#[test]
/// CR 702.94a (`OOS-DX2-1`) — `perform_one_draw`'s completed-draw path assigns
/// `miracle_pending` on BOTH branches, so a non-eligible draw clears it.
///
/// Written as a source gate because `rules::replacement::perform_one_draw` is
/// `pub(crate)` and no public channel drives a second in-turn draw on the miracle
/// fixture without also crossing the turn boundary (`mechanics_m_z/miracle.rs`'s `t5`
/// covers that half behaviourally, and says so). The FAILING shape this catches is an
/// edit to `if let Some(ev) = check_miracle_eligible(..) { p.miracle_pending = Some(..) }`
/// — which looks equivalent and leaves a stale id answerable for the rest of the turn.
fn r3_the_just_drawn_record_is_assigned_unconditionally() {
    let src = read("crates/engine/src/rules/replacement.rs");
    let anchor = src
        .find("check_miracle_eligible(")
        .expect("perform_one_draw must still call check_miracle_eligible");
    // Brace-free window: the assignment must appear as a `map`/`as_ref().map(..)` over
    // the OPTION, not inside a conditional that only fires when it is `Some`.
    let window = &src[anchor..(anchor + 700).min(src.len())];
    assert!(
        window.contains("p.miracle_pending ="),
        "CR 702.94a: the draw site must record the just-drawn object; nothing assigns \
         `miracle_pending` within 700 bytes of `check_miracle_eligible`"
    );
    let assign = window
        .find("p.miracle_pending =")
        .expect("checked just above");
    let stmt_end = window[assign..]
        .find(';')
        .map(|o| assign + o)
        .unwrap_or(window.len());
    let stmt = &window[assign..stmt_end];
    assert!(
        stmt.contains(".map("),
        "CR 702.94a (`OOS-DX2-1`): `miracle_pending` must be assigned from the OPTION \
         (`miracle_event.as_ref().map(|_| new_id)`), so a draw that is NOT miracle-\
         eligible writes `None` and clears any stale record. The statement found is \
         `{stmt}`, which does not map over the option — if it is now guarded by an \
         `if let Some(..)`, a tutored miracle card becomes answerable again for the rest \
         of the turn."
    );
}

// ── r4: the empty-but-mod'd test module census (`OOS-DX18-2`) ────────────────

#[test]
/// `OOS-DX18-2` — a `mod` line naming a file with no `#[test]` in it reads exactly like
/// coverage, and `mechanics_m_z/miracle.rs` was one for the whole life of SR-9a's
/// consolidation. That is a large part of why `OOS-DX2-1` survived: miracle's only
/// coverage in the tree was a single golden script.
///
/// Pinned by NAME, not by count, so a new empty module cannot join by coincidence
/// (PB-DX28's rule). SR-9a's own gate checks that no `mod` line was DROPPED; nothing
/// checked that a `mod`'d file contains anything.
fn r4_no_mod_declared_test_module_is_empty() {
    let groups = [
        "casting",
        "combat",
        "core",
        "mechanics_a_d",
        "mechanics_e_l",
        "mechanics_m_z",
        "primitives",
        "rules",
        "scripts",
    ];
    let mut empty: BTreeSet<String> = BTreeSet::new();
    let mut helpers: BTreeSet<String> = BTreeSet::new();
    let mut scanned = 0usize;
    for g in groups {
        let dir = repo_root().join("crates/engine/tests").join(g);
        if !dir.exists() {
            continue;
        }
        let main = std::fs::read_to_string(dir.join("main.rs")).expect("group has a main.rs");
        for line in main.lines() {
            let l = line.trim();
            let Some(rest) = l.strip_prefix("mod ") else {
                continue;
            };
            let Some(name) = rest.strip_suffix(';') else {
                continue;
            };
            let f = dir.join(format!("{name}.rs"));
            if !f.exists() {
                continue;
            }
            scanned += 1;
            let s = std::fs::read_to_string(&f).expect("readable");
            if s.contains("#[test]") {
                continue;
            }
            // TWO shapes, separated, because they are different findings. A module with
            // no tests but with `pub` items is a shared HELPER — legitimate, and the
            // first draft of this gate lumped it in and reported a false positive. A
            // module with neither is an EMPTY FILE whose `mod` line reads as coverage.
            if s.contains("pub fn ") || s.contains("pub const ") || s.contains("pub struct ") {
                helpers.insert(format!("{g}/{name}.rs"));
            } else {
                empty.insert(format!("{g}/{name}.rs"));
            }
        }
    }
    assert!(
        scanned >= 100,
        "r4 scanned only {scanned} mod-declared test modules; the walker is broken"
    );
    eprintln!("r4 empty: {empty:?}");
    eprintln!("r4 test-free helpers: {helpers:?}");
    // Test-free HELPER modules, named. These are fine — they exist so siblings can share
    // a predicate — but they are named rather than counted so a genuinely empty module
    // cannot hide among them.
    let expected_helpers: BTreeSet<String> = ["core/decision_site_walk.rs".to_string()]
        .into_iter()
        .collect();
    assert_eq!(
        helpers, expected_helpers,
        "the set of test-free `mod`-declared HELPER modules moved. A helper is a module \
         with no `#[test]` that exports something; if a new one appeared, name it here, \
         and if it exports nothing it belongs in the EMPTY set below instead."
    );
    // KNOWN and named. `rules/effects.rs` is the survivor of the same SR-9a
    // consolidation and is left as a finding rather than filled speculatively — PB-DX18
    // filled `mechanics_m_z/miracle.rs` because miracle is this batch's subject.
    let expected: BTreeSet<String> = ["rules/effects.rs".to_string()].into_iter().collect();
    assert_eq!(
        empty, expected,
        "`OOS-DX18-2`: the set of `mod`-declared test modules containing no `#[test]` \
         moved. A module whose name promises coverage and contains none is invisible to \
         SR-9a's gate (which checks that a mod line was not DROPPED) and reads as \
         coverage to every reader. Either a new empty module appeared — file it — or \
         `rules/effects.rs` was filled, in which case delete this expectation."
    );
}
