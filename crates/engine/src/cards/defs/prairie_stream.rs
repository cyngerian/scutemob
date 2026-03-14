// Prairie Stream — ({T}: Add {W} or {U}.) This land enters tapped unless you control two or more ba
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("prairie-stream"),
        name: "Prairie Stream".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Plains", "Island"]),
        oracle_text: "({T}: Add {W} or {U}.)\nThis land enters tapped unless you control two or more basic lands.".to_string(),
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
                    prompt: "Add {W} or {U}?".to_string(),
                    choices: vec![
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(1, 0, 0, 0, 0, 0) },
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 1, 0, 0, 0, 0) },
                    ],
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
