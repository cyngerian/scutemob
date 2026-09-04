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
    let row_text = &src[row_start..row_end];
    assert!(
        !row_text.contains("modes_chosen = vec![0] in both"),
        "the modal_trigger row's `site` string still claims the pre-PB-DX35 hard-code -- \
         rewrite it to describe trigger_modal_plan (execution-notes §0.3)"
    );
    assert!(
        row_text.contains("trigger_modal_plan"),
        "the modal_trigger row's `site` string should name the shared function it now \
         describes"
    );
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
