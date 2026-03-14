// Thespian's Stage — Land, {T}: Add {C}; {2},{T}: become copy of target land (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thespians-stage"),
        name: "Thespian's Stage".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}: This land becomes a copy of target land, except it has this ability.".to_string(),
        abilities: vec![
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {2}, {T}: This land becomes a copy of target land, except it has this ability
            // — "become a copy of target land" copy effect not expressible in DSL
        ],
        ..Default::default()
    }
}
