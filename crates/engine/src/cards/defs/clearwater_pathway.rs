// Clearwater Pathway // Murkwater Pathway — {T}: Add {U}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("clearwater-pathway"),
        name: "Clearwater Pathway // Murkwater Pathway".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {U}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
