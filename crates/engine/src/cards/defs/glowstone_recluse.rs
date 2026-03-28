// Glowstone Recluse — {2}{G}, Creature — Spider 2/3
// Mutate {3}{G}
// Reach
// Whenever this creature mutates, put two +1/+1 counters on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glowstone-recluse"),
        name: "Glowstone Recluse".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Spider"]),
        oracle_text: "Mutate {3}{G}\nReach\nWhenever this creature mutates, put two +1/+1 counters on it.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Mutate),
            AbilityDefinition::MutateCost {
                cost: ManaCost { generic: 3, green: 1, ..Default::default() },
            },
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenMutates,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 2,
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
