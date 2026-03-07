// Predator Dragon — {3}{R}{R}{R}, Creature — Dragon 4/4; Flying, Haste, Devour 2
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("predator-dragon"),
        name: "Predator Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 3, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying, haste\nDevour 2 (As this creature enters, you may sacrifice any number of creatures. It enters with twice that many +1/+1 counters on it.)".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Keyword(KeywordAbility::Devour(2)),
        ],
    }
}
