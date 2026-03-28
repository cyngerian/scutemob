// Commit // Memory — Put target spell or nonland permanent into its owner's library second
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("commit"),
        name: "Commit // Memory".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Put target spell or nonland permanent into its owner's library second from the top.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
