// Tormented Soul — {B}, Creature — Spirit 1/1
// Tormented Soul can't block and can't be blocked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tormented-soul"),
        name: "Tormented Soul".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Spirit"]),
        oracle_text: "Tormented Soul can't block and can't be blocked.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBlock),
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
        ],
        ..Default::default()
    }
}
