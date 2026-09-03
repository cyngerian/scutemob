//! CR 605.4a — **exactly one site in the workspace closes the effect-choice gate.**
//!
//! `EffectContext::effect_choice_gate_closed` makes the asking `Effect` arms take their
//! deterministic default instead of suspending onto PB-DP9's CR 608.2d channel. That is
//! correct in exactly one place — `rules/mana.rs`'s `WhenTappedForMana` branch, where
//! CR 605.1b/605.4a put the resolution OUTSIDE the stack so there is nothing to roll back
//! to — and it is a silent wrong answer anywhere else, because it skips an obligation
//! (offering the choice) without leaving an artefact.
//!
//! # Why this lives in `core` and not beside the test whose claim it backs
//!
//! `primitives::pb_dp9_effect_choice::test_dp9_mana_ability_gate` part (c) is the claim;
//! this file is the gate on it. They are in different test BINARIES, and that is forced
//! rather than chosen:
//!
//! * the census must walk the whole workspace, because PB-DX50's `/review` defeated the
//!   previous version by planting the defect in `rules/abilities.rs` — a file the old
//!   three-file list did not contain (PB-DX48's `SITE_SRCS` defeat and PB-DX49's `r6`
//!   defeat, for the third time in three batches);
//! * the only workspace-wide source walk in this tree, with executing non-vacuity floors,
//!   is [`workspace_src_files_checked`] in `core`;
//! * Cargo integration-test binaries are separate crates, so `primitives` cannot `use` it;
//! * a second copy of the walk is what `pb_dx50_mutate_site_roster`'s module doc rejects by
//!   name (*"the one that drifts is the one nobody re-measures"*), and a `#[path]` include
//!   is what **SR-9a's `no_stray_test_binaries` gate refuses outright** — correctly, since
//!   an attribute on a `mod` line is a way to look declared while not being compiled. That
//!   gate fired on the first draft of this fix and it was right.
//!
//! SR-9a's own layout table calls `core` the home of "the machine-checked invariant
//! gates", which is what this is.
//!
//! `g3` links the two files in BOTH directions, so neither can be deleted while the other
//! goes on claiming to be covered.

use crate::pb_dx49_saga_blanking_roster::{strip_comments, workspace_src_files_checked};

/// The two ways to write a closing, both counted.
///
/// **The second one is the `/review`'s finding.** The superseded gate keyed on the
/// ASSIGNMENT form alone (`= true`), while every `EffectContext` in this tree is built as
/// a struct LITERAL (`effect_choice_gate_closed: false`) — five such sites at HEAD. A gate
/// keyed on one of two syntactic forms measures one of two syntactic forms, which is
/// PB-DX47's `r3` finding verbatim.
const CLOSING_SPELLINGS: [&str; 2] = [
    "effect_choice_gate_closed = true",
    "effect_choice_gate_closed: true",
];

fn closings(src: &str) -> usize {
    CLOSING_SPELLINGS
        .iter()
        .map(|n| src.matches(n).count())
        .sum()
}

/// **g1** — the workspace census: one closing site, and it is `rules/mana.rs`.
///
/// **Revert to watch red**: set `rules/abilities.rs`'s
/// `effect_choice_gate_closed: false` to `true` (this is the `/review`'s own defeat, and
/// it left the superseded gate GREEN), or add `ctx.effect_choice_gate_closed = true;`
/// anywhere.
#[test]
fn g1_exactly_one_workspace_site_closes_the_cr_605_4a_gate() {
    let mut sites: Vec<String> = Vec::new();
    let mut total = 0usize;
    for (label, path) in workspace_src_files_checked() {
        // Comments are stripped, and it is load-bearing rather than defensive: the field
        // name appears in doc comments in three files, and `rules/engine.rs` now quotes
        // the spellings in prose. An unstripped scan would fail on the CORRECT tree.
        let src = strip_comments(&std::fs::read_to_string(&path).expect("read source"));
        let n = closings(&src);
        if n > 0 {
            sites.push(label);
            total += n;
        }
    }
    println!("\n=== PB-DX50 g1: CR 605.4a gate-closing sites ===");
    for s in &sites {
        println!("  {s}");
    }
    println!("  total closings {total}\n");

    assert_eq!(
        (sites.as_slice(), total),
        (
            ["crates/engine/src/rules/mana.rs".to_string()].as_slice(),
            1
        ),
        "CR 605.4a: exactly ONE site in the workspace may close the effect-choice gate, \
         and it must be `rules/mana.rs`'s `WhenTappedForMana` branch. A second site \
         invalidates TWO standing arguments and both need re-deriving rather than \
         trusting: `pb_dp9_effect_choice::test_dp9_mana_ability_gate` part (a)'s \
         corpus roster (which scans `WhenTappedForMana` triggers ALONE, because that is \
         the only route into the closed branch), and PB-DX50's structural-unreachability \
         discharge for CR 702.140c's mutate ask. Sites: {sites:?}, total {total}"
    );
}

/// **g2** — the resolution-side asker takes a LITERAL `false`, so no caller can smuggle a
/// closed gate into a stack resolution, and the mutate arm is a real caller of it.
///
/// Without the second half `g2` is about a dead helper.
#[test]
fn g2_the_resolution_asker_hardcodes_an_open_gate() {
    let effects_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/effects/mod.rs"),
    )
    .expect("src/effects/mod.rs is readable");
    let resolution_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules/resolution.rs"),
    )
    .expect("src/rules/resolution.rs is readable");

    let asker = effects_src
        .find("pub(crate) fn ask_resolution_choice")
        .expect("PB-DX50's resolution-side asker must exist");
    let body_end = effects_src[asker..]
        .find("\n}\n")
        .map(|i| asker + i)
        .expect("the asker has a body");
    let body = &effects_src[asker..body_end];
    assert!(
        body.contains("ask_or_consume_effect_choice_core(state, false, source, player, question)"),
        "`ask_resolution_choice` must pass a LITERAL `false` for the CR 605.4a gate -- a \
         stack resolution is never a mana ability, and taking the value from a parameter \
         would make that a caller's promise instead of a fact. Body: {body}"
    );

    assert!(
        resolution_src.contains("crate::effects::ask_resolution_choice("),
        "the MutatingCreatureSpell arm must be a real caller of `ask_resolution_choice`, \
         or `g2` is about a dead helper"
    );
}

/// **g3** — the two halves of this argument still point at each other.
///
/// The claim (structural unreachability of the CR 702.140c ask from the gate-closing site)
/// is stated in `primitives::pb_dp9_effect_choice::test_dp9_mana_ability_gate`; the gate on
/// it is `g1`/`g2` here. Split across two test binaries for the reason in this file's
/// module doc, which means either half can be deleted without the other noticing —
/// **exactly the failure mode this queue keeps filing, so it is machine-checked rather
/// than left to a comment.**
#[test]
fn g3_the_claim_and_its_gate_still_reference_each_other() {
    let dp9 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/primitives/pb_dp9_effect_choice.rs"),
    )
    .expect("pb_dp9_effect_choice.rs is readable");
    assert!(
        dp9.contains("fn test_dp9_mana_ability_gate()"),
        "the claim this file gates lives in \
         `primitives::pb_dp9_effect_choice::test_dp9_mana_ability_gate`; it is gone. \
         Either restore it or delete this file -- what must not happen is this gate \
         standing while nothing states what it is for."
    );
    assert!(
        dp9.contains("core::pb_dx50_effect_choice_gate_sites"),
        "`test_dp9_mana_ability_gate` must keep its pointer to this file. Without it a \
         reader checking part (c)'s structural-unreachability argument finds no gate and \
         reasonably concludes there is none -- which is how the superseded three-file \
         census survived being wrong."
    );

    // ...and the pointer must be a live path, not a stale one.
    assert!(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/core/pb_dx50_effect_choice_gate_sites.rs")
            .is_file(),
        "this file's own path, as `pb_dp9_effect_choice.rs` spells it, must resolve"
    );
}

/// **g4** — `g1`'s needles discriminate, on synthetic input.
///
/// Both spellings must be seen (the whole finding was that one of them was invisible) and
/// the PROPAGATING form must not be, since forwarding a caller's value is not a closing —
/// `effects/mod.rs` does it twice and a needle that counted those would pin `g1` at 3 and
/// make the real census unreadable.
#[test]
fn g4_the_closing_needles_discriminate() {
    assert_eq!(closings("ctx.effect_choice_gate_closed = true;"), 1);
    assert_eq!(
        closings("EffectContext { effect_choice_gate_closed: true, ..Default::default() }"),
        1
    );
    assert_eq!(closings("effect_choice_gate_closed: false,"), 0);
    assert_eq!(
        closings("effect_choice_gate_closed: ctx.effect_choice_gate_closed,"),
        0,
        "forwarding a caller's gate value is not a closing site"
    );
    assert_eq!(
        closings("pub effect_choice_gate_closed: bool,"),
        0,
        "the field DECLARATION is not a closing site"
    );
    // Comment stripping, both shapes -- this file's own prose names the spellings.
    for planted in [
        "// ctx.effect_choice_gate_closed = true;",
        "/* effect_choice_gate_closed: true, */",
    ] {
        assert_eq!(
            closings(&strip_comments(planted)),
            0,
            "a commented-out closing must not be counted: {planted:?}"
        );
    }
    assert_eq!(
        closings(&strip_comments("ctx.effect_choice_gate_closed = true;")),
        1,
        "stripping must not hide REAL code, or `g1` is vacuous"
    );
}
