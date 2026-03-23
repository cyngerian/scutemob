// Hordeling Outburst — {1}{R}{R}, Sorcery
// Create three 1/1 red Goblin creature tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hordeling-outburst"),
        name: "Hordeling Outburst".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Create three 1/1 red Goblin creature tokens.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
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
                    },
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
