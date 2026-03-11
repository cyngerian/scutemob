// Darkbore Pathway // Slitherbore Pathway — {T}: Add {B}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("darkbore-pathway"),
        name: "Darkbore Pathway // Slitherbore Pathway".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {B}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
