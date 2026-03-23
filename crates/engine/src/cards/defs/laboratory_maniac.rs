// Laboratory Maniac — {2}{U}, Creature — Human Wizard 2/2
// If you would draw a card while your library has no cards in it, you win the game
// instead.
//
// TODO: Draw replacement effect (empty library → win) not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("laboratory-maniac"),
        name: "Laboratory Maniac".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "If you would draw a card while your library has no cards in it, you win the game instead.".to_string(),
        power: Some(2),
        toughness: Some(2),
        // TODO: Draw replacement → win not expressible.
        abilities: vec![],
        ..Default::default()
    }
}
