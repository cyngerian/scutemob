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
            // CR 603.2: "Whenever Ayara or another black creature you control enters,
            // each opponent loses 1 life and you gain 1 life."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        colors: Some(im::ordset![Color::Black]),
                        ..Default::default()
                    }),
                },
                effect: Effect::DrainLife {
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 602.2: "{T}, Sacrifice another black creature: Draw a card."
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        colors: Some(im::ordset![Color::Black]),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
