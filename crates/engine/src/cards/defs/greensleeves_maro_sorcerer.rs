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
            // CR 702.16a: "Protection from planeswalkers" — blocks targeting/damage from
            // Planeswalker-type sources.
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromCardType(CardType::Planeswalker),
            )),
            // CR 702.16a: "Protection from Wizards" — blocks targeting/damage/blocking
            // from sources with the Wizard subtype.
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromSubType(SubType("Wizard".to_string())),
            )),
            // CR 604.3, 613.4a: CDA — P/T each equal to the number of lands you control.
            AbilityDefinition::CdaPowerToughness {
                power: EffectAmount::PermanentCount {
                    filter: TargetFilter { has_card_type: Some(CardType::Land), ..Default::default() },
                    controller: PlayerTarget::Controller,
                },
                toughness: EffectAmount::PermanentCount {
                    filter: TargetFilter { has_card_type: Some(CardType::Land), ..Default::default() },
                    controller: PlayerTarget::Controller,
                },
            },
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
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
