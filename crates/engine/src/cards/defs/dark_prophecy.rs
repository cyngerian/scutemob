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
            // TODO: WheneverCreatureDies is overbroad — fires on all creature deaths,
            //   not just "a creature you control".
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies,
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
