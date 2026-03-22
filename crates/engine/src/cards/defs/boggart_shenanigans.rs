// Boggart Shenanigans — {2}{R} Kindred Enchantment — Goblin
// Whenever another Goblin you control is put into a graveyard from the battlefield,
// you may have this enchantment deal 1 damage to target player or planeswalker.
//
// TODO: DSL gap — The triggered ability requires filtering creature deaths by
// subtype (Goblin) AND controller (you) AND "another" (not self). The available
// TriggerCondition::WheneverCreatureDies is overbroad (fires on all creature
// deaths regardless of type or controller). No filtered variant exists for
// "whenever a creature of subtype X you control dies". Deferred until
// WheneverCreatureDiesWithFilter or similar is available (see KI-5 pattern).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boggart-shenanigans"),
        name: "Boggart Shenanigans".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types_sub(&[CardType::Kindred, CardType::Enchantment], &["Goblin"]),
        oracle_text: "Whenever another Goblin you control is put into a graveyard from the battlefield, you may have this enchantment deal 1 damage to target player or planeswalker.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
