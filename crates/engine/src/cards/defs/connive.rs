// Connive // Concoct — Gain control of target creature with power 2 or less.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("connive"),
        name: "Connive // Concoct".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Gain control of target creature with power 2 or less.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
