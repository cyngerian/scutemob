// 1. Sol Ring — {1}, Artifact, tap: add {C}{C}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sol-ring"),
        name: "Sol Ring".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add {C}{C}.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 0, 0, 0, 0, 2),
            },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
                activation_zone: None,
        }],
        ..Default::default()
    }
}
