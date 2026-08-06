//! PB-DX25 gates (plan §6 File B, extended in the review fix cycle): G1 (the
//! registry's no-wildcard classification), G2 + G4 (source gates over the two
//! consumers — `Effect::CounterSpell` in `effects/mod.rs` and
//! `resolution::counter_stack_object` respectively — proving neither
//! re-classifies by kind), and G3 (the SR-36 corpus roster gate, plan §5 —
//! enumerate `all_cards()`, never grep the corpus).
//!
//! All source gates strip **line and block** comments before scanning — the
//! PB-DX32 M8 lesson (also applied by PB-DX24's own gates in this same
//! directory): a `/* ... */`-wrapped line defeats a line-comment-only scanner
//! while every probe stays green, because the compiler drops the commented-out
//! code and the scanner never sees it disappear.
//!
//! **Where that stripping is actually load-bearing, stated PER GATE (review
//! Finding 8(a), fix cycle — the previous version of this doc claimed a single
//! blanket property that does not hold for G1):**
//!
//! - **G1** — stripping is DEFENCE-IN-DEPTH, not load-bearing. A `/*
//!   */`-wrapped wildcard arm cannot be a live catch-all: `card_in_stack_zone`'s
//!   `match` is exhaustive with no `_` arm anywhere else, so commenting out one
//!   real arm without replacing it fails to COMPILE — `rustc`'s own
//!   exhaustiveness check catches it before this gate ever runs. Stripping here
//!   only protects against a *false negative in the scanner itself* (a real,
//!   uncommented wildcard sitting textually near a real comment) — the inverse
//!   sanity check `g1_line_comment_stripping_does_not_hide_the_wildcard_it_is_
//!   meant_to_find` tests exactly that, on a synthetic fixture, not on the
//!   gated file.
//! - **G2 and G4** — stripping IS load-bearing. Both scan the BODY of a
//!   specific region (`Effect::CounterSpell`'s match arm; `counter_stack_object`'s
//!   whole function) for forbidden literals and required call counts inside
//!   ordinary (non-exhaustive) code, where the compiler enforces nothing about
//!   comment content. An unstripped comment mentioning `card_in_stack_zone`
//!   (e.g. the arm's own explanatory prose) would inflate the call count and
//!   let a comment satisfy a code gate; an unstripped `/* StackObjectKind::
//!   Spell { .. } => ... */` would not trip the forbidden-literal check if
//!   comments were counted as code. This file's own gates prove BOTH revert
//!   shapes (`//` and `/* */`) discriminate, by executing them (see
//!   `memory/primitives/pb-DX25-execution-notes.md`'s revert matrix) — not just
//!   the line-comment one.

use mtg_engine::{
    all_cards, AbilityDefinition, CardDefinition, Effect, KeywordAbility, TargetFilter,
    TargetRequirement,
};
use std::collections::BTreeSet;
use std::path::Path;

// ── Comment-stripping (mirrors core::decision_gate / pb_dx24_trigger_zone_roster's idiom) ──
//
// Robustness note (review, additional LOW notes): `strip_line_comments` finds
// the FIRST literal "//" on each line with no awareness of string/char
// literals -- a line containing `"https://example.com"` or similar would have
// everything after the first `//` treated as a comment and stripped. Neither
// `STACK_REGISTRY_PATH` nor `EFFECTS_MOD_PATH` nor `RESOLUTION_PATH` contains
// a `//` inside a string literal today, so this is a latent robustness limit,
// not a live bug -- a future edit to any gated file that introduces one (e.g.
// a URL in a doc comment reads fine since the WHOLE line after `//` is a
// comment already, but a URL inside an actual string LITERAL would not) could
// silently over-strip. Same limit is inherited from `pb_dx24_trigger_zone_
// roster.rs`'s identical idiom, not new here.

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
const RESOLUTION_PATH: &str = "src/rules/resolution.rs";

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

/// G2 (plan §3.5 / §6; RE-AIMED by PB-DX25b §8 R6 -- see that batch's own
/// execution notes for the deliberate-edit record and its own revert proof):
/// the `Effect::CounterSpell` arm in `effects/mod.rs` (a) calls
/// `card_in_stack_zone` at least once (the CR 701.6a zone-move classification,
/// `card_owned = crate::state::stack_registry::card_in_stack_zone(&stack_obj.kind)`),
/// (a2) calls `stack_index_for_announced_target` at least once (the
/// PB-DX25b-introduced shared LOOKUP, which itself calls `card_in_stack_zone`
/// internally -- inside `state/stack_registry.rs`, not here, which is why the
/// count in THIS arm dropped from >= 2 to >= 1 when PB-DX25b routed the lookup
/// through the helper), (a3) [PB-DX25b review Finding E4, closed here] contains
/// ZERO occurrences of `stack_objects.iter()`/`iter_mut()` -- the same
/// zero-raw-scan conjunct R4 (`pb_dx25b_announced_target_roster.rs`) holds the
/// two `effects/mod.rs` consumer arms to, now extended to this arm too, so a
/// "fast path" lookup added ALONGSIDE the helper call cannot slip past (a2)
/// unnoticed, (b) calls `fizzle_move_object_to_zone` exactly once,
/// (c) never spells out `StackObjectKind::Spell` or
/// `StackObjectKind::MutatingCreatureSpell` as a literal. Message: the
/// zone-move AND the announced-target lookup are each driven off
/// `state::stack_registry`, never off a per-kind match -- do not add an arm,
/// extend the registry.
#[test]
fn g2_counter_spell_arm_does_not_reclassify_by_kind() {
    let stripped = strip_comments(&read_source(EFFECTS_MOD_PATH));
    let body = extract_match_arm_body(&stripped, "Effect::CounterSpell {");

    let card_in_stack_zone_calls = body.matches("card_in_stack_zone").count();
    assert!(
        card_in_stack_zone_calls >= 1,
        "the zone-move is driven off state::stack_registry, never off a per-kind \
         match -- do not add an arm, extend the registry. Expected >= 1 call to \
         card_in_stack_zone (the CR 701.6a zone-move classification) in the \
         Effect::CounterSpell arm, got {}",
        card_in_stack_zone_calls
    );

    let stack_index_for_announced_target_calls =
        body.matches("stack_index_for_announced_target").count();
    assert!(
        stack_index_for_announced_target_calls >= 1,
        "PB-DX25b (`OOS-DX25-3`): the announced-target LOOKUP is driven off the \
         shared state::stack_registry::stack_index_for_announced_target helper, \
         never off a re-open-coded `so.id == id` scan. Expected >= 1 call in the \
         Effect::CounterSpell arm, got {}",
        stack_index_for_announced_target_calls
    );

    // PB-DX25b review Finding E4: unlike the two `effects/mod.rs` arms R4
    // (`pb_dx25b_announced_target_roster.rs`) covers, this arm had no
    // zero-`stack_objects.iter()`/`iter_mut()` conjunct -- a future edit could
    // add a "fast path" or second lookup ALONGSIDE the helper call above and
    // both prior conjuncts would stay green. Add the same conjunct R4 uses.
    let raw_iter_calls = body.matches("stack_objects.iter()").count()
        + body.matches("stack_objects.iter_mut()").count();
    assert_eq!(
        raw_iter_calls, 0,
        "PB-DX25b review Finding E4: the Effect::CounterSpell arm must contain \
         ZERO occurrences of stack_objects.iter()/iter_mut() -- any lookup must \
         go through stack_index_for_announced_target, not a re-open-coded scan \
         added alongside it. Got {}",
        raw_iter_calls
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
            && !body.contains("StackObjectKind::MutatingCreatureSpell {")
            // Hardened (review finding, PB-DX25 fix cycle): `stack_registry.rs`
            // itself imports `use StackObjectKind as K;` -- if the
            // Effect::CounterSpell arm were ever rewritten to alias the enum
            // the same way and match on `K::Spell {`/`K::MutatingCreatureSpell
            // {`, the fully-qualified literal check above would miss it. Proven
            // load-bearing by executing exactly that revert shape (see
            // `memory/primitives/pb-DX25-execution-notes.md`).
            && !body.contains("K::Spell {")
            && !body.contains("K::MutatingCreatureSpell {"),
        "the zone-move is driven off state::stack_registry, never off a per-kind \
         match -- do not add an arm, extend the registry. Found a literal \
         StackObjectKind::Spell or StackObjectKind::MutatingCreatureSpell (or \
         their `K::` alias form) inside the Effect::CounterSpell arm."
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

// ── G4: `counter_stack_object` does not re-classify by kind either ──────────────
//
// Added in the PB-DX25 review fix cycle. G2 gates ONLY `effects/mod.rs`'s
// `Effect::CounterSpell` arm; before this gate existed there was NO source
// gate at all over `resolution.rs::counter_stack_object` -- the plan's own
// acceptance-criterion-6232 mapping (Stage 6) conceded that its "single
// classification, driving BOTH counter paths" half rested on argument plus
// T7 alone, not on a machine. G4 is G2's exact shape, aimed at the second
// function.

/// G4: `counter_stack_object`'s body (a) calls `card_in_stack_zone` at least
/// once, (b) never spells out `StackObjectKind::Spell` or
/// `StackObjectKind::MutatingCreatureSpell` as a literal (including the `K::`
/// alias form `stack_registry.rs` itself uses). Message: the zone-move is
/// driven off `state::stack_registry`, never off a per-kind match -- do not
/// add an arm, extend the registry.
#[test]
fn g4_counter_stack_object_does_not_reclassify_by_kind() {
    let stripped = strip_comments(&read_source(RESOLUTION_PATH));
    let body = extract_function_body(&stripped, "counter_stack_object");

    let card_in_stack_zone_calls = body.matches("card_in_stack_zone").count();
    assert!(
        card_in_stack_zone_calls >= 1,
        "the zone-move is driven off state::stack_registry, never off a per-kind \
         match -- do not add an arm, extend the registry. Expected >= 1 call to \
         card_in_stack_zone in counter_stack_object, got {}",
        card_in_stack_zone_calls
    );

    let move_calls = body.matches("move_object_to_zone").count();
    assert_eq!(
        move_calls, 1,
        "counter_stack_object must call move_object_to_zone exactly once (CR \
         400.7 zone move on the card-owning branch) -- got {}",
        move_calls
    );

    assert!(
        !body.contains("StackObjectKind::Spell {")
            && !body.contains("StackObjectKind::MutatingCreatureSpell {")
            && !body.contains("K::Spell {")
            && !body.contains("K::MutatingCreatureSpell {"),
        "the zone-move is driven off state::stack_registry, never off a per-kind \
         match -- do not add an arm, extend the registry. Found a literal \
         StackObjectKind::Spell or StackObjectKind::MutatingCreatureSpell (or \
         their `K::` alias form) inside counter_stack_object. Note: the \
         ActivatedAbility/TriggeredAbility diagnostics-naming arm is NOT part \
         of the zone-move decision (it cannot lose a card, OOS-DX25-4) and is \
         exempt from this check by construction -- it names an ability's \
         SOURCE, not a card-owning kind."
    );
}

/// G4 non-vacuity: the extracted body must be non-trivially sized, so a
/// collapsed extraction cannot make G4 pass vacuously.
#[test]
fn g4_scan_is_not_vacuous() {
    let stripped = strip_comments(&read_source(RESOLUTION_PATH));
    let body = extract_function_body(&stripped, "counter_stack_object");
    assert!(
        body.len() >= 400,
        "the extracted counter_stack_object body looks too small ({} chars) to \
         contain the full lookup + zone-move logic -- extraction may be broken",
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

/// True if any `AbilityDefinition::Spell` on `def` -- **either face** (review
/// Finding 3's "smaller scoping gap": the original walk was front-face only,
/// asymmetric with `has_mutate_keyword`'s own back-face walk right above it) --
/// declares a spell-level target requirement: either a non-empty `targets`
/// (the flat, non-modal path) or a `mode_targets` entry that is itself
/// non-empty (the modal path, PB-AC4). This is the §2.2 "does a Ward on the
/// mutate target have anything to announce against" measurement -- it is
/// scoped to whether the spell declares ANY target, not specifically whether
/// that target is the mutate target (the mutate target itself is carried in
/// `AdditionalCost::Mutate` and is invisible to `spell_targets` entirely, per
/// plan §0.2 F1 / `OOS-DX25-1` -- out of scope here). Moot on the current
/// corpus (no `Complete` Mutate def has a back face at all), but the walk
/// itself is now symmetric with M1's rather than merely documented as
/// asymmetric.
fn has_spell_level_target_requirement(def: &CardDefinition) -> bool {
    let face_has_it = |abilities: &[AbilityDefinition]| {
        abilities.iter().any(|a| match a {
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
    };
    face_has_it(&def.abilities)
        || def
            .back_face
            .as_ref()
            .is_some_and(|f| face_has_it(&f.abilities))
}

/// Recursive walk (plan §5 C1: "anywhere, incl. inside Modal") over an
/// `Effect` tree for `Effect::CounterSpell` **and** `Effect::CounterUnlessPays`
/// (review Finding 3: `effects/mod.rs:4401-4411` delegates `CounterUnlessPays`
/// straight into the `CounterSpell` arm under repair -- CR 118.12a -- so every
/// `CounterUnlessPays` def was equally live-wrong against a mutate spell and
/// this walk must see it or the roster undercounts). Recurses into every
/// `Effect`-nesting variant: `Sequence`, `Conditional` (both branches),
/// `ForEach`, and `Choose` (the `Effect`-level modal stub, SR-33 -- distinct
/// from `AbilityDefinition::Spell.modes`, which is walked separately by the
/// caller since it is a sibling field, not a nested `Effect`).
fn effect_contains_counter_spell(effect: &Effect) -> bool {
    match effect {
        Effect::CounterSpell { .. } => true,
        // CR 118.12a: "Counter target spell unless its controller pays [cost]."
        // `effects/mod.rs` resolves this deterministically straight through
        // `Effect::CounterSpell`'s own arm -- same zone-move, same stack-object
        // classification -- so it is the identical live-wrong class, not a
        // different one.
        Effect::CounterUnlessPays { .. } => true,
        Effect::Sequence(effects) => effects.iter().any(effect_contains_counter_spell),
        Effect::Conditional {
            if_true, if_false, ..
        } => effect_contains_counter_spell(if_true) || effect_contains_counter_spell(if_false),
        Effect::ForEach { effect, .. } => effect_contains_counter_spell(effect),
        Effect::Choose { choices, .. } => choices.iter().any(effect_contains_counter_spell),
        _ => false,
    }
}

/// True if ANY ability on `def` -- `Spell`, `Activated`, or `Triggered` --
/// contains `Effect::CounterSpell`/`Effect::CounterUnlessPays` anywhere in its
/// top-level effect or any of its modal modes (`ModeSelection.modes`, CR
/// 700.2). Review Finding 3 (scoping gap): the original walk saw only
/// `AbilityDefinition::Spell`, so a counter on an activated or triggered
/// ability was invisible to this roster. Measured: no corpus def currently
/// puts a counter effect on an `Activated`/`Triggered` ability (C1 is unchanged
/// by this widening), but the walk must be able to see one, not merely fail to
/// find one today.
fn ability_contains_counter_spell(ability: &AbilityDefinition) -> bool {
    match ability {
        AbilityDefinition::Spell { effect, modes, .. }
        | AbilityDefinition::Activated { effect, modes, .. }
        | AbilityDefinition::Triggered { effect, modes, .. } => {
            effect_contains_counter_spell(effect)
                || modes
                    .as_ref()
                    .is_some_and(|m| m.modes.iter().any(effect_contains_counter_spell))
        }
        _ => false,
    }
}

/// C1 (plan §5): defs whose abilities -- **either face** (review Finding 3's
/// "smaller scoping gap": the original walk was front-face only) -- contain
/// `Effect::CounterSpell`/`Effect::CounterUnlessPays` anywhere (incl. inside a
/// modal `ModeSelection`), by card name. Moot on the current corpus (no
/// `Complete` counter-carrying def has a back face), but the walk is now
/// symmetric with M1's rather than merely documented as asymmetric.
fn counterspell_defs(cards: &[CardDefinition]) -> BTreeSet<String> {
    cards
        .iter()
        .filter(|d| {
            d.abilities.iter().any(ability_contains_counter_spell)
                || d.back_face
                    .as_ref()
                    .is_some_and(|f| f.abilities.iter().any(ability_contains_counter_spell))
        })
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

/// C3 (plan §5, WIDENED by review Finding 3): C2 (C1 ∩ `is_complete()`) whose
/// counter target requirement is UNRESTRICTED. Two syntactic shapes count as
/// unrestricted, not one:
///
/// 1. The bare `TargetRequirement::TargetSpell` (the plan's original pin).
/// 2. `TargetRequirement::TargetSpellWithFilter(f)` where `f ==
///    TargetFilter::default()`. Review Finding 3's argument, verified here by
///    reading `TargetFilter`'s own field defaults
///    (`card_definition.rs:3036-3080`): every field is `None`/`false`/
///    `TargetController::Any` (the `#[default]` variant, `:3244`), and
///    `casting.rs:6430-6453`'s `matches_filter` check is a pure AND over those
///    fields -- an all-default filter accepts every legal spell, exactly like
///    the bare `TargetSpell` variant. `mana_leak`, `mana_tithe`, and
///    `make_disappear` (all `Complete`, all `CounterUnlessPays`) declare
///    `TargetSpellWithFilter(TargetFilter::default())` for "counter target
///    spell" with no oracle-text restriction -- syntactically different from
///    `TargetSpell`, semantically identical, and this pin now measures the
///    semantic class rather than the one literal spelling of it.
///
/// Deliberately still excludes a filter that is non-default but happens to
/// admit a creature spell (`red_elemental_blast`'s blue filter) -- that
/// remains a separate, unevaluated note (no `matches_filter` run against a
/// synthetic creature-spell `Characteristics`), exactly as the plan scoped
/// C3's syntactic-only measurement.
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
            match counter_target_requirement(def) {
                Some(TargetRequirement::TargetSpell) => true,
                Some(TargetRequirement::TargetSpellWithFilter(f)) => f == TargetFilter::default(),
                _ => false,
            }
        })
        .collect()
}

/// G3 (plan §5 / §6): the SR-36 corpus roster, pinned by NAME where the
/// population is small, with the `all_cards().len() >= 1_700` non-vacuity
/// floor asserted in the SAME test (the PB-DX24 R2 lesson: a broken
/// enumeration must not make an empty roster look correct). Message names
/// `OOS-SIM3-5` and tells a future author that a new mutate def, a new
/// unrestricted `Effect::CounterSpell` def, OR a new unrestricted
/// `Effect::CounterUnlessPays` def widens the class -- review Finding 3
/// corrected this gate to actually see the third case (it was previously
/// blind to `CounterUnlessPays` and its own message claimed a class it
/// could not see).
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

    // C1: defs carrying Effect::CounterSpell OR Effect::CounterUnlessPays
    // anywhere (incl. inside Modal). Review Finding 3: the first-runner
    // measurement (23) was itself an undercount -- it was blind to
    // Effect::CounterUnlessPays, which effects/mod.rs:4401-4411 (CR 118.12a)
    // delegates straight into the Effect::CounterSpell arm this batch
    // repaired, making every CounterUnlessPays def equally live-wrong. The
    // widened walk adds exactly 6: Flusterstorm, Izzet Charm, Make Disappear,
    // Mana Leak, Mana Tithe, Spell Pierce.
    let c1 = counterspell_defs(&cards);
    assert_eq!(
        c1.len(),
        29,
        "OOS-SIM3-5 roster C1 (defs carrying Effect::CounterSpell or \
         Effect::CounterUnlessPays anywhere) moved from the MEASURED 29 -- \
         got {}: {c1:?}. (The original 23-count was blind to \
         Effect::CounterUnlessPays, which delegates into the same arm this \
         batch repaired -- review Finding 3. Separately, the plan's own §0.3 \
         grep estimate of 24 for the CounterSpell-only subset was itself \
         wrong: it substring-matched the literal text \"Effect::CounterSpell\" \
         inside a TODO *comment* on Transcendent Dragon, which has no such \
         effect in code -- an SR-36 example of exactly the failure this \
         enumeration replaces.)",
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
        24,
        "OOS-SIM3-5 roster C2 (Complete counter defs) moved from the MEASURED \
         24 (18 CounterSpell + 6 CounterUnlessPays, all 6 Complete -- review \
         Finding 3) -- got {}: {c2:?}",
        c2.len()
    );

    // C3: C2 whose counter target requirement is UNRESTRICTED -- the bare
    // TargetRequirement::TargetSpell OR TargetSpellWithFilter(TargetFilter::
    // default()) (review Finding 3: mana_leak/mana_tithe/make_disappear all
    // declare the latter, all Complete, all semantically "counter target
    // spell" with no restriction). TargetSpellWithFilter admitting a creature
    // spell through a NON-default filter (e.g. Red Elemental Blast's blue
    // filter) is a separate note, not folded into this pin -- see the plan §5
    // note.
    let c3 = unrestricted_target_spell_defs(&cards);
    assert_eq!(
        c3.len(),
        11,
        "OOS-SIM3-5 roster C3 (Complete counter defs with an unrestricted \
         target requirement) moved from the MEASURED 11 (8 bare TargetSpell + \
         3 TargetSpellWithFilter(TargetFilter::default()) -- review Finding 3) \
         -- got {}: {c3:?}",
        c3.len()
    );

    // P: measured live-wrong pairs = |M2| x |C3|. The queue row's "6 x 24 =
    // 144" and the plan's own "~48" estimate (itself an undercount -- review
    // Finding 3, blind to Effect::CounterUnlessPays) are both superseded by
    // this measured number -- report it, do not hand-edit the queue rows here
    // (a later runner corrects seed-rerank-2026-08-02.md and
    // decision-point-audit.md's OOS-SIM3-5 row).
    let p = m2.len() * c3.len();
    assert_eq!(
        p,
        66,
        "OOS-SIM3-5 roster P (live-wrong pairs = |M2| x |C3|) moved -- expected \
         66 (6 x 11), got {p} ({} x {}). Correct the queue row and the seed row \
         with this measured number.",
        m2.len(),
        c3.len()
    );
}
