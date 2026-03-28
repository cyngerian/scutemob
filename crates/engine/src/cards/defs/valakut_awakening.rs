// Valakut Awakening // Valakut Stoneforge — Put any number of cards from your hand on the bottom of your library,
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("valakut-awakening"),
        name: "Valakut Awakening // Valakut Stoneforge".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Put any number of cards from your hand on the bottom of your library, then draw that many cards plus one.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
