// Pashalik Mons — {2}{R} Legendary Creature — Goblin Warrior 2/2
// Whenever Pashalik Mons or another Goblin you control dies, Pashalik Mons deals
// 1 damage to any target.
// {3}{R}, Sacrifice a Goblin: Create two 1/1 red Goblin creature tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pashalik-mons"),
        name: "Pashalik Mons".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Goblin", "Warrior"],
        ),
        oracle_text: "Whenever Pashalik Mons or another Goblin you control dies, Pashalik Mons deals 1 damage to any target.\n{3}{R}, Sacrifice a Goblin: Create two 1/1 red Goblin creature tokens.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Whenever ~ or another Goblin you control dies" — needs subtype-filtered
            // death trigger (WheneverCreatureYouControlDies with subtype filter) not in DSL.
            // {3}{R}, Sacrifice a Goblin: Create two 1/1 red Goblin creature tokens.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, red: 1, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter {
                        has_subtype: Some(SubType("Goblin".to_string())),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::Red].into_iter().collect(),
                        supertypes: im::OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        keywords: im::OrdSet::new(),
                        count: 2,
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
