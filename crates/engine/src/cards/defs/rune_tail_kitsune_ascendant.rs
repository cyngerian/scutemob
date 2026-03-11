// Rune-Tail, Kitsune Ascendant // Rune-Tail's Essence — When you have 30 or more life, flip Rune-Tail.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rune-tail-kitsune-ascendant"),
        name: "Rune-Tail, Kitsune Ascendant // Rune-Tail's Essence".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Fox", "Monk"]),
        oracle_text: "When you have 30 or more life, flip Rune-Tail.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}
