// Arch of Orazca — Land, Ascend; {T}: Add {C}; {5},{T}: Draw a card with city's blessing (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arch-of-orazca"),
        name: "Arch of Orazca".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Ascend (If you control ten or more permanents, you get the city's blessing for the rest of the game.)\n{T}: Add {C}.\n{5}, {T}: Draw a card. Activate only if you have the city's blessing.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: Ascend — city's blessing tracking not in DSL.
            // TODO: {5}, {T}: Draw a card. Activate only if you have the city's blessing.
            // DSL gap: conditional activation (city's blessing) not expressible.
        ],
        ..Default::default()
    }
}
