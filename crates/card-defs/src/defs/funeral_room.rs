// Funeral Room // Awakening Hall — Whenever a creature you control dies, each opponent loses 1 life and y
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("funeral-room"),
        name: "Funeral Room // Awakening Hall".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Room"]),
        oracle_text: "Whenever a creature you control dies, each opponent loses 1 life and you gain 1 life.\n(You may cast either half. That door unlocks on the battlefield. As a sorcery, you may pay the mana cost of a locked door to unlock it.)".to_string(),
        abilities: vec![],
        completeness: Completeness::inert("Blocked on the Room / door-unlock mechanic (CR 725): no two-door split-enchantment representation, no per-door mana costs, no unlock action, and no 'when you unlock this door' trigger. The Funeral Room door's own body is expressible today (WheneverCreatureDies{controller: You} + LoseLife EachOpponent + GainLife Controller); Awakening Hall's door is unauthored. Shares this blocker with bottomless_pool.rs."),
        ..Default::default()
    }
}
