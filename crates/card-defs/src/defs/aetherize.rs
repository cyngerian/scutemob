// Aetherize — {3}{U}, Instant
// Return all attacking creatures to their owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aetherize"),
        name: "Aetherize".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return all attacking creatures to their owner's hand.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::BounceAll {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    is_attacking: true,
                    ..Default::default()
                },
                max_toughness_amount: None,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
