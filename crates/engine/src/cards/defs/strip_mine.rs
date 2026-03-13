// Strip Mine — Land
// {T}: Add {C}. {T}, Sacrifice: Destroy target land (sacrifice ability not expressible).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("strip-mine"),
        name: "Strip Mine".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice this land: Destroy target land.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}, Sacrifice this land: Destroy target land — sacrifice-as-cost
            // activated abilities with targeted land destruction are not expressible
            // in the DSL (no Cost::SacrificeSelf variant)
        ],
        ..Default::default()
    }
}
