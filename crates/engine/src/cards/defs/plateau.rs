// Plateau — ({T}: Add {R} or {W}.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("plateau"),
        name: "Plateau".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Mountain", "Plains"]),
        oracle_text: "({T}: Add {R} or {W}.)".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Choose {
                    prompt: "Add {R} or {W}?".to_string(),
                    choices: vec![
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 1, 0, 0) },
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(1, 0, 0, 0, 0, 0) },
                    ],
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
