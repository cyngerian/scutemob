// Cragcrown Pathway // Timbercrown Pathway — {T}: Add {R}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cragcrown-pathway"),
        name: "Cragcrown Pathway // Timbercrown Pathway".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {R}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
