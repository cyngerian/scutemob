// Martial Coup — {X}{W}{W}, Sorcery
// Create X 1/1 white Soldier creature tokens. If X is 5 or more, destroy all other creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("martial-coup"),
        name: "Martial Coup".to_string(),
        mana_cost: Some(ManaCost { white: 2, ..Default::default() }),
        types: full_types(&[], &[CardType::Sorcery], &[]),
        oracle_text: "Create X 1/1 white Soldier creature tokens. If X is 5 or more, destroy all other creatures.".to_string(),
        abilities: vec![
            // TODO: X-based token count + conditional board wipe (X >= 5) not in DSL
            // Would need EffectAmount::XValue for token count and Conditional with X threshold
        ],
        ..Default::default()
    }
}
