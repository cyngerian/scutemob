// Norn's Choirmaster — {3}{W}{W}, Creature — Phyrexian Angel 5/4
// Flying, first strike
// Whenever a commander you control enters or attacks, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("norns-choirmaster"),
        name: "Norn's Choirmaster".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 2, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Angel"]),
        oracle_text: "Flying, first strike\nWhenever a commander you control enters or attacks, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            // TODO: "Whenever a commander you control enters or attacks, proliferate" —
            // no TriggerCondition for "when a commander you control enters the battlefield"
            // or "when a commander you control attacks" exists in the DSL. The Designations
            // bitfield tracks commander status but there is no trigger hook for it.
            // Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
