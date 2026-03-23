// Nadir Kraken — {1}{U}{U}, Creature — Kraken 2/3
// Whenever you draw a card, you may pay {1}. If you do, put a +1/+1 counter on
// Nadir Kraken and create a 1/1 blue Tentacle creature token.
//
// TODO: "May pay {1}" optional cost — no MayPaySelf pattern in DSL.
//   Implementing as unconditional counter + token (approximation).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nadir-kraken"),
        name: "Nadir Kraken".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: creature_types(&["Kraken"]),
        oracle_text: "Whenever you draw a card, you may pay {1}. If you do, put a +1/+1 counter on Nadir Kraken and create a 1/1 blue Tentacle creature token.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Tentacle".to_string(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Tentacle".to_string())].into_iter().collect(),
                            colors: [Color::Blue].into_iter().collect(),
                            power: 1,
                            toughness: 1,
                            count: 1,
                            supertypes: im::OrdSet::new(),
                            keywords: im::OrdSet::new(),
                            tapped: false,
                            enters_attacking: false,
                            mana_color: None,
                            mana_abilities: vec![],
                            activated_abilities: vec![],
                        },
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
