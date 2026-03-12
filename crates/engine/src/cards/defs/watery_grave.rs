// Watery Grave
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("watery-grave"),
        name: "Watery Grave".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island", "Swamp"]),
        oracle_text: "({T}: Add {U} or {B}.)\nAs this land enters, you may pay 2 life. If you don't, it enters tapped.".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {U} or {B}.)
            // TODO: Static — As this land enters, you may pay 2 life. If you don't, it enters tapped.
        ],
        ..Default::default()
    }
}
