// Consign // Oblivion — Return target nonland permanent to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("consign"),
        name: "Consign // Oblivion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant, CardType::Sorcery]),
        oracle_text: "Return target nonland permanent to its owner's hand.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
