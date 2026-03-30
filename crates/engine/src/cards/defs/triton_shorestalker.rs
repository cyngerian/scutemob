// Triton Shorestalker — {U}, Creature — Merfolk Rogue 1/1
// Triton Shorestalker can't be blocked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("triton-shorestalker"),
        name: "Triton Shorestalker".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Merfolk", "Rogue"]),
        oracle_text: "Triton Shorestalker can't be blocked.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
        ],
        ..Default::default()
    }
}
