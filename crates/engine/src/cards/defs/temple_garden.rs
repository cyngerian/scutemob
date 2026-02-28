// Temple Garden — Land — Forest Plains.
// Shock land: "As Temple Garden enters the battlefield, you may pay 2 life.
// If you don't, it enters the battlefield tapped."
// {T}: Add {G}. {T}: Add {W}.
// Simplification: shock ETB choice deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("temple-garden"),
        name: "Temple Garden".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Forest", "Plains"]),
        oracle_text: "As Temple Garden enters the battlefield, you may pay 2 life. If you don't, it enters the battlefield tapped.\n{T}: Add {G}.\n{T}: Add {W}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
