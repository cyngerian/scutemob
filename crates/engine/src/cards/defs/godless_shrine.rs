// Godless Shrine — Land — Plains Swamp.
// "As Godless Shrine enters the battlefield, you may pay 2 life. If you don't,
// it enters the battlefield tapped."
// {T}: Add {W}. {T}: Add {B}.
//
// Simplification: shock ETB choice (pay 2 life or enters tapped) is deferred —
// no MayPay replacement support in the DSL. Modeled as always-untapped dual land.
// The ETB cost should be added when MayPayReplacement is implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("godless-shrine"),
        name: "Godless Shrine".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Plains", "Swamp"]),
        oracle_text: "As Godless Shrine enters the battlefield, you may pay 2 life. If you don't, it enters the battlefield tapped.\n{T}: Add {W}.\n{T}: Add {B}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
