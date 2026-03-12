// Hallowed Fountain
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hallowed-fountain"),
        name: "Hallowed Fountain".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Plains", "Island"]),
        oracle_text: "({T}: Add {W} or {U}.)\nAs this land enters, you may pay 2 life. If you don't, it enters tapped.".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {W} or {U}.)
            // TODO: Static — As this land enters, you may pay 2 life. If you don't, it enters tapped.
        ],
        ..Default::default()
    }
}
