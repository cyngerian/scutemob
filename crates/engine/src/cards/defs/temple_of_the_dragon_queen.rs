// Temple of the Dragon Queen
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("temple-of-the-dragon-queen"),
        name: "Temple of the Dragon Queen".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, you may reveal a Dragon card from your hand. This land enters tapped unless you revealed a Dragon card this way or you control a Dragon.\nAs this land enters, choose a color.\n{T}: Add one mana of the chosen color.".to_string(),
        abilities: vec![
            // TODO: As this land enters, you may reveal a Dragon card from your hand. This land ente
            // TODO: As this land enters, choose a color.
            // TODO: Activated — {T}: Add one mana of the chosen color.
        ],
        ..Default::default()
    }
}
