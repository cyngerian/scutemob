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
            },
            // TODO: DSL gap — "{2}, Remove a +1/+1 counter: Put a +1/+1 counter on target
            // creature." Cost::RemoveCounter does not exist in Cost enum.
            // TODO: DSL gap — "{1}, Remove a +1/+1 counter: Prevent all combat damage."
            // Both Cost::RemoveCounter and Effect::PreventAllCombatDamage are missing.
        ],
        ..Default::default()
    }
}
