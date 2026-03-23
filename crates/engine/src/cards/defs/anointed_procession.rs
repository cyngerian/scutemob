// Anointed Procession — {3}{W}, Enchantment
// If an effect would create one or more tokens under your control, it creates twice that
// many of those tokens instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("anointed-procession"),
        name: "Anointed Procession".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Enchantment], &[]),
        oracle_text: "If an effect would create one or more tokens under your control, it creates twice that many of those tokens instead.".to_string(),
        abilities: vec![
            // TODO: Token doubling replacement effect not in DSL
        ],
        ..Default::default()
    }
}
