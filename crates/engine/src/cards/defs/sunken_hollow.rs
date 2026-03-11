// Sunken Hollow — ({T}: Add {U} or {B}.) This land enters tapped unless you control two or more ba
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sunken-hollow"),
        name: "Sunken Hollow".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island", "Swamp"]),
        oracle_text: "({T}: Add {U} or {B}.)\nThis land enters tapped unless you control two or more basic lands.".to_string(),
        abilities: vec![
            // TODO: Conditional ETB — enters tapped unless you control two or more basic lands
            // DSL gap: ReplacementModification::EntersTapped has no condition field
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Choose {
                    prompt: "Add {U} or {B}?".to_string(),
                    choices: vec![
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 1, 0, 0, 0, 0) },
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 1, 0, 0, 0) },
                    ],
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
