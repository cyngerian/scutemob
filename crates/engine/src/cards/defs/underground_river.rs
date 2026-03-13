// Underground River — painland, {T}: Add {C}. {T}: Add {U} or {B} (deals 1 damage, TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("underground-river"),
        name: "Underground River".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {U} or {B}. This land deals 1 damage to you.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}: Add {U} or {B}. This land deals 1 damage to you.
            // DSL gap: no self-damage side effect on mana abilities.
        ],
        ..Default::default()
    }
}
