// Susurian Voidborn — {2}{B}, Creature — Vampire Soldier 2/2
// Whenever this creature or another creature or artifact you control dies, target opponent
// loses 1 life and you gain 1 life.
// Warp {B}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("susurian-voidborn"),
        name: "Susurian Voidborn".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Soldier"]),
        oracle_text: "Whenever Susurian Voidborn or another creature or artifact you control dies, target opponent loses 1 life and you gain 1 life.\nWarp {B}".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: DSL gap — death trigger with controller filter (creature or artifact
            // you control) + Warp keyword (not in DSL).
        ],
        ..Default::default()
    }
}
