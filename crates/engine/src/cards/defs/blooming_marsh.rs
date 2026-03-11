// Blooming Marsh — This land enters tapped unless you control two or fewer other lands. {T}: Add {B
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blooming-marsh"),
        name: "Blooming Marsh".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control two or fewer other lands.\n{T}: Add {B} or {G}.".to_string(),
        abilities: vec![
            // Enters tapped (CR 614.1c)
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
                    prompt: "Add {B} or {G}?".to_string(),
                    choices: vec![
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 1, 0, 0, 0) },
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 0, 1, 0) },
                    ],
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
