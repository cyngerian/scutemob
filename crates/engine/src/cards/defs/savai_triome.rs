// Savai Triome
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("savai-triome"),
        name: "Savai Triome".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Plains", "Swamp", "Mountain"]),
        oracle_text: "({T}: Add {R}, {W}, or {B}.)\nThis land enters tapped.\nCycling {3} ({3}, Discard this card: Draw a card.)".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {R}, {W}, or {B}.)
            // TODO: This land enters tapped.
            // TODO: Keyword — Cycling {3} ({3}, Discard this card: Draw a card.)
        ],
        ..Default::default()
    }
}
