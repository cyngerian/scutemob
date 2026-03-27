// 31. Swan Song — {U}, Instant; counter target instant, sorcery, or enchantment
// spell. Its controller creates a 2/2 blue Bird creature token with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("swan-song"),
        name: "Swan Song".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target instant, sorcery, or enchantment spell. Its controller creates a 2/2 blue Bird creature token with flying.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Bird".to_string(),
                        power: 2,
                        toughness: 2,
                        colors: [Color::Blue].into_iter().collect(),
                        supertypes: OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Bird".to_string())].into_iter().collect(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
                        count: 1,
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
            ]),
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
