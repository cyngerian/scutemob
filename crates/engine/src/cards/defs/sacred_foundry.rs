// Sacred Foundry
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sacred-foundry"),
        name: "Sacred Foundry".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Plains", "Mountain"]),
        oracle_text: "({T}: Add {R} or {W}.)\nAs this land enters, you may pay 2 life. If you don't, it enters tapped.".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {R} or {W}.)
            // TODO: Static — As this land enters, you may pay 2 life. If you don't, it enters tapped.
        ],
        ..Default::default()
    }
}
