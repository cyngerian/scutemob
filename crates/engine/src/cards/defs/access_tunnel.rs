// Access Tunnel — Land, {T}: Add {C}; {3},{T}: target creature (power 3 or less) can't be blocked
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("access-tunnel"),
        name: "Access Tunnel".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{3}, {T}: Target creature with power 3 or less can't be blocked this turn.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {3}, {T}: Target creature with power 3 or less can't be blocked this turn.
            // DSL gap: activated ability with targets (Activated has no targets field),
            // and "can't be blocked" effect is not in Effect enum.
        ],
        ..Default::default()
    }
}
