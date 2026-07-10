// Nadir Kraken — {1}{U}{U}, Creature — Kraken 2/3
// Whenever you draw a card, you may pay {1}. If you do, put a +1/+1 counter on
// Nadir Kraken and create a 1/1 blue Tentacle creature token.
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
            // PB-AC2 (CR 118.12): "you may pay {1}. If you do, ..." beneficial
            // optional-pay wrapper — counter + token only granted if {1} is paid.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::Sequence(vec![
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
                                count: EffectAmount::Fixed(1),
                                supertypes: imbl::OrdSet::new(),
                                keywords: imbl::OrdSet::new(),
                                tapped: false,
                                enters_attacking: false,
                                mana_color: None,
                                mana_abilities: vec![],
                                activated_abilities: vec![],
                                ..Default::default()
                            },
                        },
                    ])),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
