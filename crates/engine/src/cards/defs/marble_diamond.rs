// Marble Diamond — This artifact enters tapped. {T}: Add {W}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("marble-diamond"),
        name: "Marble Diamond".to_string(),
        mana_cost: None,
        types: types(&[CardType::Artifact]),
        oracle_text: "This artifact enters tapped.\n{T}: Add {W}.".to_string(),
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
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(1, 0, 0, 0, 0, 0) },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
