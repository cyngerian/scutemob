// Brightclimb Pathway // Grimclimb Pathway — {T}: Add {W}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brightclimb-pathway"),
        name: "Brightclimb Pathway // Grimclimb Pathway".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {W}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
