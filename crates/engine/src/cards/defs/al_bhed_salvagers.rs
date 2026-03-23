// Al Bhed Salvagers — {2}{B}, Creature — Human Artificer Warrior 2/3
// Whenever this creature or another creature or artifact you control dies, target opponent
// loses 1 life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("al-bhed-salvagers"),
        name: "Al Bhed Salvagers".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Artificer", "Warrior"]),
        oracle_text: "Whenever Al Bhed Salvagers or another creature or artifact you control dies, target opponent loses 1 life and you gain 1 life.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // TODO: DSL gap — "this creature or another creature or artifact you control dies"
            // trigger with controller filter + multi-type filter (creature OR artifact).
        ],
        ..Default::default()
    }
}
