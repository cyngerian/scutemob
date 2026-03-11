// Funeral Room // Awakening Hall — Whenever a creature you control dies, each opponent loses 1 life and y
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("funeral-room"),
        name: "Funeral Room // Awakening Hall".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Room"]),
        oracle_text: "Whenever a creature you control dies, each opponent loses 1 life and you gain 1 life.\n(You may cast either half. That door unlocks on the battlefield. As a sorcery, you may pay the mana cost of a locked door to unlock it.)".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
