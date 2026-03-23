// Kazuul, Tyrant of the Cliffs — {3}{R}{R}, Legendary Creature — Ogre Warrior 5/4
// Whenever a creature an opponent controls attacks, if you're the defending player, create
// a 3/3 red Ogre creature token unless that creature's controller pays {3}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kazuul-tyrant-of-the-cliffs"),
        name: "Kazuul, Tyrant of the Cliffs".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Ogre", "Warrior"]),
        oracle_text: "Whenever a creature an opponent controls attacks, if you're the defending player, create a 3/3 red Ogre creature token unless that creature's controller pays {3}.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            // TODO: "Whenever a creature an opponent controls attacks" — per-creature attack
            // trigger with "unless pays {3}" choice not in DSL
        ],
        ..Default::default()
    }
}
