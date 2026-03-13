// Buried Ruin — Land, {T}: Add {C}; {2},{T}, sacrifice: return artifact from graveyard (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("buried-ruin"),
        name: "Buried Ruin".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}, Sacrifice this land: Return target artifact card from your graveyard to your hand.".to_string(),
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
            // TODO: {2}, {T}, Sacrifice this land: Return target artifact card from your graveyard
            // to your hand.
            // DSL gap: return_from_graveyard (with type filter) not in Effect enum;
            // sacrifice-self cost not expressible in Activated.
        ],
        ..Default::default()
    }
}
