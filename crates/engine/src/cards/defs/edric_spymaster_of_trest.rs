// Edric, Spymaster of Trest — {1}{G}{U}, Legendary Creature — Elf Rogue 2/2
// Whenever a creature deals combat damage to one of your opponents, its controller
// may draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("edric-spymaster-of-trest"),
        name: "Edric, Spymaster of Trest".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Rogue"]),
        oracle_text: "Whenever a creature deals combat damage to one of your opponents, its controller may draw a card.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: DSL gap — "Whenever a creature deals combat damage to one of your opponents"
            // requires a combat-damage trigger condition on any creature (not self-only).
            // No WhenAnyCreatureDealsCombatDamageToPlayer trigger condition exists in the DSL.
        ],
        ..Default::default()
    }
}
