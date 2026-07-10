// Dusk Legion Duelist — {1}{W}, Creature — Vampire Soldier 2/2
// Vigilance
// Whenever one or more +1/+1 counters are put on this creature, draw a card. This ability
// triggers only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dusk-legion-duelist"),
        name: "Dusk Legion Duelist".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Soldier"]),
        oracle_text: "Vigilance\nWhenever one or more +1/+1 counters are put on this creature, draw a card. This ability triggers only once each turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // CR 122.6 / 122.7 / 603.2h: "Whenever one or more +1/+1 counters are put on
            // this creature, draw a card. This ability triggers only once each turn."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenCounterPlaced {
                    counter: Some(CounterType::PlusOnePlusOne),
                    filter: None,
                    on_self: true,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
                once_per_turn: true,
            },
        ],
        ..Default::default()
    }
}
