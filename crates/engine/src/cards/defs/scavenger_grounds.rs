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
            // TODO: {2},{T}, Sacrifice a Desert: Exile all graveyards — PB-19 (mass exile)
            // Cost: Cost::Sacrifice(TargetFilter { has_subtype: Desert }) available
            // Blocked on exile-all-graveyards effect (no Effect::ExileAllGraveyards)
        ],
        ..Default::default()
    }
}
