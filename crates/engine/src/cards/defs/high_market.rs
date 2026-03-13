// High Market — Land, {T}: Add {C}. {T}, Sacrifice a creature: Gain 1 life (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("high-market"),
        name: "High Market".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice a creature: You gain 1 life.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}, Sacrifice a creature: You gain 1 life — Cost::SacrificeCreature not in DSL
        ],
        ..Default::default()
    }
}
