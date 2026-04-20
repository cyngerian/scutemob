// Mossborn Hydra — {2}{G}, Creature — Elemental Hydra 0/0
// Trample
// This creature enters with a +1/+1 counter on it.
// Landfall — Whenever a land you control enters, double the number of +1/+1 counters on this creature.
//
// Trample and enters-with-counter ETB are implemented.
//
// TODO: DSL gap — Effect::AddCounter with amount equal to current +1/+1 counters on source
// (self-doubling). Landfall trigger itself is covered by
// TriggerCondition::WheneverPermanentEntersBattlefield { Land + You } (CR 207.2c), but the
// "double the number of +1/+1 counters on this creature" effect is not expressible — no
// EffectAmount::CurrentCountersOnSource variant exists. The Landfall ability is omitted.
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
        ],
        ..Default::default()
    }
}
