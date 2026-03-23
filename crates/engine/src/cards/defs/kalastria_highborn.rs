// Kalastria Highborn — {B}{B}, Creature — Vampire Shaman 2/2
// Whenever this creature or another Vampire you control dies, you may pay {B}. If you do,
// target player loses 2 life and you gain 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kalastria-highborn"),
        name: "Kalastria Highborn".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "Whenever Kalastria Highborn or another Vampire you control dies, you may pay {B}. If you do, target player loses 2 life and you gain 2 life.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: DSL gap — "this creature or another Vampire you control dies" trigger
            // with controller + subtype filter + optional mana payment at resolution.
        ],
        ..Default::default()
    }
}
