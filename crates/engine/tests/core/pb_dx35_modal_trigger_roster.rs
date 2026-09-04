//! PB-DX35 Half A (`OOS-DX4-2`): the modal-triggered-ability **census** gate.
//!
//! Every population here is a WALK of `all_cards()` (SR-36) — never a grep — and every
//! row PRINTS what it counted (`t_census_report`), never leaves a bare figure to be
//! transcribed into prose (PB-DX8's rule, PB-DX28's MEDIUM).
//!
//! ## Rows
//!
//! * **r1** — the 7 modal triggered abilities, by name, with their `mode_targets` state.
//! * **r2** — the index-space alignment census (execution-notes §0.5): for every modal
//!   triggered ability, the REGISTRY index (position in `def.abilities`) vs the RUNTIME
//!   index (position `trigger_modal_plan`'s `Normal`-kind lookup actually reaches, derived
//!   by CALLING the production lowering, `enrich_spec_from_def`/`build_face_ability_vectors`
//!   -- never by counting `AbilityDefinition::Triggered` entries by hand, since an
//!   unhandled `TriggerCondition` could silently drop an entry from the runtime vec and a
//!   hand count would never see it). The misaligned set is pinned at EXACTLY
//!   `{hullbreaker_horror, glissa_sunslayer, junji_the_midnight_sky}`.
//! * **r3** — every misaligned member is non-`Complete` (the zero-deck-legal-blast-radius
//!   claim, gated rather than asserted in prose).
//! * **r4** — `max_modes == 1` for every modal triggered ability (the A1 step-5 premise
//!   `trigger_modal_plan`'s `debug_assert!` relies on).
//! * **r5** — no def combines a NONEMPTY flat `targets` with `mode_targets: Some(_)` on a
//!   Triggered ability -- the author invariant the cast path already enforces at
//!   `casting.rs:3848`.
//! * **r6** — the defect population (nonempty flat `targets`, `mode_targets: None`) is
//!   EXACTLY the three members this batch filed (`hullbreaker_horror`, `glissa_sunslayer`,
//!   `junji_the_midnight_sky`); the three repaired members now carry `mode_targets: Some`.
//! * **r7** — the `modal_trigger` `decision_site_walk` row's `site` string no longer claims
//!   `modes_chosen = vec![0]` in both arms.

use std::collections::BTreeSet;

use mtg_engine::{all_cards, AbilityDefinition, CardDefinition, Completeness};

// ─────────────────────────────────────────────────────────────────────────────
// The population walk
// ─────────────────────────────────────────────────────────────────────────────

/// One modal `AbilityDefinition::Triggered` ability, found by walking `all_cards()`'s
/// FRONT face abilities only -- the census (execution-notes §0.5) measured zero back-face
/// members, and `r_no_back_face_member` below gates that so a new one cannot join silently.
struct ModalTriggerMember {
    name: String,
    complete: bool,
    /// Position of the modal ability within `def.abilities` (the REGISTRY index space
    /// `rules::abilities::trigger_modal_plan`'s `ModeSelection` lookup indexes into).
    registry_index: usize,
    has_mode_targets: bool,
    flat_targets_nonempty: bool,
    /// Does any `mode_targets` slice hold a `TargetRequirement::UpToN`? See `r5`'s second
    /// conjunct for why this is a separate axis from `flat_targets_nonempty`.
    mode_targets_contain_up_to_n: bool,
    max_modes: usize,
}

fn modal_trigger_members() -> Vec<ModalTriggerMember> {
    let mut out = Vec::new();
    for def in all_cards() {
        for (idx, ability) in def.abilities.iter().enumerate() {
            if let AbilityDefinition::Triggered {
                modes: Some(modes),
                targets,
                ..
            } = ability
            {
                out.push(ModalTriggerMember {
                    name: def.name.clone(),
                    complete: def.completeness == Completeness::Complete,
                    registry_index: idx,
                    has_mode_targets: modes.mode_targets.is_some(),
                    flat_targets_nonempty: !targets.is_empty(),
                    mode_targets_contain_up_to_n: modes.mode_targets.as_ref().is_some_and(|mt| {
                        mt.iter().any(|slice| {
                            slice.iter().any(|r| {
                                matches!(
                                    r,
                                    mtg_engine::cards::card_definition::TargetRequirement::UpToN {
                                        ..
                                    }
                                )
                            })
                        })
                    }),
                    max_modes: modes.max_modes,
                });
            }
        }
        // r_no_back_face_member's own evidence: walk the back face too, but do not
        // add it to the population unless one is ever found (the census claims zero).
        if let Some(back) = &def.back_face {
            for ability in &back.abilities {
                assert!(
                    !matches!(ability, AbilityDefinition::Triggered { modes: Some(_), .. }),
                    "a BACK-FACE modal triggered ability was found on {:?} -- the census \
                     (execution-notes §0.5) measured zero; this file's population must be \
                     widened to cover it, not just this assertion silenced",
                    def.name
                );
            }
        }
    }
    out
}

/// The RUNTIME index `trigger_modal_plan`'s registry lookup is compared against for a
/// `PendingTriggerKind::Normal` trigger: the position of `def.abilities[registry_index]`
/// within the LOWERED `characteristics.triggered_abilities` vec, derived by CALLING the
/// production lowering (never by counting `Triggered` entries by hand -- an unhandled
/// `TriggerCondition` could silently produce zero runtime entries for an ability, which a
/// hand count would never catch).
///
/// Every one of the 7 modal members has EXACTLY ONE `AbilityDefinition::Triggered` on its
/// card (verified below, non-vacuity), so if the lowering produces exactly one runtime
/// entry, that entry's index is trivially 0 -- and if it produced zero (the "unhandled
/// condition" failure mode), that is itself a MORE SERIOUS defect this function surfaces
/// by panicking rather than silently returning a wrong number.
fn runtime_index(def: &CardDefinition) -> usize {
    let triggered_count_in_registry = def
        .abilities
        .iter()
        .filter(|a| matches!(a, AbilityDefinition::Triggered { .. }))
        .count();
    assert_eq!(
        triggered_count_in_registry, 1,
        "{:?}: this file's runtime-index derivation assumes exactly ONE \
         AbilityDefinition::Triggered per modal member (true for all 7 at census time) -- \
         re-derive this function if that ever changes",
        def.name
    );
    let defs_map: std::collections::HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let spec = mtg_engine::enrich_spec_from_def(
        mtg_engine::ObjectSpec::creature(mtg_engine::PlayerId(1), &def.name, 1, 1)
            .with_card_id(def.card_id.clone()),
        &defs_map,
    );
    assert_eq!(
        spec.triggered_abilities.len(),
        1,
        "{:?}: the production lowering (enrich_spec_from_def) must produce EXACTLY ONE \
         runtime TriggeredAbilityDef for this card's sole AbilityDefinition::Triggered -- \
         zero would mean its TriggerCondition is unhandled by the lowering, a defect this \
         gate exists to surface rather than silently miscount around",
        def.name
    );
    0
}

// ─────────────────────────────────────────────────────────────────────────────
// r1 -- the 7 modal triggered abilities, by name, with mode_targets state
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn r1_the_seven_modal_triggered_abilities_by_name() {
    let members = modal_trigger_members();
    let names: BTreeSet<String> = members.iter().map(|m| m.name.clone()).collect();
    let expected: BTreeSet<String> = [
        "Felidar Retreat",
        "Retreat to Coralhelm",
        "Retreat to Kazandu",
        "Shambling Ghast",
        "Hullbreaker Horror",
        "Glissa Sunslayer",
        "Junji, the Midnight Sky",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(
        names, expected,
        "the modal-triggered-ability population must be EXACTLY these 7 -- a new member \
         (or the loss of one) must be a red test, not a silent drift"
    );
    assert_eq!(members.len(), 7, "non-vacuity: exactly 7 members walked");
}

// ─────────────────────────────────────────────────────────────────────────────
// r2 -- the index-space alignment census
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn r2_index_space_alignment_census() {
    let all = all_cards();
    let mut misaligned: BTreeSet<String> = BTreeSet::new();
    let mut aligned: BTreeSet<String> = BTreeSet::new();

    for member in modal_trigger_members() {
        let def = all
            .iter()
            .find(|d| d.name == member.name)
            .expect("member must be in all_cards()");
        let runtime_idx = runtime_index(def);
        eprintln!(
            "r2: {:<28} registry_index={} runtime_index={} aligned={}",
            member.name,
            member.registry_index,
            runtime_idx,
            member.registry_index == runtime_idx
        );
        if member.registry_index == runtime_idx {
            aligned.insert(member.name.clone());
        } else {
            misaligned.insert(member.name.clone());
        }
    }

    let expected_misaligned: BTreeSet<String> = [
        "Hullbreaker Horror",
        "Glissa Sunslayer",
        "Junji, the Midnight Sky",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(
        misaligned, expected_misaligned,
        "the index-space-misaligned set must be EXACTLY these three -- if this set ever \
         empties, `OOS-DX35-1` is closed and the three markers this batch wrote must be \
         re-adjudicated; if it grows, a new member has the same defect and needs the same \
         disposition"
    );
    let expected_aligned: BTreeSet<String> = [
        "Felidar Retreat",
        "Retreat to Coralhelm",
        "Retreat to Kazandu",
        "Shambling Ghast",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(aligned, expected_aligned);
}

// ─────────────────────────────────────────────────────────────────────────────
// r3 -- every misaligned member is non-Complete (gated, not asserted in prose)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn r3_every_misaligned_member_is_non_complete() {
    let members = modal_trigger_members();
    let misaligned_names = [
        "Hullbreaker Horror",
        "Glissa Sunslayer",
        "Junji, the Midnight Sky",
    ];
    let mut checked = 0;
    for name in misaligned_names {
        let m = members
            .iter()
            .find(|m| m.name == name)
            .unwrap_or_else(|| panic!("{name} must be a modal-trigger member"));
        assert!(
            !m.complete,
            "{name}: the index-space mismatch (OOS-DX35-1) is only a zero-deck-legal-blast-\
             radius claim while this def stays non-Complete -- validate_deck refuses it"
        );
        checked += 1;
    }
    assert_eq!(
        checked, 3,
        "non-vacuity: all three misaligned members checked"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r4 -- max_modes == 1 for every modal triggered ability
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn r4_max_modes_is_one_for_every_member() {
    let members = modal_trigger_members();
    assert_eq!(members.len(), 7, "non-vacuity");
    for m in &members {
        assert_eq!(
            m.max_modes, 1,
            "{}: trigger_modal_plan's A1-step-5 debug_assert! (max_modes > 1 combined with \
             mode_targets: Some(_) is unsupported) relies on this population being uniformly \
             max_modes: 1 -- a new member with max_modes > 1 must be a red test here before \
             it can silently hit that debug_assert! in a debug build",
            m.name
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// r5 -- no def combines a nonempty flat `targets` with `mode_targets: Some(_)`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn r5_no_member_combines_flat_targets_with_mode_targets() {
    let members = modal_trigger_members();
    let mut with_mode_targets = 0;
    for m in &members {
        if m.has_mode_targets {
            with_mode_targets += 1;
            assert!(
                !m.flat_targets_nonempty,
                "{}: declares BOTH a nonempty flat `targets` list AND `mode_targets: Some(_)` \
                 -- casting.rs:3848 enforces this author invariant on the cast path; a \
                 triggered ability must obey the same rule",
                m.name
            );
            // **↻ The SECOND author invariant, added after this batch's own `/review`.**
            // Both peer modal paths hard-reject `UpToN` inside `mode_targets` with an explicit
            // `InvalidCommand` -- `casting.rs:3856` (spell) and `abilities.rs:481-486`
            // (activated) -- and this gate's first draft mirrored only the flat-targets one,
            // naming ONE of two rules that sit five lines apart in the same file. The reviewer
            // planted `UpToN` into `retreat_to_kazandu`'s mode-0 slice and all eight roster
            // gates stayed GREEN, while the behaviour was genuinely wrong: an `UpToN` slot is
            // `optional`, so `trigger_modal_mode_is_legal` calls the mode unconditionally legal,
            // CR 700.2b's fall-through to the next mode dies, and mode 0 is chosen with no
            // target. *A gate that mirrors one of two adjacent invariants measures one of them.*
            assert!(
                !m.mode_targets_contain_up_to_n,
                "{}: puts a `TargetRequirement::UpToN` inside `mode_targets`. Both peer paths \
                 reject that combination outright (casting.rs:3856, abilities.rs:481-486) \
                 because a variable-count per-mode slice is unsupported -- and on the TRIGGER \
                 path it is worse than unsupported: an UpToN slot is `optional`, so the mode is \
                 judged CR 700.2b-legal unconditionally and the fall-through to a mode that \
                 really is legal never happens",
                m.name
            );
        }
    }
    assert_eq!(
        with_mode_targets, 3,
        "non-vacuity: the three members this batch re-shaped into mode_targets"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r6 -- the defect population is exactly the three filed members
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn r6_the_defect_population_is_exactly_the_three_filed_members() {
    let members = modal_trigger_members();
    let defective: BTreeSet<String> = members
        .iter()
        .filter(|m| m.flat_targets_nonempty && !m.has_mode_targets)
        .map(|m| m.name.clone())
        .collect();
    let expected: BTreeSet<String> = [
        "Hullbreaker Horror",
        "Glissa Sunslayer",
        "Junji, the Midnight Sky",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(
        defective, expected,
        "the 'nonempty flat targets, no mode_targets' defect population must be EXACTLY the \
         three members this batch filed rather than repaired"
    );

    let repaired: BTreeSet<String> = members
        .iter()
        .filter(|m| m.has_mode_targets)
        .map(|m| m.name.clone())
        .collect();
    let expected_repaired: BTreeSet<String> = [
        "Shambling Ghast",
        "Retreat to Kazandu",
        "Retreat to Coralhelm",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(
        repaired, expected_repaired,
        "the three members this batch re-shaped into mode_targets"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// r7 -- the decision_site_walk `modal_trigger` row's site string is honest
// ─────────────────────────────────────────────────────────────────────────────

/// **↻ RE-KEYED after this batch's own `/review` defeated the first draft TWICE by execution.**
/// The first draft asserted `!row_text.contains("modes_chosen = vec![0] in both")` plus a
/// `contains("trigger_modal_plan")` floor — i.e. it forbade ONE SPELLING of the lie and required
/// one token. Two defeats, both reproduced here before the fix:
///
/// * **Reword.** `"… trigger_modal_plan (PB-DX35) -- hard-codes mode 0 in both the min_modes==0
///   and min_modes!=0 arms"` re-asserts the exact false claim in different words *while still
///   naming `trigger_modal_plan`*, so the negative needle misses and the positive floor passes.
/// * **Line continuation.** Splitting the original needle across a Rust `\`-newline leaves the
///   RENDERED string byte-identical to the pre-batch lie while the SOURCE no longer contains the
///   needle contiguously. That is `OOS-DX51-6`'s class verbatim, committed in the batch that
///   inherited the lesson.
///
/// Re-keyed on the MECHANISM rather than on a spelling, in three conjuncts:
///
/// 1. The row text is NORMALISED first — `\`+newline+indent collapsed — so a split needle cannot
///    hide. A gate that reads Rust source and does not do this is measuring the formatter.
/// 2. A DENYLIST of hard-code assertions, matched on the normalised text and deliberately
///    over-collecting (`hard-code`, `hard code`, `hardcode`, `= vec![0]`, `always mode 0`,
///    `mode 0 in both`). Over-collection can only make this redder.
/// 3. A positive requirement that the row names BOTH the shared function AND the rule that
///    replaced the hard-code, so "names `trigger_modal_plan`" cannot be satisfied by a sentence
///    that mentions it only to say it hard-codes mode 0.
#[test]
fn r7_decision_site_walk_row_no_longer_claims_the_hard_coded_mode_zero() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/core/decision_site_walk.rs"),
    )
    .expect("decision_site_walk.rs must be readable");
    let row_start = src
        .find("id: \"modal_trigger\"")
        .expect("the modal_trigger row must exist");
    let row_end = src[row_start..]
        .find("predicate: p_modal_trigger")
        .map(|off| row_start + off)
        .expect("the modal_trigger row's predicate field must exist");
    // Conjunct 1: collapse Rust line continuations (`\` + newline + indent) so the text this
    // gate reads is the text the row RENDERS, not the text the formatter happened to lay out.
    let row_text = collapse_line_continuations(&src[row_start..row_end]);

    // Conjunct 2: the denylist, over-collecting on purpose.
    const HARD_CODE_CLAIMS: [&str; 6] = [
        "modes_chosen = vec![0]",
        "= vec![0]",
        "hard-code",
        "hard code",
        "hardcode",
        "always mode 0",
    ];
    // A denylist over prose needs a NEGATION guard, and finding that out cost one red run:
    // the row's own honest phrasing is "picks the first CR 700.2b-legal mode by declared order,
    // **not always mode 0**", which asserts the opposite of the claim being forbidden. So a hit
    // counts only when it is NOT inside a negating clause. The window is 32 bytes before the
    // hit, bounded on a char boundary.
    //
    // **Stated residual**: a sufficiently contrived double negative ("it is not true that this
    // does not hard-code mode 0") evades this, and no prose gate closes that. What it DOES catch
    // is both defeats the `/review` actually executed -- the reword asserted "hard-codes mode 0
    // in both the min_modes==0 and min_modes!=0 arms" with no negator anywhere near it.
    const NEGATORS: [&str; 5] = ["not ", "no longer", "never", "rather than", "instead of"];
    let offenders: Vec<&str> = HARD_CODE_CLAIMS
        .iter()
        .copied()
        .filter(|n| match row_text.find(n) {
            None => false,
            Some(at) => {
                let mut lo = at.saturating_sub(32);
                while lo < at && !row_text.is_char_boundary(lo) {
                    lo += 1;
                }
                !NEGATORS.iter().any(|neg| row_text[lo..at].contains(neg))
            }
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "the modal_trigger row's `site` string still asserts the pre-PB-DX35 hard-code \
         ({offenders:?}). Since PB-DX35 the mode is chosen by CR 700.2b legality, not by a \
         constant -- rewrite the row (execution-notes §0.3). Matched on the NORMALISED row text, \
         so splitting the claim across a line continuation does not evade this."
    );

    // Conjunct 3: naming the function is not enough -- it must name the RULE that replaced the
    // hard-code, so a sentence mentioning `trigger_modal_plan` only in order to restate the lie
    // cannot satisfy the positive half.
    assert!(
        row_text.contains("trigger_modal_plan"),
        "the modal_trigger row's `site` string should name the shared function it now describes"
    );
    assert!(
        row_text.contains("700.2b"),
        "the modal_trigger row's `site` string must cite CR 700.2b -- the rule that replaced the \
         hard-coded mode 0. Naming `trigger_modal_plan` alone is satisfiable by a sentence that \
         names it and then restates the very claim this gate forbids (proved by execution in \
         this batch's own `/review`)."
    );
}

/// Collapse Rust's `\` + newline + indentation line continuations, so a needle split across two
/// source lines is still found. See `r7`'s doc for the executed defeat that forced this.
fn collapse_line_continuations(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'\n') {
            chars.next();
            while chars.peek().is_some_and(|c| *c == ' ' || *c == '\t') {
                chars.next();
            }
            continue;
        }
        out.push(c);
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// r8 -- the MECHANISM gate behind AC 7327's "ONE shared arithmetic"
// ─────────────────────────────────────────────────────────────────────────────

/// Every extraction of a TRIGGERED ability's `targets` inside `crates/engine/src/rules/` lives
/// inside `trigger_modal_plan`.
///
/// **↻ Added after this batch's own `/review` proved that nothing enforced the headline claim.**
/// `t9` (`rules/abilities.rs`'s `#[cfg(test)]` module) is a DIFFERENTIAL probe: it asserts site 3
/// agrees with the shared plan BY VALUE. The reviewer re-planted the original `OOS-DX4-2` defect
/// in site 3 behind `if trigger.kind == PendingTriggerKind::CardDefETB` — a hand-rolled fifth copy
/// reading the flat registry `targets` and ignoring `mode_targets` — and the entire `mtg-engine`
/// crate stayed green, `t9` included, because `t9`'s two cases both drove
/// `PendingTriggerKind::Normal`. `t9` gained a `CardDefETB` case for that specific defeat; **this
/// gate is the general answer**, and the difference matters: a differential probe proves agreement
/// on the branches it drives and nothing about the branches it does not, whereas this one is keyed
/// on the MECHANISM (PB-DX48's `r1` and PB-DX49's `r7` shape).
///
/// **The population is measured, and this gate's own first run REFUTED the figure its author had
/// written one paragraph above it.** The draft said *"exactly THREE such extractions exist in
/// `rules/`, and `rules/mana.rs`'s site 4 does not match — it never extracts `targets` at all"*.
/// It does: `mana.rs:821-828` destructures `targets` straight out of an
/// `AbilityDefinition::Triggered` pattern. The throwaway script behind that sentence searched for
/// `AbilityDefinition::Triggered {` **with the brace**, and `mana.rs` puts the brace on the next
/// line. A gate wrote its own author's correction — which is the entire argument for having one.
///
/// So the population is **SIX**: two branches of `trigger_modal_plan`'s single lookup, `t9`'s own
/// `#[cfg(test)]` fixture, and three inside `fire_mana_triggered_abilities`. Site 4 is
/// allowlisted rather than unified, and its exemption is NARROW and re-checked in source: it uses
/// the binding at exactly one place, `targets.is_empty()` — a presence test deciding whether the
/// ability must use the stack (CR 605.5a) — and never announces, slices or indexes it. It also
/// queues `PendingTriggerKind::CardDefETB` precisely so the index spaces line up. See
/// execution-notes §0.5.
///
/// Over-collection is deliberate: the scan matches BOTH spellings (an
/// `AbilityDefinition::Triggered { .. targets .. }` destructure and a
/// `characteristics.triggered_abilities` read followed by `.targets`), because over-collecting can
/// only make this redder.
#[test]
fn r8_every_triggered_target_extraction_in_rules_lives_in_trigger_modal_plan() {
    let rules_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules");
    let mut sites: Vec<(String, usize, String)> = Vec::new();
    let mut files_scanned = 0usize;
    for entry in std::fs::read_dir(&rules_dir).expect("crates/engine/src/rules must be readable") {
        let path = entry.expect("readable dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        files_scanned += 1;
        let src = std::fs::read_to_string(&path).expect("rules source must be readable");
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        for (needle, window) in [
            ("AbilityDefinition::Triggered", 400usize),
            ("triggered_abilities", 260),
        ] {
            let mut from = 0usize;
            while let Some(rel) = src[from..].find(needle) {
                let at = from + rel;
                let mut end = (at + window).min(src.len());
                while end > at && !src.is_char_boundary(end) {
                    end -= 1;
                }
                if src[at..end].contains("targets") {
                    let line = src[..at].matches('\n').count() + 1;
                    sites.push((name.clone(), line, enclosing_fn(&src, at)));
                }
                from = at + needle.len();
            }
        }
    }
    // Non-vacuity: a scan that reads no files, or finds no sites, would pass the equality below
    // vacuously. `OOS-DX8-7`: a gate whose denominator can silently go to zero is not a gate.
    assert!(
        files_scanned >= 10,
        "non-vacuity: only {files_scanned} .rs files found under src/rules -- the scan is \
         probably pointed at the wrong directory"
    );
    assert!(
        sites.len() >= 3,
        "non-vacuity: found only {} triggered-target extraction(s) in rules/; the needles are \
         probably stale against a rename",
        sites.len()
    );

    /// Enclosing functions permitted to extract a triggered ability's `targets`, each with the
    /// reason it is exempt. The reason is re-checked in source below — an allowlist whose reason
    /// nothing verifies is a comment (`OOS-DX47`).
    const ALLOWED: [(&str, &str); 3] = [
        (
            "trigger_modal_plan",
            "THE shared arithmetic (CR 700.2b + CR 700.2c); sites 1/2/D and site 3 all delegate",
        ),
        (
            "modal_subject",
            "t9's own `#[cfg(test)]` fixture, which BUILDS a def rather than reading one",
        ),
        (
            "fire_mana_triggered_abilities",
            "site 4 (CR 605.4a/605.5a): uses the binding at exactly one place, \
             `targets.is_empty()` -- a presence test deciding whether the ability must use the \
             stack -- and never announces, slices or indexes it",
        ),
    ];
    let offenders: Vec<&(String, usize, String)> = sites
        .iter()
        .filter(|(_, _, f)| !ALLOWED.iter().any(|(a, _)| a == f))
        .collect();
    assert!(
        offenders.is_empty(),
        "these functions in crates/engine/src/rules/ extract a TRIGGERED ability's `targets` \
         outside `trigger_modal_plan`, which is the fifth hand-rolled copy AC 7327 exists to \
         prevent: {offenders:?}. If one is legitimate, add it to ALLOWED with its reason -- and \
         note the reason is re-checked in source by the assertion below."
    );
    // The allowlist's reasons, verified rather than trusted.
    let abilities_src = std::fs::read_to_string(rules_dir.join("abilities.rs"))
        .expect("abilities.rs must be readable");
    assert!(
        abilities_src.contains("fn trigger_modal_plan("),
        "ALLOWED names `trigger_modal_plan` as THE shared arithmetic, but no such function \
         exists in rules/abilities.rs any more -- the exemption has outlived its reason"
    );
    // Site 4's exemption is the narrow one, so it is the one whose reason is checked hardest:
    // it may TEST the list's emptiness and nothing else. If it ever starts announcing, slicing
    // or indexing those targets, it is a fifth copy and belongs in the shared arithmetic.
    let mana_src = std::fs::read_to_string(rules_dir.join("mana.rs")).expect("mana.rs readable");
    assert!(
        mana_src.contains("targets.is_empty()"),
        "rules/mana.rs (site 4) is allowlisted BECAUSE its only use of the extracted `targets` \
         is the presence test `targets.is_empty()` (CR 605.5a). That call is gone, so the \
         exemption has outlived its stated reason -- re-derive what it does now."
    );
    for forbidden in ["targets.get(", "targets.iter()", "targets[", "mode_targets"] {
        assert!(
            !mana_src.contains(forbidden),
            "rules/mana.rs (site 4) now contains `{forbidden}`, so it does more than TEST the \
             triggered ability's target list -- its narrow exemption no longer holds and it is a \
             fifth hand-rolled copy of the arithmetic `trigger_modal_plan` owns"
        );
    }
}

/// The name of the `fn` lexically enclosing byte offset `at`, or `"<top level>"`.
fn enclosing_fn(src: &str, at: usize) -> String {
    src[..at]
        .rmatch_indices("fn ")
        .find_map(|(i, _)| {
            // Require `fn` to start a token (preceded by whitespace or line start), so
            // `unsafe fn`/`pub fn` are found and identifiers ending in "fn " are not.
            let ok = i == 0 || src[..i].ends_with([' ', '\n', '\t']);
            if !ok {
                return None;
            }
            let rest = &src[i + 3..];
            let end = rest
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            (end > 0).then(|| rest[..end].to_string())
        })
        .unwrap_or_else(|| "<top level>".to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// t_census_report -- every population, PRINTED (never transcribed)
// ─────────────────────────────────────────────────────────────────────────────

/// Run with `cargo test -p mtg-engine --test core pb_dx35 -- --nocapture`.
#[test]
fn t_census_report() {
    eprintln!("\n=== PB-DX35 Half A census (walked from all_cards(), never grepped) ===");
    let members = modal_trigger_members();
    eprintln!("r1 -- modal triggered abilities: {}", members.len());
    for m in &members {
        eprintln!(
            "  {:<28} complete={:<5} registry_idx={} mode_targets={:<5} flat_targets={:<5} \
             max_modes={}",
            m.name,
            m.complete,
            m.registry_index,
            m.has_mode_targets,
            m.flat_targets_nonempty,
            m.max_modes
        );
    }
    // r2's alignment is printed by r2 itself (it needs `all_cards()` a second time to
    // resolve each member's full CardDefinition for `runtime_index`), which the mandatory
    // `--nocapture` invocation for THIS test does not re-run; run r2 with `--nocapture`
    // separately to see the per-member alignment table.
    eprintln!("(see `r2_index_space_alignment_census -- --nocapture` for the per-member table)");
}
