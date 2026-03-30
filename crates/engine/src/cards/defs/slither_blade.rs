// Slither Blade — {U}, Creature — Snake Rogue 1/2
// Slither Blade can't be blocked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("slither-blade"),
        name: "Slither Blade".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Snake", "Rogue"]),
        oracle_text: "Slither Blade can't be blocked.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
        ],
        ..Default::default()
    }
}
