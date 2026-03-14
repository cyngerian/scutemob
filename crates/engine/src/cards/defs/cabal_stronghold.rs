// Cabal Stronghold — Land, {T}: Add {C}. {3},{T}: Add {B} per basic Swamp (TODO: count-based mana).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cabal-stronghold"),
        name: "Cabal Stronghold".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{3}, {T}: Add {B} for each basic Swamp you control.".to_string(),
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
            // TODO: {3},{T}: Add {B} for each basic Swamp you control — count-based mana generation not in DSL
        ],
        ..Default::default()
    }
}
