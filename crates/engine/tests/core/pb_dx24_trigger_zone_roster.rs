//! PB-DX24 gates (plan §2.6 G-A/G-B, §4.3 R1/R2), SR-36 -- enumerate `all_cards()`,
//! never grep the corpus.
//!
//! G-A/G-B are structural source gates over
//! `crates/engine/src/testing/replay_harness.rs`: the trigger-lowering function must
//! never see `trigger_zone` (the CR 113.6b/113.6m filter lives at its single call
//! site, `lowers_onto_the_battlefield`), and there must be exactly one such call
//! site, fed the filtered binding. R1/R2 pin the two corpus populations this batch's
//! probes are written against (§4.1/§4.2 of the plan).
//!
//! Both source gates strip **line and block** comments before scanning -- the
//! PB-DX32 M8 lesson: a `/* ... */`-wrapped row defeats a line-comment-only scanner
//! while every probe stays green, because the compiler drops the commented-out code
//! and the scanner never sees it disappear.

use mtg_engine::{all_cards, AbilityDefinition, CardDefinition, KeywordAbility, TriggerCondition};
use std::collections::BTreeSet;
use std::path::Path;

// ── Comment-stripping (mirrors core::decision_gate's idiom, PB-DX32 M8) ────────

/// Strips `//` line comments. Naive about `//` inside string literals -- adequate
/// for this data file, not a general Rust tokenizer (same caveat as the precedent).
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Strips `/* ... */` block comments (PB-DX32 fix cycle, review finding M8 --
/// `strip_line_comments` alone lets a block-commented row escape detection because
/// the compiler drops it but a line-only scanner still "sees" the text inside the
/// comment as unchanged source).
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

fn replay_harness_source() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/testing/replay_harness.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

/// Extract the body of `fn build_face_triggered_abilities` (the OPEN brace after
/// its signature through the matching CLOSE brace) from already comment-stripped
/// source, by simple brace balancing. Naive about braces inside string/char
/// literals -- adequate here (no such literal appears in this function's body).
fn extract_function_body<'a>(stripped: &'a str, fn_name: &str) -> &'a str {
    let sig_marker = format!("fn {fn_name}(");
    let sig_start = stripped
        .find(&sig_marker)
        .unwrap_or_else(|| panic!("`fn {fn_name}(` not found in stripped source"));
    let open_brace = stripped[sig_start..]
        .find('{')
        .map(|i| sig_start + i)
        .unwrap_or_else(|| panic!("no opening brace found after `fn {fn_name}(`"));
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
    let end = end.unwrap_or_else(|| panic!("unbalanced braces in `fn {fn_name}` body"));
    &stripped[open_brace..end]
}

// ── G-A: the lowering function must never see `trigger_zone` ───────────────────

/// G-A (plan §2.6): `trigger_zone` occurs ZERO times inside
/// `build_face_triggered_abilities`'s body. A 41st lowering arm that destructures
/// `trigger_zone` (the only way an arm can swallow it) fails this gate.
#[test]
fn g_a_lowering_function_never_sees_trigger_zone() {
    let stripped = strip_comments(&replay_harness_source());
    let body = extract_function_body(&stripped, "build_face_triggered_abilities");
    assert!(
        !body.contains("trigger_zone"),
        "CR 113.6b/113.6m: build_face_triggered_abilities's body must never see \
         `trigger_zone` -- the filter lives at its single call site in \
         build_face_ability_vectors (`lowers_onto_the_battlefield`). Do NOT add a \
         per-arm guard here -- extend the filter at the call site instead."
    );
}

/// G-A is proven non-vacuous: the function body must actually exist and be
/// non-trivially sized (it contains dozens of `trigger_condition:` match arms), so
/// a broken `extract_function_body` that returns an empty slice cannot make G-A
/// pass by accident.
#[test]
fn g_a_scan_is_not_vacuous() {
    let stripped = strip_comments(&replay_harness_source());
    let body = extract_function_body(&stripped, "build_face_triggered_abilities");
    let arm_count = body.matches("trigger_condition:").count();
    assert!(
        arm_count >= 30,
        "build_face_triggered_abilities's body must contain dozens of \
         `trigger_condition:` match arms (measured at PB-DX24 stage 3: 34) -- got \
         {arm_count}. A collapsed or empty extraction would make G-A pass \
         vacuously."
    );
}

// ── G-B: exactly one (filtered) call site ───────────────────────────────────────

/// G-B (plan §2.6): `build_face_triggered_abilities(` appears exactly TWICE in the
/// stripped file (the `fn` definition + one call), and the call's argument is the
/// filtered binding `battlefield_triggers` by name. A second, unfiltered call site
/// is the other way the invariant can be lost.
#[test]
fn g_b_call_site_is_unique_and_filtered() {
    let stripped = strip_comments(&replay_harness_source());
    let occurrences = stripped.matches("build_face_triggered_abilities(").count();
    assert_eq!(
        occurrences, 2,
        "`build_face_triggered_abilities(` must appear exactly twice in \
         replay_harness.rs (the fn definition + its single call site) -- got \
         {occurrences}. A second call site would bypass the CR 113.6b/113.6m \
         filter."
    );
    assert!(
        stripped.contains("build_face_triggered_abilities(&battlefield_triggers)"),
        "the single call site must pass the FILTERED binding `battlefield_triggers` \
         by name, not `abilities` (the unfiltered parameter) or any other \
         expression."
    );
}

// ── R1: the trigger_zone: Some(_) corpus population (§4.3) ──────────────────────

/// Fix cycle (review Finding 8): the ORIGINAL version walked `def.abilities`
/// only (front face). R1's own doc comment says a new `trigger_zone` def
/// "must ALSO have a dispatch arm ... or it will silently never fire" -- a
/// def declaring `trigger_zone: Some(_)` on its BACK face is exactly that
/// population, and was invisible to this roster. Now walks both faces.
fn trigger_zone_population(defs: &[CardDefinition]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in defs {
        for ability in &def.abilities {
            if let AbilityDefinition::Triggered {
                trigger_zone: Some(_),
                ..
            } = ability
            {
                out.insert(def.name.clone());
            }
        }
        if let Some(back) = &def.back_face {
            for ability in &back.abilities {
                if let AbilityDefinition::Triggered {
                    trigger_zone: Some(_),
                    ..
                } = ability
                {
                    out.insert(def.name.clone());
                }
            }
        }
    }
    out
}

/// R1 (plan §4.3): the `trigger_zone: Some(_)` population, pinned BY SYMBOL (card
/// names, never file paths), exactly `{Bloodghast, Squee Goblin Nabob, Nether
/// Traitor}`. A new `trigger_zone` def must be added here AND must have a dispatch
/// arm in `collect_graveyard_carddef_triggers`, or it will silently never fire.
#[test]
fn r1_trigger_zone_population_is_pinned() {
    let cards = all_cards();
    assert!(
        cards.len() >= 1_700,
        "non-vacuity floor: all_cards() must return at least 1,700 defs (measured \
         at PB-DX24: 1,803) -- got {}. A broken enumeration cannot make an empty \
         roster look correct.",
        cards.len()
    );
    let population = trigger_zone_population(&cards);
    let expected: BTreeSet<String> = ["Bloodghast", "Squee, Goblin Nabob", "Nether Traitor"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        population, expected,
        "PB-DX24 R1: the trigger_zone: Some(_) corpus population moved. A new \
         `trigger_zone` def must ALSO have a dispatch arm in \
         collect_graveyard_carddef_triggers, or it will silently never fire. \
         Expected {expected:?}, got {population:?}."
    );
}

// ── R2: the back-face population this batch's Q1-Q7 probes are measured against ─

/// R2 (plan §4.3): the `back_face: Some(_)` corpus population, with its own
/// non-vacuity floor (a roster pinned empty rots silently). Measured at PB-DX24
/// stage 1: 15 defs, none of which carries any of the 7 OOS-DX1-4 shapes on its
/// back face -- which is WHY every Q1/Q3/Q4/Q6 probe in
/// `pb_dx24_trigger_zone_and_index_spaces.rs` uses a SYNTHETIC fixture rather than
/// a real corpus card.
///
/// Fix cycle (review Finding 8): the ORIGINAL version asserted only the
/// def-count (15) and told a human, in its failure message, to "re-run the
/// shape scan by hand" -- so the actual §4.2 finding this test claims to back
/// ("0 real corpus cards exercise any Q-shape") was pinned by NOTHING
/// machine-checked. A def could gain `Keyword(Backup(_))` on its back face
/// and this gate would stay green. Now asserts the SEVEN per-shape counts
/// directly (each measured 0 at PB-DX24 stage 1), keeping the def-count pin
/// as an eighth, still-real assertion.
#[test]
fn r2_back_face_population_is_pinned_with_a_non_vacuity_floor() {
    let cards = all_cards();
    let back_face_defs: BTreeSet<String> = cards
        .iter()
        .filter(|d| d.back_face.is_some())
        .map(|d| d.name.clone())
        .collect();
    assert!(
        !back_face_defs.is_empty(),
        "non-vacuity floor: the back_face: Some(_) population must be non-empty, \
         or this roster (and the Q1-Q7 'zero real cards exercise this shape' \
         finding it backs) is measuring nothing."
    );
    assert_eq!(
        back_face_defs.len(),
        15,
        "PB-DX24 R2: the back_face: Some(_) corpus population moved from the \
         PB-DX24-stage-1-measured 15 -- got {}. Population: {back_face_defs:?}",
        back_face_defs.len()
    );

    let backup = count_back_face_shape(&cards, |a| {
        matches!(a, AbilityDefinition::Keyword(KeywordAbility::Backup(_)))
    });
    assert_eq!(
        backup, 0,
        "Q1 (abilities.rs's Backup ETB lowering) moved off its PB-DX24-stage-1 \
         measurement of 0 real back-face Keyword(Backup(_)) abilities -- a Q1 \
         probe can now be written against a REAL corpus card instead of a \
         synthetic fixture. Got {backup}."
    );
    let when_you_cast = count_back_face_shape(&cards, |a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenYouCastThisSpell,
                ..
            }
        )
    });
    assert_eq!(
        when_you_cast, 0,
        "Q2 (abilities.rs's WhenYouCastThisSpell queue site) moved off 0 real \
         back-face abilities of this shape. Got {when_you_cast}."
    );
    let when_exerted = count_back_face_shape(&cards, |a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenExertedAsAttacks,
                ..
            }
        )
    });
    assert_eq!(
        when_exerted, 0,
        "Q3 (abilities.rs's WhenExertedAsAttacks queue site) moved off 0 real \
         back-face abilities of this shape. Got {when_exerted}."
    );
    let combat_damage = count_back_face_shape(&cards, |a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                ..
            }
        )
    });
    assert_eq!(
        combat_damage, 0,
        "Q4 (abilities.rs's WhenDealsCombatDamageToPlayer queue site) moved off \
         0 real back-face abilities of this shape. Got {combat_damage}."
    );
    let turned_face_up = count_back_face_shape(&cards, |a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenTurnedFaceUp,
                ..
            }
        )
    });
    assert_eq!(
        turned_face_up, 0,
        "Q5 (resolution.rs's WhenTurnedFaceUp queue site, re-scoped not fixed) \
         moved off 0 real back-face abilities of this shape. Got {turned_face_up}."
    );
    let ring_tempts = count_back_face_shape(&cards, |a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverRingTemptsYou,
                ..
            }
        )
    });
    assert_eq!(
        ring_tempts, 0,
        "Q6 (abilities.rs's WheneverRingTemptsYou queue site) moved off 0 real \
         back-face abilities of this shape. Got {ring_tempts}."
    );
    let trigger_zone_shape = count_back_face_shape(&cards, |a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_zone: Some(_),
                ..
            }
        )
    });
    assert_eq!(
        trigger_zone_shape, 0,
        "Q7 (abilities.rs's graveyard sweep, collect_graveyard_carddef_triggers) \
         moved off 0 real back-face abilities carrying trigger_zone: Some(_). \
         Got {trigger_zone_shape}."
    );
}

/// Counts abilities matching `pred` across every def's BACK face only (the
/// §4.2 measurement is specifically about the back-face population -- the
/// front face is already covered by R1 and by ordinary corpus authoring).
fn count_back_face_shape(
    cards: &[CardDefinition],
    pred: impl Fn(&AbilityDefinition) -> bool,
) -> usize {
    cards
        .iter()
        .filter_map(|d| d.back_face.as_ref())
        .flat_map(|f| f.abilities.iter())
        .filter(|a| pred(a))
        .count()
}
