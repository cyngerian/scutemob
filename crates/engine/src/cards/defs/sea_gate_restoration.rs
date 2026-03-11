// Sea Gate Restoration // Sea Gate, Reborn — Draw cards equal to the number of cards in your hand plus one. You hav
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sea-gate-restoration"),
        name: "Sea Gate Restoration // Sea Gate, Reborn".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 3, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw cards equal to the number of cards in your hand plus one. You have no maximum hand size for the rest of the game.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
