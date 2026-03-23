// Badgermole Cub — {1}{G}, Creature — Badger Mole 2/2
// When this creature enters, earthbend 1.
// Whenever you tap a creature for mana, add an additional {G}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("badgermole-cub"),
        name: "Badgermole Cub".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Badger", "Mole"]),
        oracle_text: "When this creature enters, earthbend 1. (Target land you control becomes a 0/0 creature with haste that's still a land. Put a +1/+1 counter on it. When it dies or is exiled, return it to the battlefield tapped.)\nWhenever you tap a creature for mana, add an additional {G}.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: DSL gap — Earthbend (keyword action) not in DSL.
            // TODO: DSL gap — "Whenever you tap a creature for mana, add an additional {G}."
            // Mana ability trigger not in DSL.
        ],
        ..Default::default()
    }
}
