// Witch Enchanter // Witch-Blessed Meadow — When this creature enters, destroy target artifact or enchantment an o
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("witch-enchanter"),
        name: "Witch Enchanter // Witch-Blessed Meadow".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Warlock"]),
        oracle_text: "When this creature enters, destroy target artifact or enchantment an opponent controls.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}
