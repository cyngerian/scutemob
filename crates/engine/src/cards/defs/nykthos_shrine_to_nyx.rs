// Nykthos, Shrine to Nyx — Legendary Land
// {T}: Add {C}. {2},{T}: devotion mechanic — not expressible in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nykthos-shrine-to-nyx"),
        name: "Nykthos, Shrine to Nyx".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Legendary], &[CardType::Land], &["Shrine"]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}: Choose a color. Add an amount of mana of that color equal to your devotion to that color. (Your devotion to a color is the number of mana symbols of that color in the mana costs of permanents you control.)".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {2},{T}: Choose a color, add mana equal to devotion to that color —
            // devotion counting and choice-of-color variable mana are not expressible
            // in the DSL
        ],
        ..Default::default()
    }
}
