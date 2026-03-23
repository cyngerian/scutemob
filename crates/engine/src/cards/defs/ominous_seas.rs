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
            // TODO: "Remove eight foreshadow counters" — Cost::RemoveCounters not in DSL.
            // Cycling {2}
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 2, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
