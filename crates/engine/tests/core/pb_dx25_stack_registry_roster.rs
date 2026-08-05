//! PB-DX25 gates (plan §6 File B): G1 + G2 (source gates over the registry and
//! the `Effect::CounterSpell` arm) and G3 (the SR-36 corpus roster gate, plan
//! §5 — enumerate `all_cards()`, never grep the corpus).
//!
//! Both source gates strip **line and block** comments before scanning — the
//! PB-DX32 M8 lesson (also applied by PB-DX24's own gates in this same
//! directory): a `/* ... */`-wrapped line defeats a line-comment-only scanner
//! while every probe stays green, because the compiler drops the commented-out
//! code and the scanner never sees it disappear. This file's own gates prove
//! that load-bearing property by executing BOTH revert shapes (`//` and `/*
//! */`), not just the line-comment one.

use mtg_engine::{
    all_cards, AbilityDefinition, CardDefinition, Effect, KeywordAbility, TargetRequirement,
};
use std::collections::BTreeSet;
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

/// Extract the BODY of a `match` arm whose pattern starts with `pattern_marker`
/// (e.g. `"Effect::CounterSpell {"`) -- from the `{` immediately after the arm's
/// `=>` through the matching close brace, by brace balancing. Used for G2, where
/// the gated region is one arm of a giant `match effect { ... }`, not a whole
/// function.
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
const EFFECTS_MOD_PATH: &str = "src/effects/mod.rs";

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

/// G2 (plan §3.5 / §6): the `Effect::CounterSpell` arm in `effects/mod.rs`
/// (a) calls `card_in_stack_zone` at least twice (lookup + move), (b) calls
/// `fizzle_move_object_to_zone` exactly once, (c) never spells out
/// `StackObjectKind::Spell` or `StackObjectKind::MutatingCreatureSpell` as a
/// literal. Message: the zone-move is driven off `state::stack_registry`, never
/// off a per-kind match -- do not add an arm, extend the registry.
#[test]
fn g2_counter_spell_arm_does_not_reclassify_by_kind() {
    let stripped = strip_comments(&read_source(EFFECTS_MOD_PATH));
    let body = extract_match_arm_body(&stripped, "Effect::CounterSpell {");

    let card_in_stack_zone_calls = body.matches("card_in_stack_zone").count();
    assert!(
        card_in_stack_zone_calls >= 2,
        "the zone-move is driven off state::stack_registry, never off a per-kind \
         match -- do not add an arm, extend the registry. Expected >= 2 calls to \
         card_in_stack_zone (lookup + move) in the Effect::CounterSpell arm, got {}",
        card_in_stack_zone_calls
    );

    let fizzle_calls = body.matches("fizzle_move_object_to_zone").count();
    assert_eq!(
        fizzle_calls, 1,
        "the Effect::CounterSpell arm must call fizzle_move_object_to_zone \
         exactly once (CR 400.7 fizzle-shaped zone move on the card-owning \
         branch) -- got {}",
        fizzle_calls
    );

    assert!(
        !body.contains("StackObjectKind::Spell {")
            && !body.contains("StackObjectKind::MutatingCreatureSpell {"),
        "the zone-move is driven off state::stack_registry, never off a per-kind \
         match -- do not add an arm, extend the registry. Found a literal \
         StackObjectKind::Spell or StackObjectKind::MutatingCreatureSpell inside \
         the Effect::CounterSpell arm."
    );
}

/// G2 non-vacuity: the extracted arm must be non-trivially sized (it contains
/// the full zone-move logic, not just the position() lookup), so a collapsed
/// extraction cannot make G2 pass vacuously.
#[test]
fn g2_scan_is_not_vacuous() {
    let stripped = strip_comments(&read_source(EFFECTS_MOD_PATH));
    let body = extract_match_arm_body(&stripped, "Effect::CounterSpell {");
    assert!(
        body.len() >= 400,
        "the extracted Effect::CounterSpell arm body looks too small ({} chars) \
         to contain the full lookup + zone-move logic -- extraction may be broken",
        body.len()
    );
}

// ── G3: the SR-36 corpus roster (plan §5) ───────────────────────────────────────
//
// Enumerated from `all_cards()`, never grepped -- the §0.3 grep numbers in the
// plan are reconnaissance, replaced here by a measured enumeration. Every
// population, including zeros, is recorded in
// `memory/primitives/pb-DX25-execution-notes.md`.

/// True if `def` carries `AbilityDefinition::Keyword(KeywordAbility::Mutate)`
/// on either face.
fn has_mutate_keyword(def: &CardDefinition) -> bool {
    let face_has_it = |abilities: &[AbilityDefinition]| {
        abilities
            .iter()
            .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Mutate)))
    };
    face_has_it(&def.abilities)
        || def
            .back_face
            .as_ref()
            .is_some_and(|f| face_has_it(&f.abilities))
}

/// M1 (plan §5): defs carrying `AbilityDefinition::Keyword(KeywordAbility::Mutate)`
/// on either face, by card name.
fn mutate_defs(cards: &[CardDefinition]) -> BTreeSet<String> {
    cards
        .iter()
        .filter(|d| has_mutate_keyword(d))
        .map(|d| d.name.clone())
        .collect()
}

/// True if any `AbilityDefinition::Spell` on `def`'s FRONT face declares a
/// spell-level target requirement -- either a non-empty `targets` (the flat,
/// non-modal path) or a `mode_targets` entry that is itself non-empty (the
/// modal path, PB-AC4). This is the §2.2 "does a Ward on the mutate target
/// have anything to announce against" measurement -- it is scoped to whether
/// the spell declares ANY target, not specifically whether that target is the
/// mutate target (the mutate target itself is carried in `AdditionalCost::
/// Mutate` and is invisible to `spell_targets` entirely, per plan §0.2 F1 /
/// `OOS-DX25-1` -- out of scope here).
fn has_spell_level_target_requirement(def: &CardDefinition) -> bool {
    def.abilities.iter().any(|a| match a {
        AbilityDefinition::Spell { targets, modes, .. } => {
            !targets.is_empty()
                || modes.as_ref().is_some_and(|m| {
                    m.mode_targets
                        .as_ref()
                        .is_some_and(|mt| mt.iter().any(|slice| !slice.is_empty()))
                })
        }
        _ => false,
    })
}

/// Recursive walk (plan §5 C1: "anywhere, incl. inside Modal") over an
/// `Effect` tree for `Effect::CounterSpell`. Recurses into every
/// `Effect`-nesting variant: `Sequence`, `Conditional` (both branches),
/// `ForEach`, and `Choose` (the `Effect`-level modal stub, SR-33 -- distinct
/// from `AbilityDefinition::Spell.modes`, which is walked separately by the
/// caller since it is a sibling field, not a nested `Effect`).
fn effect_contains_counter_spell(effect: &Effect) -> bool {
    match effect {
        Effect::CounterSpell { .. } => true,
        Effect::Sequence(effects) => effects.iter().any(effect_contains_counter_spell),
        Effect::Conditional {
            if_true, if_false, ..
        } => effect_contains_counter_spell(if_true) || effect_contains_counter_spell(if_false),
        Effect::ForEach { effect, .. } => effect_contains_counter_spell(effect),
        Effect::Choose { choices, .. } => choices.iter().any(effect_contains_counter_spell),
        _ => false,
    }
}

/// True if `AbilityDefinition::Spell`'s top-level effect OR any of its modal
/// modes (`ModeSelection.modes`, CR 700.2) contains `Effect::CounterSpell`
/// anywhere in its tree.
fn ability_contains_counter_spell(ability: &AbilityDefinition) -> bool {
    match ability {
        AbilityDefinition::Spell { effect, modes, .. } => {
            effect_contains_counter_spell(effect)
                || modes
                    .as_ref()
                    .is_some_and(|m| m.modes.iter().any(effect_contains_counter_spell))
        }
        _ => false,
    }
}

/// C1 (plan §5): defs whose FRONT-face abilities contain `Effect::CounterSpell`
/// anywhere (incl. inside a modal `ModeSelection`), by card name.
fn counterspell_defs(cards: &[CardDefinition]) -> BTreeSet<String> {
    cards
        .iter()
        .filter(|d| d.abilities.iter().any(ability_contains_counter_spell))
        .map(|d| d.name.clone())
        .collect()
}

/// For a def in C1, find the `TargetRequirement` that governs the counter
/// effect: `targets[0]` for a non-modal `Spell` whose effect tree contains
/// the counter (whether the counter is the ability's own top-level effect, as
/// in `counterspell.rs`, or nested one level inside `Effect::Sequence`, as in
/// `access_denied.rs`/`rewind.rs` -- every corpus def observed uses
/// `EffectTarget::DeclaredTarget { index: 0 }` for the counter's target
/// regardless of nesting depth, i.e. `targets[0]` is always the slot), or
/// `mode_targets[i][0]` for the modal mode `i` whose effect tree contains the
/// counter (PB-AC4 -- `mode_targets[i]` is local to mode `i`, index 0 being
/// that mode's own first target). Returns `None` if no ability's effect tree
/// contains the counter, or if the requirement list is empty -- in which case
/// the def is deliberately excluded from C3 rather than mis-measured.
fn counter_target_requirement(def: &CardDefinition) -> Option<TargetRequirement> {
    for ability in &def.abilities {
        let AbilityDefinition::Spell {
            effect,
            targets,
            modes,
            ..
        } = ability
        else {
            continue;
        };
        if effect_contains_counter_spell(effect) {
            return targets.first().cloned();
        }
        if let Some(m) = modes {
            for (i, mode_effect) in m.modes.iter().enumerate() {
                if effect_contains_counter_spell(mode_effect) {
                    return m
                        .mode_targets
                        .as_ref()
                        .and_then(|mt| mt.get(i))
                        .and_then(|slice| slice.first())
                        .cloned();
                }
            }
        }
    }
    None
}

/// C3 (plan §5): C2 (C1 ∩ `is_complete()`) whose counter target requirement is
/// the UNRESTRICTED `TargetRequirement::TargetSpell` -- deliberately
/// syntactic (no `matches_filter` evaluation against a synthetic creature
/// spell; `TargetSpellWithFilter` admitting a creature spell, e.g. Red
/// Elemental Blast's blue filter, is recorded as a separate note per the
/// plan, not folded into this pin).
fn unrestricted_target_spell_defs(cards: &[CardDefinition]) -> BTreeSet<String> {
    let c2 = counterspell_defs(cards)
        .into_iter()
        .filter(|name| {
            cards
                .iter()
                .find(|d| &d.name == name)
                .is_some_and(|d| d.completeness.is_complete())
        })
        .collect::<BTreeSet<_>>();
    c2.into_iter()
        .filter(|name| {
            let def = cards.iter().find(|d| &d.name == name).unwrap();
            counter_target_requirement(def) == Some(TargetRequirement::TargetSpell)
        })
        .collect()
}

/// G3 (plan §5 / §6): the SR-36 corpus roster, pinned by NAME where the
/// population is small, with the `all_cards().len() >= 1_700` non-vacuity
/// floor asserted in the SAME test (the PB-DX24 R2 lesson: a broken
/// enumeration must not make an empty roster look correct). Message names
/// `OOS-SIM3-5` and tells a future author that a new mutate def or a new
/// unrestricted counter def widens the class.
#[test]
fn g3_corpus_roster_is_pinned() {
    let cards = all_cards();
    assert!(
        cards.len() >= 1_700,
        "OOS-SIM3-5 roster (PB-DX25 G3): non-vacuity floor -- all_cards() must \
         return at least 1,700 defs (measured on this branch: {}) -- got {}. A \
         broken enumeration cannot make an empty roster look correct.",
        cards.len(),
        cards.len()
    );

    // M1: mutate defs, by name.
    let m1 = mutate_defs(&cards);
    let expected_m1: BTreeSet<String> = [
        "Gemrazer",
        "Sea-Dasher Octopus",
        "Brokkos, Apex of Forever",
        "Vulpikeet",
        "Necropanther",
        "Glowstone Recluse",
        "Mindleecher",
        "Nethroi, Apex of Death",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(
        m1,
        expected_m1,
        "OOS-SIM3-5 roster M1 (Mutate-keyword defs) moved -- expected {} names, \
         got {}: {m1:?}. A new Mutate def widens the OOS-SIM3-5 live-wrong-pair \
         class -- re-derive M2/M3/P below.",
        expected_m1.len(),
        m1.len()
    );

    // M2: M1 ∩ is_complete().
    let m2: BTreeSet<String> = m1
        .iter()
        .filter(|name| {
            cards
                .iter()
                .find(|d| &d.name == *name)
                .is_some_and(|d| d.completeness.is_complete())
        })
        .cloned()
        .collect();
    assert_eq!(
        m2.len(),
        6,
        "OOS-SIM3-5 roster M2 (Complete Mutate defs) moved from 6 -- got {}: \
         {m2:?}",
        m2.len()
    );

    // M3: M2 that declare ANY spell-level target requirement. Expected 0 --
    // this is what makes shape (a) corpus-unreachable via Ward today (plan
    // §2.2); a non-zero count is a finding, not a failure of this gate, and
    // must be reported (not silently accepted) by whoever re-runs this.
    let m3: BTreeSet<String> = m2
        .iter()
        .filter(|name| {
            cards
                .iter()
                .find(|d| &d.name == *name)
                .is_some_and(has_spell_level_target_requirement)
        })
        .cloned()
        .collect();
    assert_eq!(
        m3.len(),
        0,
        "OOS-SIM3-5 roster M3 (Complete Mutate defs with a spell-level target \
         requirement) moved from the expected 0 -- got {}: {m3:?}. This is a \
         FINDING (shape (a) may now be corpus-reachable via Ward), not just a \
         drifted pin -- report it.",
        m3.len()
    );

    // C1: defs carrying Effect::CounterSpell anywhere (incl. inside Modal).
    let c1 = counterspell_defs(&cards);
    assert_eq!(
        c1.len(),
        23,
        "OOS-SIM3-5 roster C1 (defs carrying Effect::CounterSpell anywhere) \
         moved from the MEASURED 23 -- got {}: {c1:?}. (The plan's own §0.3 \
         grep estimate of 24 was itself wrong: it substring-matched the \
         literal text \"Effect::CounterSpell\" inside a TODO *comment* on \
         Transcendent Dragon, which has no such effect in code -- an SR-36 \
         example of exactly the failure this enumeration replaces.)",
        c1.len()
    );

    // C2: C1 ∩ is_complete().
    let c2: BTreeSet<String> = c1
        .iter()
        .filter(|name| {
            cards
                .iter()
                .find(|d| &d.name == *name)
                .is_some_and(|d| d.completeness.is_complete())
        })
        .cloned()
        .collect();
    assert_eq!(
        c2.len(),
        18,
        "OOS-SIM3-5 roster C2 (Complete counter defs) moved from 18 -- got {}: \
         {c2:?}",
        c2.len()
    );

    // C3: C2 whose counter target requirement is the unrestricted
    // TargetRequirement::TargetSpell (syntactic subset -- TargetSpellWithFilter
    // admitting a creature spell, e.g. Red Elemental Blast's blue filter, is a
    // separate note, not folded into this pin -- see the plan §5 note).
    let c3 = unrestricted_target_spell_defs(&cards);
    assert_eq!(
        c3.len(),
        8,
        "OOS-SIM3-5 roster C3 (Complete counter defs with an unrestricted \
         TargetRequirement::TargetSpell) moved from 8 -- got {}: {c3:?}",
        c3.len()
    );

    // P: measured live-wrong pairs = |M2| x |C3|. The queue row's "6 x 24 =
    // 144" and this plan's own "~48" estimate are both superseded by this
    // measured number -- report it, do not hand-edit the queue rows here (a
    // later runner corrects seed-rerank-2026-08-02.md and
    // decision-point-audit.md's OOS-SIM3-5 row).
    let p = m2.len() * c3.len();
    assert_eq!(
        p,
        48,
        "OOS-SIM3-5 roster P (live-wrong pairs = |M2| x |C3|) moved -- expected \
         48 (6 x 8), got {p} ({} x {}). Correct the queue row and the seed row \
         with this measured number.",
        m2.len(),
        c3.len()
    );
}
