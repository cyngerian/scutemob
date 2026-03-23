// Academy Manufactor — {3}, Artifact Creature — Assembly-Worker 1/3
// If you would create a Clue, Food, or Treasure token, instead create one of each.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("academy-manufactor"),
        name: "Academy Manufactor".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Assembly-Worker"]),
        oracle_text: "If you would create a Clue, Food, or Treasure token, instead create one of each.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // TODO: Token replacement effect — "instead create one of each" not in DSL
        ],
        ..Default::default()
    }
}
