// Unclaimed Territory — Land; ETB choose creature type, tap for colorless or
// any color (restricted to chosen creature type spells).
// TODO: ETB choice of creature type and mana-type restriction not expressible in DSL.
// Implementing the two tap abilities without the type restriction.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("unclaimed-territory"),
        name: "Unclaimed Territory".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, choose a creature type.\n{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: second ability adds any-color mana restricted to creature spells of chosen type
            // Requires ETB choice tracking and mana restriction enforcement — not yet in DSL.
        ],
        ..Default::default()
    }
}
