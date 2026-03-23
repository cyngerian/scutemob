// Eldrazi Monument — {5}, Artifact
// Creatures you control get +1/+1 and have flying and indestructible.
// At the beginning of your upkeep, sacrifice a creature. If you can't, sacrifice this artifact.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eldrazi-monument"),
        name: "Eldrazi Monument".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Creatures you control get +1/+1 and have flying and indestructible.\nAt the beginning of your upkeep, sacrifice a creature. If you can't, sacrifice this artifact.".to_string(),
        abilities: vec![
            // TODO: All abilities stripped per W5 policy — +1/+1, flying, and indestructible
            // to all creatures with NO downside is wrong game state. The upkeep sacrifice
            // ("sacrifice a creature or sacrifice this artifact") is the balancing cost.
            // DSL gaps: upkeep trigger with mandatory sacrifice choice (player picks which
            // creature), "if you can't" fallback to self-sacrifice. Re-implement all
            // three statics (+1/+1, flying, indestructible) once the upkeep trigger works.
        ],
        ..Default::default()
    }
}
