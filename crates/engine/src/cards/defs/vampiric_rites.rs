// Vampiric Rites — {B}, Enchantment
// {1}{B}, Sacrifice a creature: You gain 1 life and draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampiric-rites"),
        name: "Vampiric Rites".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "{1}{B}, Sacrifice a creature: You gain 1 life and draw a card.".to_string(),
        abilities: vec![
            // TODO: "Sacrifice a creature" cost — Cost::Sacrifice with creature filter
            //   exists (PB-4). Using it.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, black: 1, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
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
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
