// Oran-Rief, the Vastwood
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oran-rief-the-vastwood"),
        name: "Oran-Rief, the Vastwood".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {G}.\n{T}: Put a +1/+1 counter on each green creature that entered this turn.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {G}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 0, 1, 0) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: Activated — {T}: Put a +1/+1 counter on each green creature that entered this turn.
        ],
        ..Default::default()
    }
}
