// Scavenger Regent // Exude Toxin — Flying\nWard—Discard a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scavenger-regent"),
        name: "Scavenger Regent // Exude Toxin".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWard—Discard a card.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![],
        ..Default::default()
    }
}
