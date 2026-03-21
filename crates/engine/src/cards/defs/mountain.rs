// Mountain
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mountain"),
        name: "Mountain".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Basic], &[CardType::Land], &["Mountain"]),
        oracle_text: "{T}: Add {R}.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 0, 0, 1, 0, 0),
            },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
        }],
        ..Default::default()
    }
}
