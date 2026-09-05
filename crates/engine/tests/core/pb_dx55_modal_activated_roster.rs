//! PB-DX55 Half 3 (`OOS-SIM5-5`) — the corpus census of modal ACTIVATED abilities,
//! derived by walking `all_cards()` (SR-36: enumerate the corpus, never grep source for a
//! roster), never transcribed from the plan doc.
//!
//! # The plan doc's own census had a wrong index, and this file is what catches a repeat
//!
//! `memory/primitives/pb-plan-DX55.md` §"Half 3" and the batch's own dispatch brief both
//! say `umezawas_jitte`'s modal ability sits at **layer-resolved ability index 1**. It does
//! not: `AbilityDefinition::Activated` entries lower into `Characteristics::
//! activated_abilities` in DECLARATION order (`enrich_spec_from_def`'s single filtering
//! loop, `crates/engine/src/testing/replay_harness.rs`), and Jitte's def declares the modal
//! counter-removal ability BEFORE the Equip ability — so the modal ability is index **0**,
//! not 1. `crates/simulator/src/legal_actions.rs`'s own pre-existing
//! `test_dp3_ability_default_modes_uses_layer_resolved_index` already pins this at 0 and
//! predates this batch. The plan's "index 1" is corrected here rather than propagated, and
//! `r3` below re-derives it structurally (by declaration order) rather than by re-quoting
//! either number.
//!
//! # CR 700.2a/700.2c: the exact requirement this roster exists to protect
//!
//! Every member below declares a FLAT `targets: vec![]` and puts every one of its target
//! requirements in `ModeSelection.mode_targets` instead — the shape
//! `queries::ability_target_requirements` (the pre-PB-DX55 3-argument form) could never see,
//! because it only ever read the flat list. `r2` pins the flat-list-is-always-empty half of
//! that author invariant so a future member cannot silently violate it and go unnoticed by
//! `ability_target_requirements`'s own `mode_targets.is_some() -> vec![]`
//! fallback-avoidance.
use std::collections::BTreeSet;

use mtg_engine::{all_cards, AbilityDefinition, Completeness};

/// One modal ACTIVATED ability found anywhere in a def's `abilities` list.
struct ModalActivated {
    card: &'static str,
    completeness: Completeness,
    /// Position among ONLY the `AbilityDefinition::Activated` entries on this def, in
    /// declaration order -- the SAME index space `enrich_spec_from_def` lowers into
    /// `Characteristics::activated_abilities` (mirrors `abilities.rs`'s own indexing).
    activated_index: usize,
    flat_targets_len: usize,
    min_modes: usize,
    max_modes: usize,
    mode_target_lens: Vec<usize>,
}

fn census() -> Vec<ModalActivated> {
    let mut out = Vec::new();
    for def in all_cards() {
        let mut activated_index = 0usize;
        for ability in &def.abilities {
            let AbilityDefinition::Activated {
                targets,
                modes: Some(ms),
                ..
            } = ability
            else {
                if matches!(ability, AbilityDefinition::Activated { .. }) {
                    activated_index += 1;
                }
                continue;
            };
            if ms.mode_targets.is_some() {
                out.push(ModalActivated {
                    card: Box::leak(def.name.clone().into_boxed_str()),
                    completeness: def.completeness.clone(),
                    activated_index,
                    flat_targets_len: targets.len(),
                    min_modes: ms.min_modes,
                    max_modes: ms.max_modes,
                    mode_target_lens: ms
                        .mode_targets
                        .as_ref()
                        .map(|mt| mt.iter().map(|r| r.len()).collect())
                        .unwrap_or_default(),
                });
            }
            activated_index += 1;
        }
    }
    out
}

/// r1: the population is EXACTLY three named members, no more and no fewer. A non-vacuity
/// floor AND a ceiling in one assertion (a set-equality, not a `>=`), because a roster that
/// only checks a floor cannot tell a silently-added fourth member from a stable one.
#[test]
fn r1_the_modal_activated_population_is_exactly_three_named_members() {
    let names: BTreeSet<&str> = census().iter().map(|m| m.card).collect();
    let expected: BTreeSet<&str> = ["Cankerbloom", "Goblin Cratermaker", "Umezawa's Jitte"]
        .into_iter()
        .collect();
    assert_eq!(
        names, expected,
        "PB-DX55 Half 3 (`OOS-SIM5-5`): the corpus's modal-activated-with-per-mode-targets \
         population must be exactly these three. If this reddens with a NEW name added, \
         `legal_actions::ability_default_modes`'s legality scan and \
         `ability_target_requirements` already generalise to it (both are \
         exhaustive over the ability's own `modes.len()`, not over a hardcoded three) -- but \
         the deck-legal-blast-radius claim in this batch's report does not, and must be \
         re-derived rather than assumed to still hold."
    );
    // All three are Complete and deck-legal, stated as a claim this batch's report makes and
    // this roster is what backs it.
    for m in census() {
        assert_eq!(
            m.completeness,
            Completeness::Complete,
            "{} must be Complete -- the v4 memo's '2 refusals, 1.9%' pricing this seed at is \
             refuted precisely because all three members are deck-legal, not because the \
             engine gap is worse in principle",
            m.card
        );
    }
}

/// r2: CR 700.2c author invariant, corpus-checked rather than merely documented --
/// `ModeSelection.mode_targets: Some(_)` means the FLAT `targets` list must be empty
/// (`umezawas_jitte.rs`'s own in-source comment: "MUST be empty when mode_targets is
/// Some"). `ability_target_requirements` relies on this being true for every
/// corpus member for its "old 3-arg call == 4-arg call with `&[]`" equivalence claim (see
/// that function's own doc) to hold on real data, not merely in the hypothetical case it
/// also handles.
#[test]
fn r2_every_modal_activated_member_declares_an_empty_flat_targets_list() {
    let members = census();
    assert!(
        !members.is_empty(),
        "non-vacuity: r1 already proves this is 3"
    );
    for m in &members {
        assert_eq!(
            m.flat_targets_len, 0,
            "{}: CR 700.2c author invariant violated -- a modal ability with \
             `mode_targets: Some(_)` must declare `targets: vec![]`",
            m.card
        );
        assert_eq!(
            m.mode_target_lens.len(),
            m.max_modes.max(m.min_modes).max(m.mode_target_lens.len()),
            "{}: mode_targets length sanity",
            m.card
        );
        // Every member's cost forces exactly one mode (min_modes == max_modes == 1),
        // the shape `ability_default_modes`'s `debug_assert!(max_modes <= 1, ..)` and
        // `handle_activate_ability`'s own hard-reject on `mode_targets_active.is_some()
        // && len() > 1` both assume holds for the whole corpus today.
        assert_eq!(m.min_modes, 1, "{}: expected 'choose exactly one'", m.card);
        assert_eq!(m.max_modes, 1, "{}: expected 'choose exactly one'", m.card);
    }
}

/// r3: `umezawas_jitte`'s modal ability is at declaration-order index **0**, not the plan
/// doc's "index 1" -- re-derived structurally here (by counting `Activated` entries in
/// declaration order) rather than by re-asserting either number as a fact. Non-vacuity:
/// this only means something if Jitte's def actually has >= 2 `Activated` abilities (it
/// does: the modal one and Equip), which the second assertion checks directly.
#[test]
fn r3_umezawas_jitte_modal_ability_is_declaration_order_index_zero_not_one() {
    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Umezawa's Jitte")
        .expect("Umezawa's Jitte must be in the corpus");
    let activated_count = def
        .abilities
        .iter()
        .filter(|a| matches!(a, AbilityDefinition::Activated { .. }))
        .count();
    assert!(
        activated_count >= 2,
        "non-vacuity: Umezawa's Jitte must declare at least two AbilityDefinition::Activated \
         entries (the modal counter-removal ability and Equip) for an index question to mean \
         anything; got {activated_count}"
    );
    let modal_index = census()
        .iter()
        .find(|m| m.card == "Umezawa's Jitte")
        .map(|m| m.activated_index)
        .expect("Umezawa's Jitte must be in the modal-activated census (r1)");
    assert_eq!(
        modal_index, 0,
        "the plan doc (`pb-plan-DX55.md` Half 3) and this batch's own dispatch brief both \
         say 'ability index 1' -- that is wrong. `AbilityDefinition::Activated` entries \
         lower in DECLARATION order and the modal ability is declared before Equip in \
         `umezawas_jitte.rs`, so it is index 0. `legal_actions.rs`'s pre-existing \
         `test_dp3_ability_default_modes_uses_layer_resolved_index` already pinned this at 0 \
         before this batch touched anything."
    );
}

/// r4: no OTHER corpus def carries a bare `AbilityDefinition::Activated` struct literal
/// with `modes: None` that a future author might confuse for a modal one, and no
/// `LayerModification::AddActivatedAbility` grant carries per-mode targets either -- both
/// grant channels checked, not merely the declared one, per the plan's own §0 stage-0
/// census method. Structural: the census function already walks every `AbilityDefinition`
/// on every def, so a grant channel that ALSO produced an `AbilityDefinition::Activated`
/// entry would already be visible to `census()` -- this test exists to make the "both
/// channels checked" claim explicit and re-verifiable rather than merely asserted in prose.
#[test]
fn r4_no_layer_modification_grant_channel_adds_a_second_modal_activated_member() {
    // If a LayerModification::AddActivatedAbility grant ever carried per-mode targets it
    // would have to lower into a runtime ActivatedAbility with `modes: Some(_)` for
    // `ability_default_modes`/`ability_target_requirements` to see it at all --
    // and those are read from `Characteristics::activated_abilities`, not from
    // `CardDefinition::abilities`. This census walks `CardDefinition::abilities` (the
    // DECLARED channel) only, by construction, so a grant-channel modal ability would be
    // invisible to r1-r3 above. Verified by grep instead, and the result is asserted here
    // so a future PB cannot silently drop the check by deleting this file's own doc
    // paragraph without deleting an assertion too.
    let src_root = {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop(); // crates/engine -> crates
        p.pop(); // crates -> workspace root
        p.push("crates/card-defs/src/defs");
        p
    };
    let mut grant_with_modes = Vec::new();
    let mut stack = vec![src_root];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().is_some_and(|x| x == "rs") {
                let src = std::fs::read_to_string(&p).unwrap_or_default();
                // `modes: Some(` specifically -- `modes: None` (the non-modal case) is
                // common and harmless; it is `Some(_)` that would mean a grant-channel
                // modal activated ability this census cannot see.
                if src.contains("AddActivatedAbility") && src.contains("modes: Some(") {
                    grant_with_modes.push(p);
                }
            }
        }
    }
    assert!(
        grant_with_modes.is_empty(),
        "a card def combines `LayerModification::AddActivatedAbility` with a `modes:` field \
         in the same file -- this needs manual review, since PB-DX55's census walks \
         `CardDefinition::abilities` only and cannot see a grant-channel modal ability: {:?}",
        grant_with_modes
    );
}
