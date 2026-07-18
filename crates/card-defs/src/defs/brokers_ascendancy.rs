// Brokers Ascendancy — {G}{W}{U}, Enchantment
// At the beginning of your end step, put a +1/+1 counter on each creature you control
// and a loyalty counter on each planeswalker you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brokers-ascendancy"),
        name: "Brokers Ascendancy".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            white: 1,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your end step, put a +1/+1 counter on each creature you \
                      control and a loyalty counter on each planeswalker you control."
            .to_string(),
        abilities: vec![
            // "At the beginning of your end step, put a +1/+1 counter on each creature you
            // control and a loyalty counter on each planeswalker you control."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::Sequence(vec![
                    Effect::ForEach {
                        over: ForEachTarget::EachCreatureYouControl,
                        effect: Box::new(Effect::AddCounter {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            counter: CounterType::PlusOnePlusOne,
                            count: 1,
                        }),
                    },
                    Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                            has_card_type: Some(CardType::Planeswalker),
                            controller: TargetController::You,
                            ..Default::default()
                        })),
                        effect: Box::new(Effect::AddCounter {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            counter: CounterType::Loyalty,
                            count: 1,
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
