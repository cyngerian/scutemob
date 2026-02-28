// Overgrown Tomb — Land — Swamp Forest.
// Shock land: "As Overgrown Tomb enters the battlefield, you may pay 2 life.
// If you don't, it enters the battlefield tapped."
// {T}: Add {B}. {T}: Add {G}.
// Simplification: shock ETB choice deferred (same as Godless Shrine, Blood Crypt).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("overgrown-tomb"),
        name: "Overgrown Tomb".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Swamp", "Forest"]),
        oracle_text: "As Overgrown Tomb enters the battlefield, you may pay 2 life. If you don't, it enters the battlefield tapped.\n{T}: Add {B}.\n{T}: Add {G}.".to_string(),
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
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
