// Invisible Stalker — {1}{U}, Creature — Human Rogue 1/1
// "Hexproof (This creature can't be the target of spells or abilities your opponents control.)
// This creature can't be blocked."
//
// Hexproof is implemented.
//
// TODO: DSL gap — "This creature can't be blocked" requires a CantBeBlocked static keyword
// or continuous effect. No such KeywordAbility variant exists in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("invisible-stalker"),
        name: "Invisible Stalker".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Rogue"]),
        oracle_text: "Hexproof (This creature can't be the target of spells or abilities your opponents control.)\nThis creature can't be blocked.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Hexproof),
        ],
        ..Default::default()
    }
}
