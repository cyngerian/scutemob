// Vulpikeet — {3}{W}, Creature — Fox Bird 2/3
// Mutate {2}{W}
// Flying
// Whenever this creature mutates, put a +1/+1 counter on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vulpikeet"),
        name: "Vulpikeet".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: creature_types(&["Fox", "Bird"]),
        oracle_text: "Mutate {2}{W}\nFlying\nWhenever this creature mutates, put a +1/+1 counter on it.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Mutate),
            AbilityDefinition::MutateCost {
                cost: ManaCost { generic: 2, white: 1, ..Default::default() },
            },
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenMutates,
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
