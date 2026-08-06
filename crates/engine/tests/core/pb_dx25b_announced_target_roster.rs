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

use mtg_engine::{
    all_cards, AbilityDefinition, CardDefinition, Completeness, Effect, TargetRequirement,
};
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

// ── R1: the requirement roster ──────────────────────────────────────────────

/// True if `req` is `TargetSpellWithSingleTarget` or
/// `TargetSpellOrAbilityWithSingleTarget`, including inside a `TargetRequirement::UpToN`.
fn is_single_target_spell_requirement(req: &TargetRequirement) -> bool {
    match req {
        TargetRequirement::TargetSpellWithSingleTarget
        | TargetRequirement::TargetSpellOrAbilityWithSingleTarget => true,
        TargetRequirement::UpToN { inner, .. } => is_single_target_spell_requirement(inner),
        _ => false,
    }
}

/// True if any `targets` list on `def` -- **either face**, walking `Spell`,
/// `Activated`, AND `Triggered` abilities (all three carry `targets` and
/// `modes`) -- declares `TargetSpellWithSingleTarget` or
/// `TargetSpellOrAbilityWithSingleTarget`, either in the flat non-modal
/// `targets` field or inside `ModeSelection.mode_targets` (the modal path,
/// PB-AC4).
fn has_single_target_spell_requirement(def: &CardDefinition) -> bool {
    let ability_has_it = |ability: &AbilityDefinition| -> bool {
        let (targets, modes): (&[TargetRequirement], &Option<_>) = match ability {
            AbilityDefinition::Spell { targets, modes, .. } => (targets, modes),
            AbilityDefinition::Activated { targets, modes, .. } => (targets, modes),
            AbilityDefinition::Triggered { targets, modes, .. } => (targets, modes),
            _ => return false,
        };
        targets.iter().any(is_single_target_spell_requirement)
            || modes.as_ref().is_some_and(|m| {
                m.mode_targets.as_ref().is_some_and(|mt| {
                    mt.iter()
                        .any(|slice| slice.iter().any(is_single_target_spell_requirement))
                })
            })
    };
    let face_has_it = |abilities: &[AbilityDefinition]| abilities.iter().any(ability_has_it);
    face_has_it(&def.abilities)
        || def
            .back_face
            .as_ref()
            .is_some_and(|f| face_has_it(&f.abilities))
}

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
        .filter(|d| has_single_target_spell_requirement(d))
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
    // HEAD (plan §2.4) -- Untimely Malfunction is `partial` for an unrelated
    // reason (mode 2's variable target count).
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

fn effect_contains_change_targets(effect: &Effect) -> bool {
    match effect {
        Effect::ChangeTargets { .. } => true,
        Effect::Sequence(effects) => effects.iter().any(effect_contains_change_targets),
        Effect::Conditional {
            if_true, if_false, ..
        } => effect_contains_change_targets(if_true) || effect_contains_change_targets(if_false),
        Effect::ForEach { effect, .. } => effect_contains_change_targets(effect),
        Effect::Choose { choices, .. } => choices.iter().any(effect_contains_change_targets),
        _ => false,
    }
}

fn ability_contains_change_targets(ability: &AbilityDefinition) -> bool {
    match ability {
        AbilityDefinition::Spell { effect, modes, .. }
        | AbilityDefinition::Activated { effect, modes, .. }
        | AbilityDefinition::Triggered { effect, modes, .. } => {
            effect_contains_change_targets(effect)
                || modes
                    .as_ref()
                    .is_some_and(|m| m.modes.iter().any(effect_contains_change_targets))
        }
        _ => false,
    }
}

/// R2 (plan §5.3): defs whose abilities -- either face -- contain
/// `Effect::ChangeTargets` anywhere (incl. inside a modal `ModeSelection`), by
/// NAME. Includes Deflecting Swat (`must_change: false`), which the dispatch
/// brief's site analysis missed (plan §0.4 F-A): it remains a documented
/// no-op after this batch (`must_change: false` -> `effects/mod.rs`'s
/// deterministic-fallback `continue`), so membership here does NOT mean
/// "works" for every row.
#[test]
fn r2_change_targets_roster_is_pinned() {
    let cards = all_cards();
    let roster: BTreeSet<String> = cards
        .iter()
        .filter(|d| {
            d.abilities.iter().any(ability_contains_change_targets)
                || d.back_face
                    .as_ref()
                    .is_some_and(|f| f.abilities.iter().any(ability_contains_change_targets))
        })
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

fn effect_contains_copy_spell_on_stack(effect: &Effect) -> bool {
    match effect {
        Effect::CopySpellOnStack { .. } => true,
        Effect::Sequence(effects) => effects.iter().any(effect_contains_copy_spell_on_stack),
        Effect::Conditional {
            if_true, if_false, ..
        } => {
            effect_contains_copy_spell_on_stack(if_true)
                || effect_contains_copy_spell_on_stack(if_false)
        }
        Effect::ForEach { effect, .. } => effect_contains_copy_spell_on_stack(effect),
        Effect::Choose { choices, .. } => choices.iter().any(effect_contains_copy_spell_on_stack),
        _ => false,
    }
}

fn effect_contains_draw_cards(effect: &Effect) -> bool {
    match effect {
        Effect::DrawCards { .. } => true,
        Effect::Sequence(effects) => effects.iter().any(effect_contains_draw_cards),
        Effect::Conditional {
            if_true, if_false, ..
        } => effect_contains_draw_cards(if_true) || effect_contains_draw_cards(if_false),
        Effect::ForEach { effect, .. } => effect_contains_draw_cards(effect),
        Effect::Choose { choices, .. } => choices.iter().any(effect_contains_draw_cards),
        _ => false,
    }
}

fn ability_contains<F: Fn(&Effect) -> bool>(ability: &AbilityDefinition, pred: &F) -> bool {
    match ability {
        AbilityDefinition::Spell { effect, modes, .. }
        | AbilityDefinition::Activated { effect, modes, .. }
        | AbilityDefinition::Triggered { effect, modes, .. } => {
            pred(effect) || modes.as_ref().is_some_and(|m| m.modes.iter().any(pred))
        }
        _ => false,
    }
}

fn def_contains<F: Fn(&Effect) -> bool>(def: &CardDefinition, pred: &F) -> bool {
    def.abilities.iter().any(|a| ability_contains(a, pred))
        || def
            .back_face
            .as_ref()
            .is_some_and(|f| f.abilities.iter().any(|a| ability_contains(a, pred)))
}

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
#[test]
fn r3_copy_spell_on_stack_roster_is_empty_with_liveness_control() {
    let cards = all_cards();

    let draw_cards_control: BTreeSet<String> = cards
        .iter()
        .filter(|d| def_contains(d, &effect_contains_draw_cards))
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
        .filter(|d| def_contains(d, &effect_contains_copy_spell_on_stack))
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

// ── R4: source gate over the two effects/mod.rs arms ────────────────────────

/// R4 (plan §5.3): after comment-stripping, the `Effect::ChangeTargets` and
/// `Effect::CopySpellOnStack` arm bodies must each (a) contain
/// `stack_index_for_announced_target` at least once, and (b) contain ZERO
/// occurrences of `stack_objects.iter()` / `stack_objects.iter_mut()`.
///
/// **Residual, stated honestly**: this gate sees only the two arms it names.
/// A brand-new arm elsewhere in `effects/mod.rs` that takes an announced id
/// and re-open-codes `stack_objects.iter().find(...)` is invisible to it --
/// exactly as PB-DX25's G2 was blind to `resolution.rs` until its review added
/// G4. R5 below is the closest thing to a wide net, and it too is scoped (see
/// its own doc).
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

// ── R5: the helper has no second implementation ─────────────────────────────

/// R5 (plan §5.3): scan `crates/engine/src/` (comment-stripped) for the
/// literal rule shape `card_in_stack_zone(` appearing in the same expression
/// as `so.id ==` / `s.id ==`, OUTSIDE `state/stack_registry.rs` -- the shape
/// `stack_index_for_announced_target`'s body itself has, and the shape a
/// future author re-open-coding the rule would reproduce. Assert zero.
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
        // Look for `card_in_stack_zone(` within 150 chars BEFORE an id-equality
        // comparison (`so.id ==` / `s.id ==`), where the intervening span ALSO
        // contains `||` and does NOT contain `;` -- i.e. the two are joined by
        // OR inside the SAME statement, the exact shape
        // `stack_index_for_announced_target`'s body has
        // (`so.id == announced || (!so.is_copy && card_in_stack_zone(&so.kind)
        // == Some(announced))`). Mere co-occurrence within a function (e.g.
        // `resolution.rs::counter_stack_object`, which legitimately does an
        // `so.id ==` lookup in one statement and a SEPARATE, later
        // `card_in_stack_zone(...)` classification call for the zone-move
        // decision, plan §2.3/§3.4 -- deliberately NOT unified) must NOT flag.
        for window_start in stripped
            .match_indices("card_in_stack_zone(")
            .map(|(i, _)| i)
        {
            let ctx_start = window_start.saturating_sub(150);
            let before = &stripped[ctx_start..window_start];
            let has_id_eq = before.contains("so.id ==") || before.contains("s.id ==");
            let has_or = before.contains("||");
            let same_statement = !before.contains(';');
            if has_id_eq && has_or && same_statement {
                offending.push(format!("{relative} (byte offset {window_start})"));
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
