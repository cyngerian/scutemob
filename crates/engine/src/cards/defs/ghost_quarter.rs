// Ghost Quarter — Land
// {T}: Add {C}. {T}, Sacrifice: Destroy target land (opponent may search for basic).
// Sacrifice-as-cost ability not expressible in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ghost-quarter"),
        name: "Ghost Quarter".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice this land: Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield, then shuffle.".to_string(),
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
            // TODO: {T}, Sacrifice: Destroy target land, opponent may search — PB-5 (targeted)
            // Cost::SacrificeSelf available; blocked on targeted destroy + opponent search effect
        ],
        ..Default::default()
    }
}
