// Keeper of Fables — {3}{G}{G}, Creature — Cat 4/5
// Whenever one or more non-Human creatures you control deal combat damage to a player,
// draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("keeper-of-fables"),
        name: "Keeper of Fables".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: creature_types(&["Cat"]),
        oracle_text: "Whenever one or more non-Human creatures you control deal combat damage to a player, draw a card.".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            // TODO: DSL gap — "Whenever one or more non-Human creatures you control deal
            // combat damage to a player" requires a per-combat-damage trigger with a
            // type-filtered (non-Human) creature-you-control condition. No such trigger
            // condition exists in the DSL.
        ],
        ..Default::default()
    }
}
