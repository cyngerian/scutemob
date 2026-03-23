// Teferi's Ageless Insight — {2}{U}{U}, Legendary Enchantment
// If you would draw a card except the first one you draw in each of your draw steps,
// draw two cards instead.
//
// TODO: Draw replacement effect with draw-step exception too complex for DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teferis-ageless-insight"),
        name: "Teferi's Ageless Insight".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Enchantment]),
        oracle_text: "If you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.".to_string(),
        // TODO: Draw replacement effect not expressible.
        abilities: vec![],
        ..Default::default()
    }
}
