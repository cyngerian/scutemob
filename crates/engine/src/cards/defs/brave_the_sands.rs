// Brave the Sands — {1}{W}, Enchantment
// TODO: DSL gap — static ability "Creatures you control have vigilance."
//   (global keyword grant to all controlled creatures not supported in card DSL)
// TODO: DSL gap — static ability "Each creature you control can block an additional creature
//   each combat." (additional blocker assignment not supported in card DSL)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brave-the-sands"),
        name: "Brave the Sands".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control have vigilance.\nEach creature you control can block an additional creature each combat.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
