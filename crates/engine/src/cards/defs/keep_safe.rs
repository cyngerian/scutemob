// Keep Safe — {1}{U}, Instant
// Counter target spell that targets a permanent you control.
// Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("keep-safe"),
        name: "Keep Safe".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell that targets a permanent you control.\nDraw a card.".to_string(),
        // TODO: "Counter target spell that targets a permanent you control" — requires
        // spell-target-filter (must target your permanent). Overbroad targeting is wrong
        // game state (KI-2). Stripped per W6 policy.
        // When fixed, also add: Draw a card.
        abilities: vec![],
        ..Default::default()
    }
}
