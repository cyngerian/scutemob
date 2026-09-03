//! PB-DX50 half 3 — CR 707.10: which `AdditionalCost` variants a spell COPY inherits.
//!
//! `rules::copy::copy_spell_on_stack` filters `original.additional_costs` through a
//! hardcoded allowlist. Full per-variant audit, with the read site for each:
//! `memory/primitives/pb-DX50-additional-cost-copy-audit.md`.
//!
//! # What this file pins, and what it deliberately does not
//!
//! It pins the allowlist's **membership** and the `is_copy` guard's **existence**, both
//! as source gates. It does not pin behaviour, and the reason is a hard bound the audit
//! measured rather than assumed: **no card definition in this corpus can copy another
//! card's spell.** `Effect::CopySpellOnStack` has ZERO genuine declarations across 1,803
//! defs (its only two textual occurrences are a `Completeness::partial(..)` note in
//! `plumb_the_forbidden.rs` and a `//` comment in `complete_the_circuit.rs`), and the six
//! deck-legal `Complete` copy sources — `empty_the_warrens`, `radstorm`, `flusterstorm`
//! (Storm), `follow_the_bodies` (Gravestorm), `train_of_thought` (Replicate),
//! `make_disappear` (Casualty) — are all **self**-copying instants or sorceries.
//!
//! So a behavioural probe for any of this would need a synthetic card the corpus cannot
//! produce, and would measure the fixture rather than the engine. A source gate that makes
//! an edit to the list VISIBLE is the honest instrument, and it is stated as such.
//!
//! CR citations: CR 707.10 (a copy copies decisions, including additional and alternative
//! costs; choices normally made on resolution are NOT copied), CR 707.2 (text-changing
//! effects are not copied), CR 702.140a/c (mutate), CR 702.47c (splice), CR 707.10f /
//! CR 608.3f (a copy of a permanent spell becomes a token — unimplemented, `OOS-DX50-3`).

use std::path::{Path, PathBuf};

fn engine_src(rel: &str) -> String {
    let p: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

/// The `matches!` body inside `copy_spell_on_stack`'s `additional_costs` filter, with
/// comments stripped so a variant named in prose cannot be counted as allowlisted.
///
/// Brace-matched from the `.filter(|c| {` that follows `additional_costs:`, never a
/// fixed-width window — PB-DX49's `/review` caught exactly that construct over-scanning
/// by a kilobyte into the next arm.
fn allowlist_body() -> String {
    let src = engine_src("rules/copy.rs");
    let anchor = src
        .find("additional_costs: original")
        .expect("copy_spell_on_stack must build `additional_costs` from the original");
    let rest = &src[anchor..];
    let open = rest
        .find("matches!(")
        .expect("the filter must be a `matches!` over the allowlist");
    let bytes = rest.as_bytes();
    let start = open + "matches!(".len();
    let mut depth = 1usize;
    let mut i = start;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    assert!(
        depth == 0,
        "unbalanced `matches!(` in copy.rs — fail closed"
    );
    strip_comments(&rest[start..i - 1])
}

fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        match line.find("//") {
            Some(i) => out.push_str(&line[..i]),
            None => out.push_str(line),
        }
        out.push('\n');
    }
    out
}

/// Every `AdditionalCost` variant name, parsed from the ENUM'S OWN DECLARATION rather
/// than hand-listed — PB-DX49's `r8` lesson, and `OOS-DX24-4`'s: a hand-listed set is a
/// claim, and a claim is what this queue keeps finding stale.
fn declared_variants() -> Vec<String> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../card-types/src/state/types.rs")
        .canonicalize()
        .expect("card-types/src/state/types.rs resolves");
    let src = std::fs::read_to_string(&p).expect("types.rs is readable");
    let start = src
        .find("pub enum AdditionalCost {")
        .expect("the AdditionalCost declaration must be findable");
    let body_start = start + "pub enum AdditionalCost {".len();
    let bytes = src.as_bytes();
    let mut depth = 1usize;
    let mut i = body_start;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    assert!(
        depth == 0,
        "unbalanced braces in the AdditionalCost declaration"
    );
    let body = strip_comments(&src[body_start..i - 1]);
    let mut out = Vec::new();
    let mut nest = 0usize;
    for line in body.lines() {
        let t = line.trim();
        if nest == 0 {
            if let Some(name) = t
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()
                .filter(|n| !n.is_empty() && n.chars().next().is_some_and(|c| c.is_uppercase()))
            {
                out.push(name.to_string());
            }
        }
        nest += t.matches('{').count();
        nest = nest.saturating_sub(t.matches('}').count());
    }
    out
}

/// The variants PB-DX50 leaves the copy path dropping, each with the disposition the
/// audit recorded. Kept beside `EXPECTED_ALLOWED` so the two together must partition the
/// declaration — an unlisted new variant reddens `r1` rather than sitting unclassified.
const EXPECTED_DROPPED: &[(&str, &str)] = &[
    (
        "Sacrifice",
        "FILE (MEDIUM) — CR 707.10's cost-object clause; latent, 0 copyable",
    ),
    (
        "Discard",
        "CORRECT-AS-IS — unobservable, no resolution read site",
    ),
    ("EscapeExile", "CORRECT-AS-IS — unobservable"),
    (
        "CollectEvidenceExile",
        "CORRECT-AS-IS — the linked bit rides `evidence_collected`",
    ),
    (
        "Assist",
        "CORRECT-AS-IS — CR 702.132a is a payment-rules modification",
    ),
    (
        "Replicate",
        "CORRECT-AS-IS — the trigger carries its own `copy_count`",
    ),
    (
        "Squad",
        "FILE (LOW-MED) — gated on CR 707.10f, `OOS-DX50-3`",
    ),
    (
        "Splice",
        "CORRECT-AS-IS — CR 702.47c text-changing, CR 707.2 excludes",
    ),
    (
        "Offspring",
        "FILE (LOW) — gated on CR 707.10f, `OOS-DX50-3`",
    ),
    (
        "Gift",
        "FILE (MEDIUM) — CR 702.174a is a decision; copy-reachable read site",
    ),
    ("ExileFromHand", "CORRECT-AS-IS — unobservable"),
];

const EXPECTED_ALLOWED: &[(&str, &str)] = &[
    ("Entwine", "CR 702.42b — a decision, CR 707.10 sentence 2"),
    ("Fuse", "CR 702.102d — a decision"),
    ("EscalateModes", "CR 702.120a — a decision"),
    (
        "Mutate",
        "CR 702.140a — the host is a TARGET, and CR 707.10 sentence 2 copies targets. \
         ADDED BY PB-DX50. CR 707.10 sentence 3 (resolution choices are not copied) is \
         satisfied by construction: half 2 removed `on_top` from the variant.",
    ),
];

/// **r1** — the allowlist is exactly `EXPECTED_ALLOWED`, and allowed ∪ dropped is exactly
/// the enum's own declared variant set.
///
/// The second half is what makes an edit to `AdditionalCost` visible here: a 16th variant
/// is unclassified and this reddens, rather than being silently dropped by a filter nobody
/// re-read.
///
/// **Revert to watch red**: remove `AdditionalCost::Mutate { .. }` from `copy.rs`'s
/// `matches!`, or add any other variant to it.
#[test]
fn r1_the_copy_allowlist_is_exactly_the_classified_set() {
    let body = allowlist_body();
    let declared = declared_variants();
    assert_eq!(
        declared.len(),
        15,
        "PB-DX50 classified 15 `AdditionalCost` variants; the declaration now has {}. \
         Classify the new one in EXPECTED_ALLOWED or EXPECTED_DROPPED (with its CR \
         reason) before touching `copy.rs`. Declared: {declared:?}",
        declared.len()
    );

    let mut classified: Vec<&str> = EXPECTED_ALLOWED.iter().map(|(v, _)| *v).collect();
    classified.extend(EXPECTED_DROPPED.iter().map(|(v, _)| *v));
    let mut sorted_classified = classified.clone();
    sorted_classified.sort_unstable();
    let mut sorted_declared: Vec<&str> = declared.iter().map(|s| s.as_str()).collect();
    sorted_declared.sort_unstable();
    assert_eq!(
        sorted_classified, sorted_declared,
        "every declared `AdditionalCost` variant must be classified exactly once"
    );

    for (variant, reason) in EXPECTED_ALLOWED {
        assert!(
            body.contains(&format!("AdditionalCost::{variant}")),
            "`{variant}` must be propagated to a spell copy ({reason}). Allowlist body: {body}"
        );
    }
    for (variant, reason) in EXPECTED_DROPPED {
        assert!(
            !body.contains(&format!("AdditionalCost::{variant}")),
            "`{variant}` must NOT be propagated to a spell copy ({reason}). If you are \
             fixing one of the FILE rows, move it to EXPECTED_ALLOWED in the same commit. \
             Allowlist body: {body}"
        );
    }
}

/// **r2** — the brace matcher does not over-scan, and the extractor is not vacuous.
///
/// # HONESTLY UNDISCRIMINATED, disclosed here and not only in `memory/`
///
/// The revert matrix for this batch has one row that could not be made to fail: **making
/// `strip_comments` the identity leaves `r1` GREEN.** The reason is structural, not a
/// missing probe — the brace-matched region is the inside of `copy.rs`'s `matches!(..)`,
/// and there are no comments in there today; every variant named in prose sits ABOVE the
/// `additional_costs:` anchor and is therefore outside the window by construction.
///
/// So the stripping is **defensive, not currently load-bearing**, and it is kept for one
/// stated reason rather than by inertia: the day someone annotates a variant inside the
/// `matches!` — `// AdditionalCost::Gift, see the audit` is exactly the shape this
/// codebase writes — `r1`'s `!body.contains(..)` half would go silently green on a
/// variant that is still dropped. This is PB-DX47's `r3`/`r3b` finding recurring: a guard
/// whose subject is empty today is not a guard nobody needs, it is a guard nobody can
/// currently prove. **Do not delete it on the grounds that no test fails — no test CAN
/// fail, and that is the point of writing it down here.**
///
/// What `r2` DOES discriminate is the brace matcher: a fixed-width window that ran past
/// the `matches!` into the neighbouring field initialisers would make `r1`'s `!contains`
/// half silently green, and the size bound below catches it.
#[test]
fn r2_the_allowlist_extractor_is_neither_vacuous_nor_over_wide() {
    let body = allowlist_body();
    assert!(
        !body.trim().is_empty(),
        "the extracted allowlist body must not be empty — fail closed"
    );
    assert!(
        body.len() < 600,
        "the extracted body is {} bytes, which is far more than a four-variant \
         `matches!` — the brace matcher has over-scanned into neighbouring code and \
         `r1`'s `!contains` half would go silently green. Body: {body}",
        body.len()
    );
    // The stripping itself is DEFENSIVE, not currently load-bearing — see this test's
    // own doc for the disclosure and for why it stays anyway.
    assert!(!body.contains("//"), "comments must be stripped: {body}");
    // The prose that WOULD defeat `r1` if the window ever widened: `copy.rs`'s comment
    // names every dropped variant, a few hundred bytes above the anchor. This assertion
    // is what makes the size bound above a real bound rather than an arbitrary number —
    // it pins that the defeating input exists and is nearby.
    let src = engine_src("rules/copy.rs");
    assert!(
        src.contains("`Sacrifice`, `Gift`, `Squad` and `Offspring` are dropped"),
        "copy.rs's own comment must name the dropped variants — that prose is the input \
         a wider window would swallow, and the size bound above is only meaningful while \
         it exists"
    );
}

/// **r3** — `copy.rs`'s comment states CR 707.10 rather than the invented
/// choice-vs-one-shot-cost dichotomy it used to claim.
///
/// A gate on PROSE, and that is the point: the old comment's stated RULE was refuted by
/// the CR it cited, and its LIST named 6 of the 12 dropped variants. A comment asserting a
/// property nothing enforces is the defect this project keeps filing, so the corrected
/// text is pinned.
#[test]
fn r3_the_allowlist_comment_cites_the_rule_that_actually_governs() {
    let src = engine_src("rules/copy.rs");
    assert!(
        src.contains("CR 707.10"),
        "the allowlist comment must cite CR 707.10, the rule that governs what a copy \
         inherits"
    );
    assert!(
        !src.contains("Copies copy choices (entwine, escalate, fuse) but not one-shot"),
        "the refuted comment is back: CR 707.10 says a copy copies \"additional or \
         alternative costs\" in as many words, and CR 707.2's own example list includes \
         \"whether it was kicked\". The choice-vs-one-shot-cost dichotomy does not exist."
    );
    // The two genuinely CR-correct drops must carry their reasons, which the old
    // comment omitted entirely.
    assert!(
        src.contains("CR 702.47c"),
        "Splice's drop must state its reason (CR 702.47c makes it text-changing, and \
         CR 707.2 excludes text-changing effects)"
    );
    assert!(
        src.contains("CR 702.140c"),
        "the mutate over/under drop must state its reason (CR 702.140c makes it a \
         resolution choice, and CR 707.10 excludes those)"
    );
}

/// **r4** — the `MutatingCreatureSpell` resolution arm consults `is_copy`.
///
/// `copy.rs` clones `original.kind` wholesale, so a copy of a mutating creature spell
/// names the ORIGINAL's card in its `source_object`. Without a guard, the copy's
/// resolution calls `move_object_to_zone` on that card (CR 702.140b branch) or merges it
/// into the target (CR 729.2 branch) — either way consuming another object's card and
/// leaving the original to resolve against a dead `ObjectId` (CR 400.7).
///
/// The sibling `StackObjectKind::Spell` arm guards exactly this, and its comment says so:
/// *"The source_object belongs to the original spell and must not be moved by a copy's
/// resolution."* The counter path guards it a second time. **The mutate arm was the third
/// such site and had ZERO `is_copy` mentions.**
///
/// Source-level and not behavioural, for this file's stated reason: no corpus def can copy
/// a creature spell, so the path is unreachable and a probe would measure a synthetic
/// fixture. **Disclosed here rather than only in `memory/`.**
///
/// **Revert to watch red**: delete the `if stack_obj.is_copy { … return … }` block at the
/// head of the `MutatingCreatureSpell` arm.
#[test]
fn r4_the_mutate_resolution_arm_guards_on_is_copy() {
    let src = engine_src("rules/resolution.rs");
    let arm = src
        .find("StackObjectKind::MutatingCreatureSpell {")
        .and_then(|i| src[i..].find("} => {").map(|j| i + j))
        .expect("the MutatingCreatureSpell resolution arm must be findable");
    // Brace-match the arm rather than taking a fixed window (PB-DX49 `/review`).
    let bytes = src.as_bytes();
    let mut i = arm + "} => {".len();
    let mut depth = 1usize;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    assert!(
        depth == 0,
        "unbalanced braces in the mutate arm — fail closed"
    );
    let body = &src[arm..i];
    // Non-vacuity: the extracted body really is the mutate arm.
    assert!(
        body.contains("CR 702.140b"),
        "the extracted body must be the mutate arm; got {} bytes",
        body.len()
    );
    assert!(
        body.contains("stack_obj.is_copy"),
        "CR 707.10: the MutatingCreatureSpell arm must consult `is_copy`, or a resolving \
         COPY moves or merges the ORIGINAL's card. The sibling `Spell` arm has guarded \
         this since before PB-DX50, under a comment saying exactly why."
    );

    // ── The conjunct the `/review` proved this gate needed ──────────────────────
    //
    // The assertion above is satisfied by BOTH shapes of the guard: the correct
    // `if … { … } else { … }` and the first draft's
    // `if stack_obj.is_copy { … return Ok(events); }`. The second HANGS THE GAME --
    // `return` leaves `resolve_top_of_stack_inner` altogether, skipping
    // `check_triggers_with_timing`, `check_and_apply_sbas`, `flush_pending_triggers` and
    // `priority::grant_priority_to_active_player`, so both seats have passed and nobody
    // holds priority. **`r4` was green through the entire episode**, which is the whole
    // finding: a needle that both the fix and the defect contain measures nothing.
    //
    // # Why POSITION, and not "the arm contains no `return`"
    //
    // The arm has exactly one LEGITIMATE early return -- CR 608.2d's suspend, the
    // `None => return Ok(events)` arm of the `ask_resolution_choice` match, which is
    // every other suspending site's idiom in this engine and must stay allowed. Two
    // candidate discriminators were available and both are used, because each catches
    // what the other misses:
    //
    //  * **position** -- every `return Ok(events)` must sit AFTER the
    //    `ask_resolution_choice(` call. This is the one that catches the shipped defect:
    //    the first draft's return was at the HEAD of the arm, ~13,000 bytes earlier.
    //  * **the surrounding arm** -- each one must be on a line carrying `None =>`. This
    //    catches a second early return added *below* the ask (say in the merge branch),
    //    which position alone would wave through.
    //
    // **The second conjunct is a FORM gate and its cost is stated, not hidden.** The
    // executed defeat that proves it discriminates -- rewriting the suspend as
    // `None => { return Ok(events); }` -- is behaviour-NEUTRAL, so this conjunct will
    // also fire on an honest reformatting, and on the equally honest
    // `if answer.is_none() { return Ok(events); }` refactor. That is a deliberate trade
    // in this project's direction (*a ratchet's slack IS its blind spot*, PB-DX47): the
    // defect class it uniquely catches is a NEW early return placed below the ask, in
    // the merge branch, after state has been written -- which neither the count nor the
    // position conjunct can see, because the count stays 1 if the suspend return is what
    // it replaced. **If it fires on a refactor, widen the accepted idiom here; do not
    // delete the check.**
    //
    // Comments are stripped first, and that is load-bearing here rather than defensive:
    // the arm's own doc quotes the defective line verbatim (*"shipped as an early
    // `return Ok(events);`"*) while explaining why it is wrong, so an unstripped scan
    // would find a `return Ok(events)` at the head of the arm and fail on the CORRECT
    // code -- fail-open's opposite, a gate that can only be satisfied by deleting the
    // explanation.
    let stripped = strip_comments(body);
    let ask = stripped.find("ask_resolution_choice(").expect(
        "the mutate arm must ask CR 702.140c's over/under question via \
         `ask_resolution_choice` -- if this moved, re-derive the position anchor below \
         rather than deleting it",
    );
    let returns: Vec<usize> = stripped
        .match_indices("return Ok(events)")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        returns.len(),
        1,
        "the MutatingCreatureSpell arm must contain EXACTLY ONE `return Ok(events)` \
         (CR 608.2d's suspend). Found {}. A second early return skips the shared \
         resolution tail -- `check_triggers_with_timing`, `check_and_apply_sbas`, \
         `flush_pending_triggers` and `priority::grant_priority_to_active_player` -- \
         which strands the stack with nobody holding priority. See \
         `primitives::pb_dx50_mutate_on_top_timing::t8`.",
        returns.len()
    );
    for at in returns {
        assert!(
            at > ask,
            "a `return Ok(events)` sits at byte {at} of the MutatingCreatureSpell arm, \
             BEFORE the CR 608.2d ask at byte {ask}. That is the first draft's \
             `is_copy` guard shape and it HANGS THE GAME: the shared resolution tail \
             (SBAs, trigger flush, `grant_priority_to_active_player`) is skipped, so \
             both seats have passed and `priority_holder` is None. The sibling `:819` \
             guard is an `if / else if` chain that FALLS THROUGH to that tail -- use an \
             `else`, not a `return`."
        );
        let line_start = stripped[..at].rfind('\n').map_or(0, |i| i + 1);
        let line_end = stripped[at..].find('\n').map_or(stripped.len(), |i| at + i);
        let line = &stripped[line_start..line_end];
        assert!(
            line.contains("None =>"),
            "the only permitted `return Ok(events)` in this arm is CR 608.2d's suspend \
             (`None => return Ok(events)`), where returning without applying anything is \
             the documented contract and `resolve_top_of_stack` rolls the resolution \
             back. This one is on `{}`, which is a different control-flow decision and \
             inherits the obligations of every statement it skips (PB-DP8).",
            line.trim()
        );
    }
}
