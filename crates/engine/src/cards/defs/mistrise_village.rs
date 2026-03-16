// Mistrise Village — This land enters tapped unless you control a Mountain or a Forest.
// {T}: Add {U}. {U}, {T}: The next spell you cast this turn can't be countered. (complex activated ability — TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mistrise-village"),
        name: "Mistrise Village".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Mountain or a Forest.\n{T}: Add {U}.\n{U}, {T}: The next spell you cast this turn can't be countered.".to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType("Mountain".to_string()), SubType("Forest".to_string())])),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: Activated — {U}, {T}: The next spell you cast this turn can't be countered.
        ],
        ..Default::default()
    }
}
