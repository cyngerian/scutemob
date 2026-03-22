// Dragon's Hoard — {3}, Artifact
// Whenever a Dragon you control enters, put a gold counter on Dragon's Hoard.
// {T}, Remove a gold counter from Dragon's Hoard: Draw a card.
// {T}: Add one mana of any color.
//
// TODO: "{T}, Remove a gold counter: Draw a card" — Cost::RemoveCounters not in DSL.
//   Implementing the Dragon ETB trigger (gold counter) and mana ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragons-hoard"),
        name: "Dragon's Hoard".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever a Dragon you control enters, put a gold counter on Dragon's Hoard.\n{T}, Remove a gold counter from Dragon's Hoard: Draw a card.\n{T}: Add one mana of any color.".to_string(),
        abilities: vec![
            // Whenever a Dragon you control enters, put a gold counter on this.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Custom("gold".to_string()),
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: {T}, Remove gold counter: Draw a card (Cost::RemoveCounters not in DSL)
            // {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
