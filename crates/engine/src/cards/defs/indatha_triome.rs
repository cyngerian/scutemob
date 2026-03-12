// Indatha Triome
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("indatha-triome"),
        name: "Indatha Triome".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Plains", "Swamp", "Forest"]),
        oracle_text: "({T}: Add {W}, {B}, or {G}.)\nThis land enters tapped.\nCycling {3} ({3}, Discard this card: Draw a card.)".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {W}, {B}, or {G}.)
            // TODO: This land enters tapped.
            // TODO: Keyword — Cycling {3} ({3}, Discard this card: Draw a card.)
        ],
        ..Default::default()
    }
}
