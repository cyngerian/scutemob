// Monster Manual — {1}{G}, {T}: You may put a creature card from your hand onto the battl
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("monster-manual"),
        name: "Monster Manual".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery, CardType::Artifact]),
        oracle_text: "{1}{G}, {T}: You may put a creature card from your hand onto the battlefield.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
