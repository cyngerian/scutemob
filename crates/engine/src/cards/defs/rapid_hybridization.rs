// Rapid Hybridization — {U} Instant
// Destroy target creature. It can't be regenerated. That creature's controller creates a 3/3
// green Frog Lizard creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rapid-hybridization"),
        name: "Rapid Hybridization".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text:
            "Destroy target creature. It can't be regenerated. That creature's controller creates a 3/3 green Frog Lizard creature token."
                .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: true,
                },
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Frog Lizard".to_string(),
                        power: 3,
                        toughness: 3,
                        colors: [Color::Green].into_iter().collect(),
                        supertypes: OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [
                            SubType("Frog".to_string()),
                            SubType("Lizard".to_string()),
                        ]
                        .into_iter()
                        .collect(),
                        keywords: OrdSet::new(),
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
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
