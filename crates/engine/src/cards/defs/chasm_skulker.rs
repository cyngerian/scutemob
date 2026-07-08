// Chasm Skulker — {2}{U}, Creature — Squid Horror 1/1
// Whenever you draw a card, put a +1/+1 counter on Chasm Skulker.
// When Chasm Skulker dies, create X 1/1 blue Squid creature tokens with islandwalk,
// where X is the number of +1/+1 counters on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chasm-skulker"),
        name: "Chasm Skulker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Squid", "Horror"]),
        oracle_text: "Whenever you draw a card, put a +1/+1 counter on Chasm Skulker.\nWhen Chasm Skulker dies, create X 1/1 blue Squid creature tokens with islandwalk, where X is the number of +1/+1 counters on Chasm Skulker.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // Whenever you draw a card, put a +1/+1 counter on Chasm Skulker.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // When Chasm Skulker dies, create X 1/1 blue Squid creature tokens with islandwalk,
            // where X is the number of +1/+1 counters on it.
            // CR 603.10a: counter count read from LKI snapshot (counters cease on zone change per CR 122.2).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Squid".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::Blue].into_iter().collect(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Squid".to_string())].into_iter().collect(),
                        keywords: [KeywordAbility::Landwalk(LandwalkType::BasicType(
                            SubType("Island".to_string()),
                        ))]
                        .into_iter()
                        .collect(),
                        count: EffectAmount::CounterCountAtLastKnownInformation {
                            counter: CounterType::PlusOnePlusOne,
                        },
                        ..Default::default()
                    },
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
