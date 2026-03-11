// Decadent Dragon — Flying, trample\nWhenever this creature attacks, create a Treasure tok
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("decadent-dragon"),
        name: "Decadent Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying, trample\nWhenever this creature attacks, create a Treasure token.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![],
        ..Default::default()
    }
}
