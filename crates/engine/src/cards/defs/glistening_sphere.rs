// Glistening Sphere
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glistening-sphere"),
        name: "Glistening Sphere".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "This artifact enters tapped.\nWhen this artifact enters, proliferate.\n{T}: Add one mana of any color.\nCorrupted — {T}: Add three mana of any one color. Activate only if an opponent has three or more poison counters.".to_string(),
        abilities: vec![
            // TODO: This artifact enters tapped.
            // TODO: Triggered — When this artifact enters, proliferate.
            // TODO: Activated — {T}: Add one mana of any color.
            // TODO: Static — Corrupted — {T}: Add three mana of any one color. Activate only if an opponent h
        ],
        ..Default::default()
    }
}
