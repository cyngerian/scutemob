// Ramunap Excavator — {2}{G}, Creature — Snake Cleric 2/3
// You may play lands from your graveyard.
//
// CR 601.3, CR 305.1: Graveyard land play implemented via StaticPlayFromGraveyard (PB-B).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ramunap-excavator"),
        name: "Ramunap Excavator".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Snake", "Cleric"]),
        oracle_text: "You may play lands from your graveyard.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // CR 601.3, CR 305.1: "You may play lands from your graveyard."
            AbilityDefinition::StaticPlayFromGraveyard {
                filter: PlayFromTopFilter::LandsOnly,
                condition: None,
            },
        ],
        ..Default::default()
    }
}
