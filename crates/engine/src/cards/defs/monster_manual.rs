// Monster Manual // Zoological Study — Adventure Artifact
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("monster-manual"),
        name: "Monster Manual // Zoological Study".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{1}{G}, {T}: You may put a creature card from your hand onto the battlefield.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
