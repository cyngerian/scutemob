// Moldervine Reclamation — {3}{B}{G}, Enchantment
// Whenever a creature you control dies, you gain 1 life and draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("moldervine-reclamation"),
        name: "Moldervine Reclamation".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control dies, you gain 1 life and draw a card.".to_string(),
        abilities: vec![
            // CR 603.10a: "Whenever a creature you control dies, you gain 1 life and draw a card."
            // PB-23: controller_you filter applied via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: false, nontoken_only: false },
                effect: Effect::Sequence(vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
