// Crypt of Agadeem
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crypt-of-agadeem"),
        name: "Crypt of Agadeem".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {B}.\n{2}, {T}: Add {B} for each black creature card in your graveyard.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped.
            // TODO: Activated — {T}: Add {B}.
            // TODO: Activated — {2}, {T}: Add {B} for each black creature card in your graveyard.
        ],
        ..Default::default()
    }
}
