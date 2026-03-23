// Falkenrath Pit Fighter — {R}, Creature — Vampire Warrior 2/1
// {1}{R}, Discard a card, Sacrifice a Vampire: Draw two cards. Activate only if an
// opponent lost life this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("falkenrath-pit-fighter"),
        name: "Falkenrath Pit Fighter".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Warrior"]),
        oracle_text: "{1}{R}, Discard a card, Sacrifice a Vampire: Draw two cards. Activate only if an opponent lost life this turn.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // TODO: "Sacrifice a Vampire" (not self) + "opponent lost life this turn"
            //   activation condition not expressible. SacrificeSelf was wrong (oracle
            //   says sacrifice any Vampire, not specifically self). Removed to avoid
            //   wrong game state.
        ],
        ..Default::default()
    }
}
