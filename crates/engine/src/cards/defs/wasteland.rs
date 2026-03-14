// Wasteland — Land
// {T}: Add {C}. {T}, Sacrifice: Destroy target nonbasic land (PB-5: targeted activated).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wasteland"),
        name: "Wasteland".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice this land: Destroy target nonbasic land.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}, Sacrifice this land: Destroy target nonbasic land — PB-5
            // Cost::SacrificeSelf is available; blocked on targeted land destruction effect
        ],
        ..Default::default()
    }
}
