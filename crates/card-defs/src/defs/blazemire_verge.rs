// Blazemire Verge — Land
// {T}: Add {B}. {T}: Add {R}. Activate only if you control a Swamp or a Mountain.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blazemire-verge"),
        name: "Blazemire Verge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {B}.\n{T}: Add {R}. Activate only if you control a Swamp or a Mountain.".to_string(),
        abilities: vec![
            // {T}: Add {B}. (unconditional)
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {T}: Add {R}. Activate only if you control a Swamp or a Mountain.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::ControlLandWithSubtypes(vec![
                    SubType("Swamp".to_string()),
                    SubType("Mountain".to_string()),
                ])),

                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
