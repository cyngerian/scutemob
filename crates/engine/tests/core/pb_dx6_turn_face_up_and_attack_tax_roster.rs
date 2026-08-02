//! PB-DX6 roster sweep (SR-36 — enumerate `all_cards()`, never grep source): the two
//! payment-path shapes this batch fixed.
//!
//! R1/R2 — `Command::TurnFaceUp` (`handle_turn_face_up`, `rules/engine.rs`): a
//! hybrid/Phyrexian pip can reach this special-action payment path from either of
//! two independent sources —
//!   - R1: the printed `mana_cost` of a manifested/cloaked creature card
//!     (`TurnFaceUpMethod::ManaCost`, CR 701.40b/701.40g)
//!   - R2: a `Morph`/`Megamorph`/`Disguise` ability's own cost
//!     (`TurnFaceUpMethod::MorphCost`/`DisguiseCost`, CR 702.37e/702.168d)
//!
//! Both are pinned separately because they are read from different fields of the
//! `CardDefinition` and neither implies the other.
//!
//! R3/R4 — `Command::DeclareAttackers`'s CR 508.1h attack tax
//! (`AbilityDefinition::StaticRestriction { restriction: GameRestriction::
//! CantAttackYouUnlessPay { cost_per_creature } }`, `rules/combat.rs`'s
//! `accumulate_attack_tax_total` / `queries::attack_tax_total`):
//!   - R3: every def that produces the restriction at all
//!   - R4: of those, the subset whose `cost_per_creature` carries a hybrid/Phyrexian
//!     pip OR a nonzero `x_count` — the only members that exercise the fix, since a
//!     plain-generic-only `cost_per_creature` was already payable before this batch.
//!
//! All four sets are pinned EXACT (not a floor), matching
//! `pb_rs2_hybrid_phyrexian_activation_roster.rs`'s reasoning: none of these four
//! shapes is an actively-growing authoring target, so the next card that adds one
//! should fail this test until a human confirms its cost is actually charged (the
//! whole point of the residue guard in `crates/card-types/src/state/player.rs` and
//! this sweep working together).
//!
//! What an entry in any of these sets asserts, and does NOT assert (PB-DX4's
//! `BASELINE` lesson, same wording): membership means only that this def's cost
//! carries a pip (or, for R4, an X) at this specific site. It says nothing about
//! whether the def is otherwise oracle-correct.

use mtg_engine::{AbilityDefinition, CardDefinition, CardType, GameRestriction};
use std::collections::BTreeSet;

fn has_pip(cost: &mtg_engine::ManaCost) -> bool {
    !cost.hybrid.is_empty() || !cost.phyrexian.is_empty()
}

/// R1: printed `mana_cost` carries a pip AND the def is a creature — the
/// `TurnFaceUpMethod::ManaCost` roster (CR 701.40b).
fn roster_r1(defs: &[CardDefinition]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in defs {
        let is_creature = def.types.card_types.contains(&CardType::Creature);
        if is_creature {
            if let Some(mc) = &def.mana_cost {
                if has_pip(mc) {
                    out.insert(def.name.clone());
                }
            }
        }
    }
    out
}

/// R2: a `Morph`/`Megamorph`/`Disguise` ability's own cost carries a pip — the
/// `TurnFaceUpMethod::MorphCost`/`DisguiseCost` roster.
fn roster_r2(defs: &[CardDefinition]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in defs {
        for ability in &def.abilities {
            let cost = match ability {
                AbilityDefinition::Morph { cost } => Some(cost),
                AbilityDefinition::Megamorph { cost } => Some(cost),
                AbilityDefinition::Disguise { cost } => Some(cost),
                _ => None,
            };
            if let Some(cost) = cost {
                if has_pip(cost) {
                    out.insert(def.name.clone());
                }
            }
        }
    }
    out
}

/// R3: every def producing `GameRestriction::CantAttackYouUnlessPay` at all (CR
/// 508.1h).
fn roster_r3(defs: &[CardDefinition]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in defs {
        for ability in &def.abilities {
            if let AbilityDefinition::StaticRestriction { restriction } = ability {
                if matches!(restriction, GameRestriction::CantAttackYouUnlessPay { .. }) {
                    out.insert(def.name.clone());
                }
            }
        }
    }
    out
}

/// R4: of R3, those whose `cost_per_creature` has a pip OR `x_count > 0` — the
/// subset that actually exercises the PB-DX6 fix.
fn roster_r4(defs: &[CardDefinition]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in defs {
        for ability in &def.abilities {
            if let AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::CantAttackYouUnlessPay { cost_per_creature },
            } = ability
            {
                if has_pip(cost_per_creature) || cost_per_creature.x_count > 0 {
                    out.insert(def.name.clone());
                }
            }
        }
    }
    out
}

fn names(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| s.to_string()).collect()
}

/// R1 — pinned exact set of 5. `blade_historian` declares NO `completeness` field
/// at all (`crates/card-defs/src/defs/blade_historian.rs`) and is therefore
/// `Complete` only by the `Completeness` `#[default]` derive, not an explicit
/// author decision — the twice-demonstrated silent-defect generator PB-DX3b and
/// PB-DX4 both hit. Recorded here so a future reader does not mistake three
/// explicit markers for three decisions. `kitchen_finks`, `boggart_ram_gang`,
/// `deathrite_shaman`, `vexing_shusher` all declare an explicit `completeness`
/// marker.
#[test]
fn r1_turn_face_up_mana_cost_pip_roster_is_pinned() {
    let defs = mtg_engine::all_cards();
    let found = roster_r1(&defs);
    let expected = names(&[
        "Kitchen Finks",
        "Blade Historian",
        "Boggart Ram-Gang",
        "Deathrite Shaman",
        "Vexing Shusher",
    ]);
    assert_eq!(
        found, expected,
        "R1 (TurnFaceUpMethod::ManaCost, hybrid/Phyrexian pip in printed mana_cost of a \
         creature) has changed. This entry asserts ONLY that the def's printed cost carries \
         a pip -- nothing about whether the def is otherwise oracle-correct. If a card was \
         ADDED, confirm its manifest/cloak-turn-face-up cost is actually charged (PB-DX6 \
         fixed `handle_turn_face_up`'s payment path; the player.rs residue guard will \
         panic in debug tests if it isn't flattened before reaching \
         ManaPool::can_spend/spend) and update this pinned set. If REMOVED, confirm that \
         was intentional.\nFound:    {found:?}\nExpected: {expected:?}"
    );
}

/// R2 — pinned exact EMPTY set: no `Morph`/`Megamorph`/`Disguise` ability in the
/// corpus carries a hybrid/Phyrexian pip in its own cost today. Guarded by the
/// non-vacuity floor below.
#[test]
fn r2_morph_megamorph_disguise_pip_roster_is_pinned_empty() {
    let defs = mtg_engine::all_cards();
    let found = roster_r2(&defs);
    let expected: BTreeSet<String> = BTreeSet::new();
    assert_eq!(
        found, expected,
        "R2 (Morph/Megamorph/Disguise cost, hybrid/Phyrexian pip) has changed from the \
         pinned empty set. This entry asserts ONLY that the def's morph-family cost \
         carries a pip -- nothing about whether the def is otherwise oracle-correct. If a \
         card was ADDED, confirm its turn-face-up cost is actually charged (PB-DX6 fixed \
         `handle_turn_face_up`'s payment path for this arm too) and update this pinned \
         set.\nFound:    {found:?}\nExpected: {expected:?}"
    );
}

/// Non-vacuity floor for R2: an exact-set assertion where both sides are empty
/// passes silently even if the walk itself is broken. At least one
/// Morph/Megamorph/Disguise ability must exist somewhere in the corpus (7 defs
/// carry one per the plan) or this floor fails loudly instead.
#[test]
fn r2_walk_is_not_vacuous() {
    let defs = mtg_engine::all_cards();
    let mut seen = 0usize;
    for def in &defs {
        for ability in &def.abilities {
            if matches!(
                ability,
                AbilityDefinition::Morph { .. }
                    | AbilityDefinition::Megamorph { .. }
                    | AbilityDefinition::Disguise { .. }
            ) {
                seen += 1;
            }
        }
    }
    assert!(
        seen >= 1,
        "no Morph/Megamorph/Disguise abilities found anywhere in the corpus -- the R2 walk \
         is broken (this is a real, populated corpus; PB-DX6's R2 roster gate would be \
         vacuous). Found {seen}."
    );
}

/// Non-vacuity floor for R1: at least one def in the corpus must have a non-`None`
/// `mana_cost` at all, or the R1 walk's field access is broken.
#[test]
fn r1_walk_is_not_vacuous() {
    let defs = mtg_engine::all_cards();
    let with_mana_cost = defs.iter().filter(|d| d.mana_cost.is_some()).count();
    assert!(
        with_mana_cost >= 1,
        "no def in the corpus has a non-None mana_cost -- the R1 walk is broken (this is a \
         real, populated corpus). Found {with_mana_cost}."
    );
}

/// R3 — pinned exact set of 2 (CR 508.1h).
#[test]
fn r3_cant_attack_you_unless_pay_roster_is_pinned() {
    let defs = mtg_engine::all_cards();
    let found = roster_r3(&defs);
    let expected = names(&["Propaganda", "Ghostly Prison"]);
    assert_eq!(
        found, expected,
        "R3 (GameRestriction::CantAttackYouUnlessPay) has changed. This entry asserts ONLY \
         that the def produces this restriction at all -- nothing about whether the def is \
         otherwise oracle-correct. If a card was ADDED, confirm its \
         cost_per_creature is actually charged at declare-attackers time via \
         `rules::queries::attack_tax_total` and update this pinned set. If REMOVED, confirm \
         that was intentional.\nFound:    {found:?}\nExpected: {expected:?}"
    );
}

/// R4 — pinned exact EMPTY set: neither Propaganda's nor Ghostly Prison's
/// `cost_per_creature` carries a pip or a nonzero x_count today (both are a flat
/// `{2}`). Guarded by the non-vacuity floor below (R3 must be non-empty).
#[test]
fn r4_pip_or_x_attack_tax_roster_is_pinned_empty() {
    let defs = mtg_engine::all_cards();
    let found = roster_r4(&defs);
    let expected: BTreeSet<String> = BTreeSet::new();
    assert_eq!(
        found, expected,
        "R4 (of R3, cost_per_creature has a pip or x_count > 0) has changed from the pinned \
         empty set. This entry asserts ONLY that the def's attack-tax cost_per_creature \
         carries a pip or an X -- nothing about whether the def is otherwise \
         oracle-correct. If a card was ADDED, confirm the hybrid/Phyrexian/X attack tax is \
         actually charged (PB-DX6's copy-major replication + \
         `accumulate_attack_tax_total`/`queries::attack_tax_total`) and update this pinned \
         set.\nFound:    {found:?}\nExpected: {expected:?}"
    );
}

/// Non-vacuity floor for R4: R3 must be non-empty, or an empty R4 would be
/// trivially and uninformatively true.
#[test]
fn r4_walk_is_not_vacuous() {
    let defs = mtg_engine::all_cards();
    let r3 = roster_r3(&defs);
    assert!(
        !r3.is_empty(),
        "R3 (GameRestriction::CantAttackYouUnlessPay) is empty -- R4's pinned-empty \
         assertion would be vacuously true rather than a real check. This is a real, \
         populated corpus (Propaganda, Ghostly Prison); the R3 walk is broken."
    );
}
