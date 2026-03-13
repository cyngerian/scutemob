// Kodama of the East Tree — {4}{G}{G}, Legendary Creature — Spirit 6/6
// Reach, Partner
// Whenever another permanent you control enters, if it wasn't put onto the battlefield with this ability,
// you may put a permanent card with equal or lesser mana value from your hand onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kodama-of-the-east-tree"),
        name: "Kodama of the East Tree".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Spirit"],
        ),
        oracle_text: "Reach\nWhenever another permanent you control enters, if it wasn't put onto the battlefield with this ability, you may put a permanent card with equal or lesser mana value from your hand onto the battlefield.\nPartner (You can have two commanders if both have partner.)".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            AbilityDefinition::Keyword(KeywordAbility::Partner),
            // TODO: Triggered ability — whenever another permanent you control enters, put a permanent
            // card from hand with equal or lesser MV onto the battlefield.
            // DSL gap: no mana-value comparison filter for hand-to-battlefield effect; no self-exclusion
            // for the "wasn't put with this ability" condition.
        ],
        ..Default::default()
    }
}
