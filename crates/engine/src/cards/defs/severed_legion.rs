// 58. Severed Legion — {1BB}, Creature — Zombie 2/2; Fear.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("severed-legion"),
        name: "Severed Legion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text: "Fear (This creature can't be blocked except by artifact creatures and/or black creatures.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Fear),
        ],
        back_face: None,
    }
}
