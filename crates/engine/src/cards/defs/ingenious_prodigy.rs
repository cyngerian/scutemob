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
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Skulk\nThis creature enters with X +1/+1 counters on it.\nAt the beginning of your upkeep, if this creature has one or more +1/+1 counters on it, you may remove a +1/+1 counter from it. If you do, draw a card.".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Skulk),
            // TODO: "Enters with X +1/+1 counters" — X-value ETB not in DSL.
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
