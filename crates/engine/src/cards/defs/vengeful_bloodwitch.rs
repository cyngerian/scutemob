// Vengeful Bloodwitch — {1}{B}, Creature — Vampire Warlock 1/1
// Whenever this creature or another creature you control dies, target opponent loses 1
// life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vengeful-bloodwitch"),
        name: "Vengeful Bloodwitch".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Warlock"]),
        oracle_text: "Whenever Vengeful Bloodwitch or another creature you control dies, target opponent loses 1 life and you gain 1 life.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // TODO: DSL gap — "this creature or another creature you control dies" trigger
            // needs controller filter on WheneverCreatureDies. Using unfiltered trigger
            // would fire on opponents' creatures dying (wrong game state per W5).
        ],
        ..Default::default()
    }
}
