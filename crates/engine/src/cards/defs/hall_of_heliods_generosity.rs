// Hall of Heliod's Generosity — Legendary Land, {T}: Add {C}; {1}{W},{T}: graveyard recovery (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hall-of-heliods-generosity"),
        name: "Hall of Heliod's Generosity".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}{W}, {T}: Put target enchantment card from your graveyard on top of your library.".to_string(),
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
            // TODO: {1}{W}, {T}: Put target enchantment card from your graveyard on top of library
            // — graveyard targeting + return_to_library not expressible in DSL
        ],
        ..Default::default()
    }
}
