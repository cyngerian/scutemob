// The Meathook Massacre
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-meathook-massacre"),
        name: "The Meathook Massacre".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Enchantment], &[]),
        oracle_text: "When The Meathook Massacre enters, each creature gets -X/-X until end of turn.
Whenever a creature you control dies, each opponent loses 1 life.
Whenever a creature an opponent controls dies, you gain 1 life.".to_string(),
        abilities: vec![
            // TODO: DSL gap — X cost ETB: "each creature gets -X/-X" needs X mana value
            // at resolution + mass ApplyContinuousEffect to AllCreatures.
            // CR 603.10a: "Whenever a creature you control dies, each opponent loses 1 life."
            // PB-23: controller_you filter applied via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                },
                effect: Effect::LoseLife {
                    player: PlayerTarget::EachOpponent,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            // CR 603.10a: "Whenever a creature an opponent controls dies, you gain 1 life."
            // PB-23: controller_opponent filter applied via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::Opponent),
                    exclude_self: false,
                    nontoken_only: false,
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
