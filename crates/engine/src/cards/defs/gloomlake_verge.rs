// Gloomlake Verge — Land
// {T}: Add {U}. {T}: Add {B}. Activate only if you control an Island or a Swamp.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gloomlake-verge"),
        name: "Gloomlake Verge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {U}.\n{T}: Add {B}. Activate only if you control an Island or a Swamp.".to_string(),
        abilities: vec![
            // {T}: Add {U}. (unconditional)
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // {T}: Add {B}. Activate only if you control an Island or a Swamp.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::ControlLandWithSubtypes(vec![
                    SubType("Island".to_string()),
                    SubType("Swamp".to_string()),
                ])),
            },
        ],
        ..Default::default()
    }
}
