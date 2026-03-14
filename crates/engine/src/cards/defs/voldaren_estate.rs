// Voldaren Estate — Land, {T}: Add {C}. {T}: Add {B} or {R} (Vampire-only restriction, TODO). {5},{T}: Create Blood token (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("voldaren-estate"),
        name: "Voldaren Estate".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast a Vampire spell.\n{5}, {T}: Create a Blood token. This ability costs {1} less to activate for each Vampire you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast a
            // Vampire spell. DSL gap: no life-payment cost variant and no mana restriction
            // (spend-only-for-subtype).
            // TODO: {5}, {T}: Create a Blood token. This ability costs {1} less to activate for
            // each Vampire you control. DSL gap: no variable cost reduction based on board state.
        ],
        ..Default::default()
    }
}
