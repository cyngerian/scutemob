// Snow-Covered Swamp — ({T}: Add {B}.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("snow-covered-swamp"),
        name: "Snow-Covered Swamp".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Basic, SuperType::Snow], &[CardType::Land], &["Swamp"]),
        oracle_text: "({T}: Add {B}.)".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 1, 0, 0, 0) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
