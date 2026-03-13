// Yavimaya Coast — painland, {T}: Add {C}. {T}: Add {G} or {U} (deals 1 damage, TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("yavimaya-coast"),
        name: "Yavimaya Coast".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {G} or {U}. This land deals 1 damage to you.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}: Add {G} or {U}. This land deals 1 damage to you.
            // DSL gap: no self-damage side effect on mana abilities.
        ],
        ..Default::default()
    }
}
