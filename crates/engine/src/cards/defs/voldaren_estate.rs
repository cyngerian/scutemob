// Voldaren Estate — Land
// {T}: Add {C}.
// {T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast a Vampire spell.
// {5}, {T}: Create a Blood token. This ability costs {1} less for each Vampire you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("voldaren-estate"),
        name: "Voldaren Estate".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast a Vampire spell.\n{5}, {T}: Create a Blood token. This ability costs {1} less to activate for each Vampire you control.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // {T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast a Vampire spell.
            // TODO: Pay 1 life cost is not expressible (Cost enum lacks Cost::PayLife variant).
            // DSL gap: use Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]) when available.
            // Modeled as tap-only until then; game state is incorrect (no life payment required).
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColorRestricted {
                    player: PlayerTarget::Controller,
                    restriction: ManaRestriction::SubtypeOnly(SubType("Vampire".to_string())),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {5}, {T}: Create a Blood token. This ability costs {1} less to activate for
            // each Vampire you control. DSL gap: no variable cost reduction based on board state.
        ],
        ..Default::default()
    }
}
