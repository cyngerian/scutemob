// Doubling Season — {4}{G}, Enchantment
// If an effect would create one or more tokens under your control, it creates twice that
// many of those tokens instead.
// If an effect would put one or more counters on a permanent you control, it puts twice
// that many of those counters on that permanent instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("doubling-season"),
        name: "Doubling Season".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "If an effect would create one or more tokens under your control, it creates twice that many of those tokens instead.\nIf an effect would put one or more counters on a permanent you control, it puts twice that many of those counters on that permanent instead.".to_string(),
        abilities: vec![
            // TODO: Token doubling replacement effect not in DSL.
            // TODO: Counter doubling replacement effect not in DSL.
        ],
        ..Default::default()
    }
}
