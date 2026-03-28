// Krenko, Tin Street Kingpin — {2}{R}, Legendary Creature — Goblin 1/2
// Whenever Krenko attacks, put a +1/+1 counter on it, then create a number of 1/1 red
// Goblin creature tokens equal to Krenko's power.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("krenko-tin-street-kingpin"),
        name: "Krenko, Tin Street Kingpin".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Goblin"],
        ),
        oracle_text: "Whenever Krenko, Tin Street Kingpin attacks, put a +1/+1 counter on it, then create a number of 1/1 red Goblin creature tokens equal to Krenko's power.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    // TODO: "Tokens equal to power" — EffectAmount lacks power-based count.
                    //   Using fixed 2 as approximation.
                    Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Goblin".to_string(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                            colors: [Color::Red].into_iter().collect(),
                            power: 1,
                            toughness: 1,
                            count: 2,
                            supertypes: im::OrdSet::new(),
                            keywords: im::OrdSet::new(),
                            tapped: false,
                            enters_attacking: false,
                            mana_color: None,
                            mana_abilities: vec![],
                            activated_abilities: vec![],
                            ..Default::default()
                        },
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
