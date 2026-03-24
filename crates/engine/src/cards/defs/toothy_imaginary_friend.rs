// Toothy, Imaginary Friend — {3}{U}, Legendary Creature — Illusion 1/1
// Partner with Pir, Imaginative Rascal (ETB trigger handled by PartnerWith keyword).
// "Whenever you draw a card, put a +1/+1 counter on Toothy." — triggered ability.
// "When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it." — triggered.
// Both WheneverYouDrawACard and WhenLeavesBattlefield are now implemented (PB-26).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("toothy-imaginary-friend"),
        name: "Toothy, Imaginary Friend".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Illusion"]),
        oracle_text:
            "Partner with Pir, Imaginative Rascal (When this creature enters the battlefield, \
             target player may search their library for a card named Pir, Imaginative Rascal, \
             reveal it, put it into their hand, then shuffle.)\n\
             Whenever you draw a card, put a +1/+1 counter on Toothy, Imaginary Friend.\n\
             When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it."
                .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 702.124j: "Partner with [name]" — ETB trigger searches for named partner.
            AbilityDefinition::Keyword(KeywordAbility::PartnerWith(
                "Pir, Imaginative Rascal".to_string(),
            )),
            // Whenever you draw a card, put a +1/+1 counter on Toothy.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it.
            // Note: LKI — source is in graveyard/exile but counter count is preserved by move_object_to_zone.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenLeavesBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::CounterCount {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
