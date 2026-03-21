// 23. Generous Gift — {2W}, Instant; destroy target permanent, its controller
// creates a 3/3 green Elephant creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("generous-gift"),
        name: "Generous Gift".to_string(),
        mana_cost: Some(ManaCost { white: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target permanent. Its controller creates a 3/3 green Elephant creature token.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Elephant".to_string(),
                        power: 3,
                        toughness: 3,
                        colors: [Color::Green].into_iter().collect(),
                        supertypes: OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Elephant".to_string())].into_iter().collect(),
                        keywords: OrdSet::new(),
                        count: 1,
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
            ]),
            targets: vec![TargetRequirement::TargetPermanent],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
