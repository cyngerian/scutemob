// Phyrexian Tower — Legendary Land, {T}: Add {C}; sacrifice creature for {B}{B} (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phyrexian-tower"),
        name: "Phyrexian Tower".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice a creature: Add {B}{B}.".to_string(),
        abilities: vec![
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}, Sacrifice a creature: Add {B}{B}
            // — sacrifice-a-creature cost not expressible in DSL (Cost enum lacks SacrificeCreature)
        ],
        ..Default::default()
    }
}
