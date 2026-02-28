// Breeding Pool — Land — Forest Island.
// Shock land: "As Breeding Pool enters the battlefield, you may pay 2 life.
// If you don't, it enters the battlefield tapped."
// {T}: Add {G}. {T}: Add {U}.
// Simplification: shock ETB choice deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("breeding-pool"),
        name: "Breeding Pool".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Forest", "Island"]),
        oracle_text: "As Breeding Pool enters the battlefield, you may pay 2 life. If you don't, it enters the battlefield tapped.\n{T}: Add {G}.\n{T}: Add {U}.".to_string(),
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
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
