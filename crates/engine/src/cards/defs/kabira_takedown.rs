// Kabira Takedown // Kabira Plateau — Kabira Takedown deals damage equal to the number of creatures you cont
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kabira-takedown"),
        name: "Kabira Takedown // Kabira Plateau".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Kabira Takedown deals damage equal to the number of creatures you control to target creature or planeswalker.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
