// Wastewood Verge — Land
// {T}: Add {G}. {T}: Add {B}. Activate only if you control a Swamp or a Forest.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wastewood-verge"),
        name: "Wastewood Verge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {G}.\n{T}: Add {B}. Activate only if you control a Swamp or a Forest.".to_string(),
        abilities: vec![
            // {T}: Add {G}. (unconditional)
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {T}: Add {B}. Activate only if you control a Swamp or a Forest.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::ControlLandWithSubtypes(vec![
                    SubType("Swamp".to_string()),
                    SubType("Forest".to_string()),
                ])),

                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
