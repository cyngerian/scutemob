// Spawning Pit — {2}, Artifact
// Sacrifice a creature: Put a charge counter on this artifact.
// {1}, Remove two charge counters from this artifact: Create a 2/2 colorless Spawn artifact creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spawning-pit"),
        name: "Spawning Pit".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Sacrifice a creature: Put a charge counter on this artifact.\n{1}, Remove two charge counters from this artifact: Create a 2/2 colorless Spawn artifact creature token.".to_string(),
        abilities: vec![
            // CR 602.2: Sacrifice a creature: Put a charge counter on this artifact.
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Charge,
                    count: 1,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // CR 602.2: {1}, Remove two charge counters: Create a 2/2 colorless Spawn token.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::RemoveCounter { counter: CounterType::Charge, count: 2 },
                ]),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Spawn".to_string(),
                        card_types: [CardType::Artifact, CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Spawn".to_string())].into_iter().collect(),
                        colors: im::OrdSet::new(),
                        power: 2,
                        toughness: 2,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
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
