// Panharmonicon — {4}, Artifact
// If an artifact or creature entering causes a triggered ability of a permanent
// you control to trigger, that ability triggers an additional time.
//
// CR 603.2d: Trigger doubling for artifacts and creatures ETB.
// Panharmonicon ruling 2021-03-19: "Panharmonicon affects a permanent's own
// enters-the-battlefield triggered abilities as well as other triggered abilities
// that trigger when that permanent enters the battlefield."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("panharmonicon"),
        name: "Panharmonicon".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "If an artifact or creature entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.".to_string(),
        abilities: vec![
            // CR 603.2d: ETB trigger doubling for artifacts and creatures.
            AbilityDefinition::TriggerDoubling {
                filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
                additional_triggers: 1,
            },
        ],
        ..Default::default()
    }
}
