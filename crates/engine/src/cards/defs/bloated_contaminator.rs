// Bloated Contaminator — {2}{G}, Creature — Phyrexian Beast 4/4
// Trample
// Toxic 1 (Players dealt combat damage by this creature also get a poison counter.)
// Whenever this creature deals combat damage to a player, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloated-contaminator"),
        name: "Bloated Contaminator".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Beast"]),
        oracle_text: "Trample\nToxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nWhenever this creature deals combat damage to a player, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::Proliferate,
                intervening_if: None,
            },
        ],
        ..Default::default()
    }
}
