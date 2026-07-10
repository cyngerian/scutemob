// Mist-Cloaked Herald — {U}, Creature — Merfolk Warrior 1/1
// Mist-Cloaked Herald can't be blocked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mist-cloaked-herald"),
        name: "Mist-Cloaked Herald".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Merfolk", "Warrior"]),
        oracle_text: "Mist-Cloaked Herald can't be blocked.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
        ],
        ..Default::default()
    }
}
