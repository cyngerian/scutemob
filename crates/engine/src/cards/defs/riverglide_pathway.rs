// Riverglide Pathway // Lavaglide Pathway — {T}: Add {U}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("riverglide-pathway"),
        name: "Riverglide Pathway // Lavaglide Pathway".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {U}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
