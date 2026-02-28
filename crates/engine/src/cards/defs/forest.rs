// Forest
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forest"),
        name: "Forest".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Basic], &[CardType::Land], &["Forest"]),
        oracle_text: "{T}: Add {G}.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 0, 0, 0, 1, 0),
            },
            timing_restriction: None,
        }],
        ..Default::default()
    }
}
