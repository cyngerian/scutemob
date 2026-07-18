// Contagion Clasp — {2}, Artifact
// When this artifact enters, put a -1/-1 counter on target creature.
// {4}, {T}: Proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("contagion-clasp"),
        name: "Contagion Clasp".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "When this artifact enters, put a -1/-1 counter on target creature.\n{4}, \
                      {T}: Proliferate. (Choose any number of permanents and/or players, then \
                      give each another counter of each kind already there.)"
            .to_string(),
        abilities: vec![
            // When this artifact enters, put a -1/-1 counter on target creature.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddCounter {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    counter: CounterType::MinusOneMinusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                trigger_zone: None,
            },
            // {4}, {T}: Proliferate.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 4,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::Proliferate,
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
