// Nykthos, Shrine to Nyx — Legendary Land
// {T}: Add {C}.
// {2}, {T}: Choose a color. Add an amount of mana of that color equal to your
// devotion to that color.
//
// EffectAmount::DevotionTo(color) is now available, but the "choose a color"
// interactive selection requires M10 (Command::ChooseColor). Devotion ability
// deferred until color choice is implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nykthos-shrine-to-nyx"),
        name: "Nykthos, Shrine to Nyx".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}: Choose a color. Add an amount of mana of that color equal to your devotion to that color. (Your devotion to a color is the number of mana symbols of that color in the mana costs of permanents you control.)".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // TODO: {2},{T}: Choose a color, add mana equal to devotion to that color.
            // DevotionTo(color) is implemented but "choose a color" interactive selection
            // requires M10 (Command::ChooseColor). Deferred.
        ],
        ..Default::default()
    }
}
