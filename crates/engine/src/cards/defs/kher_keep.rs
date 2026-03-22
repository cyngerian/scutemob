// Kher Keep — Legendary Land, {T}: Add {C}. {1}{R}, {T}: Create a 0/1 red Kobold token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kher-keep"),
        name: "Kher Keep".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}{R}, {T}: Create a 0/1 red Kobold creature token named Kobolds of Kher Keep.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // {1}{R}, {T}: Create a 0/1 red Kobold creature token named Kobolds of Kher Keep.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, red: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Kobolds of Kher Keep".to_string(),
                        power: 0,
                        toughness: 1,
                        colors: [Color::Red].into_iter().collect(),
                        supertypes: im::OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Kobold".to_string())].into_iter().collect(),
                        keywords: im::OrdSet::new(),
                        count: 1,
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
