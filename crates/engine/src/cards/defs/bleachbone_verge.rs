// Bleachbone Verge — Land
// {T}: Add {B}. {T}: Add {W}. Activate only if you control a Plains or a Swamp.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bleachbone-verge"),
        name: "Bleachbone Verge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {B}.\n{T}: Add {W}. Activate only if you control a Plains or a Swamp.".to_string(),
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
            },
            // {T}: Add {W}. Activate only if you control a Plains or a Swamp.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::ControlLandWithSubtypes(vec![
                    SubType("Plains".to_string()),
                    SubType("Swamp".to_string()),
                ])),

                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
