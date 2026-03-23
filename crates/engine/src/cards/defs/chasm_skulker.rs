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
            // TODO: Death trigger with variable token count (based on +1/+1 counters)
            //   not expressible — TokenSpec.count is fixed u32.
        ],
        ..Default::default()
    }
}
