// Rune-Tail, Kitsune Ascendant // Rune-Tail's Essence — When you have 30 or more life, flip Rune-Tail.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rune-tail-kitsune-ascendant"),
        name: "Rune-Tail, Kitsune Ascendant // Rune-Tail's Essence".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Fox", "Monk"]),
        oracle_text: "When you have 30 or more life, flip Rune-Tail.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        completeness: Completeness::inert("Blocked on flip-card support (CR 711): no Effect flips a permanent to its flipped face, and no CardDefinition field carries a flipped face (back_face/CardFace model DFC/MDFC, not flip). The intervening-if 'when you have 30 or more life' is expressible (Condition::ControllerLifeAtLeast); the flip action and Rune-Tail's Essence's damage-prevention static are not."),
        ..Default::default()
    }
}
