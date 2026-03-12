// Mishra, Claimed by Gix
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mishra-claimed-by-gix"),
        name: "Mishra, Claimed by Gix".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Artificer", "Phyrexian"]),
        oracle_text: "Whenever you attack, each opponent loses X life and you gain X life, where X is the number of attacking creatures. If Mishra, Claimed by Gix and a creature named Phyrexian Dragon Engine are attacking, and you both own and control them, exile them, then meld them into Mishra, Lost to Phyrexia. It enters tapped and attacking.".to_string(),
        abilities: vec![
            // TODO: Triggered — Whenever you attack, each opponent loses X life and you gain X life, where X is 
        ],
        power: Some(3),
        toughness: Some(5),
        ..Default::default()
    }
}
