// Martial Coup — {X}{W}{W}, Sorcery
// Create X 1/1 white Soldier creature tokens. If X is 5 or more, destroy all other creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("martial-coup"),
        name: "Martial Coup".to_string(),
        mana_cost: Some(ManaCost { white: 2, x_count: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Sorcery], &[]),
        oracle_text: "Create X 1/1 white Soldier creature tokens. If X is 5 or more, destroy all other creatures.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 107.3m: Create X 1/1 white Soldier tokens.
                Effect::Repeat {
                    count: EffectAmount::XValue,
                    effect: Box::new(Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Soldier".to_string(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                            colors: [Color::White].into_iter().collect(),
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
                        },
                    }),
                },
                // CR 107.3m: "If X is 5 or more, destroy all other creatures."
                Effect::Conditional {
                    condition: Condition::XValueAtLeast(5),
                    if_true: Box::new(Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        },
                        cant_be_regenerated: false,
                    }),
                    if_false: Box::new(Effect::Nothing),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
