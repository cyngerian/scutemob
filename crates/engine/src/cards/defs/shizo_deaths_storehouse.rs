// Shizo, Death's Storehouse — Legendary Land, {T}: Add {B}; fear grant ability (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shizo-deaths-storehouse"),
        name: "Shizo, Death's Storehouse".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {B}.\n{B}, {T}: Target legendary creature gains fear until end of turn. (It can't be blocked except by artifact creatures and/or black creatures.)".to_string(),
        abilities: vec![
            // {T}: Add {B}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {B}, {T}: Target legendary creature gains fear until end of turn
            // — targeted activated ability not expressible in DSL (no targets field on Activated);
            // fear keyword also not implemented
        ],
        ..Default::default()
    }
}
