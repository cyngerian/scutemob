//! PB-DX25c (`OOS-DX25b-3`) gates: R1-R5 of the plan's §5.4. (R6, the
//! candidate-universe behavioural comparison, lives as an in-source unit
//! test inside `crates/engine/src/rules/retarget.rs` -- `retarget_candidates`
//! is `pub(crate)`, unreachable from this external test crate; the
//! `casting.rs::validate_target_spell_with_single_target_self_and_kind_check`
//! precedent PB-DX25b's own T8 doc cites is the same shape.)
//!
//! Reuses `pb_dx25b_announced_target_roster.rs`'s `strip_comments`
//! (line-AND-block, the PB-DX32 M8 lesson), `balanced_body`,
//! `extract_match_arm_body` and `sanitized_debug` corpus walker rather than
//! writing new ones (plan §5.4's explicit instruction) -- copied verbatim
//! below rather than imported, since that file has no `pub` surface of its
//! own (it is itself a `tests/core/` module, not a library).

use mtg_engine::{all_cards, CardDefinition, Completeness};
use std::collections::BTreeSet;
use std::path::Path;

// ── Comment-stripping / extraction helpers (verbatim from
// pb_dx25b_announced_target_roster.rs -- see that file's own module doc for
// the PB-DX32 M8 lesson these encode) ───────────────────────────────────────

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

/// Verbatim from `pb_dx25b_announced_target_roster.rs` -- see that file's
/// module doc for the full rationale (a sanitized-Debug scan over the WHOLE
/// struct, immune to new recursive `Effect`/`AbilityDefinition` variants,
/// with the stated residual: a FUTURE free-text field this sanitization
/// doesn't know about).
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

fn all_rs_files_under(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
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
    visit(dir, &mut out);
    out
}

// ── R1: the must_change: true corpus roster, with the single-target claim
//        measured, not assumed ────────────────────────────────────────────

/// R1 (plan §5.4): defs carrying `Effect::ChangeTargets { must_change: true,
/// .. }` anywhere (either face), by NAME -- re-measured, NOT hard-coded from
/// the plan's recon guess (`{Bolt Bend, Misdirection, Untimely Malfunction}`).
/// For EACH roster member, also pins whether its own `TargetRequirement` is
/// one of the two single-target variants -- this is what makes §3.5's "the
/// all-or-nothing clause is unreachable" a MEASUREMENT, not a claim.
/// Non-vacuity floor in the same test (PB-DX24 R2 lesson).
#[test]
fn r1_must_change_true_roster_and_single_target_claim_are_pinned() {
    let cards = all_cards();
    assert!(
        cards.len() >= 1_700,
        "PB-DX25c R1 non-vacuity: all_cards() must return at least 1,700 \
         defs, got {}",
        cards.len()
    );

    let roster: BTreeSet<String> = cards
        .iter()
        .filter(|d| {
            let debug = sanitized_debug(d);
            debug.contains("must_change: true")
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
        "PB-DX25c R1 (must_change: true roster) moved -- expected {} names, \
         got {}: {roster:?}. A new member widens the population §3.5's \
         reachability claim depends on -- re-derive it.",
        expected.len(),
        roster.len()
    );

    // For each roster member, confirm its OWN TargetRequirement is single-
    // target-shaped (TargetSpellWithSingleTarget /
    // TargetSpellOrAbilityWithSingleTarget) -- the property that makes CR
    // 115.7a's all-or-nothing clause (§3.5) unreachable for n=1.
    //
    // **Residual, stated rather than glossed (`/review` Finding T11)**: this
    // check greps the WHOLE DEF's sanitized Debug, not the specific ability
    // that carries `must_change: true`. A future def with `must_change: true`
    // on ONE ability and an unrelated single-target requirement on a
    // DIFFERENT ability would satisfy this assertion vacuously -- the needle
    // would find the single-target string somewhere in the def without it
    // governing the ChangeTargets-carrying ability at all. This is a
    // DEF-LEVEL approximation, not an ability-scoped proof; today's three
    // roster members each have exactly one relevant ability, so the
    // approximation and the precise property coincide, but a def added later
    // with multiple abilities is not distinguished from one that is
    // genuinely single-target-shaped.
    for name in &roster {
        let def = cards.iter().find(|d| &d.name == name).unwrap();
        let debug = sanitized_debug(def);
        assert!(
            debug.contains("TargetSpellWithSingleTarget")
                || debug.contains("TargetSpellOrAbilityWithSingleTarget"),
            "PB-DX25c R1: roster member {name:?} carries must_change: true \
             but its OWN TargetRequirement is not single-target-shaped -- \
             §3.5's 'all-or-nothing is unreachable because n==1' claim is \
             now FALSE for this def; re-derive OOS-DX25c-1's reachability."
        );
    }
}

// ── R2: the 115.7b/115.7c population is EMPTY, with a liveness control ─────

/// R2 (plan §5.4). **Renamed by `/review` (Finding T4): the old name
/// (`r2_115_7b_115_7c_population_is_empty_with_a_control_and_deflecting_
/// swat_is_pinned`) claimed a CR 115.7b/115.7c *population* pin the body
/// never asserted over the CORPUS** -- it asserted walker liveness plus the
/// `must_change: false` roster, neither of which measures 115.7b/115.7c at
/// all. This version does two things instead: (1) what the old name promised
/// but the old body didn't check -- a source-level assertion that the
/// `Effect` enum (`card_definition.rs`) declares NO variant shaped for CR
/// 115.7b ("change A target") or CR 115.7c ("change ANY targets"), i.e. the
/// DSL genuinely has no representation for either, not merely that no CORPUS
/// def happens to use one; (2) the corpus-side checks the old body actually
/// ran: a walker-liveness control (PB-DX25b R3 lesson: an empty pin needs a
/// control, not just a corpus floor) and `deflecting_swat` pinned as the sole
/// `must_change: false` user, restating `OOS-DX25b-4`: membership here does
/// NOT mean the card gains anything -- `must_change: false` is a
/// deterministic no-op by construction (§3.3), unchanged by this batch.
#[test]
fn r2_effect_enum_has_no_115_7b_115_7c_variant_and_must_change_false_roster_is_pinned() {
    // ── (1) source-level: the Effect enum itself has no 115.7b/115.7c shape ──
    let card_def_src = strip_comments(&read_source("../card-types/src/cards/card_definition.rs"));
    let enum_marker = "pub enum Effect {";
    let enum_start = card_def_src
        .find(enum_marker)
        .unwrap_or_else(|| panic!("`{enum_marker}` not found in card_definition.rs"));
    let open_brace = enum_start + enum_marker.len() - 1;
    let enum_body = balanced_body(&card_def_src, open_brace, enum_marker);
    assert!(
        enum_body.contains("ChangeTargets"),
        "PB-DX25c R2 non-vacuity: the extracted Effect enum body must contain \
         the ChangeTargets variant it is supposed to be scanning -- \
         extraction is broken"
    );
    for needle in ["ChangeSomeTargets", "ChangeATarget", "ChangeAnyTargets"] {
        assert!(
            !enum_body.contains(needle),
            "PB-DX25c R2: the Effect enum must declare NO variant named \
             {needle:?} -- if one now exists, CR 115.7b/115.7c has gained a \
             DSL shape and this test's whole premise (no representation \
             exists) is stale; update it to cover the new variant instead of \
             asserting its absence."
        );
    }

    // ── (2) corpus-side: walker liveness + the must_change: false roster ──
    let cards = all_cards();

    // Liveness control: the SAME walker mechanism must find a non-empty set
    // for a common needle.
    let control: BTreeSet<String> = cards
        .iter()
        .filter(|d| sanitized_debug(d).contains("must_change"))
        .map(|d| d.name.clone())
        .collect();
    assert!(
        !control.is_empty(),
        "PB-DX25c R2 walker-liveness control: 'must_change' must be found on \
         at least one corpus def by this walker -- an empty result means the \
         walk itself is broken"
    );

    // The DSL encodes CR 115.7a (must_change: true) and CR 115.7d
    // (must_change: false) only -- there is no `ChangeSomeTargets { count }`
    // or similar CR 115.7b/115.7c shape at all (confirmed above at the
    // source level), so a corpus text-needle scan for one would be
    // meaningless; what CAN be measured on the corpus is that
    // `must_change: false` has exactly the one known user, and that this
    // batch changes nothing for it.
    let must_change_false: BTreeSet<String> = cards
        .iter()
        .filter(|d| sanitized_debug(d).contains("must_change: false"))
        .map(|d| d.name.clone())
        .collect();
    let expected: BTreeSet<String> = ["Deflecting Swat"].into_iter().map(String::from).collect();
    assert_eq!(
        must_change_false,
        expected,
        "PB-DX25c R2 (must_change: false roster, CR 115.7d) moved -- expected \
         {} names, got {}: {must_change_false:?}. OOS-DX25b-4 stays open: \
         membership here does not mean the card gains anything, must_change: \
         false is a deterministic no-op by construction (§3.3).",
        expected.len(),
        must_change_false.len()
    );
}

// ── R3: the population gate -- every StackObject literal that writes
//        `.targets` also writes `target_requirements` ────────────────────

/// R3 (plan §5.4): scan `crates/engine/src` (comment-stripped) for every
/// `StackObject {` literal; assert each one that mentions `targets:` (a
/// non-empty announcement) also mentions `target_requirements:` in the SAME
/// literal.
///
/// **Residuals, stated honestly rather than overclaimed (`/review` Finding
/// T5) -- TWO of them, not one:**
///
/// 1. **This test is MOSTLY redundant with rustc, and that is worth saying
///    plainly rather than presenting it as this gate's main protection.** A
///    `StackObject { .. }` literal that omits `target_requirements` entirely
///    does not COMPILE unless it uses a `..base` spread (every field is
///    required otherwise) -- so for a literal with NO spread, rustc itself
///    already forces the field to be written, and this gate adds nothing.
///    The ONE shape this gate adds anything over the compiler for is a
///    literal that DOES use a `..spread` (e.g. `..blank_stack_object()`)
///    while setting `targets:` explicitly and relying on the spread's
///    DEFAULT for `target_requirements` -- that compiles fine, silently
///    inheriting whatever the spread source's `target_requirements` happens
///    to be (typically empty), and is exactly the configuration this gate
///    exists to catch. It cannot prove the recorded list is the RIGHT one
///    either way (a site could write `target_requirements: vec![]` explicitly
///    next to a non-empty `targets:` and this gate would not object -- that
///    is what §3.4's fail-closed guard and T9c exist to make merely a LOST
///    feature, never a wrong one).
/// 2. **This gate scans `crates/engine/src` only.** A production `StackObject
///    { .. }` literal in `crates/simulator`, `crates/view-model`, or
///    `tools/` is invisible to it. Measured, not assumed: none exist today
///    (every production site outside `crates/engine/src` goes through
///    `StackObject::trigger_default`, a function, not a literal -- its BODY
///    is inside the swept directory and IS covered).
///
/// It also cannot see a literal built through `..Default::default()` with
/// `targets` set via a later `.targets = ` assignment outside the literal --
/// P2's 9 sites (§3.1) are checked separately by inspection, not by this
/// gate, and are named in this test's own doc so a reviewer can re-verify by
/// hand.
#[test]
fn r3_stack_object_literals_pair_targets_with_target_requirements() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = all_rs_files_under(&src_dir);
    assert!(
        files.len() >= 40,
        "PB-DX25c R3 non-vacuity: expected at least 40 .rs files under \
         crates/engine/src/, got {}",
        files.len()
    );

    let mut offending: Vec<String> = Vec::new();
    let mut literals_scanned = 0usize;
    for path in &files {
        let relative = path
            .strip_prefix(&src_dir)
            .unwrap()
            .to_string_lossy()
            .to_string();
        let raw = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let stripped = strip_comments(&raw);
        let mut search_from = 0usize;
        while let Some(rel_pos) = stripped[search_from..].find("StackObject {") {
            let marker_pos = search_from + rel_pos;
            let open_brace = marker_pos + "StackObject {".len() - 1;
            let body = balanced_body(&stripped, open_brace, &relative);
            literals_scanned += 1;
            // `/review` Finding T5: the outer `body.contains("targets:") &&`
            // this used to be gated behind was redundant -- `has_targets`
            // already contains that exact conjunct.
            let has_targets = body.contains("targets:") && !body.contains("target_requirements:");
            if has_targets {
                offending.push(format!(
                    "{relative} (StackObject literal at byte {marker_pos})"
                ));
            }
            search_from = open_brace + 1;
        }
    }

    assert!(
        literals_scanned >= 1,
        "PB-DX25c R3 non-vacuity: at least one StackObject {{ }} literal must \
         be found in crates/engine/src -- got {literals_scanned}"
    );
    assert!(
        offending.is_empty(),
        "PB-DX25c R3: every StackObject {{ }} literal that sets `targets:` \
         must also set `target_requirements:` in the same literal -- found: \
         {offending:?}"
    );
}

// ── R4: the ChangeTargets arm contains no second decision ──────────────────

/// R4 (plan §5.4): over `effects/mod.rs`'s `Effect::ChangeTargets` arm body
/// (comment-stripped): (a) >= 1 occurrence of `retarget::plan_target_change`;
/// (b) ZERO occurrences of `state.objects`, `.objects.iter()`,
/// `state.players`, `has_lost`, `candidates.sort()` -- the shapes the
/// pre-PB-DX25c open-coded scan used; (c) a re-measured size floor.
///
/// **Residual, stated honestly**: this gate sees ONLY the arm it names. A
/// future author who re-open-codes a candidate scan inside a DIFFERENT arm
/// (or a different file entirely) is invisible to it.
#[test]
fn r4_change_targets_arm_contains_no_second_decision() {
    let stripped = strip_comments(&read_source(EFFECTS_MOD_PATH));

    // `/review` Finding T6: `extract_match_arm_body` anchors on the FIRST
    // textual occurrence of the marker -- a future non-comment
    // `matches!(e, Effect::ChangeTargets { .. })` earlier in this file would
    // silently retarget the gate at a different (and misleading) arm body.
    // It would fail CLOSED (plan_calls would read 0 against the wrong body)
    // but with a message that blames the wrong cause. Guard the anchor
    // itself: the marker must occur EXACTLY once in the stripped source, so
    // "first occurrence" and "the only occurrence" are provably the same
    // thing here.
    let marker = "Effect::ChangeTargets {";
    let marker_count = stripped.matches(marker).count();
    assert_eq!(
        marker_count, 1,
        "PB-DX25c R4: `{marker:?}` must occur EXACTLY once in the \
         comment-stripped src/effects/mod.rs -- found {marker_count}. A \
         second occurrence (e.g. a `matches!(e, Effect::ChangeTargets {{ .. \
         }})` guard added earlier in the file) would make \
         extract_match_arm_body anchor on the WRONG arm body, and this gate \
         would then be measuring the wrong code without saying so."
    );

    let body = extract_match_arm_body(&stripped, marker);

    // Re-measured floor (plan §1 fact 14 / §9 checklist): PB-DX25b's own R4
    // floor (200 chars) was predicted AT RISK from this batch's shrink and
    // stayed green because it measures the WHOLE arm body (the `for` loop,
    // target resolution, and event construction, none of which shrank) --
    // not just the candidate-scan portion that moved into `rules::retarget`.
    // See `memory/primitives/pb-DX25c-execution-notes.md` for the measured
    // body length and the diagnosis.
    assert!(
        body.len() >= 400,
        "PB-DX25c R4 non-vacuity: the extracted Effect::ChangeTargets arm \
         body looks too small ({} chars) -- extraction may be broken",
        body.len()
    );

    let plan_calls = body.matches("retarget::plan_target_change").count();
    assert!(
        plan_calls >= 1,
        "PB-DX25c R4: the Effect::ChangeTargets arm must call \
         retarget::plan_target_change at least once, got {plan_calls}"
    );

    for needle in [
        "state.objects",
        ".objects.iter()",
        "state.players",
        "has_lost",
        "candidates.sort()",
    ] {
        assert_eq!(
            body.matches(needle).count(),
            0,
            "PB-DX25c R4: the Effect::ChangeTargets arm must contain ZERO \
             occurrences of {needle:?} -- the pre-PB-DX25c open-coded \
             candidate scan's own shape. A hit here means a second decision \
             has been re-introduced inline."
        );
    }
}

// ── R5: one TargetsChanged emitter ──────────────────────────────────────────

/// R5 (plan §5.4): `GameEvent::TargetsChanged` is constructed at exactly ONE
/// place in `crates/engine/src` (comment-stripped) -- distinguished from a
/// PATTERN-MATCH site (e.g. `state/hash.rs`'s per-variant hasher, which must
/// destructure every `GameEvent` variant including this one) by requiring
/// `push(` within a short backward window, matching the real emitter's own
/// shape (`events.push(GameEvent::TargetsChanged { .. })`). Measured: without
/// this distinction the naive text scan finds TWO occurrences
/// (`effects/mod.rs` and `state/hash.rs`), which would make this gate
/// permanently red for a reason that has nothing to do with a second
/// decision site.
///
/// **Residual, stated honestly (PB-DX25b's own R5 lesson -- its reviewer
/// defeated the original form three ways, and the accepted answer was
/// disclosure, not a bigger regex)**: this is the NARROWEST available
/// machine check on the census (§2.1). A second retarget decision that
/// mutates `StackObject.targets` WITHOUT emitting this event is invisible to
/// it; so is a construction that does not go through `.push(` directly (e.g.
/// built into a local variable first, then pushed on a later line) -- and
/// nothing else in this tree closes either residual.
#[test]
fn r5_targets_changed_has_one_emitter() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = all_rs_files_under(&src_dir);
    assert!(
        files.len() >= 40,
        "PB-DX25c R5 non-vacuity: expected at least 40 .rs files, got {}",
        files.len()
    );

    // Distinguish CONSTRUCTION sites (`events.push(GameEvent::TargetsChanged
    // { .. })`, building a new instance) from PATTERN-MATCH sites
    // (`GameEvent::TargetsChanged { .. } => { .. }` inside a `match`, e.g.
    // `state/hash.rs`'s per-variant hasher, which must destructure EVERY
    // event variant and is not a second emitter). A construction site has
    // `push(` within a short backward window; a match arm does not.
    const BACK_WINDOW: usize = 40;
    let mut sites: Vec<String> = Vec::new();
    for path in &files {
        let relative = path
            .strip_prefix(&src_dir)
            .unwrap()
            .to_string_lossy()
            .to_string();
        let raw = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let stripped = strip_comments(&raw);
        for (pos, _) in stripped.match_indices("GameEvent::TargetsChanged") {
            let ctx_lo = pos.saturating_sub(BACK_WINDOW);
            let ctx = &stripped[ctx_lo..pos];
            if ctx.contains("push(") {
                sites.push(relative.clone());
            }
        }
    }

    assert_eq!(
        sites.len(),
        1,
        "PB-DX25c R5: GameEvent::TargetsChanged must be constructed at \
         EXACTLY one place in crates/engine/src -- found {}: {sites:?}",
        sites.len()
    );
    assert_eq!(
        sites[0], "effects/mod.rs",
        "PB-DX25c R5: the one TargetsChanged emitter must be effects/mod.rs, \
         found it in {:?} instead",
        sites[0]
    );
}
