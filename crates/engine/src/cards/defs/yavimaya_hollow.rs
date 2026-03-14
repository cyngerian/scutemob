// Yavimaya Hollow — Legendary Land, {T}: Add {C}. {G},{T}: Regenerate target creature (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("yavimaya-hollow"),
        name: "Yavimaya Hollow".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Legendary], &[CardType::Land], &[]),
        oracle_text: "{T}: Add {C}.\n{G}, {T}: Regenerate target creature.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {G},{T}: Regenerate target creature — Regenerate effect not in DSL
        ],
        ..Default::default()
    }
}
