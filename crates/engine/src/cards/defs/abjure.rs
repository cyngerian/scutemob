// Abjure — {U}, Instant
// As an additional cost to cast this spell, sacrifice a blue permanent.
// Counter target spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abjure"),
        name: "Abjure".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a blue permanent.\nCounter target spell.".to_string(),
        // TODO: "As an additional cost, sacrifice a blue permanent. Counter target spell."
        // Without the sacrifice cost, this is a {U} unconditional counter (KI-2).
        // Stripped per W6 policy until color-filtered sacrifice additional cost is in DSL.
        abilities: vec![],
        ..Default::default()
    }
}
