// Sulfurous Springs — Land (painland)
// {T}: Add {C}. {T}: Add {B} or {R} (deals 1 damage to you — damage part omitted, TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sulfurous-springs"),
        name: "Sulfurous Springs".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {B} or {R}. This land deals 1 damage to you.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}: Add {B} or {R}. This land deals 1 damage to you.
            // DSL gap: no self-damage side effect on mana abilities.
        ],
        ..Default::default()
    }
}
