// Abomination of Llanowar — {1}{B}{G}, Legendary Creature — Elf Horror */*
// Vigilance, Menace; P/T = number of Elves you control + Elf cards in graveyard (CDA)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abomination-of-llanowar"),
        name: "Abomination of Llanowar".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Horror"],
        ),
        oracle_text: "Vigilance; menace (This creature can't be blocked except by two or more creatures.)\nAbomination of Llanowar's power and toughness are each equal to the number of Elves you control plus the number of Elf cards in your graveyard.".to_string(),
        power: None,   // */* CDA — P/T set dynamically by Layer 7a
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // CR 604.3, 613.4a: CDA — P/T = Elves you control + Elf cards in your graveyard.
            AbilityDefinition::CdaPowerToughness {
                power: EffectAmount::Sum(
                    Box::new(EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Elf".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    }),
                    Box::new(EffectAmount::CardCount {
                        zone: ZoneTarget::Graveyard { owner: PlayerTarget::Controller },
                        player: PlayerTarget::Controller,
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Elf".to_string())),
                            ..Default::default()
                        }),
                    }),
                ),
                toughness: EffectAmount::Sum(
                    Box::new(EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Elf".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    }),
                    Box::new(EffectAmount::CardCount {
                        zone: ZoneTarget::Graveyard { owner: PlayerTarget::Controller },
                        player: PlayerTarget::Controller,
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Elf".to_string())),
                            ..Default::default()
                        }),
                    }),
                ),
            },
        ],
        ..Default::default()
    }
}
