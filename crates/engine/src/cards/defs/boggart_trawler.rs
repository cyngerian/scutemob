// Boggart Trawler // Boggart Bog — When this creature enters, exile target player's graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boggart-trawler"),
        name: "Boggart Trawler // Boggart Bog".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "When this creature enters, exile target player's graveyard.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}
