// Ancient Gold Dragon — {5}{W}{W}, Creature — Elder Dragon 7/10
// Flying
// Whenever this creature deals combat damage to a player, roll a d20. You create a number
// of 1/1 blue Faerie Dragon creature tokens with flying equal to the result.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-gold-dragon"),
        name: "Ancient Gold Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 5, white: 2, ..Default::default() }),
        types: creature_types(&["Elder", "Dragon"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, roll a d20. You create a number of 1/1 blue Faerie Dragon creature tokens with flying equal to the result.".to_string(),
        power: Some(7),
        toughness: Some(10),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 706.2: Combat damage — roll d20, create that many 1/1 blue Faerie
            // Dragon tokens with flying. Any token-doubling replacement (e.g. Doubling
            // Season) applies on top of the resolved roll count via the normal
            // CreateToken chokepoint.
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
                                name: "Faerie Dragon".to_string(),
                                power: 1,
                                toughness: 1,
                                colors: [Color::Blue].into_iter().collect(),
                                supertypes: OrdSet::new(),
                                card_types: [CardType::Creature].into_iter().collect(),
                                subtypes: [SubType("Faerie".to_string()), SubType("Dragon".to_string())]
                                    .into_iter()
                                    .collect(),
                                keywords: [KeywordAbility::Flying].into_iter().collect(),
                                count: EffectAmount::LastDiceRoll,
                                tapped: false,
                                enters_attacking: false,
                                mana_color: None,
                                mana_abilities: vec![],
                                activated_abilities: vec![],
                                ..Default::default()
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
