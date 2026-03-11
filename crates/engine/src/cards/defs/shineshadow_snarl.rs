// Shineshadow Snarl — As this land enters, you may reveal a Plains or Swamp card from your hand. If yo
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shineshadow-snarl"),
        name: "Shineshadow Snarl".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, you may reveal a Plains or Swamp card from your hand. If you don't, this land enters tapped.\n{T}: Add {W} or {B}.".to_string(),
        abilities: vec![
            // TODO: Conditional ETB — may reveal a Plains or Swamp card, enters tapped if you don't
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
                    prompt: "Add {W} or {B}?".to_string(),
                    choices: vec![
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(1, 0, 0, 0, 0, 0) },
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 1, 0, 0, 0) },
                    ],
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
