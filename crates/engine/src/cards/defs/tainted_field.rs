// Tainted Field — Land
// {T}: Add {C}. {T}: Add {W} or {B}. Activate only if you control a Swamp.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tainted-field"),
        name: "Tainted Field".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {W} or {B}. Activate only if you control a Swamp.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // {T}: Add {W} or {B}. Activate only if you control a Swamp.
            // Modeled as producing {W} (first option); engine lacks mana choice.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType("Swamp".to_string())])),
                activation_zone: None,
            },
            // Second color option: {B}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType("Swamp".to_string())])),
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
