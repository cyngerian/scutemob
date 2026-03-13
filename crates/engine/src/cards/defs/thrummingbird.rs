// Thrummingbird — {1}{U}, Creature — Phyrexian Bird Horror 1/1
// Flying
// Whenever this creature deals combat damage to a player, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thrummingbird"),
        name: "Thrummingbird".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Bird", "Horror"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::Proliferate,
                intervening_if: None,
            },
        ],
        ..Default::default()
    }
}
