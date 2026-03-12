// Raugrin Triome
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("raugrin-triome"),
        name: "Raugrin Triome".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Plains", "Island", "Mountain"]),
        oracle_text: "({T}: Add {U}, {R}, or {W}.)\nThis land enters tapped.\nCycling {3} ({3}, Discard this card: Draw a card.)".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {U}, {R}, or {W}.)
            // TODO: This land enters tapped.
            // TODO: Keyword — Cycling {3} ({3}, Discard this card: Draw a card.)
        ],
        ..Default::default()
    }
}
