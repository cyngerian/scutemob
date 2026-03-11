// Sundering Eruption // Volcanic Fissure — Destroy target land. Its controller may search their library for a bas
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sundering-eruption"),
        name: "Sundering Eruption // Volcanic Fissure".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield tapped, then shuffle. Creatures without flying can't block this turn.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
