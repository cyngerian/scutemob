// Dreamtide Whale — {2}{U}, Creature — Whale 7/5
// Vanishing 2
// Whenever a player casts their second spell each turn, proliferate.
//
// Vanishing 2 is fully supported. "Second spell each turn" trigger is a DSL gap.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dreamtide-whale"),
        name: "Dreamtide Whale".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Whale"]),
        oracle_text: "Vanishing 2 (This creature enters with two time counters on it. At the beginning of your upkeep, remove a time counter from it. When the last is removed, sacrifice it.)\nWhenever a player casts their second spell each turn, proliferate.".to_string(),
        power: Some(7),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vanishing(2)),
            // TODO: "second spell each turn" trigger — no TriggerCondition for
            // WheneverPlayerCastsNthSpellThisTurn. Proliferate effect works but
            // trigger condition is blocked.
        ],
        ..Default::default()
    }
}
