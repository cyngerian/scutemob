// Dryad Arbor — (This land isn't a spell, it's affected by summoning sickness, and it has "{T}: 
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dryad-arbor"),
        name: "Dryad Arbor".to_string(),
        mana_cost: None,
        types: creature_types(&["Forest", "Dryad"]),
        oracle_text: "(This land isn't a spell, it's affected by summoning sickness, and it has \"{T}: Add {G}.\")".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 0, 1, 0) },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
