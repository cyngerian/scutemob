// Watery Grave — ({T}: Add {U} or {B}.) As this land enters, you may pay 2 life. If you don't, it enters tapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("watery-grave"),
        name: "Watery Grave".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island", "Swamp"]),
        oracle_text: "({T}: Add {U} or {B}.)\nAs this land enters, you may pay 2 life. If you don't, it enters tapped.".to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTappedUnlessPayLife(2),
                is_self: true,
                unless_condition: None,
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
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
