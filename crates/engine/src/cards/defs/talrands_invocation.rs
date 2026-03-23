// Talrand's Invocation — {2}{U}{U}, Sorcery
// Create two 2/2 blue Drake creature tokens with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("talrands-invocation"),
        name: "Talrand's Invocation".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Create two 2/2 blue Drake creature tokens with flying.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Drake".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Drake".to_string())].into_iter().collect(),
                        colors: [Color::Blue].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 2,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
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
