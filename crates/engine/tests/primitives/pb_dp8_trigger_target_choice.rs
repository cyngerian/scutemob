//! PB-DP8 (DP-6 / OOS-M11-4) — triggered-ability targets become a player choice.
//!
//! CR 603.3d: "The remainder of the process for putting a triggered ability on the
//! stack is identical to the process for casting a spell listed in rules 601.2c-d."
//! CR 601.2c: "The player announces their choice of an appropriate object or player
//! for each target the spell requires."
//! CR 603.3b: the CR 603.3b batch is placed in APNAP order, one ability at a time.

use mtg_card_defs::all_cards;
use mtg_card_types::cards::card_definition::{AbilityDefinition, Completeness};

/// CR 603.3d / SR-36 — the PB-DP8 roster, derived by enumerating `all_cards()`
/// rather than by grepping source.
///
/// A def is in the roster iff some `AbilityDefinition::Triggered` on **any** of its
/// faces (front, `back_face`, `adventure_face`) declares a non-empty `targets`, and
/// the def is `Completeness::Complete` (i.e. legal in a deck, per SR-2). Those are
/// exactly the defs whose trigger reaches
/// `rules::abilities::flush_pending_triggers`'s CR 603.3d announcement.
///
/// The assertion is `>=` on purpose: the authoring campaign adds cards continuously
/// and an `==` pin would redden on unrelated work.
#[test]
fn test_dp8_roster_enumeration() {
    fn has_targeted_trigger(abilities: &[AbilityDefinition]) -> bool {
        abilities.iter().any(|a| {
            matches!(a, AbilityDefinition::Triggered { targets, .. } if !targets.is_empty())
        })
    }

    let mut roster: Vec<String> = Vec::new();
    let mut incomplete = 0usize;
    for def in all_cards() {
        let mut hit = has_targeted_trigger(&def.abilities);
        if let Some(face) = def.back_face.as_ref() {
            hit |= has_targeted_trigger(&face.abilities);
        }
        if let Some(face) = def.adventure_face.as_ref() {
            hit |= has_targeted_trigger(&face.abilities);
        }
        if !hit {
            continue;
        }
        if def.completeness == Completeness::Complete {
            roster.push(def.name.clone());
        } else {
            incomplete += 1;
        }
    }
    roster.sort();
    println!(
        "PB-DP8 roster: {} effectively-Complete defs with a targeted triggered ability \
         ({} more carry a non-Complete marker)",
        roster.len(),
        incomplete
    );
    for name in &roster {
        println!("  {name}");
    }
    assert!(
        roster.len() >= 60,
        "PB-DP8 roster collapsed to {} defs (expected >= 60)",
        roster.len()
    );
}
