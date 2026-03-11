// Bridgeworks Battle // Tanglespan Bridgeworks — Target creature you control gets +2/+2 until end of turn. It fights up
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bridgeworks-battle"),
        name: "Bridgeworks Battle // Tanglespan Bridgeworks".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Target creature you control gets +2/+2 until end of turn. It fights up to one target creature you don't control. (Each deals damage equal to its power to the other.)".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
