// Cruel Celebrant — {W}{B}, Creature — Vampire 1/2
// Whenever this creature or another creature or planeswalker you control dies, each
// opponent loses 1 life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cruel-celebrant"),
        name: "Cruel Celebrant".to_string(),
        mana_cost: Some(ManaCost { white: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever Cruel Celebrant or another creature or planeswalker you control dies, each opponent loses 1 life and you gain 1 life.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // TODO: DSL gap — "Whenever this creature or another creature or planeswalker
            // you control dies" needs WheneverCreatureDies with controller filter (You).
            // WheneverCreatureDies triggers on ANY creature dying — using it would fire
            // on opponents' creatures dying too (wrong game state per W5).
        ],
        ..Default::default()
    }
}
