// Stomping Ground — Land — Mountain Forest.
// Shock land: "As Stomping Ground enters the battlefield, you may pay 2 life.
// If you don't, it enters the battlefield tapped."
// {T}: Add {R}. {T}: Add {G}.
// Simplification: shock ETB choice deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stomping-ground"),
        name: "Stomping Ground".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Mountain", "Forest"]),
        oracle_text: "As Stomping Ground enters the battlefield, you may pay 2 life. If you don't, it enters the battlefield tapped.\n{T}: Add {R}.\n{T}: Add {G}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
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
