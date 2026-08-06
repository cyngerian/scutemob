//! PB-DX25b (`OOS-DX25-3`) gates: R1-R5 of the plan's §5.3.
//!
//! R1/R2/R3 are SR-36 corpus rosters (enumerate `all_cards()`, never grep the
//! corpus -- the dispatch brief's own grep-derived claim about
//! `Effect::CopySpellOnStack` usage was refuted this way, plan §1 fact 9). R4/R5
//! are structural source gates over `crates/engine/src/effects/mod.rs` and
//! `crates/engine/src/`, proving the shared helper is actually used (not
//! re-open-coded) at the two `effects/mod.rs` consumer sites and nowhere else.
//!
//! Both source gates strip **line and block** comments before scanning -- the
//! PB-DX32 M8 lesson, reapplied by every PB-DX2x roster/gate file since: a
//! `/* ... */`-wrapped literal defeats a line-comment-only scanner while every
//! probe stays green, because the compiler drops the commented-out code and the
//! scanner never sees it disappear.

use mtg_engine::{all_cards, CardDefinition, Completeness};
use std::collections::BTreeSet;
use std::path::Path;

// ── Comment-stripping (mirrors pb_dx25_stack_registry_roster.rs's idiom) ───────

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

fn extract_match_arm_body<'a>(stripped: &'a str, pattern_marker: &str) -> &'a str {
    let pat_start = stripped
        .find(pattern_marker)
        .unwrap_or_else(|| panic!("`{pattern_marker}` not found in stripped source"));
    let arrow = stripped[pat_start..]
        .find("=> {")
        .map(|i| pat_start + i)
        .unwrap_or_else(|| panic!("no `=> {{` found after `{pattern_marker}`"));
    let open_brace = arrow + "=> ".len();
    balanced_body(stripped, open_brace, pattern_marker)
}

const EFFECTS_MOD_PATH: &str = "src/effects/mod.rs";

// ── Sanitized-Debug walker (PB-DX25b review Finding E3) ─────────────────────
//
// R1/R2/R3 originally used HAND-WRITTEN structural walkers over `Effect` /
// `AbilityDefinition` (matching `Spell`/`Activated`/`Triggered` and
// `Sequence`/`Conditional`/`ForEach`/`Choose`, `_ => false` elsewhere). The
// PB-DX25b review (Finding E3) measured their blind spots against
// `card_definition.rs`: four recursive `Effect` carriers were never descended
// into (`Repeat`, `MayPayOrElse`, `MayPayThenEffect`, `CoinFlip`), and four
// `AbilityDefinition` variants were never examined at all (`LoyaltyAbility`,
// `SagaChapter`, `ClassLevel`, `Forecast`) -- all correctly excluded from
// today's corpus, but a `_ => {}`/`_ => false` arm cannot tell "correctly
// excluded" from "silently missed", and a FUTURE def in one of those shapes
// would move R1/R2/R3's rosters without moving these tests.
//
// **Chosen fix: plan §5.3 option (b), the sanitized-Debug scan** -- total over
// the WHOLE struct by construction (every field must implement `Debug` for
// `#[derive(Debug)]` to compile at all), so it is immune to a new recursive
// `Effect`/`AbilityDefinition` variant in a way a hand-written walker never
// can be. `Effect`/`TargetRequirement`/`AbilityDefinition` all derive `Debug`
// with NO custom impl, so the emitted text is the bare variant name (e.g.
// `ChangeTargets { .. }`, not `Effect::ChangeTargets { .. }`) -- confirmed by
// grepping their `#[derive(..)]` attributes before relying on it.
//
// **The blind spot of THIS approach, stated per the plan's own instruction
// ("the choice and its blind spot go in the file's doc comment")**: a
// sanitized `Debug` scan is a SUBSTRING search over free text. It is immune to
// new *code* shapes (new enum variants, new nesting), but NOT immune to a
// FUTURE free-text `String` field elsewhere on `CardDefinition` that happens
// to contain one of the needle strings (`"ChangeTargets"`,
// `"CopySpellOnStack"`, `"TargetSpellWithSingleTarget"`,
// `"TargetSpellOrAbilityWithSingleTarget"`, `"DrawCards"`) in hand-authored
// English. `sanitize()` below closes the ONE such field this batch found by
// experiment (`Completeness`'s note, `plumb_the_forbidden.rs`'s own
// `Completeness::partial(...)` prose literally contains the string
// `Effect::CopySpellOnStack` -- see the executed proof at R3's own site
// below) plus `oracle_text` on every face (front, back, adventure) as a
// defensive measure (printed game text is Rust-identifier-shaped only by
// coincidence, but nothing stops a future author's note referencing an
// engine type name the way `plumb_the_forbidden.rs`'s did). A field this
// sanitization does NOT know about (e.g. a future free-text `description` on
// `AbilityDefinition::Activated`) is a residual, exactly as R4/R5's own doc
// comments state their residuals rather than implying total coverage.

/// Returns a `{:?}`-formatted string of `def` with every known free-text
/// prose field cleared/normalized, so a substring search over it cannot
/// false-positive on hand-authored English that happens to contain a Rust
/// type name. See the module note above for the mechanism and its own
/// residual.
fn sanitized_debug(def: &CardDefinition) -> String {
    let mut clone = def.clone();
    clone.oracle_text = String::new();
    if let Some(face) = clone.back_face.as_mut() {
        face.oracle_text = String::new();
    }
    if let Some(face) = clone.adventure_face.as_mut() {
        face.oracle_text = String::new();
    }
    clone.completeness = Completeness::Complete;
    format!("{clone:?}")
}

// ── R1: the requirement roster ──────────────────────────────────────────────

/// R1 (plan §5.3): defs carrying `TargetSpellWithSingleTarget` or
/// `TargetSpellOrAbilityWithSingleTarget` anywhere, by NAME. Non-vacuity floor
/// asserted in the SAME test (PB-DX24 R2 lesson: a broken enumeration must not
/// make an empty roster look correct).
#[test]
fn r1_single_target_spell_requirement_roster_is_pinned() {
    let cards = all_cards();
    assert!(
        cards.len() >= 1_700,
        "PB-DX25b R1: non-vacuity floor -- all_cards() must return at least \
         1,700 defs (measured on this branch: {}) -- got {}. A broken \
         enumeration cannot make an empty roster look correct.",
        cards.len(),
        cards.len()
    );

    let roster: BTreeSet<String> = cards
        .iter()
        .filter(|d| {
            let debug = sanitized_debug(d);
            debug.contains("TargetSpellWithSingleTarget")
                || debug.contains("TargetSpellOrAbilityWithSingleTarget")
        })
        .map(|d| d.name.clone())
        .collect();

    let expected: BTreeSet<String> = ["Bolt Bend", "Misdirection", "Untimely Malfunction"]
        .into_iter()
        .map(String::from)
        .collect();

    assert_eq!(
        roster,
        expected,
        "PB-DX25b R1 (TargetSpellWithSingleTarget / \
         TargetSpellOrAbilityWithSingleTarget requirement roster) moved -- \
         expected {} names, got {}: {roster:?}. A new def carrying either \
         requirement widens the class this batch repairs -- re-derive R2 below.",
        expected.len(),
        roster.len()
    );

    // Of the roster, exactly the two named `Complete` defs are live-wrong at
    // HEAD (plan §2.4). Untimely Malfunction stays `partial`, but NOT for the
    // C1/C2 lookup defect this batch fixes: mode 2's variable-target-count
    // gap (TargetRequirement::UpToN has no minimum) is a SEPARATE, unrelated
    // limitation. This is now a MEASURED claim, not an assumed one -- PB-DX25b
    // review Finding C2 required a modal-index probe before "unrelated" could
    // be trusted; `t10_untimely_malfunction_mode1_target_index` casts mode 1
    // for real and confirms it redirects correctly post-fix, so mode 2's gap
    // really is the only thing keeping this def out of `Complete`.
    let complete: BTreeSet<String> = roster
        .iter()
        .filter(|name| {
            cards
                .iter()
                .find(|d| &d.name == *name)
                .is_some_and(|d| d.completeness == Completeness::Complete)
        })
        .cloned()
        .collect();
    let expected_complete: BTreeSet<String> = ["Bolt Bend", "Misdirection"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        complete,
        expected_complete,
        "PB-DX25b R1 Complete-subset moved -- expected {} names, got {}: \
         {complete:?}",
        expected_complete.len(),
        complete.len()
    );
}

// ── R2: the Effect::ChangeTargets roster ────────────────────────────────────

/// R2 (plan §5.3): defs whose abilities -- either face -- contain
/// `Effect::ChangeTargets` anywhere (incl. inside a modal `ModeSelection`), by
/// NAME. Includes Deflecting Swat (`must_change: false`), which the dispatch
/// brief's site analysis missed (plan §0.4 F-A): it remains a documented
/// no-op after this batch (`must_change: false` -> `effects/mod.rs`'s
/// deterministic-fallback `continue`), so membership here does NOT mean
/// "works" for every row.
///
/// **PB-DX25b review Finding E3: now built on the sanitized-Debug walker**
/// (see the module note above), not a hand-written structural walk -- total
/// over the whole `Effect` tree by construction, including
/// `LoyaltyAbility`/`SagaChapter`/`ClassLevel`/`Forecast` and
/// `Repeat`/`MayPayOrElse`/`MayPayThenEffect`/`CoinFlip`, none of which the
/// old walker examined.
#[test]
fn r2_change_targets_roster_is_pinned() {
    let cards = all_cards();
    let roster: BTreeSet<String> = cards
        .iter()
        .filter(|d| sanitized_debug(d).contains("ChangeTargets"))
        .map(|d| d.name.clone())
        .collect();

    let expected: BTreeSet<String> = [
        "Bolt Bend",
        "Deflecting Swat",
        "Misdirection",
        "Untimely Malfunction",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    assert_eq!(
        roster,
        expected,
        "PB-DX25b R2 (Effect::ChangeTargets roster) moved -- expected {} \
         names, got {}: {roster:?}. Deflecting Swat is a documented no-op \
         (must_change: false, F-A) -- membership here does not mean 'works'.",
        expected.len(),
        roster.len()
    );
}

// ── R3: the Effect::CopySpellOnStack roster (expected EMPTY) ───────────────

/// R3 (plan §5.3): the `Effect::CopySpellOnStack` roster, expected EMPTY --
/// with a walker-liveness CONTROL, not just a corpus floor. PB-DX25's own T6
/// advertised non-vacuity while comparing a hand-written fixture to itself;
/// this is the same trap for an EMPTY expected roster. The control
/// (`Effect::DrawCards`, a common effect) proves the identical walker
/// mechanism returns a NON-empty set when the effect actually exists in the
/// corpus, so the empty `CopySpellOnStack` roster below is not indistinguishable
/// from a broken walk.
///
/// **Refutes the dispatch brief's grep-derived claim (plan §1 fact 9):**
/// `plumb_the_forbidden` and `complete_the_circuit` were claimed to use
/// `Effect::CopySpellOnStack`; neither actually constructs it in code (one
/// mentions the literal string only inside `Completeness::partial(...)` PROSE,
/// the other only inside a Rust comment) -- exactly the SR-36 failure mode
/// this structural walk exists to avoid.
///
/// **PB-DX25b review Finding E3: now built on `sanitized_debug` (module note
/// above), not a hand-written structural walk.** `plumb_the_forbidden.rs`'s
/// `Completeness::partial(...)` prose LITERALLY CONTAINS the string
/// `Effect::CopySpellOnStack` -- see `r3_sanitization_is_load_bearing` below,
/// which executes the UNSANITIZED variant and shows it false-positives on
/// exactly this def, proving the sanitization step is not decorative.
#[test]
fn r3_copy_spell_on_stack_roster_is_empty_with_liveness_control() {
    let cards = all_cards();

    let draw_cards_control: BTreeSet<String> = cards
        .iter()
        .filter(|d| sanitized_debug(d).contains("DrawCards"))
        .map(|d| d.name.clone())
        .collect();
    assert!(
        !draw_cards_control.is_empty(),
        "PB-DX25b R3 walker-liveness control: Effect::DrawCards must be found \
         on at least one corpus def by the SAME walker mechanism used for \
         Effect::CopySpellOnStack below -- an empty result here would mean the \
         walk itself is broken, not that the corpus is empty of DrawCards. Got \
         {} names.",
        draw_cards_control.len()
    );

    let copy_spell_on_stack_roster: BTreeSet<String> = cards
        .iter()
        .filter(|d| sanitized_debug(d).contains("CopySpellOnStack"))
        .map(|d| d.name.clone())
        .collect();
    assert!(
        copy_spell_on_stack_roster.is_empty(),
        "PB-DX25b R3 (Effect::CopySpellOnStack roster) moved from the expected \
         EMPTY -- got {} names: {copy_spell_on_stack_roster:?}. C4's fix \
         (effects/mod.rs, plan §3.2) is no longer purely synthetic if this is \
         non-empty -- re-derive its completeness impact.",
        copy_spell_on_stack_roster.len()
    );
}

/// PB-DX25b review Finding E3: executes the UNSANITIZED variant of R3's own
/// scan (`format!("{:?}", def)` with no `oracle_text`/`completeness`
/// clearing) and asserts it FALSE-POSITIVES on `plumb_the_forbidden` -- the
/// trap the plan named (§5.3): that def's `Completeness::partial(...)` note
/// literally contains the string `Effect::CopySpellOnStack` in hand-authored
/// English, even though the def never constructs that effect in code. This is
/// the executed proof that `sanitized_debug`'s cleanup is load-bearing, not
/// decorative -- required by the review rather than merely asserted.
#[test]
fn r3_sanitization_is_load_bearing() {
    let cards = all_cards();
    let plumb = cards
        .iter()
        .find(|d| d.name == "Plumb the Forbidden")
        .expect("Plumb the Forbidden must exist in the corpus for this proof");

    // The SANITIZED scan (R3's real mechanism) must NOT flag it.
    assert!(
        !sanitized_debug(plumb).contains("CopySpellOnStack"),
        "sanitized_debug(Plumb the Forbidden) must NOT contain \
         \"CopySpellOnStack\" -- the def does not construct that effect in \
         code; if this fails, sanitization regressed"
    );

    // The UNSANITIZED scan (the naive `format!("{:?}", def)` the plan warned
    // against) MUST flag it -- proving the sanitization step, not the corpus,
    // is what keeps R3 from false-positiving.
    let unsanitized = format!("{plumb:?}");
    assert!(
        unsanitized.contains("CopySpellOnStack"),
        "UNSANITIZED format!(\"{{:?}}\", def) must contain \"CopySpellOnStack\" \
         for Plumb the Forbidden -- if this fails, the def's note no longer \
         contains the literal string this proof depends on, and the trap this \
         test exists to demonstrate no longer applies here (re-derive against \
         a different def before deleting this test)"
    );
}

// ── R4: source gate over the two effects/mod.rs arms ────────────────────────

/// R4 (plan §5.3): after comment-stripping, the `Effect::ChangeTargets` and
/// `Effect::CopySpellOnStack` arm bodies must each (a) contain
/// `stack_index_for_announced_target` at least once, and (b) contain ZERO
/// occurrences of `stack_objects.iter()` / `stack_objects.iter_mut()`.
///
/// **Residual, stated honestly (PB-DX25b review Finding E2 correction)**: this
/// gate sees only the two arms it names. A brand-new arm elsewhere in
/// `effects/mod.rs`, OR a brand-new `TargetRequirement` arm in `casting.rs`,
/// that takes an announced id and re-open-codes `stack_objects.iter().find
/// (...)` is invisible to it -- exactly as PB-DX25's G2 was blind to
/// `resolution.rs` until its review added G4. R6 below extends this SAME
/// per-arm structural check to the two `casting.rs` arms (C1/C2), closing that
/// half of the gap for the FOUR arms this batch actually touched. **R5 below
/// is NOT a wide net** (the review found and this batch confirmed three ways
/// to defeat its original form -- see R5's own doc for what its hardened form
/// still cannot catch): a genuinely NEW fifth site that never calls
/// `card_in_stack_zone` at all is invisible to every gate in this file. No
/// mechanism in this tree detects that shape; only R4+R6 (per-arm, per known
/// site) and R5 (per-shape, corpus-wide) exist, and each is scoped to what its
/// own doc says.
#[test]
fn r4_change_targets_and_copy_spell_on_stack_arms_use_the_shared_helper() {
    let stripped = strip_comments(&read_source(EFFECTS_MOD_PATH));

    for (marker, label) in [
        ("Effect::ChangeTargets {", "Effect::ChangeTargets"),
        (
            "Effect::CopySpellOnStack { target, count } => {",
            "Effect::CopySpellOnStack",
        ),
    ] {
        let body = extract_match_arm_body(&stripped, marker);
        assert!(
            body.len() >= 200,
            "PB-DX25b R4 non-vacuity: the extracted {label} arm body looks too \
             small ({} chars) to contain the full lookup logic -- extraction \
             may be broken",
            body.len()
        );
        let helper_calls = body.matches("stack_index_for_announced_target").count();
        assert!(
            helper_calls >= 1,
            "PB-DX25b R4: the {label} arm must call \
             stack_index_for_announced_target at least once -- do not re-open-code \
             the announced-target lookup here, got {helper_calls} calls",
            label = label
        );
        assert_eq!(
            body.matches("stack_objects.iter()").count()
                + body.matches("stack_objects.iter_mut()").count(),
            0,
            "PB-DX25b R4: the {label} arm must contain ZERO occurrences of \
             stack_objects.iter()/iter_mut() -- the lookup must go through \
             stack_index_for_announced_target, not a re-open-coded scan",
            label = label
        );
    }
}

// ── R6: source gate over the two casting.rs arms (PB-DX25b review Finding E2) ──

const CASTING_RS_PATH: &str = "src/rules/casting.rs";

/// `casting.rs`'s C1/C2 sites are `if matches!(req, TargetRequirement::X) {
/// .. }` blocks inside `validate_object_satisfies_requirement`, not `match`
/// arms with `=> {` -- `extract_match_arm_body`'s marker shape does not apply.
/// Finds the marker, then the FIRST `{` after it, and extracts the balanced
/// body from there (reusing `balanced_body`, which is shape-agnostic).
fn extract_if_block_body<'a>(stripped: &'a str, pattern_marker: &str) -> &'a str {
    let pat_start = stripped
        .find(pattern_marker)
        .unwrap_or_else(|| panic!("`{pattern_marker}` not found in stripped source"));
    let brace_rel = stripped[pat_start..]
        .find('{')
        .unwrap_or_else(|| panic!("no `{{` found after `{pattern_marker}`"));
    let open_brace = pat_start + brace_rel;
    balanced_body(stripped, open_brace, pattern_marker)
}

/// R6 (PB-DX25b review Finding E2, the PREFERRED fix): the SAME structural
/// check R4 applies to the two `effects/mod.rs` arms (C3/C4), applied to the
/// two `casting.rs` arms (C1/C2) -- the only production sites with a
/// behavioural regression test (T1/T2/the in-source `casting.rs` tests) but,
/// before this fix, NO structural source gate at all. After
/// comment-stripping, the `TargetSpellOrAbilityWithSingleTarget` and
/// `TargetSpellWithSingleTarget` `if` blocks must each (a) contain
/// `stack_index_for_announced_target` at least once, and (b) contain ZERO
/// occurrences of `stack_objects.iter()` / `stack_objects.iter_mut()`.
///
/// **Residual, stated honestly (same shape as R4's)**: this gate sees only
/// the two arms it names. A BRAND-NEW `TargetRequirement` arm elsewhere in
/// this function that re-open-codes the lookup is invisible to it.
#[test]
fn r6_casting_c1_c2_arms_use_the_shared_helper() {
    let stripped = strip_comments(&read_source(CASTING_RS_PATH));

    for (marker, label) in [
        (
            "if matches!(req, TargetRequirement::TargetSpellOrAbilityWithSingleTarget) {",
            "TargetSpellOrAbilityWithSingleTarget (C1)",
        ),
        (
            "if matches!(req, TargetRequirement::TargetSpellWithSingleTarget) {",
            "TargetSpellWithSingleTarget (C2)",
        ),
    ] {
        let body = extract_if_block_body(&stripped, marker);
        assert!(
            body.len() >= 200,
            "PB-DX25b R6 non-vacuity: the extracted {label} block body looks \
             too small ({} chars) to contain the full lookup logic -- \
             extraction may be broken",
            body.len()
        );
        let helper_calls = body.matches("stack_index_for_announced_target").count();
        assert!(
            helper_calls >= 1,
            "PB-DX25b R6: the {label} block must call \
             stack_index_for_announced_target at least once -- do not \
             re-open-code the announced-target lookup here, got \
             {helper_calls} calls"
        );
        assert_eq!(
            body.matches("stack_objects.iter()").count()
                + body.matches("stack_objects.iter_mut()").count(),
            0,
            "PB-DX25b R6: the {label} block must contain ZERO occurrences of \
             stack_objects.iter()/iter_mut() -- the lookup must go through \
             stack_index_for_announced_target, not a re-open-coded scan"
        );
    }
}

// ── R5: the helper has no second implementation ─────────────────────────────

/// R5 (plan §5.3): scan `crates/engine/src/` (comment-stripped) for the
/// literal rule shape `card_in_stack_zone(` appearing in the same expression
/// as `so.id ==` / `s.id ==`, OUTSIDE `state/stack_registry.rs` -- the shape
/// `stack_index_for_announced_target`'s body itself has, and the shape a
/// future author re-open-coding the rule would reproduce. Assert zero.
///
/// **PB-DX25b review Finding E2: hardened, order- and preceding-statement-
/// insensitive.** The original version scanned a FIXED 150-byte window
/// strictly BEFORE each `card_in_stack_zone(` match and required no `;`
/// anywhere in that window. The review defeated it two ways, both executed
/// against the real gate and confirmed to defeat the ORIGINAL implementation
/// before this fix (see the batch's fix-cycle notes for the captured
/// before/after): (b) a PRECEDING statement's own trailing `;` (e.g. `let
/// announced = id;` two lines above a faithfully re-open-coded
/// `so.id == announced || (!so.is_copy && card_in_stack_zone(&so.kind) ==
/// Some(announced))`) falls inside the fixed window and trips the `;`
/// veto even though it has nothing to do with the two literals' own
/// statement; (c) writing the disjuncts in the OTHER order
/// (`card_in_stack_zone(...) == Some(announced) || so.id == announced`) puts
/// the id comparison AFTER the match position, invisible to a
/// backward-only window.
///
/// The fix: for each `card_in_stack_zone(` occurrence, search a SYMMETRIC
/// window in both directions for the nearest `so.id ==`/`s.id ==`
/// occurrence, then evaluate the span STRICTLY BETWEEN the two literals
/// (not a fixed byte count) for `||` (must be present -- this is what
/// legitimately excludes `resolution.rs::counter_stack_object`, whose
/// `so.id ==` lookup and later, separate `card_in_stack_zone(...)`
/// classification call have NO `||` connecting them at all, verified by
/// grep before relying on it) and the absence of `;`/`}` in that inter-
/// literal span (both would mean the two literals are in different
/// statements/blocks, not one joined expression). This closes (b) and (c);
/// it does NOT close every possible defeat -- see the module note on
/// `sanitized_debug` above for the general principle, and Finding E2's own
/// remaining residual stated at R4's doc: a genuinely new lookup that never
/// calls `card_in_stack_zone` at all has no `card_in_stack_zone(` occurrence
/// for this scan to anchor on, and is invisible to it BY CONSTRUCTION, not
/// by an implementation gap this hardening could close.
#[test]
fn r5_the_announced_target_rule_has_no_second_open_coded_copy() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offending: Vec<String> = Vec::new();
    let mut files_scanned = 0usize;

    fn visit(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("failed to read dir {}: {e}", dir.display()))
        {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                visit(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }

    let mut files = Vec::new();
    visit(&src_dir, &mut files);
    assert!(
        files.len() >= 40,
        "PB-DX25b R5 non-vacuity: expected at least 40 .rs files under \
         crates/engine/src/ (measured on this branch: 43), got {} -- the \
         directory walk may be broken",
        files.len()
    );

    for path in &files {
        files_scanned += 1;
        let relative = path
            .strip_prefix(&src_dir)
            .unwrap()
            .to_string_lossy()
            .to_string();
        if relative == "state/stack_registry.rs" {
            continue; // the one legitimate implementation
        }
        let raw = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let stripped = strip_comments(&raw);
        // Symmetric window, statement-boundary-aware -- see the doc comment
        // above (PB-DX25b review Finding E2) for why this replaced the
        // original fixed-backward-window/`;`-only heuristic.
        const SEARCH_WINDOW: usize = 250;
        let id_eq_positions: Vec<(usize, usize)> = ["so.id ==", "s.id =="]
            .iter()
            .flat_map(|needle| {
                stripped
                    .match_indices(needle)
                    .map(|(i, m)| (i, m.len()))
                    .collect::<Vec<_>>()
            })
            .collect();
        for czi_pos in stripped
            .match_indices("card_in_stack_zone(")
            .map(|(i, _)| i)
        {
            let ctx_lo = czi_pos.saturating_sub(SEARCH_WINDOW);
            let ctx_hi = (czi_pos + SEARCH_WINDOW).min(stripped.len());
            for &(id_eq_pos, id_eq_len) in &id_eq_positions {
                if id_eq_pos < ctx_lo || id_eq_pos > ctx_hi {
                    continue;
                }
                let (span_start, span_end) = if id_eq_pos < czi_pos {
                    (id_eq_pos, czi_pos + "card_in_stack_zone(".len())
                } else {
                    (czi_pos, id_eq_pos + id_eq_len)
                };
                let span = &stripped[span_start..span_end];
                let has_or = span.contains("||");
                let same_statement =
                    !span.contains(';') && !span.contains('}') && !span.contains("let ");
                if has_or && same_statement {
                    offending.push(format!(
                        "{relative} (card_in_stack_zone at byte offset {czi_pos}, \
                         id-eq at byte offset {id_eq_pos})"
                    ));
                }
            }
        }
    }

    assert!(
        files_scanned >= 40,
        "PB-DX25b R5 non-vacuity: files_scanned must be >= 40, got {files_scanned}"
    );
    assert!(
        offending.is_empty(),
        "PB-DX25b R5: the announced-target rule (`card_in_stack_zone(...)` \
         paired with a direct `so.id ==`/`s.id ==` comparison) must live ONLY \
         in state/stack_registry.rs -- found a second open-coded copy in: \
         {offending:?}"
    );
}
