// Frontier Siege — {3}{G}, Enchantment
// As this enchantment enters, choose Khans or Dragons.
// Khans — At the beginning of each of your main phases, add {G}{G}.
// Dragons — Whenever a creature you control with flying enters, you may have it fight target creature you don't control.
//
// TODO: Modal ETB choice (Khans vs Dragons), phase-specific mana trigger, and
// "whenever creature with flying ETB, fight" conditional trigger all exceed
// the current DSL. No faithful partial implementation is possible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("frontier-siege"),
        name: "Frontier Siege".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "As this enchantment enters, choose Khans or Dragons.\n• Khans — At the beginning of each of your main phases, add {G}{G}.\n• Dragons — Whenever a creature you control with flying enters, you may have it fight target creature you don't control.".to_string(),
        abilities: vec![
            // TODO: modal ETB choice mechanic not in DSL.
            // Khans mode: mana add at beginning of each main phase (not a standard trigger condition).
            // Dragons mode: flying creature ETB → conditional fight target — no Fight Effect in DSL.
        ],
        ..Default::default()
    }
}
