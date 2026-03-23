// Sanctum Seeker — {2}{B}{B}, Creature — Vampire Knight 3/4
// Whenever a Vampire you control attacks, each opponent loses 1 life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sanctum-seeker"),
        name: "Sanctum Seeker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Knight"]),
        oracle_text: "Whenever a Vampire you control attacks, each opponent loses 1 life and you gain 1 life.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // TODO: DSL gap — "Whenever a Vampire you control attacks" requires a
            // creature-type-filtered attack trigger (WheneverCreatureWithSubtypeAttacks)
            // which does not exist in the DSL.
        ],
        ..Default::default()
    }
}
