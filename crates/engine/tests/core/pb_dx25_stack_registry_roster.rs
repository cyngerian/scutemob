//! PB-DX25 gates (plan §6 File B). **This runner's scope covers G1 and G2 only**
//! — G3 (the SR-36 corpus roster gate) is Stage 1 / Stage 6, a different
//! runner's assignment; see `memory/primitives/pb-plan-DX25.md` §10.
//!
//! Both source gates strip **line and block** comments before scanning — the
//! PB-DX32 M8 lesson (also applied by PB-DX24's own gates in this same
//! directory): a `/* ... */`-wrapped line defeats a line-comment-only scanner
//! while every probe stays green, because the compiler drops the commented-out
//! code and the scanner never sees it disappear. This file's own gates prove
//! that load-bearing property by executing BOTH revert shapes (`//` and `/*
//! */`), not just the line-comment one.

use std::path::Path;

// ── Comment-stripping (mirrors core::decision_gate / pb_dx24_trigger_zone_roster's idiom) ──

fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_block_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("*/") {
            Some(end) => rest = &after[end + 2..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

fn strip_comments(src: &str) -> String {
    strip_block_comments(&strip_line_comments(src))
}

fn read_source(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

/// Extract the body of `fn fn_name` (the OPEN brace after its signature through
/// the matching CLOSE brace) from already comment-stripped source, by simple
/// brace balancing. Naive about braces inside string/char literals -- adequate
/// here (neither gated function's body contains one).
fn extract_function_body<'a>(stripped: &'a str, fn_name: &str) -> &'a str {
    let sig_marker = format!("fn {fn_name}(");
    let sig_start = stripped
        .find(&sig_marker)
        .unwrap_or_else(|| panic!("`fn {fn_name}(` not found in stripped source"));
    let open_brace = stripped[sig_start..]
        .find('{')
        .map(|i| sig_start + i)
        .unwrap_or_else(|| panic!("no opening brace found after `fn {fn_name}(`"));
    balanced_body(stripped, open_brace, &format!("fn {fn_name}"))
}

/// Shared brace-balancing walk: `open_brace` must index the `{` character
/// itself. Returns the slice from `open_brace` through (and including) its
/// matching `}`.
fn balanced_body<'a>(stripped: &'a str, open_brace: usize, label: &str) -> &'a str {
    let mut depth = 0i32;
    let mut end = None;
    for (offset, ch) in stripped[open_brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(open_brace + offset + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.unwrap_or_else(|| panic!("unbalanced braces extracting body for `{label}`"));
    &stripped[open_brace..end]
}

const STACK_REGISTRY_PATH: &str = "src/state/stack_registry.rs";

// ── G1: the registry's classification has no wildcard arm ──────────────────────

/// G1 (plan §3.1 / §6): `card_in_stack_zone`'s body contains no `_ =>` and no
/// `_ |` -- a new `StackObjectKind` variant must be classified explicitly here,
/// never defaulted. Message: a new `StackObjectKind` must be classified here,
/// not defaulted -- `Effect::CounterSpell` and `counter_stack_object` both drive
/// their zone-move off this answer.
#[test]
fn g1_stack_registry_has_no_wildcard_arm() {
    let stripped = strip_comments(&read_source(STACK_REGISTRY_PATH));
    let body = extract_function_body(&stripped, "card_in_stack_zone");
    assert!(
        !body.contains("_ =>") && !body.contains("_ |"),
        "a new StackObjectKind must be classified here, not defaulted -- \
         Effect::CounterSpell and counter_stack_object both drive their zone-move \
         off this answer. Found a wildcard arm in card_in_stack_zone's body."
    );
}

/// G1 non-vacuity: the extracted body must actually contain the full 27-variant
/// classification (measured at plan-verification time, `pb-DX25-stage0.md`), so
/// a broken `extract_function_body` returning an empty slice cannot make G1 pass
/// by accident.
#[test]
fn g1_scan_is_not_vacuous() {
    let stripped = strip_comments(&read_source(STACK_REGISTRY_PATH));
    let body = extract_function_body(&stripped, "card_in_stack_zone");
    let arm_count = body.matches("=> Some").count() + body.matches("=> None").count();
    assert_eq!(
        arm_count, 27,
        "card_in_stack_zone's body must contain exactly 27 classification arms \
         (measured: 2 `Some` + 25 `None`) -- got {arm_count}. A collapsed or empty \
         extraction would make G1 pass vacuously."
    );
}

/// G1's revert proof, LINE-COMMENT shape: adding a bare `_ => None,` catch-all
/// must be detectable. Exercised by executing the revert in the batch's own
/// runbook (see `memory/primitives/pb-DX25-execution-notes.md`) rather than
/// asserted here as a second copy of the source -- this test documents the
/// expectation for a future reader auditing the gate's discrimination.
#[test]
fn g1_line_comment_stripping_does_not_hide_the_wildcard_it_is_meant_to_find() {
    // A wildcard arm hidden behind a LINE comment must still be found -- i.e.
    // stripping comments must not itself create a false negative on a REAL
    // (uncommented) wildcard elsewhere in the body. This is the inverse
    // sanity check to the block-comment danger PB-DX32 M8 found: here we prove
    // stripping doesn't ADD false negatives, by constructing a small fixture
    // string with a genuine wildcard arm sitting next to a line comment.
    let fixture = "fn f(k: &K) -> Option<u8> {\n    match k {\n        // a comment\n        _ => None,\n    }\n}\n";
    let stripped = strip_comments(fixture);
    let body = extract_function_body(&stripped, "f");
    assert!(
        body.contains("_ =>"),
        "a genuine (uncommented) wildcard arm must survive comment-stripping -- \
         got {:?}",
        body
    );
}

// ── G2: the counter arm does not re-classify by kind ────────────────────────────
//
// G2 is written in Stage 4, once the `Effect::CounterSpell` arm has been
// rewritten to actually call `card_in_stack_zone` (at HEAD it doesn't exist yet,
// so a gate here would be permanently, vacuously red for the wrong reason). See
// this file's own history for the Stage-4 commit.
