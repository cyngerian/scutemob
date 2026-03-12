// Breeding Pool
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("breeding-pool"),
        name: "Breeding Pool".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island", "Forest"]),
        oracle_text: "({T}: Add {G} or {U}.)\nAs this land enters, you may pay 2 life. If you don't, it enters tapped.".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {G} or {U}.)
            // TODO: Static — As this land enters, you may pay 2 life. If you don't, it enters tapped.
        ],
        ..Default::default()
    }
}
