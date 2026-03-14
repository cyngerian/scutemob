// Gloomlake Verge — Land, {T}: Add {U}; {T}: Add {B} only if you control Island or Swamp
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gloomlake-verge"),
        name: "Gloomlake Verge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {U}.\n{T}: Add {B}. Activate only if you control an Island or a Swamp.".to_string(),
        abilities: vec![
            // {T}: Add {U}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {T}: Add {B}. Activate only if you control an Island or a Swamp.
            // DSL gap: conditional mana activation (requires "control an Island or a Swamp" filter)
            // not expressible in current DSL.
        ],
        ..Default::default()
    }
}
