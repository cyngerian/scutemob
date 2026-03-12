// Steam Vents
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("steam-vents"),
        name: "Steam Vents".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island", "Mountain"]),
        oracle_text: "({T}: Add {U} or {R}.)\nAs this land enters, you may pay 2 life. If you don't, it enters tapped.".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {U} or {R}.)
            // TODO: Static — As this land enters, you may pay 2 life. If you don't, it enters tapped.
        ],
        ..Default::default()
    }
}
