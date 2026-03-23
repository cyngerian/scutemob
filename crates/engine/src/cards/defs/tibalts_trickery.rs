// Tibalt's Trickery — {1}{R}, Instant
// Counter target spell. Choose 1, 2, or 3 at random. Its controller mills that many cards,
// then exiles cards from the top of their library until they exile a nonland card with a
// different name. They may cast that card without paying its mana cost. Then they put the
// exiled cards on the bottom of their library in a random order.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tibalts-trickery"),
        name: "Tibalt's Trickery".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. Choose 1, 2, or 3 at random. Its controller mills that many cards, then exiles cards from the top of their library until they exile a nonland card with a different name than that spell. They may cast that card without paying its mana cost. Then they put the exiled cards on the bottom of their library in a random order.".to_string(),
        // TODO: Counter target spell + random mill + exile-until-nonland + free cast for opponent.
        // Hard counter in red without compensation is color-pie violation (KI-2).
        // Stripped per W6 policy until the full effect chain is expressible.
        abilities: vec![],
        ..Default::default()
    }
}
