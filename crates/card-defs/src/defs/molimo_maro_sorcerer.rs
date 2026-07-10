// Molimo, Maro-Sorcerer — {4}{G}{G}{G}, Legendary Creature — Elemental Sorcerer */*
// Trample
// Molimo's power and toughness are each equal to the number of lands you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("molimo-maro-sorcerer"),
        name: "Molimo, Maro-Sorcerer".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 3, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elemental", "Sorcerer"]),
        oracle_text: "Trample (This creature can deal excess combat damage to the player or planeswalker it's attacking.)\nMolimo's power and toughness are each equal to the number of lands you control.".to_string(),
        power: None,   // */* CDA — P/T set dynamically by Layer 7a
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
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
        ],
        ..Default::default()
    }
}
