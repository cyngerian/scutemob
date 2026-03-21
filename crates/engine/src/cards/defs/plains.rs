// 14-18: Basic lands (each produces one mana of its color).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("plains"),
        name: "Plains".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Basic], &[CardType::Land], &["Plains"]),
        oracle_text: "{T}: Add {W}.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(1, 0, 0, 0, 0, 0),
            },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
        }],
        ..Default::default()
    }
}
