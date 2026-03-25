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
            // DEVIATION: CR 614.1c — "enters with [counters]" is a replacement effect, not a
            // triggered ability. The counters should be placed as part of entering the battlefield
            // (simultaneously, without using the stack). The DSL lacks an EntersWithCounters
            // replacement effect primitive, so this is modeled as a triggered ETB ability instead.
            // Consequence: opponents can respond before counters are placed; SBAs see the creature
            // without counters during that window. For Ingenious Prodigy (0/1 base) the creature
            // survives either way, but timing is incorrect. Fix requires a future
            // EntersWithCounters { counter, count: EffectAmount } replacement effect primitive.
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
            // Upkeep: remove counter → draw.
            // DEVIATION: Oracle says "you MAY remove a +1/+1 counter." This ability is
            // unconditional — the DSL has no way to make "remove counter as cost" optional
            // without MayPayOrElse integration that is out of scope for PB-27.
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
