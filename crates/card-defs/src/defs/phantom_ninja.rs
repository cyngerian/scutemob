// Phantom Ninja — {1}{U}{U}, Creature — Illusion Ninja 2/2
// Phantom Ninja can't be blocked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phantom-ninja"),
        name: "Phantom Ninja".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: creature_types(&["Illusion", "Ninja"]),
        oracle_text: "Phantom Ninja can't be blocked.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
        ],
        ..Default::default()
    }
}
