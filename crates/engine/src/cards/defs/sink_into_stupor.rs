// Sink into Stupor // Soporific Springs — Return target spell or nonland permanent an opponent controls to its o
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sink-into-stupor"),
        name: "Sink into Stupor // Soporific Springs".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return target spell or nonland permanent an opponent controls to its owner's hand.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
