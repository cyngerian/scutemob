// Choked Estuary — As this land enters, you may reveal an Island or Swamp card from your hand. If y
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("choked-estuary"),
        name: "Choked Estuary".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, you may reveal an Island or Swamp card from your hand. If you don't, this land enters tapped.\n{T}: Add {U} or {B}.".to_string(),
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
