// Bloodmark Mentor — {1}{R}, Creature — Goblin Warrior 1/1
// "Red creatures you control have first strike."
//
// TODO: DSL gap — "Red creatures you control have first strike" is a continuous keyword-grant
// effect (Layer 6) filtered by color. No static continuous ability with a color filter
// for "all creatures you control" is expressible in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodmark-mentor"),
        name: "Bloodmark Mentor".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Red creatures you control have first strike.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}
