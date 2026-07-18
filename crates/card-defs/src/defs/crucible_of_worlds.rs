// Crucible of Worlds — {3}, Artifact
// You may play lands from your graveyard.
//
// CR 601.3, CR 305.1: Graveyard land play implemented via StaticPlayFromGraveyard (PB-B).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crucible-of-worlds"),
        name: "Crucible of Worlds".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "You may play lands from your graveyard.".to_string(),
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
