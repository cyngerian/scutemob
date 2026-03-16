// Collector Ouphe — {1}{G}, Creature — Ouphe 2/2
// Activated abilities of artifacts can't be activated.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("collector-ouphe"),
        name: "Collector Ouphe".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Ouphe"]),
        oracle_text: "Activated abilities of artifacts can't be activated.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::ArtifactAbilitiesCantBeActivated,
            },
        ],
        ..Default::default()
    }
}
