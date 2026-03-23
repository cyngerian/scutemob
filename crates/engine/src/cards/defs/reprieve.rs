// Reprieve — {1}{W}, Instant
// Return target spell to its owner's hand.
// Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reprieve"),
        name: "Reprieve".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return target spell to its owner's hand.\nDraw a card.".to_string(),
        // TODO: "Return target spell to its owner's hand" — requires MoveZone from Stack
        // to Hand (bounce-spell, not counter). Draw a card is expressible but partial without
        // the bounce makes this wrong game state (KI-2). Stripped per W6 policy.
        abilities: vec![],
        ..Default::default()
    }
}
