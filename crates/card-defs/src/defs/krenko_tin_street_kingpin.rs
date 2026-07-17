// Krenko, Tin Street Kingpin — {2}{R}, Legendary Creature — Goblin 1/2
// Whenever Krenko attacks, put a +1/+1 counter on it, then create a number of 1/1 red
// Goblin creature tokens equal to Krenko's power.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("krenko-tin-street-kingpin"),
        name: "Krenko, Tin Street Kingpin".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Goblin"]),
        oracle_text: "Whenever Krenko, Tin Street Kingpin attacks, put a +1/+1 counter on it, \
                      then create a number of 1/1 red Goblin creature tokens equal to Krenko's \
                      power."
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenAttacks,
            effect: Effect::Sequence(vec![
                Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                // CR 111.1: "a number of tokens equal to Krenko's power" — the +1/+1
                // counter above resolves first in the Sequence, so PowerOf(Source)
                // sees the freshly-added counter via calculate_characteristics.
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::PowerOf(EffectTarget::Source),
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
            ]),
            intervening_if: None,
            targets: vec![],

            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
