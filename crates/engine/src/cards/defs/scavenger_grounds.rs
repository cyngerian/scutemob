// Scavenger Grounds — Land — Desert
// {T}: Add {C}. {2},{T}, Sacrifice a Desert: Exile all graveyards (not expressible).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scavenger-grounds"),
        name: "Scavenger Grounds".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Desert"]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}, Sacrifice a Desert: Exile all graveyards.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {2},{T}, Sacrifice a Desert: Exile all graveyards —
            // sacrifice-a-permanent-of-type as cost and exile-all-graveyards
            // effect are not expressible in the DSL
        ],
        ..Default::default()
    }
}
