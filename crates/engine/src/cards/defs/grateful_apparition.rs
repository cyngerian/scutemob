// Grateful Apparition — {1}{W}, Creature — Spirit 1/1
// Flying
// Whenever this creature deals combat damage to a player or planeswalker, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grateful-apparition"),
        name: "Grateful Apparition".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Spirit"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player or planeswalker, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
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
