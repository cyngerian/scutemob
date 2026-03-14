// Tainted Isle — Land, {T}: Add {C}; conditional {U} or {B} if you control a Swamp
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tainted-isle"),
        name: "Tainted Isle".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {U} or {B}. Activate only if you control a Swamp.".to_string(),
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
            // TODO: {T}: Add {U} or {B}. Activate only if you control a Swamp.
            // DSL gap: conditional mana activation (requires "control a Swamp" filter)
            // and the full {U}/{B} choice are not expressible together.
        ],
        ..Default::default()
    }
}
