// Kaya's Ghostform — {B}, Enchantment — Aura
// Enchant creature or planeswalker you control
// When enchanted permanent dies or is put into exile, return that card to the battlefield
// under your control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kayas-ghostform"),
        name: "Kaya's Ghostform".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature or planeswalker you control\nWhen enchanted permanent dies or is put into exile, return that card to the battlefield under your control.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // TODO: DSL gap — "Enchant creature or planeswalker" (not just creature).
            // Also: "When enchanted permanent dies or is exiled, return it" needs a
            // trigger on attached permanent's zone change + return from GY/exile.
        ],
        ..Default::default()
    }
}
