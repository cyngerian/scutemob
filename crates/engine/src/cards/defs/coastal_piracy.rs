// Coastal Piracy — {2}{U}{U}, Enchantment
// Whenever a creature you control deals combat damage to an opponent, you may draw a card.
//
// TODO: Per-creature combat damage trigger not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("coastal-piracy"),
        name: "Coastal Piracy".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control deals combat damage to an opponent, you may draw a card.".to_string(),
        // TODO: Per-creature combat damage trigger not in DSL.
        abilities: vec![],
        ..Default::default()
    }
}
