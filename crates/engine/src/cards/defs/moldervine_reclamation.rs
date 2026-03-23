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
            // TODO: WheneverCreatureDies is overbroad — fires on all creature deaths,
            //   not just "a creature you control". No controller filter available.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You) },
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
            },
        ],
        ..Default::default()
    }
}
