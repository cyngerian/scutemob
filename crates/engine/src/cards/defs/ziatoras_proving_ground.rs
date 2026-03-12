// Ziatora's Proving Ground
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ziatoras-proving-ground"),
        name: "Ziatora's Proving Ground".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Swamp", "Mountain", "Forest"]),
        oracle_text: "({T}: Add {B}, {R}, or {G}.)\nThis land enters tapped.\nCycling {3} ({3}, Discard this card: Draw a card.)".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {B}, {R}, or {G}.)
            // TODO: This land enters tapped.
            // TODO: Keyword — Cycling {3} ({3}, Discard this card: Draw a card.)
        ],
        ..Default::default()
    }
}
