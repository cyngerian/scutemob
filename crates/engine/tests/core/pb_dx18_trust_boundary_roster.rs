//! PB-DX18 — the structural gates the batch's own source comments promise.
//!
//! Three claims in production source say "see this file". Each is here, and each is
//! keyed on a MECHANISM rather than on a spelling, because this queue keeps recording
//! gates that measured the one syntactic form their author happened to write
//! (`OOS-DX47`'s `r3`, PB-DX26, PB-DX43, PB-DX45).
//!
//! * **r1** — every `ZoneChangeAction::Redirect` arm in the engine discharges the CR
//!   701.24 shuffle obligation. `Redirect` is destructured with `..` at every consumer,
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

// ── r1: every Redirect consumer discharges the CR 701.24 obligation ───────────

#[test]
/// CR 701.24 (`OOS-DP2-7`) — a `ZoneChangeAction::Redirect` match arm that moves the
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
            // Find where the PATTERN ends and the body begins. Two consumer forms exist
            // and BOTH must be seen:
            //
            //   * a `match` arm  — `ZoneChangeAction::Redirect { .. } => {`
            //   * an `if let`    — `if let ZoneChangeAction::Redirect { .. } = action {`
            //
            // The first draft of this gate looked for `} => {` alone and was DEFEATED by
            // the `/review`, which planted an ordinary `if let` consumer that moves the
            // object to the destination library and never discharges the obligation — i.e.
            // it re-created `OOS-DP2-7` exactly — and every row in this file stayed GREEN.
            // That is the second time this gate measured a SPELLING (the first was revert
            // row R2), inside the file whose module doc says a gate must be keyed on the
            // mechanism. A construction site (a struct literal) has neither terminator
            // before its closing brace and is still skipped.
            let after = &src[start..];
            let arm = after.find("} => {").map(|o| (o, "} => {".len()));
            let iflet = after.find("} = ").and_then(|o| {
                // `if let PATTERN { .. } = expr {` — the body starts at the `{` that ends
                // that line. Bounded to the same window so an unrelated later `} = ` in
                // the file cannot be mistaken for this pattern's terminator.
                after[o..]
                    .find(" {\n")
                    .map(|k| (o + k + " {".len(), 1usize))
            });
            let terminator = match (arm, iflet) {
                (Some(a), Some(i)) if i.0 < a.0 => Some(i),
                (Some(a), _) => Some((a.0, a.1)),
                (None, Some(i)) => Some(i),
                (None, None) => None,
            };
            let Some((rel, skip)) = terminator else {
                continue;
            };
            // A construction site's `}` closes the literal; require the terminator to be
            // close (within the pattern's own field list) rather than anywhere later.
            if rel > 400 {
                continue;
            }
            arms += 1;
            // Brace-match the arm body from its opening `{`.
            let body_start = start + rel + skip;
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
            // (`r1c` did not exist when this comment was first written — a comment
            // asserting an enforcement that was not there, caught by the `/review`,
            // inside the file about gates that claim more than they check. It exists now.)
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
        "CR 701.24 (`OOS-DP2-7`): {} `ZoneChangeAction::Redirect` arm(s) move the object \
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
    let mut test_consumers: Vec<String> = Vec::new();
    let mut roots: Vec<PathBuf> = Vec::new();
    // WALKED, not listed. The first draft named six directories and omitted
    // `crates/network`, `crates/card-db`, `crates/card-pipeline` and every `tests/` tree —
    // "a hardcoded file list is a claim" (`OOS-DX49-6`), one function below the doc that
    // cites it. Everything under `crates/` and `tools/` except the engine's own `src` is
    // in scope now.
    for top in ["crates", "tools"] {
        let base = repo_root().join(top);
        if !base.exists() {
            continue;
        }
        for e in std::fs::read_dir(&base).expect("readable") {
            let p = e.expect("entry").path();
            if p.is_dir() {
                roots.push(p);
            }
        }
    }
    let engine_src = repo_root().join("crates/engine/src");
    for dir in roots {
        if dir == repo_root().join("crates/engine") {
            // The engine's own `tests/` tree still counts; only its `src` is r1's scope.
        }
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
        let mut files = Vec::new();
        walk(&dir, &mut files);
        for f in files {
            if f.starts_with(&engine_src) {
                continue; // r1's own scope
            }
            let s = std::fs::read_to_string(&f).expect("readable");
            if !s.contains("ZoneChangeAction::Redirect") {
                continue;
            }
            let rel = f
                .strip_prefix(repo_root())
                .unwrap_or(&f)
                .display()
                .to_string();
            // TEST and BENCH consumers are listed, not asserted against — and they are
            // PRINTED rather than silently filtered, so "out of scope" stays visible. A
            // test that destructures the action to inspect it owes nothing; a test that
            // MOVES the object (the darksteel probe) discharges through
            // `test_util::finish_redirect_shuffle`, which is exactly what makes it a probe
            // of the production obligation rather than a re-implementation of it.
            if rel.contains("/tests/") || rel.contains("/benches/") {
                test_consumers.push(rel);
            } else {
                outside.push(rel);
            }
        }
    }
    test_consumers.sort();
    eprintln!("r1b test/bench consumers (out of r1's scope, listed): {test_consumers:?}");
    assert!(
        outside.is_empty(),
        "r1 walks `crates/engine/src` only, and that is sound ONLY while every PRODUCTION \
         `ZoneChangeAction::Redirect` consumer lives there. These do not: {:?}",
        outside
    );
    // NON-VACUITY: the walk really does reach other crates' trees. If this floor trips,
    // the walker broke and `outside.is_empty()` above is meaningless.
    assert!(
        !test_consumers.is_empty(),
        "r1b found no consumer anywhere outside `crates/engine/src`, not even in the test \
         tree — the walk is broken, and the emptiness of `outside` proves nothing"
    );
}

#[test]
/// `r1`'s `MOVE_HELPERS` list, gated against the engine's OWN set (`/review` finding 4).
///
/// `r1` decides whether a `Redirect` arm owes a discharge by asking whether it calls one
/// of three move helpers, and that list was hand-written — a fourth helper would make
/// every arm using it invisible to `r1`. `r1`'s own comment claimed "`r1c` gates that
/// enumeration against the engine's own set"; **`r1c` did not exist**, which is a comment
/// asserting an enforcement that was not there, inside the file about gates that claim
/// more than they check. It exists now.
///
/// The engine's set is derived from `GameState`'s own declarations: every inherent method
/// whose name ends in `move_object_to_zone`.
fn r1c_the_move_helper_list_matches_the_engine() {
    // Walked, not listed: the helpers are split across `state/mod.rs` (the primitive) and
    // `state/diagnostics.rs` (SR-4's `expect_*` / `fizzle_*` wrappers), and the first draft
    // of THIS gate read only `mod.rs` and found one of three — its own non-vacuity floor
    // caught that, which is what the floor is for.
    let mut declared: BTreeSet<String> = BTreeSet::new();
    let mut src = String::new();
    for f in engine_src_files() {
        src.push_str(&std::fs::read_to_string(&f).expect("readable"));
        src.push('\n');
    }
    for line in src.lines() {
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        if let Some(rest) = t.split("fn ").nth(1) {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.ends_with("move_object_to_zone") {
                declared.insert(name);
            }
        }
    }
    let expected: BTreeSet<String> = [
        "expect_move_object_to_zone",
        "fizzle_move_object_to_zone",
        "move_object_to_zone",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    // `state/test_util.rs`'s escape hatch is a free function that DELEGATES to the
    // primitive; it is not a fourth way to move an object, so it is excluded by name with
    // the reason rather than by the walk happening not to see it.
    declared.remove("test_only_move_object_to_zone");
    assert!(
        declared.len() >= 3,
        "r1c found only {declared:?} `*move_object_to_zone` helpers; the scanner is broken"
    );
    assert_eq!(
        declared, expected,
        "`GameState`'s set of `*move_object_to_zone` helpers moved. `r1`'s `MOVE_HELPERS` \
         is what decides whether a `ZoneChangeAction::Redirect` arm OWES a CR 701.24 \
         discharge, so a helper it does not know about makes every arm using that helper \
         invisible to `r1`. Update `MOVE_HELPERS` and this expectation together."
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
            // EVERY source `casting::card_def_target_requirements` can draw the host's
            // requirements from, not just `Spell` — the first draft counted `Spell` alone
            // and the `/review` pointed out that `Aftermath.targets` and `Fuse.targets`
            // feed the same list, and that a modal host's `ModeSelection.mode_targets`
            // REPLACES it. All three are the axis this precondition actually rests on.
            let host_targets: usize = d
                .abilities
                .iter()
                .map(|a| match a {
                    AbilityDefinition::Spell { targets, modes, .. } => {
                        let modal: usize = modes
                            .as_ref()
                            .and_then(|m| m.mode_targets.as_ref())
                            .map(|mt| mt.iter().map(|v| v.len()).sum())
                            .unwrap_or(0);
                        targets.len() + modal
                    }
                    AbilityDefinition::Aftermath { targets, .. } => targets.len(),
                    AbilityDefinition::Fuse { targets, .. } => targets.len(),
                    _ => 0,
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
    let window = &src[anchor..(anchor + 900).min(src.len())];
    let assign = window.find("p.miracle_pending =").unwrap_or_else(|| {
        panic!(
            "CR 702.94a: the draw site must record the just-drawn object; nothing assigns \
             `miracle_pending` within 900 bytes of `check_miracle_eligible`"
        )
    });
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
         `{stmt}`."
    );

    // THE MECHANISM, and the reason the check above is not enough. The `/review` defeated
    // this gate's first draft by leaving the `.map(..)` statement exactly as it is and
    // wrapping the WHOLE thing in `if miracle_event.is_some() { .. }` — the gate stayed
    // GREEN and all five behavioural probes stayed green, while a stale `miracle_pending`
    // survived a later non-eligible draw. The gate's own failure message says it catches
    // "an `if let Some(..)`"; it caught only the `Some(..)` SPELLING.
    //
    // So: walk from the end of the `check_miracle_eligible` statement to the assignment
    // and require that the ONLY block opened on the way is the player lookup. Any `if`,
    // `match`, `while` or `for` between them is a conditional the assignment now sits
    // under, whatever it is spelled.
    let call_end = window[..assign]
        .rfind(");")
        .map(|o| o + 2)
        .expect("the check_miracle_eligible call must end before the assignment");
    let between = &window[call_end..assign];
    let expected_opener = "if let Some(p) = state.expect_player_mut(player) {";
    let residue = between.replace(expected_opener, " ");
    for kw in ["if ", "match ", "while ", "for "] {
        assert!(
            !residue.contains(kw),
            "CR 702.94a (`OOS-DX2-1`): the `miracle_pending` assignment is nested inside a \
             `{kw}` between `check_miracle_eligible` and the write. It must be \
             UNCONDITIONAL — the whole point is that a draw which is NOT miracle-eligible \
             writes `None` and clears any stale record. Text between the call and the \
             assignment, with the expected player lookup removed: {residue:?}"
        );
    }
    // NON-VACUITY: the expected opener really is there, so the `replace` above removed
    // something and the loop is not scanning a string that never had a keyword in it.
    assert!(
        between.contains(expected_opener),
        "r3's residue check is vacuous: the player lookup it strips is not present. Text \
         between the call and the assignment: {between:?}"
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
            // `pub mod x;` counts too — the first draft stripped only `mod `, so a
            // `pub mod` naming an empty file was invisible (`/review`).
            let rest = l
                .strip_prefix("pub mod ")
                .or_else(|| l.strip_prefix("mod "));
            let Some(rest) = rest else {
                continue;
            };
            let Some(name) = rest.strip_suffix(';') else {
                continue;
            };
            // A module is either `name.rs` or `name/mod.rs`; the first draft looked only
            // for the first and skipped a directory module silently (`/review`).
            let flat = dir.join(format!("{name}.rs"));
            let nested = dir.join(name).join("mod.rs");
            let f = if flat.exists() {
                flat
            } else if nested.exists() {
                nested
            } else {
                continue;
            };
            scanned += 1;
            let s = std::fs::read_to_string(&f).expect("readable");
            // THE MECHANISM: a real `#[test]` / `#[tokio::test]` ATTRIBUTE, which begins a
            // line. The first draft asked whether the file CONTAINS the substring
            // `#[test]`, and the `/review` defeated it with a one-line doc comment
            // mentioning `#[test]` in prose — an empty module that reads as covered, which
            // is the exact thing `OOS-DX18-2` is about, inside the gate that files it.
            let has_test = s.lines().any(|line| {
                let t = line.trim_start();
                !t.starts_with("//")
                    && (t.starts_with("#[test]")
                        || t.starts_with("#[tokio::test")
                        || t.starts_with("#[rstest")
                        || t.starts_with("#[test("))
            });
            if has_test {
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
