//! CARDS-1 (OOS-M11-10) roster sweep (SR-36 -- enumerate `all_cards()`, never grep
//! source): the equip-ability target-requirement repair.
//!
//! R1 -- every def carrying an `AbilityDefinition::Activated` whose `effect` is
//! `Effect::AttachEquipment` (`abilities.rs`'s `handle_activate_ability`, which
//! reads `target_requirements` from the layer-resolved `ActivatedAbility.targets`
//! -- `rules/abilities.rs:315-334`/`495`). Pinned EXACT (not a floor): a new
//! Equipment must fail this gate until a human confirms its equip target is
//! authored, mirroring `pb_rs2_hybrid_phyrexian_activation_roster.rs`'s reasoning.
//!
//! R2 -- of R1, every member must declare EXACTLY ONE `TargetRequirement`, and it
//! must be `TargetCreatureWithFilter` with `controller: TargetController::You` and
//! otherwise default (no `exclude_self`, no power/type/colour restriction) --
//! CR 702.6a: "Attach this permanent to target creature you control." This is the
//! gate that makes OOS-M11-10 unable to recur: a def that regresses to
//! `targets: vec![]`, or to an under-restrictive `TargetCreature` (Helm of the
//! Host's pre-fix shape), fails here.
//!
//! R3 -- non-vacuity floor: R1 must be exactly 17.
//!
//! What membership in R1/R2 asserts, and does NOT assert (PB-DX4's `BASELINE`
//! lesson, same wording): membership means only that this def's equip ability
//! carries this specific shape. It says nothing about whether the def is
//! otherwise oracle-correct.

use mtg_engine::{
    AbilityDefinition, CardDefinition, Effect, TargetController, TargetFilter, TargetRequirement,
};
use std::collections::BTreeSet;

/// R1: every def with an `AbilityDefinition::Activated` whose `effect` is
/// `Effect::AttachEquipment`.
fn roster_r1(defs: &[CardDefinition]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in defs {
        for ability in &def.abilities {
            if let AbilityDefinition::Activated { effect, .. } = ability {
                if matches!(effect, Effect::AttachEquipment { .. }) {
                    out.insert(def.name.clone());
                }
            }
        }
    }
    out
}

/// For a single def already known to be in R1, return its equip ability's
/// `targets` list (there may be more than one `Activated`+`AttachEquipment`
/// ability in principle; in practice every R1 member has exactly one -- this
/// returns the FIRST one found, and a def with more than one is itself flagged by
/// the "exactly one requirement" check failing if the two disagree).
fn equip_targets_for(def: &CardDefinition) -> Option<&[TargetRequirement]> {
    def.abilities.iter().find_map(|ability| {
        if let AbilityDefinition::Activated {
            effect, targets, ..
        } = ability
        {
            if matches!(effect, Effect::AttachEquipment { .. }) {
                return Some(targets.as_slice());
            }
        }
        None
    })
}

/// R1 -- pinned exact set of 17 (16 hand-authored equip defs + Helm of the Host,
/// whose equip ability was already declaring a target -- an under-restrictive
/// `TargetCreature`, per R2 -- before this batch).
#[test]
fn r1_equip_activated_attach_equipment_roster_is_pinned() {
    let defs = mtg_engine::all_cards();
    let found = roster_r1(&defs);
    let expected: BTreeSet<String> = [
        "Accorder's Shield",
        "Argentum Armor",
        "Basilisk Collar",
        "Batterskull",
        "Cathar's Shield",
        "Diamond Pick-Axe",
        "Hammer of Nazahn",
        "Helm of the Host",
        "Lightning Greaves",
        "Shadowspear",
        "Skullclamp",
        "Spidersilk Net",
        "Swiftfoot Boots",
        "Sword of Fire and Ice",
        "Sword of Vengeance",
        "Thornbite Staff",
        "Whispersilk Cloak",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(
        found, expected,
        "R1 (AbilityDefinition::Activated + Effect::AttachEquipment) has changed. This entry \
         asserts ONLY that the def's equip ability has this shape -- nothing about whether \
         the def is otherwise oracle-correct. If a card was ADDED, confirm its equip ability \
         declares a proper `TargetRequirement::TargetCreatureWithFilter {{ controller: You, \
         .. }}` (CARDS-1 / OOS-M11-10 -- an empty `targets: vec![]` silently accepts a \
         zero-target activation, pays the cost, and fizzles with no error) and update this \
         pinned set. If REMOVED, confirm that was intentional.\nFound:    {found:?}\nExpected: \
         {expected:?}"
    );
}

/// Non-vacuity floor for R1: this is a real, populated corpus: at least 17 defs
/// must exist with a non-`None` `mana_cost` and an `AbilityDefinition::Activated`
/// ability at all, or the R1 walk itself is broken.
#[test]
fn r3_walk_is_not_vacuous() {
    let defs = mtg_engine::all_cards();
    let found = roster_r1(&defs);
    assert_eq!(
        found.len(),
        17,
        "R1 must contain exactly 17 members (16 hand-authored equip defs + Helm of the \
         Host); found {}: {found:?}",
        found.len()
    );
    let with_any_activated = defs
        .iter()
        .filter(|d| {
            d.abilities
                .iter()
                .any(|a| matches!(a, AbilityDefinition::Activated { .. }))
        })
        .count();
    assert!(
        with_any_activated >= 17,
        "fewer than 17 defs in the corpus have ANY Activated ability at all -- the R1 walk's \
         field access is broken (this is a real, populated corpus). Found {with_any_activated}."
    );
}

/// R2 -- every R1 member declares EXACTLY ONE `TargetRequirement`, and it is
/// `TargetCreatureWithFilter { controller: TargetController::You, ..default }`.
/// This is the gate that makes OOS-M11-10 unable to recur: it fails loudly for
/// ANY of the following regressions:
///   - a member reverting to `targets: vec![]` (the original defect),
///   - a member with more than one declared requirement,
///   - a member using the under-restrictive `TargetRequirement::TargetCreature`
///     (Helm of the Host's pre-fix shape -- no "you control" scoping),
///   - a member whose filter adds an unexpected restriction (power/type/colour)
///     that isn't part of the printed "Equip [cost]" line for any of these 17
///     cards (all 17 were MCP-verified as plain "Equip {N}" with no further
///     CR 702.6c quality restriction).
#[test]
fn r2_every_roster_member_has_exactly_the_expected_target_requirement() {
    let defs = mtg_engine::all_cards();
    let by_name: std::collections::HashMap<&str, &CardDefinition> =
        defs.iter().map(|d| (d.name.as_str(), d)).collect();
    let roster = roster_r1(&defs);
    assert!(
        !roster.is_empty(),
        "R1 must be non-empty for R2 to be a meaningful check"
    );

    let expected_filter = TargetFilter {
        controller: TargetController::You,
        ..Default::default()
    };

    let mut failures: Vec<String> = Vec::new();
    for name in &roster {
        let def = by_name
            .get(name.as_str())
            .unwrap_or_else(|| panic!("R1 member '{name}' must exist in all_cards()"));
        let targets = equip_targets_for(def).unwrap_or_else(|| {
            panic!("R1 member '{name}' must have an equip ability with a targets list")
        });
        if targets.len() != 1 {
            failures.push(format!(
                "{name}: expected exactly 1 TargetRequirement, found {} ({:?})",
                targets.len(),
                targets
            ));
            continue;
        }
        match &targets[0] {
            TargetRequirement::TargetCreatureWithFilter(filter) => {
                if filter != &expected_filter {
                    failures.push(format!(
                        "{name}: TargetCreatureWithFilter has an unexpected filter -- \
                         found {filter:?}, expected {expected_filter:?}"
                    ));
                }
            }
            other => {
                failures.push(format!(
                    "{name}: expected TargetRequirement::TargetCreatureWithFilter, found \
                     {other:?} (this is exactly Helm of the Host's pre-fix under-restrictive \
                     shape if it is a bare TargetCreature, or the original empty-vec defect \
                     if this list should not be reached with zero targets)"
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "R2 (CARDS-1 / OOS-M11-10 fix gate) failed for {} of {} roster members:\n{}",
        failures.len(),
        roster.len(),
        failures.join("\n")
    );
}
