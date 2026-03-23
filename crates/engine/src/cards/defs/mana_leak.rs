// Mana Leak — {1}{U}, Instant
// Counter target spell unless its controller pays {3}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-leak"),
        name: "Mana Leak".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell unless its controller pays {3}.".to_string(),
        // TODO: "Counter unless controller pays {3}" — requires CounterUnlessPays effect.
        // Unconditional CounterSpell is strictly better than Counterspell for {1}{U} (KI-2).
        // Stripped per W6 policy.
        abilities: vec![],
        ..Default::default()
    }
}
