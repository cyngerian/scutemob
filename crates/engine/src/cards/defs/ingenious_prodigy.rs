// Ingenious Prodigy — {X}{U}, Creature — Human Wizard 0/1
// Skulk
// This creature enters with X +1/+1 counters on it.
// At the beginning of your upkeep, if this creature has one or more +1/+1 counters on it,
// you may remove a +1/+1 counter from it. If you do, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ingenious-prodigy"),
        name: "Ingenious Prodigy".to_string(),
        mana_cost: Some(ManaCost { blue: 1, x_count: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Skulk\nThis creature enters with X +1/+1 counters on it.\nAt the beginning of your upkeep, if this creature has one or more +1/+1 counters on it, you may remove a +1/+1 counter from it. If you do, draw a card.".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Skulk),
            // CR 107.3m: "This creature enters with X +1/+1 counters on it."
            // x_value is propagated from the spell's StackObject to the permanent's x_value
            // field on resolution, then forwarded to the ETB trigger EffectContext in resolution.rs.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddCounterAmount {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: EffectAmount::XValue,
                },
                intervening_if: None,
                targets: vec![],
            },
            // Upkeep: remove counter → draw
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::Sequence(vec![
                    Effect::RemoveCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: Some(Condition::SourceHasCounters {
                    counter: CounterType::PlusOnePlusOne,
                    min: 1,
                }),
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
