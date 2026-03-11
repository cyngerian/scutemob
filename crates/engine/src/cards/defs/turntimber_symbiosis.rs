// Turntimber Symbiosis // Turntimber, Serpentine Wood — Look at the top seven cards of your library. You may put a creature ca
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("turntimber-symbiosis"),
        name: "Turntimber Symbiosis // Turntimber, Serpentine Wood".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 3, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Look at the top seven cards of your library. You may put a creature card from among them onto the battlefield. If that card has mana value 3 or less, it enters with three additional +1/+1 counters on it. Put the rest on the bottom of your library in a random order.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
