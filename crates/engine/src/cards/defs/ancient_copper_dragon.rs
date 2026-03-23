// Ancient Copper Dragon — {4}{R}{R}, Creature — Elder Dragon 6/5
// Flying
// Whenever this creature deals combat damage to a player, roll a d20. You create a number
// of Treasure tokens equal to the result.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-copper-dragon"),
        name: "Ancient Copper Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: creature_types(&["Elder", "Dragon"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, roll a d20. You create a number of Treasure tokens equal to the result.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Combat damage: roll d20, create that many Treasures
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::CreateToken { spec: treasure_token_spec(10) },
                // TODO: d20 roll for variable count — using fixed 10 as average approximation.
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
