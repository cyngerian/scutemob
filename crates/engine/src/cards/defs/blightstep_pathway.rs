// Blightstep Pathway // Searstep Pathway — {T}: Add {B}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blightstep-pathway"),
        name: "Blightstep Pathway // Searstep Pathway".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {B}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
