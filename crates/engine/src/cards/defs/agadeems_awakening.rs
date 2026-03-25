// Agadeem's Awakening // Agadeem, the Undercrypt — {X}{B}{B}{B} Sorcery
// Return from your graveyard to the battlefield any number of target creature cards
// that each have a different mana value X or less.
// CR 202.3e: In non-stack zones, X is treated as 0, so mana value = 3 (black only).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("agadeems-awakening"),
        name: "Agadeem's Awakening // Agadeem, the Undercrypt".to_string(),
        mana_cost: Some(ManaCost { black: 3, x_count: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return from your graveyard to the battlefield any number of target creature cards that each have a different mana value X or less.".to_string(),
        abilities: vec![
            // TODO: "Return creature cards from your graveyard with different mana values X or less."
            // Requires: multi-target graveyard selection with mana value <= X filter (dynamic),
            // and each-different-mana-value uniqueness constraint. Neither is expressible in the
            // current DSL. Deferred until dynamic MV filter and multi-target GY selection are added.
        ],
        ..Default::default()
    }
}
