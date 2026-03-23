// Searslicer Goblin — {1}{R}, Creature — Goblin Warrior 2/1
// Raid — At the beginning of your end step, if you attacked this turn, create a 1/1 red
// Goblin creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("searslicer-goblin"),
        name: "Searslicer Goblin".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Raid \u{2014} At the beginning of your end step, if you attacked this turn, create a 1/1 red Goblin creature token.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // TODO: Raid end-step trigger — "if you attacked this turn" condition not in DSL
        ],
        ..Default::default()
    }
}
