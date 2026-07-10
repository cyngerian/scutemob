// Changeling Outcast — {B}, Creature — Shapeshifter 1/1
// Changeling (This card is every creature type.)
// Changeling Outcast can't block and can't be blocked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("changeling-outcast"),
        name: "Changeling Outcast".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Shapeshifter"]),
        oracle_text: "Changeling (This card is every creature type.)\nChangeling Outcast can't block and can't be blocked.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Changeling),
            AbilityDefinition::Keyword(KeywordAbility::CantBlock),
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
        ],
        ..Default::default()
    }
}
