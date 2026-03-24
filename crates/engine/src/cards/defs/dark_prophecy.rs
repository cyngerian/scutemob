// Dark Prophecy — {B}{B}{B}, Enchantment
// Whenever a creature you control dies, you draw a card and you lose 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dark-prophecy"),
        name: "Dark Prophecy".to_string(),
        mana_cost: Some(ManaCost { black: 3, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control dies, you draw a card and you lose 1 life.".to_string(),
        abilities: vec![
            // CR 603.10a: "Whenever a creature you control dies, you draw a card and you lose 1 life."
            // PB-23: controller_you filter applied via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: false, nontoken_only: false },
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
