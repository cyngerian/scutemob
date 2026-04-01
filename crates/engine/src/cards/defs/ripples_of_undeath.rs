// Ripples of Undeath — {1}{B}, Enchantment
// At the beginning of your first main phase, mill three cards. Then you may pay
// 1 life. If you do, return a card from among those milled this way to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ripples-of-undeath"),
        name: "Ripples of Undeath".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your first main phase, mill three cards. Then you may pay 1 life. If you do, return a card from among those milled this way to your hand.".to_string(),
        abilities: vec![
            // TODO: "first main phase" trigger + mill-3 + pay-1-life conditional return.
            // Complex trigger with conditional life payment and mill-tracking not in DSL.
        ],
        ..Default::default()
    }
}
