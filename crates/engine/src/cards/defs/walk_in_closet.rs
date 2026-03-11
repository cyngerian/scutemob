// Walk-In Closet // Forgotten Cellar — You may play lands from your graveyard.\n(You may cast either half. Th
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("walk-in-closet"),
        name: "Walk-In Closet // Forgotten Cellar".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Room"]),
        oracle_text: "You may play lands from your graveyard.\n(You may cast either half. That door unlocks on the battlefield. As a sorcery, you may pay the mana cost of a locked door to unlock it.)".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
