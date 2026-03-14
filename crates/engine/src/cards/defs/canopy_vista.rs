// Canopy Vista — ({T}: Add {G} or {W}.) This land enters tapped unless you control two or more ba
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("canopy-vista"),
        name: "Canopy Vista".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Forest", "Plains"]),
        oracle_text: "({T}: Add {G} or {W}.)\nThis land enters tapped unless you control two or more basic lands.".to_string(),
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
                    prompt: "Add {G} or {W}?".to_string(),
                    choices: vec![
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 0, 1, 0) },
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(1, 0, 0, 0, 0, 0) },
                    ],
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
