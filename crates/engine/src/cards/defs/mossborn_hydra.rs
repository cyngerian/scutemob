// Mossborn Hydra — {2}{G}, Creature — Elemental Hydra 0/0
// Trample
// This creature enters with a +1/+1 counter on it.
// Landfall — Whenever a land you control enters, double the number of +1/+1 counters on this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mossborn-hydra"),
        name: "Mossborn Hydra".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elemental", "Hydra"]),
        oracle_text: "Trample (This creature can deal excess combat damage to the player or planeswalker it's attacking.)\nThis creature enters with a +1/+1 counter on it.\nLandfall — Whenever a land you control enters, double the number of +1/+1 counters on this creature.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // ETB: enters with a +1/+1 counter
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
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
            // Landfall — Whenever a land you control enters, double the number of +1/+1
            // counters on this creature. Implementation: read current N counters via
            // EffectAmount::CounterCount and add N more, yielding 2N total. Ruling
            // 2024-11-08 confirms semantics ("put a number of +1/+1 counters on it equal
            // to the number it already has"). CR 207.2c (Landfall) +
            // CR 122.1/122.6 (counters; counters being put on an object).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::AddCounterAmount {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: EffectAmount::CounterCount {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
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
