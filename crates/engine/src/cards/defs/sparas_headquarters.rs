// Spara's Headquarters
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sparas-headquarters"),
        name: "Spara's Headquarters".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Plains", "Island", "Forest"]),
        oracle_text: "({T}: Add {G}, {W}, or {U}.)\nThis land enters tapped.\nCycling {3} ({3}, Discard this card: Draw a card.)".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {G}, {W}, or {U}.)
            // TODO: This land enters tapped.
            // TODO: Keyword — Cycling {3} ({3}, Discard this card: Draw a card.)
        ],
        ..Default::default()
    }
}
