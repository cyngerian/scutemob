// Spike Weaver — {2}{G}{G}, Creature — Spike 0/0
// This creature enters with three +1/+1 counters on it.
// {2}, Remove a +1/+1 counter from this creature: Put a +1/+1 counter on target creature.
// {1}, Remove a +1/+1 counter from this creature: Prevent all combat damage that would
// be dealt this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spike-weaver"),
        name: "Spike Weaver".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Spike"]),
        oracle_text: "Spike Weaver enters with three +1/+1 counters on it.\n{2}, Remove a +1/+1 counter from Spike Weaver: Put a +1/+1 counter on target creature.\n{1}, Remove a +1/+1 counter from Spike Weaver: Prevent all combat damage that would be dealt this turn.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            // ETB: enters with three +1/+1 counters.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 3,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 602.2: {2}, Remove a +1/+1 counter: Put a +1/+1 counter on target creature.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 1 },
                ]),
                effect: Effect::AddCounter {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // CR 615.1: {1}, Remove a +1/+1 counter: Prevent all combat damage this turn.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 1 },
                ]),
                effect: Effect::PreventAllCombatDamage,
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
