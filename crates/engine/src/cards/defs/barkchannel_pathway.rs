// Barkchannel Pathway // Tidechannel Pathway — {T}: Add {G}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("barkchannel-pathway"),
        name: "Barkchannel Pathway // Tidechannel Pathway".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {G}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
