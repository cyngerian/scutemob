// Mavren Fein, Dusk Apostle — {2}{W}, Legendary Creature — Vampire Cleric 2/2
// Whenever one or more nontoken Vampires you control attack, create a 1/1 white Vampire
// creature token with lifelink.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mavren-fein-dusk-apostle"),
        name: "Mavren Fein, Dusk Apostle".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Vampire", "Cleric"]),
        oracle_text: "Whenever one or more nontoken Vampires you control attack, create a 1/1 white Vampire creature token with lifelink.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Whenever one or more nontoken Vampires you control attack" — batch attack
            // trigger with subtype + nontoken filter not in DSL. WhenAttacks fires for self only,
            // not for other Vampires. Empty to avoid wrong game state.
        ],
        ..Default::default()
    }
}
