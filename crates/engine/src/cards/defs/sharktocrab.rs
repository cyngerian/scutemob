// Sharktocrab — {2}{G}{U}, Creature — Shark Octopus Crab 4/4 (Ravnica Allegiance)
// Adapt 1: {2}{G}{U}, {T}: if no +1/+1 counters, put 1 +1/+1 counter on it.
// TODO: "Whenever one or more +1/+1 counters are put on this creature, tap target creature
// an opponent controls. That creature doesn't untap during its controller's next untap step."
// — requires a targeted triggered ability with a "doesn't untap" replacement; no DSL support yet.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sharktocrab"),
        name: "Sharktocrab".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Shark", "Octopus", "Crab"]),
        oracle_text: "{2}{G}{U}: Adapt 1. (If this creature has no +1/+1 counters on it, put a +1/+1 counter on it.)\nWhenever one or more +1/+1 counters are put on this creature, tap target creature an opponent controls. That creature doesn't untap during its controller's next untap step.".to_string(),
        abilities: vec![
            // Keyword marker for Adapt 1 (CR 701.46)
            AbilityDefinition::Keyword(KeywordAbility::Adapt(1)),
            // Activated ability: {2}{G}{U}, {T}: Adapt 1
            // At resolution, the Conditional checks whether the source has no +1/+1 counters;
            // if true, places a counter. Mana is always spent regardless (ruling 2019-01-25).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::Conditional {
                    condition: Condition::SourceHasNoCountersOfType {
                        counter: CounterType::PlusOnePlusOne,
                    },
                    if_true: Box::new(Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    }),
                    if_false: Box::new(Effect::Nothing),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        power: Some(4),
        toughness: Some(4),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
