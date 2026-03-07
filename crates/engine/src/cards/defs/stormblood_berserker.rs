// Stormblood Berserker — {1}{R}, Creature — Human Berserker 1/1; Bloodthirst 2, Menace
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stormblood-berserker"),
        name: "Stormblood Berserker".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Human", "Berserker"]),
        oracle_text: "Bloodthirst 2 (If an opponent was dealt damage this turn, this creature enters with two +1/+1 counters on it.)\nMenace (This creature can't be blocked except by two or more creatures.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Bloodthirst(2)),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
        ],
    }
}
