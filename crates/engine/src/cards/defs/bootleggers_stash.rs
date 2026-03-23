// Bootleggers' Stash — {5}{G}, Artifact
// Lands you control have "{T}: Create a Treasure token."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bootleggers-stash"),
        name: "Bootleggers' Stash".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Lands you control have \"{T}: Create a Treasure token.\"".to_string(),
        abilities: vec![
            // TODO: "Lands you control gain activated ability" — static ability granting
            //   activated abilities to other permanents not in DSL.
        ],
        ..Default::default()
    }
}
