// Smoldering Marsh — ({T}: Add {B} or {R}.) This land enters tapped unless you control two or more ba
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("smoldering-marsh"),
        name: "Smoldering Marsh".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Swamp", "Mountain"]),
        oracle_text: "({T}: Add {B} or {R}.)\nThis land enters tapped unless you control two or more basic lands.".to_string(),
        abilities: vec![            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlBasicLandsAtLeast(2)),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Choose {
                    prompt: "Add {B} or {R}?".to_string(),
                    choices: vec![
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 1, 0, 0, 0) },
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 1, 0, 0) },
                    ],
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
