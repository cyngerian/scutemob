// Elvish Promenade — {3}{G}, Kindred Sorcery — Elf
// Create a 1/1 green Elf Warrior creature token for each Elf you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-promenade"),
        name: "Elvish Promenade".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Sorcery], &["Elf"]),
        oracle_text: "Create a 1/1 green Elf Warrior creature token for each Elf you control.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "For each Elf you control" — count-based token creation not in DSL.
            //   Using fixed 3 as approximation.
            effect: Effect::CreateToken {
                spec: TokenSpec {
                    name: "Elf Warrior".to_string(),
                    card_types: [CardType::Creature].into_iter().collect(),
                    subtypes: [SubType("Elf".to_string()), SubType("Warrior".to_string())].into_iter().collect(),
                    colors: [Color::Green].into_iter().collect(),
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
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
