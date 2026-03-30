// Siege-Gang Commander — {3}{R}{R}, Creature — Goblin 2/2
// When this creature enters, create three 1/1 red Goblin creature tokens.
// {1}{R}, Sacrifice a Goblin: This creature deals 2 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("siege-gang-commander"),
        name: "Siege-Gang Commander".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "When this creature enters, create three 1/1 red Goblin creature tokens.\n{1}{R}, Sacrifice a Goblin: This creature deals 2 damage to any target.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // When this creature enters, create three 1/1 red Goblin creature tokens.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 3,
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
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // {1}{R}, Sacrifice a Goblin: This creature deals 2 damage to any target.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, red: 1, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter {
                        has_subtype: Some(SubType("Goblin".to_string())),
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetAny],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
