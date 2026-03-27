// Ominous Seas — {1}{U}, Enchantment
// Whenever you draw a card, put a foreshadow counter on Ominous Seas.
// Remove eight foreshadow counters from Ominous Seas: Create an 8/8 blue Kraken
// creature token.
// Cycling {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ominous-seas"),
        name: "Ominous Seas".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever you draw a card, put a foreshadow counter on Ominous Seas.\nRemove eight foreshadow counters from Ominous Seas: Create an 8/8 blue Kraken creature token.\nCycling {2}".to_string(),
        abilities: vec![
            // Whenever you draw a card, put a foreshadow counter on this.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Custom("foreshadow".to_string()),
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // CR 602.2: Remove eight foreshadow counters: Create an 8/8 blue Kraken token.
            AbilityDefinition::Activated {
                cost: Cost::RemoveCounter {
                    counter: CounterType::Custom("foreshadow".to_string()),
                    count: 8,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Kraken".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Kraken".to_string())].into_iter().collect(),
                        colors: [Color::Blue].into_iter().collect(),
                        power: 8,
                        toughness: 8,
                        count: 1,
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
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // Cycling {2}
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 2, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
