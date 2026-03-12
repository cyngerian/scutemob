// Overgrown Tomb
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("overgrown-tomb"),
        name: "Overgrown Tomb".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Swamp", "Forest"]),
        oracle_text: "({T}: Add {B} or {G}.)\nAs this land enters, you may pay 2 life. If you don't, it enters tapped.".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {B} or {G}.)
            // TODO: Static — As this land enters, you may pay 2 life. If you don't, it enters tapped.
        ],
        ..Default::default()
    }
}
