// Fell the Profane // Fell Mire — Destroy target creature or planeswalker.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fell-the-profane"),
        name: "Fell the Profane // Fell Mire".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target creature or planeswalker.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
