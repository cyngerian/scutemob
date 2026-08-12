//! CARDS-1 (OOS-M11-10) roster sweep (SR-36 -- enumerate `all_cards()`, never grep
//! source): the equip-ability target-requirement repair.
//!
//! > **PB-DX26 re-pinned R1 from 17 to 38 and made its `Effect` match recursive.**
//! > Read "R1 is green" as covering exactly the defs listed in R1 and nothing
//! > else: the complementary "no `Equip`-marker def lacks an ability" property is
//! > `pb_dx26_attach_keyword_roster::r2`, and the type-line-derived census that no
//! > keyword-derived roster can see is that file's R4.
//!
//! R1 -- every def carrying an `AbilityDefinition::Activated` whose `effect` reaches
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
//! R3 -- non-vacuity floor: R1 must be exactly 38 (17 + PB-DX26's 21).
//!
//! What membership in R1/R2 asserts, and does NOT assert (PB-DX4's `BASELINE`
//! lesson, same wording): membership means only that this def's equip ability
//! carries this specific shape. It says nothing about whether the def is
//! otherwise oracle-correct.

use mtg_engine::{
    AbilityDefinition, CardDefinition, Effect, TargetController, TargetFilter, TargetRequirement,
};
use std::collections::BTreeSet;

/// Does this effect tree reach an `Effect::AttachEquipment`?
///
/// **PB-DX26 made this recursive.** It was a flat `matches!`, so a def nesting its
/// attach inside an `Effect::Sequence` dropped out of R1's exact pin **silently** —
/// the hazard `memory/primitives/seed-rerank-2026-08-02.md` §2.7 names about this
/// exact line and about `cards1_equip_target_repair.rs:541`. Every `Box<Effect>` /
/// `Vec<Effect>` nesting site in the `Effect` enum is walked; the site list itself
/// is pinned by `core::pb_dx26_attach_keyword_roster::r6`, so this walk cannot
/// silently go shallow when a new nesting variant is added.
fn reaches_attach_equipment(effect: &Effect) -> bool {
    if matches!(effect, Effect::AttachEquipment { .. }) {
        return true;
    }
    match effect {
        Effect::Sequence(inner) => inner.iter().any(reaches_attach_equipment),
        Effect::Conditional {
            if_true, if_false, ..
        } => reaches_attach_equipment(if_true) || reaches_attach_equipment(if_false),
        Effect::Repeat { effect, .. } => reaches_attach_equipment(effect),
        Effect::ForEach { effect, .. } => reaches_attach_equipment(effect),
        Effect::Choose { choices, .. } => choices.iter().any(reaches_attach_equipment),
        Effect::MayPayOrElse { or_else, .. } => reaches_attach_equipment(or_else),
        Effect::MayPayThenEffect { then, .. } => reaches_attach_equipment(then),
        Effect::CoinFlip {
            on_win, on_lose, ..
        } => reaches_attach_equipment(on_win) || reaches_attach_equipment(on_lose),
        _ => false,
    }
}

/// R1: every def with an `AbilityDefinition::Activated` whose `effect` reaches
/// `Effect::AttachEquipment` (see `reaches_attach_equipment` on why "reaches"
/// rather than "is").
fn roster_r1(defs: &[CardDefinition]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in defs {
        for ability in &def.abilities {
            if let AbilityDefinition::Activated { effect, .. } = ability {
                if reaches_attach_equipment(effect) {
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
            if reaches_attach_equipment(effect) {
                return Some(targets.as_slice());
            }
        }
        None
    })
}

/// R1 -- pinned exact set of **38**.
///
/// **Re-pinned 17 -> 38 by PB-DX26 (`OOS-CARDS1-3`).** The original 17 were the
/// defs that already had an `Activated` + `AttachEquipment` ability and were
/// missing only the CR 702.6a *target* (16 hand-authored + Helm of the Host, whose
/// declared requirement was an under-restrictive bare `TargetCreature`). The 21
/// added here had no equip **ability** at all -- only an
/// `AbilityDefinition::Keyword(KeywordAbility::Equip)` marker, which
/// `keyword_registry.rs` classifies as `KeywordHandling::Marker` and which
/// therefore synthesises nothing.
///
/// **This pin was the hazard `OOS-CARDS1-3` was filed to prevent**: at 17 it read
/// as "all the equip defs are correct" while 21 more had no equip at all. It is
/// exact-pinned precisely so that a reader cannot mistake its greenness for
/// coverage of a population it never enumerated. The complementary property --
/// "no `Equip`-marker def lacks a reachable ability" -- is `pb_dx26_attach_
/// keyword_roster::r2`, and the type-line-derived census that neither of these
/// keyword-derived rosters can see is that file's R4.
#[test]
fn r1_equip_activated_attach_equipment_roster_is_pinned() {
    let defs = mtg_engine::all_cards();
    let found = roster_r1(&defs);
    let expected: BTreeSet<String> = [
        // -- CARDS-1's original 17 (target authored into an existing ability) ----
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
        // -- PB-DX26's 21 (the whole ability authored; marker-only before) -------
        "Blackblade Reforged",
        "Blade of the Bloodchief",
        "Bone Saw",
        "Commander's Plate",
        "Empyrial Plate",
        "Glimmer Lens",
        "Illusionist's Bracers",
        "Kite Shield",
        "Mask of Memory",
        "Paradise Mantle",
        "Sword of Body and Mind",
        "Sword of Feast and Famine",
        "Sword of Light and Shadow",
        "Sword of Sinew and Steel",
        "Sword of the Animist",
        "Sword of the Paruns",
        "Sword of Truth and Justice",
        "Sword of War and Peace",
        "The Reaver Cleaver",
        "Umbral Mantle",
        "Umezawa's Jitte",
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
        38,
        "R1 must contain exactly 38 members (CARDS-1's 17 + PB-DX26's 21 marker-only defs); \
         found {}: {found:?}",
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
