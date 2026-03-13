// Three Tree City — Legendary Land, ETB choose creature type, {T}: Add {C}; activated mana (TODO)
// TODO: "As Three Tree City enters, choose a creature type" — ETB choice not expressible in DSL
// TODO: {2},{T}: Choose a color. Add mana equal to # creatures you control of chosen type
// — count-based mana scaling not expressible in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("three-tree-city"),
        name: "Three Tree City".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "As Three Tree City enters, choose a creature type.\n{T}: Add {C}.\n{2}, {T}: Choose a color. Add an amount of mana of that color equal to the number of creatures you control of the chosen type.".to_string(),
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
        ],
        ..Default::default()
    }
}
