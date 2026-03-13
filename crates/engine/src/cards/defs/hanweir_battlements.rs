// Hanweir Battlements — Land, {T}: Add {C}; {R},{T}: haste; meld ability (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hanweir-battlements"),
        name: "Hanweir Battlements".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{R}, {T}: Target creature gains haste until end of turn.\n{3}{R}{R}, {T}: If you both own and control this land and a creature named Hanweir Garrison, exile them, then meld them into Hanweir, the Writhing Township.".to_string(),
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
            // TODO: {R}, {T}: Target creature gains haste until end of turn
            // — targeted activated ability not expressible in DSL (no targets field on Activated)
            // TODO: meld ability — Meld mechanic not implemented in DSL
        ],
        ..Default::default()
    }
}
