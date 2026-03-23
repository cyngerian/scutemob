// Ayara, First of Locthwain — {B}{B}{B}, Legendary Creature — Elf Noble 2/3
// Whenever Ayara or another black creature you control enters, each opponent loses
// 1 life and you gain 1 life.
// {T}, Sacrifice another black creature: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ayara-first-of-locthwain"),
        name: "Ayara, First of Locthwain".to_string(),
        mana_cost: Some(ManaCost { black: 3, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Noble"]),
        oracle_text: "Whenever Ayara, First of Locthwain or another black creature you control enters, each opponent loses 1 life and you gain 1 life.\n{T}, Sacrifice another black creature: Draw a card.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // TODO: "black creature you control enters" — color filter on ETB trigger.
            //   Using unfiltered creature ETB as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::Sequence(vec![
                    Effect::DrainLife {
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "Sacrifice another black creature" cost not expressible.
        ],
        ..Default::default()
    }
}
