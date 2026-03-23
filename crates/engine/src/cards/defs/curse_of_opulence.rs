// Curse of Opulence — {R}, Enchantment — Aura Curse
// Enchant player
// Whenever enchanted player is attacked, create a Gold token. Each opponent attacking
// that player does the same.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("curse-of-opulence"),
        name: "Curse of Opulence".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura", "Curse"]),
        oracle_text: "Enchant player\nWhenever enchanted player is attacked, create a Gold token. Each opponent attacking that player does the same.".to_string(),
        abilities: vec![
            // TODO: "Enchant player" not in EnchantTarget enum.
            // TODO: "Whenever enchanted player is attacked" trigger not in DSL.
            // TODO: Gold token spec not a helper function.
        ],
        ..Default::default()
    }
}
