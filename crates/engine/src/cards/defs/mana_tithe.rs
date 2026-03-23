// Mana Tithe — {W}, Instant
// Counter target spell unless its controller pays {1}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-tithe"),
        name: "Mana Tithe".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell unless its controller pays {1}.".to_string(),
        // TODO: "Counter unless controller pays {1}" — requires CounterUnlessPays effect.
        // Unconditional CounterSpell for {W} is wrong game state (KI-2).
        // Stripped per W6 policy.
        abilities: vec![],
        ..Default::default()
    }
}
