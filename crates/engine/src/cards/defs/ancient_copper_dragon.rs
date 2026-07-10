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
            // CR 706.2: Combat damage — roll d20, create that many Treasures. Any
            // token-doubling replacement (e.g. Doubling Season) applies on top of the
            // resolved roll count via the normal CreateToken chokepoint.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::RollDice {
                    sides: 20,
                    results: vec![(
                        1,
                        20,
                        Effect::CreateToken {
                            spec: TokenSpec {
                                count: EffectAmount::LastDiceRoll,
                                ..treasure_token_spec(1)
                            },
                        },
                    )],
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
