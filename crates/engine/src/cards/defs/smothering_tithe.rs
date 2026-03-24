// Smothering Tithe — {3}{W}, Enchantment
// Whenever an opponent draws a card, that player may pay {2}. If the player doesn't,
// you create a Treasure token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("smothering-tithe"),
        name: "Smothering Tithe".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent draws a card, that player may pay {2}. If the player doesn't, you create a Treasure token.".to_string(),
        abilities: vec![
            // Opponent-draw filter applied.
            // TODO: "that player may pay {2}, if they don't" — MayPayOrElse still a gap.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPlayerDrawsCard {
                    player_filter: Some(TargetController::Opponent),
                },
                effect: Effect::CreateToken { spec: treasure_token_spec(1) },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
