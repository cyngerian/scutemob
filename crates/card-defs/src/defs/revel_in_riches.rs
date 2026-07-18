// Revel in Riches — {4}{B}, Enchantment
// Whenever a creature an opponent controls dies, create a Treasure token.
// At the beginning of your upkeep, if you control ten or more Treasures, you win the game.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("revel-in-riches"),
        name: "Revel in Riches".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature an opponent controls dies, create a Treasure token. \
                      (It's an artifact with \"{T}, Sacrifice this token: Add one mana of any \
                      color.\")\nAt the beginning of your upkeep, if you control ten or more \
                      Treasures, you win the game."
            .to_string(),
        abilities: vec![
            // Whenever a creature an opponent controls dies, create a Treasure token.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::Opponent),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // At the beginning of your upkeep, if you control ten or more Treasures, you win
            // the game. CR 603.4: intervening-if re-checked at resolution.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::WinGame,
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 10,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Artifact),
                        has_subtype: Some(SubType("Treasure".to_string())),
                        ..Default::default()
                    },
                }),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
