// Shifting Woodland — This land enters tapped unless you control a Forest. {T}: Add {G}.
// Delirium — {2}{G}{G}: This land becomes a copy of target permanent card in your graveyard until
// end of turn. Activate only if there are four or more card types among cards in your graveyard.
// (channel/delirium ability — TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shifting-woodland"),
        name: "Shifting Woodland".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Forest.\n{T}: Add {G}.\nDelirium — {2}{G}{G}: This land becomes a copy of target permanent card in your graveyard until end of turn. Activate only if there are four or more card types among cards in your graveyard.".to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType("Forest".to_string())])),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: Activated — Delirium — {2}{G}{G}: This land becomes a copy of target permanent card in your graveyard until end of turn.
        ],
        ..Default::default()
    }
}
