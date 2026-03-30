// Ghave, Guru of Spores — {2}{W}{B}{G}, Legendary Creature — Fungus Shaman 0/0
// Ghave enters with five +1/+1 counters on it.
// {1}, Remove a +1/+1 counter from a creature you control: Create a 1/1 green Saproling creature token.
// {1}, Sacrifice a creature: Put a +1/+1 counter on target creature.
//
// Note: The first ability says "from a creature you control" — ideally it can remove
// from any creature the controller controls. Cost::RemoveCounter removes from the source
// permanent (Ghave itself). This is a limitation deferred to PB-37.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ghave-guru-of-spores"),
        name: "Ghave, Guru of Spores".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Fungus", "Shaman"]),
        oracle_text: "Ghave enters with five +1/+1 counters on it.\n{1}, Remove a +1/+1 counter from a creature you control: Create a 1/1 green Saproling creature token.\n{1}, Sacrifice a creature: Put a +1/+1 counter on target creature.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            // ETB: enters with five +1/+1 counters.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 5,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 602.2: {1}, Remove a +1/+1 counter from this creature: Create a 1/1 green Saproling.
            // Note: Oracle says "from a creature you control" but Cost::RemoveCounter removes from
            // the source permanent (Ghave). "From another creature" deferred to PB-37.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 1 },
                ]),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Saproling".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Saproling".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 1,
                        toughness: 1,
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
            // CR 602.2: {1}, Sacrifice a creature: Put a +1/+1 counter on target creature.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
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
        ],
        ..Default::default()
    }
}
