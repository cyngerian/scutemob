// Mystic Sanctuary
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mystic-sanctuary"),
        name: "Mystic Sanctuary".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island"]),
        oracle_text: "({T}: Add {U}.)\nThis land enters tapped unless you control three or more other Islands.\nWhen this land enters untapped, you may put target instant or sorcery card from your graveyard on top of your library.".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {U}.)
            // TODO: This land enters tapped unless you control three or more other Islands.
            // TODO: Triggered — When this land enters untapped, you may put target instant or sorcery card from 
        ],
        ..Default::default()
    }
}
