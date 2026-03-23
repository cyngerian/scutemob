// Greensleeves, Maro-Sorcerer — {3}{G}{G}, Legendary Creature — Elemental Sorcerer */*
// Protection from planeswalkers and from Wizards
// Greensleeves's power and toughness are each equal to the number of lands you control.
// Landfall — Whenever a land you control enters, create a 3/3 green Badger creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("greensleeves-maro-sorcerer"),
        name: "Greensleeves, Maro-Sorcerer".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elemental", "Sorcerer"],
        ),
        oracle_text: "Protection from planeswalkers and from Wizards\nGreensleeves, Maro-Sorcerer's power and toughness are each equal to the number of lands you control.\nLandfall \u{2014} Whenever a land you control enters, create a 3/3 green Badger creature token.".to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            // TODO: "Protection from planeswalkers and from Wizards" — multi-quality protection.
            // Landfall: create 3/3 Badger
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Badger".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Badger".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 3,
                        toughness: 3,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
