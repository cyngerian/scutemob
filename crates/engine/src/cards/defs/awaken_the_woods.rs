// Awaken the Woods — {X}{G}{G}, Sorcery
// Create X 1/1 green Forest Dryad land creature tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("awaken-the-woods"),
        name: "Awaken the Woods".to_string(),
        mana_cost: Some(ManaCost { green: 2, x_count: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Create X 1/1 green Forest Dryad land creature tokens. (They're affected by summoning sickness.)".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 107.3m: Repeat creates one Dryad token per chosen X value.
            effect: Effect::Repeat {
                count: EffectAmount::XValue,
                effect: Box::new(Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Forest Dryad".to_string(),
                        card_types: [CardType::Land, CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Forest".to_string()), SubType("Dryad".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: Some(ManaColor::Green),
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                }),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
