// Blood Crypt — Land — Swamp Mountain.
// "As Blood Crypt enters the battlefield, you may pay 2 life. If you don't,
// it enters the battlefield tapped."
// {T}: Add {B}. {T}: Add {R}.
//
// Simplification: shock ETB choice deferred (same as Godless Shrine).
// Modeled as always-untapped dual land producing {B} and {R}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blood-crypt"),
        name: "Blood Crypt".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Swamp", "Mountain"]),
        oracle_text: "As Blood Crypt enters the battlefield, you may pay 2 life. If you don't, it enters the battlefield tapped.\n{T}: Add {B}.\n{T}: Add {R}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
