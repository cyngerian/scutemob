// Reckless One — {3}{R}, Creature — Goblin Avatar */*
// Haste
// Reckless One's power and toughness are each equal to the number of Goblins on the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reckless-one"),
        name: "Reckless One".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Avatar"]),
        oracle_text: "Haste\nReckless One's power and toughness are each equal to the number of Goblins on the battlefield.".to_string(),
        power: None,   // */* CDA — P/T set dynamically by Layer 7a
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 604.3, 613.4a: CDA — P/T each equal to the number of Goblins on the battlefield
            // (all players' Goblins, not just yours — "on the battlefield" means EachPlayer).
            AbilityDefinition::CdaPowerToughness {
                power: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Goblin".to_string())),
                        ..Default::default()
                    },
                    controller: PlayerTarget::EachPlayer,
                },
                toughness: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Goblin".to_string())),
                        ..Default::default()
                    },
                    controller: PlayerTarget::EachPlayer,
                },
            },
        ],
        ..Default::default()
    }
}
