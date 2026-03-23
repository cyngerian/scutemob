// Quest for the Goblin Lord — {R}, Enchantment
// Whenever a Goblin you control enters, you may put a quest counter on this enchantment.
// As long as this enchantment has five or more quest counters on it, creatures you control
// get +2/+0.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("quest-for-the-goblin-lord"),
        name: "Quest for the Goblin Lord".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a Goblin you control enters, you may put a quest counter on this enchantment.\nAs long as this enchantment has five or more quest counters on it, creatures you control get +2/+0.".to_string(),
        abilities: vec![
            // Whenever a Goblin you control enters, you may put a quest counter on ~.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        has_subtype: Some(SubType("Goblin".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Custom("quest".to_string()),
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: DSL gap — conditional static: "As long as this has 5+ quest counters,
            // creatures you control get +2/+0." Needs Condition-gated Static ability.
        ],
        ..Default::default()
    }
}
