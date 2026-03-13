// Throatseeker — {2}{B}, Creature — Vampire Ninja 3/2
// Unblocked attacking Ninjas you control have lifelink.
//
// TODO: Static continuous effect giving lifelink to a subset of creatures
// (unblocked attacking Ninjas you control) requires both a creature-type filter
// AND a combat-state filter. Neither is available in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("throatseeker"),
        name: "Throatseeker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Ninja"]),
        oracle_text: "Unblocked attacking Ninjas you control have lifelink.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Unblocked attacking Ninjas you control have lifelink" — static effect
            // requiring creature-type filter (Ninja) and combat-state filter (unblocked attacker).
            // DSL gap: no such filter combination exists in ContinuousEffectDef.
        ],
        ..Default::default()
    }
}
