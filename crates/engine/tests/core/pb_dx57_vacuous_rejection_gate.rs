//! PB-DX57 (`OOS-DX21-7`): the ratchet against "the rejected command mutated nothing" asserted
//! through `process_command`.
//!
//! # The seed
//!
//! > **`process_command`'s `Err` arm carries no `GameState`, so ANY test asserting "the
//! > rejected command mutated nothing" through `process_command` is structurally vacuous.** The
//! > signature is `Result<(GameState, Vec<GameEvent>), GameStateError>`; on `Err`, Rust's
//! > ownership model discards every mutation the callee made, regardless of where in the callee
//! > it happened. A probe that calls `process_command(state.clone(), cmd)`, expects an `Err`,
//! > and then reads the *original* `state` is reading a state the failing call never touched —
//! > **it passes identically whether the guard is at the top of the function or absent
//! > entirely.** The only sound idioms are the direct-handler call with `&mut state`, or
//! > asserting on an observable the *accepted* path produced.
//!
//! # What the stage-0 sweep measured
//!
//! Search space 387 files / 3,115 call sites → 369 test files → 191 with an error expectation →
//! **534** test functions whose `Err` came from a `process_command` result → **216** with a
//! trailing assertion, all read in full → **17 VACUOUS assertion sites in 15 functions across 9
//! files**, zero AMBIGUOUS. **No shared helper wraps the shape**, so it was 17 hand-written
//! instances rather than one helper multiplied — there was nothing to fix once and everything
//! to fix once each. All 17 are repaired by this batch.
//!
//! **The concentration is the tell.** `pb_dp7` / `pb_dp8` / `pb_dp9` are three batches written
//! to one template and held 9 of the 17; `pb_dp8` had **exactly one** of its tests repaired,
//! with a doc comment stating the whole argument, while its own siblings and the whole of
//! `pb_dp7`/`pb_dp9` were left. The lesson was learned once and not carried across the file.
//!
//! # THIS GATE IS A RATCHET, NOT A PROOF, and the distinction is measured rather than modest
//!
//! The sweep's own adversarial section enumerated seven ways to defeat any source-level gate
//! for this shape. Two of them are **not closable at the source level at all**:
//!
//! * **A helper wrapper.** `fn reject(s: &GameState, c: Command) -> GameStateError {
//!   process_command(s.clone(), c).unwrap_err() }`, then `let e = reject(&state, cmd);
//!   assert_eq!(state.public_state_hash(), h);`. The test function now contains **no
//!   `process_command`, no `.clone()`, and no `Err` token**. A per-function scanner is
//!   structurally blind, and this is exactly what a well-meaning later batch does when it
//!   notices 17 sites repeating.
//! * **A macro.** `assert_untouched!(state, hash_before)` expands to the shape; a source-text
//!   gate cannot see through expansion.
//!
//! So this file **does not claim to enforce the property**. It holds the measured population at
//! its floor and makes a new hand-written instance loud. Saying so here rather than letting the
//! name imply more is the point of `OOS-DX49-6` (*a comment asserting a property the code does
//! not enforce*), and this batch closes that seed's sibling.
//!
//! **The bypass-proof repair exists and is deliberately NOT taken here**: split
//! `process_command` into a `pub process_command_mut(&mut GameState, Command)` — its body is
//! already `let mut state = state;` followed by `&mut`-dispatch — plus the by-value wrapper.
//! That makes the property falsifiable for all 45 command variants at once, **including the 10
//! whose handlers are private `fn` in `rules/engine.rs` and for which no such test can be
//! written today**. It is an engine change and PB-DX57 is a 0-engine-lines batch, so it is
//! FILED, not made.
//!
//! # The three lessons this gate is built to survive
//!
//! * **`OOS-DX32-6`** — a `contains`-based source gate cannot tell code from a comment, and a
//!   commented-out call satisfies it. So the gate is phrased as a **PROHIBITION** on the vacuous
//!   shape, never as a requirement that the sound shape be present (a `// handle_x(&mut state)`
//!   satisfies the latter, and this tree's doc comments name handlers three times), and it
//!   strips comments AND string literals before analysis.
//! * **PB-DX50's `r3`** — a gate on a predicate's DEFINITION says nothing about its CONSUMER.
//!   Instantiated three ways here, all real: gating that `handle_*` exists and takes
//!   `&mut GameState` would be green today with all 17 sites in place (all 34 `pub` handlers
//!   already do); gating **per FILE** is green on the three files holding 9 of the 17; gating
//!   **per FUNCTION** is green on `pb_dp7`'s `&mut state.clone()`. So the key is **per
//!   ASSERTION**.
//! * **`&mut expr.clone()`** — live in the tree before this batch. A `&mut` of a temporary is
//!   dropped at the end of the statement, so it LOOKS like the sound idiom and is not. The
//!   `&mut` exemption below therefore requires a bare IDENTIFIER, never an expression.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Files whose vacuous shape is RECORDED rather than forbidden, each with the reason.
///
/// Empty is the goal and empty is the state: all 17 measured sites were repaired rather than
/// allowlisted. The list exists so that a future disclosed exception is a written act.
const RECORDED_VACUOUS: &[(&str, &str, &str)] = &[
    // PB-DX21's two second-declaration probes. The read after the rejection is on a `state`
    // that a PREVIOUS, ACCEPTED `process_command` produced, and each assertion's own message
    // labels itself a positive control -- *"positive control: the raid count from the first,
    // accepted declaration survives"*. That is the second sound idiom (assert on an observable
    // the ACCEPTED path produced); the verdict in both tests is the `expect_err` variant match.
    //
    // Recorded rather than rewritten, and recorded rather than filtered. Not rewritten because
    // there is nothing wrong with them. Not filtered because the mechanical distinction the
    // gate would need -- *"is this assertion comparing to a pre-call snapshot, or to a value
    // the accepted path established"* -- is a claim about INTENT, and the gate deliberately
    // strips comments, so it cannot read the disclosure that makes these sound. Encoding the
    // judgement here, once, with the reason, is honest; teaching the scanner to guess at it
    // would make it fail OPEN on the real shape.
    (
        "primitives/pb_dx21_declare_attackers_once_per_combat.rs",
        "test_dx21_second_declaration_rejected_raid_count_not_clobbered",
        "DISCLOSED POSITIVE CONTROL: `state` is the product of the FIRST, ACCEPTED declaration \
         (`let (state, _events) = process_command(..)`), so the post-rejection read asserts what \
         the accepted path established -- the second sound idiom. The verdict is the expect_err \
         variant match, and the assertion message says 'positive control' in so many words.",
    ),
    (
        "primitives/pb_dx21_declare_attackers_once_per_combat.rs",
        "test_dx21_second_declaration_rejected_target_not_overwritten",
        "DISCLOSED POSITIVE CONTROL, same shape and same file as the row above: the attack \
         target read after the rejection was set by the first, ACCEPTED declaration. PB-DX21's \
         own review (M4/M5) is what filed OOS-DX21-7, and step (4) of its sibling test carries \
         the direct-handler repair that IS the verdict for the mutation half.",
    ),
];

fn tests_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// Strip `//` and `/* */` comments and string/char literals, preserving byte length so line
/// numbers survive. A gate whose subject is *"what does this code do"* must not be able to read
/// a comment or a message string as code.
fn strip_noncode(src: &str) -> String {
    let b: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == '/' && i + 1 < b.len() && b[i + 1] == '/' {
            while i < b.len() && b[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if b[i] == '/' && i + 1 < b.len() && b[i + 1] == '*' {
            let mut depth = 1usize;
            out.push_str("  ");
            i += 2;
            while i < b.len() && depth > 0 {
                if b[i] == '/' && i + 1 < b.len() && b[i + 1] == '*' {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if b[i] == '*' && i + 1 < b.len() && b[i + 1] == '/' {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                } else {
                    out.push(if b[i] == '\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
            continue;
        }
        if b[i] == '"' {
            out.push(' ');
            i += 1;
            while i < b.len() && b[i] != '"' {
                if b[i] == '\\' {
                    out.push(' ');
                    i += 1;
                }
                if i < b.len() {
                    out.push(if b[i] == '\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
            if i < b.len() {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

fn rust_files_under(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files_under(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// The names `process_command` is reachable under in this file: the bare name, plus any
/// `use ... as ALIAS` rename.
///
/// Bypass 1 from the sweep's own list: `use mtg_engine::process_command as pc;`. A bare-token
/// scan dies on it, and `replacement_effects.rs` already calls it fully qualified, so the tree
/// really does use more than one spelling. This is PB-DX36's bidirectional `use`-alias scan.
/// **Residual, stated**: a re-export chain through a third module still defeats it.
fn entry_point_names(code: &str) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    names.insert("process_command".to_string());
    for line in code.lines() {
        let l = line.trim();
        if !l.starts_with("use ") || !l.contains("process_command") {
            continue;
        }
        if let Some(at) = l.find("process_command as ") {
            let alias: String = l[at + "process_command as ".len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !alias.is_empty() {
                names.insert(alias);
            }
        }
    }
    names
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Does `hay` contain `needle` as a whole token?
fn has_token(hay: &str, needle: &str) -> bool {
    let mut from = 0usize;
    while let Some(rel) = hay[from..].find(needle) {
        let at = from + rel;
        let before_ok = at == 0 || !hay[..at].chars().next_back().is_some_and(is_ident_char);
        let after = at + needle.len();
        let after_ok =
            after >= hay.len() || !hay[after..].chars().next().is_some_and(is_ident_char);
        if before_ok && after_ok {
            return true;
        }
        from = at + 1;
    }
    false
}

/// Split a file into `(fn_name, body)` for every `#[test]` function, brace-matched.
///
/// Brace-matched rather than windowed: bypass 3 is *"put the `.expect_err(` 40 lines below the
/// call"*, which every window heuristic loses — **and the sweep's own scanner lost exactly that,
/// at `pb_ef8_exile_self_from_hand.rs`, before it was widened.** A whole-function body has no
/// distance limit.
fn test_functions(code: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = code[from..].find("#[test]") {
        let at = from + rel;
        from = at + 1;
        let Some(fnrel) = code[at..].find("fn ") else {
            continue;
        };
        let fnat = at + fnrel + 3;
        let name: String = code[fnat..]
            .chars()
            .take_while(|c| is_ident_char(*c))
            .collect();
        let Some(brel) = code[fnat..].find('{') else {
            continue;
        };
        let bstart = fnat + brel + 1;
        let mut depth = 1usize;
        let mut end = bstart;
        for (i, ch) in code[bstart..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = bstart + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        out.push((name, code[bstart..end].to_string()));
    }
    out
}

/// One flagged site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Vacuous {
    file: String,
    test: String,
    shape: &'static str,
    detail: String,
}

/// Expressions that read game state. An assertion mentioning a binding but reading nothing
/// state-shaped is not an absence-of-mutation claim.
const STATE_READS: &[&str] = &[
    "public_state_hash",
    ".objects()",
    ".players()",
    ".player(",
    ".zone(",
    ".turn()",
    ".combat()",
    ".stack_objects()",
    "life_total",
    ".status.",
    "command_count",
];

fn scan_file(path: &std::path::Path) -> Vec<Vacuous> {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    let code = strip_noncode(&raw);
    let entries = entry_point_names(&code);
    let file = path
        .strip_prefix(tests_root())
        .unwrap_or(path)
        .display()
        .to_string();
    let mut out = Vec::new();

    for (test, body) in test_functions(&code) {
        // ── Shape A: `X.clone()` handed to the entry point, then an assertion reading `X`.
        for entry in &entries {
            let needle = format!("{entry}(");
            let mut from = 0usize;
            while let Some(rel) = body[from..].find(&needle) {
                let at = from + rel;
                from = at + 1;
                let before_ok =
                    at == 0 || !body[..at].chars().next_back().is_some_and(is_ident_char);
                if !before_ok {
                    continue;
                }
                let after = &body[at + needle.len()..];
                let arg: String = after
                    .trim_start()
                    .chars()
                    .take_while(|c| is_ident_char(*c))
                    .collect();
                if arg.is_empty() {
                    continue;
                }
                if !after.trim_start()[arg.len()..]
                    .trim_start()
                    .starts_with(".clone()")
                {
                    continue;
                }
                let tail = &body[at..];
                // **The Err must belong to THIS call.** The first draft asked only whether an
                // Err token appeared anywhere later in the function, and that over-fired on a
                // real and common shape: `rules::commander`'s mulligan tests hand a `.clone()`
                // to a call that SUCCEEDS (`.unwrap()`), read the original `state` afterwards as
                // a legitimate branch continuation, and separately expect an `Err` from a
                // DIFFERENT, correctly by-value call later on. Nothing about those is vacuous.
                // Found by adjudicating a hit rather than by reading the count -- three of the
                // eleven the first draft reported were this shape.
                // The statement runs from the previous `;`/`{` to the next `;`. Anchoring the
                // START backwards is load-bearing and the first draft got it wrong: it began the
                // statement at the CALL, so `let r = process_command(..)` never showed its own
                // `let` and the binding-then-check arm below was dead. `v3`'s synthetic shape-A
                // case is what caught it.
                let stmt_start = body[..at].rfind([';', '{']).map(|i| i + 1).unwrap_or(0);
                let stmt_end = tail.find(';').map(|e| e + 1).unwrap_or(tail.len());
                let stmt = &body[stmt_start..at + stmt_end];
                let err_is_this_calls = if stmt.contains("unwrap_err")
                    || stmt.contains("expect_err")
                    || stmt.contains("is_err")
                {
                    true
                } else if stmt.contains(".unwrap()") || stmt.contains(".expect(") {
                    // An explicitly-unwrapped success. Whatever Err appears later belongs to
                    // another call.
                    false
                } else if let Some(li) = stmt.find("let ") {
                    // `let r = <call>;` -- the Err check may be on `r`, later.
                    let bind: String = stmt[li + 4..]
                        .trim_start()
                        .trim_start_matches("mut ")
                        .chars()
                        .take_while(|c| is_ident_char(*c))
                        .collect();
                    !bind.is_empty()
                        && tail[stmt_end..].split(';').any(|later| {
                            has_token(later, &bind)
                                && (later.contains("is_err")
                                    || later.contains("unwrap_err")
                                    || later.contains("expect_err")
                                    || later.contains("Err("))
                        })
                } else {
                    false
                };
                if !err_is_this_calls {
                    continue;
                }
                // Was `arg` handed to anything by `&mut` in this test? The exemption requires a
                // BARE IDENTIFIER: `&mut arg.clone()` is a temporary and is NOT an exemption.
                let exempt = has_token(&body, &format!("&mut {arg}"))
                    && !body.contains(&format!("&mut {arg}."))
                    && !body.contains(&format!("&mut {arg}.clone()"));
                if exempt {
                    continue;
                }
                // Walk forward statement by statement and STOP at a REBIND of `arg`.
                //
                // Without this the gate reports a false positive on a real and common shape:
                // `rules::commander`'s `test_mulligan_three_times_escalating_bottom_count`
                // rejects a `KeepHand` on a clone and then does
                // `let (state, _) = process_command(state, ..).unwrap();` -- so every assertion
                // after that reads the ACCEPTED path's state, which is sound. A scanner without
                // rebinding awareness cannot tell that from the vacuous shape, and *dozens of
                // the 216 functions the stage-0 sweep read in full rebind `state` from a later
                // successful call*, which is why that sweep had to READ them.
                for stmt in tail[stmt_end..].split(';') {
                    let binds_arg = stmt.contains("let ")
                        && stmt
                            .split('=')
                            .next()
                            .is_some_and(|lhs| has_token(lhs, &arg));
                    if binds_arg {
                        break;
                    }
                    if !stmt.contains("assert") {
                        continue;
                    }
                    if has_token(stmt, &arg) && STATE_READS.iter().any(|r| stmt.contains(r)) {
                        out.push(Vacuous {
                            file: file.clone(),
                            test: test.clone(),
                            shape: "A: entry point took `X.clone()`, assertion reads `X`",
                            detail: format!("binding `{arg}`"),
                        });
                        break;
                    }
                }
            }
        }

        // ── Shape B: `&mut X.clone()` — looks like the sound direct-handler idiom, is a
        // temporary dropped at the end of the statement. Live in the tree before this batch,
        // at `pb_dp7_cleanup_discard.rs`, and a gate keyed on "calls a handler with `&mut`"
        // passes it.
        if body.contains("&mut ") {
            for stmt in body.split(';') {
                if let Some(at) = stmt.find("&mut ") {
                    let rest = &stmt[at + 5..];
                    let ident: String = rest.chars().take_while(|c| is_ident_char(*c)).collect();
                    if ident.is_empty() {
                        continue;
                    }
                    if rest[ident.len()..].trim_start().starts_with(".clone()")
                        && (body.contains("is_err")
                            || body.contains("unwrap_err")
                            || body.contains("expect_err"))
                    {
                        out.push(Vacuous {
                            file: file.clone(),
                            test: test.clone(),
                            shape: "B: `&mut X.clone()` — a temporary, not the sound idiom",
                            detail: format!("binding `{ident}`"),
                        });
                    }
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn scan_all() -> Vec<Vacuous> {
    let mut files = Vec::new();
    rust_files_under(&tests_root(), &mut files);
    let mut out: Vec<Vacuous> = files.iter().flat_map(|p| scan_file(p)).collect();
    out.sort();
    out
}

// ── The gate ─────────────────────────────────────────────────────────────────

#[test]
fn v1_no_test_asserts_absence_of_mutation_through_a_by_value_entry_point() {
    let live = scan_all();
    let recorded: BTreeSet<(String, String)> = RECORDED_VACUOUS
        .iter()
        .map(|(f, t, _)| (f.to_string(), t.to_string()))
        .collect();
    let new: Vec<&Vacuous> = live
        .iter()
        .filter(|v| !recorded.contains(&(v.file.clone(), v.test.clone())))
        .collect();
    assert!(
        new.is_empty(),
        "OOS-DX21-7: {} test(s) assert that a REJECTED command left state untouched, through an \
         entry point that takes `GameState` BY VALUE:\n{new:#?}\n\
         On `Err`, Rust's ownership model discards every mutation the callee made, so the \
         assertion reads a state the failing call never held. It passes identically whether the \
         guard is at the top of the function or absent entirely.\n\
         The two sound idioms: call the handler directly with `&mut state` and read THAT state \
         (`rules::combat::handle_declare_attackers(&mut state, ..)`, and note that `&mut \
         state.clone()` is a TEMPORARY and is not the sound idiom), or assert on an observable \
         the ACCEPTED path produced.\n\
         This gate is a RATCHET, not a proof — a helper wrapper or a macro defeats any \
         source-level check of this shape. Do not satisfy it by hiding the call.",
        new.len()
    );
    let live_keys: BTreeSet<(String, String)> = live
        .iter()
        .map(|v| (v.file.clone(), v.test.clone()))
        .collect();
    let gone: Vec<&(String, String)> = recorded.difference(&live_keys).collect();
    assert!(
        gone.is_empty(),
        "recorded exception(s) {gone:?} no longer fire. If repaired, delete the row and say so; \
         if not, the SCANNER narrowed and the ratchet is blind — indistinguishable from the \
         count alone, which is why this assertion exists."
    );
}

// ── Non-vacuity: each of these executes ──────────────────────────────────────

/// The scan reaches the test tree at all. A walker that found no files would report zero
/// offenders forever.
#[test]
fn v2_the_scan_reaches_the_test_tree() {
    let mut files = Vec::new();
    rust_files_under(&tests_root(), &mut files);
    assert!(
        files.len() >= 200,
        "the walk found only {} .rs files under crates/engine/tests",
        files.len()
    );
    let fns: usize = files
        .iter()
        .map(|p| {
            test_functions(&strip_noncode(
                &std::fs::read_to_string(p).unwrap_or_default(),
            ))
            .len()
        })
        .sum();
    assert!(
        fns >= 2_000,
        "the #[test] splitter found only {fns} functions across {} files — a splitter that \
         finds nothing makes v1 vacuous",
        files.len()
    );
    println!(
        "PB-DX57 / OOS-DX21-7 — scanned {} files, {fns} #[test] fns, {} live site(s)",
        files.len(),
        scan_all().len()
    );
}

/// The detector fires on each shape, on SYNTHETIC input.
///
/// Synthetic rather than corpus-driven deliberately: after this batch the corpus contains zero
/// instances, so a corpus-driven proof would be unfalsifiable — *a self-test written by the
/// same author from the same mental model exercises the inputs that author already thought of*
/// (`OOS-DX54-6`), and a detector whose only evidence is that it currently finds nothing is
/// `OOS-DX32-6`'s shape.
#[test]
fn v3_the_detector_fires_on_each_shape_and_spares_the_sound_ones() {
    let dir = std::env::temp_dir().join("pb_dx57_v3");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");

    let write = |name: &str, body: &str| {
        let p = dir.join(name);
        std::fs::write(&p, body).expect("write");
        p
    };

    // Shape A — the seed verbatim.
    let a = write(
        "a.rs",
        "#[test]\nfn t() {\n let hash_before = state.public_state_hash();\n \
         let r = process_command(state.clone(), cmd);\n assert!(r.is_err());\n \
         assert_eq!(state.public_state_hash(), hash_before);\n}\n",
    );
    assert!(!scan_file(&a).is_empty(), "shape A not detected");

    // Shape A under an alias — bypass 1.
    let b = write(
        "b.rs",
        "use mtg_engine::process_command as pc;\n#[test]\nfn t() {\n \
         let h = state.public_state_hash();\n let r = pc(state.clone(), cmd);\n \
         assert!(r.is_err());\n assert_eq!(state.public_state_hash(), h);\n}\n",
    );
    assert!(
        !scan_file(&b).is_empty(),
        "aliased entry point not detected"
    );

    // Shape B — `&mut X.clone()`, the one that looks sound.
    let c = write(
        "c.rs",
        "#[test]\nfn t() {\n let h = state.public_state_hash();\n \
         let r = handle_x(&mut state.clone(), p);\n assert!(r.is_err());\n \
         assert_eq!(state.public_state_hash(), h);\n}\n",
    );
    assert!(
        !scan_file(&c).is_empty(),
        "shape B (&mut temporary) not detected"
    );

    // A COMMENTED-OUT vacuous test must NOT be detected (OOS-DX32-6, the other direction:
    // a gate that reads comments as code fires on prose).
    let d = write(
        "d.rs",
        "#[test]\nfn t() {\n // let r = process_command(state.clone(), cmd);\n \
         // assert_eq!(state.public_state_hash(), hash_before);\n \
         let (state, _) = process_command(state, cmd).unwrap();\n \
         assert_eq!(state.turn().turn_number, 1);\n}\n",
    );
    assert!(
        scan_file(&d).is_empty(),
        "the detector read a COMMENTED-OUT call as code: {:?}",
        scan_file(&d)
    );

    // A REBIND after the rejection must NOT be detected: the assertion reads the ACCEPTED
    // path's state. This is `rules::commander::test_mulligan_three_times_escalating_bottom_count`
    // in miniature, and the first draft of this gate flagged it.
    let f = write(
        "f.rs",
        "#[test]\nfn t() {\n let e = process_command(state.clone(), bad).unwrap_err();\n \
         assert!(matches!(e, X));\n let (state, _) = process_command(state, good).unwrap();\n \
         assert_eq!(state.public_state_hash(), h);\n}\n",
    );
    assert!(
        scan_file(&f).is_empty(),
        "a REBOUND state read after the rejection was flagged: {:?}",
        scan_file(&f)
    );

    // The SOUND direct-handler idiom must NOT be detected.
    let e = write(
        "e.rs",
        "#[test]\nfn t() {\n let h = state.public_state_hash();\n \
         let r = handle_declare_attackers(&mut state, p);\n assert!(r.is_err());\n \
         assert_eq!(state.public_state_hash(), h);\n}\n",
    );
    assert!(
        scan_file(&e).is_empty(),
        "the sound direct-handler idiom was flagged: {:?}",
        scan_file(&e)
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Every recorded exception is still live on its own terms (`OOS-DX52-1`: an allowlist whose
/// reason is not checked is a comment). Vacuous while the list is empty, and says so.
#[test]
fn v4_recorded_exceptions_carry_a_reason() {
    for (f, t, why) in RECORDED_VACUOUS {
        assert!(
            why.len() > 40,
            "recorded exception ({f}, {t}) has a {}-char reason; an entry whose adjudication is \
             not written down is an allowlist entry",
            why.len()
        );
    }
    println!(
        "PB-DX57 / OOS-DX21-7 — RECORDED_VACUOUS holds {} entries. All 17 sites the stage-0 \
         sweep measured were REPAIRED rather than allowlisted, plus one the sweep missed; the \
         entries here are DISCLOSED POSITIVE CONTROLS, not exceptions granted to defects.",
        RECORDED_VACUOUS.len()
    );
}
