// Spark Double — {3}{U}, Creature — Illusion 0/0
// You may have this enter as a copy of a creature or planeswalker you control,
// except it isn't legendary, and it enters with an additional +1/+1 counter
// (creature) or loyalty counter (planeswalker).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spark-double"),
        name: "Spark Double".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: creature_types(&["Illusion"]),
        oracle_text: "You may have Spark Double enter as a copy of a creature or planeswalker you control, except it isn't legendary if that permanent is legendary, and it enters with an additional +1/+1 counter on it if it's a creature and an additional loyalty counter on it if it's a planeswalker.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            // TODO: ETB-replacement clone (BecomeCopyOf + remove legendary + extra counter).
            // BecomeCopyOf exists but ETB-replacement clone choice not in DSL.
        ],
        ..Default::default()
    }
}
