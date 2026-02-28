//! Deck construction helpers for the simulator.
//!
//! Builds decks from the existing 66+ CardDefinitions in the engine.
//! Picks a legendary creature as commander, fills with cards matching
//! color identity, and pads with basic lands.

use mtg_engine::{
    all_cards, compute_color_identity, CardDefinition, CardId, CardRegistry, CardType, Color,
    SuperType,
};
use rand::prelude::*;
use std::sync::Arc;

/// Configuration for a deck to use in a simulated game.
#[derive(Clone, Debug)]
pub struct DeckConfig {
    /// CardId of the commander.
    pub commander: CardId,
    /// CardIds of the 99 main deck cards (including basic lands).
    pub main_deck: Vec<CardId>,
}

/// Build a random Commander deck from available CardDefinitions.
///
/// Strategy:
/// 1. Pick a random legendary creature as commander
/// 2. Compute its color identity
/// 3. Gather all cards that fit within that color identity
/// 4. Fill to 99 cards, padding with matching basic lands
pub fn random_deck(rng: &mut StdRng, cards: &[CardDefinition]) -> Option<DeckConfig> {
    // Find all legendary creatures (potential commanders)
    let commanders: Vec<&CardDefinition> = cards
        .iter()
        .filter(|c| {
            c.types.supertypes.contains(&SuperType::Legendary)
                && c.types.card_types.contains(&CardType::Creature)
        })
        .collect();

    if commanders.is_empty() {
        return None;
    }

    let commander = commanders[rng.gen_range(0..commanders.len())];
    let color_identity = compute_color_identity(commander);

    // Gather cards fitting within the commander's color identity.
    // Exclude lands (we'll add basics), exclude the commander itself.
    let eligible: Vec<&CardDefinition> = cards
        .iter()
        .filter(|c| {
            if c.card_id == commander.card_id {
                return false;
            }
            // Check color identity fits within commander's identity
            let card_ci = compute_color_identity(c);
            card_ci.iter().all(|color| color_identity.contains(color))
        })
        .collect();

    // Split into non-lands and lands
    let non_lands: Vec<&CardDefinition> = eligible
        .iter()
        .filter(|c| !c.types.card_types.contains(&CardType::Land))
        .copied()
        .collect();

    let non_basic_lands: Vec<&CardDefinition> = eligible
        .iter()
        .filter(|c| {
            c.types.card_types.contains(&CardType::Land)
                && !c.types.supertypes.contains(&SuperType::Basic)
        })
        .copied()
        .collect();

    let mut main_deck: Vec<CardId> = Vec::new();

    // Add non-land cards (up to 60 non-lands, singleton)
    let mut shuffled_nonlands: Vec<&CardDefinition> = non_lands.clone();
    shuffled_nonlands.shuffle(rng);
    for card in shuffled_nonlands.into_iter().take(60) {
        main_deck.push(card.card_id.clone());
    }

    // Add non-basic lands (up to 5)
    let mut shuffled_lands: Vec<&CardDefinition> = non_basic_lands.clone();
    shuffled_lands.shuffle(rng);
    for card in shuffled_lands.into_iter().take(5) {
        main_deck.push(card.card_id.clone());
    }

    // Pad to 99 with basic lands matching the color identity
    let basics = basics_for_colors(&color_identity);
    while main_deck.len() < 99 {
        if basics.is_empty() {
            // Colorless commander — use Wastes (or just any basic)
            main_deck.push(CardId("forest".to_string()));
        } else {
            let basic = &basics[rng.gen_range(0..basics.len())];
            main_deck.push(basic.clone());
        }
    }

    // Truncate if we somehow got over 99
    main_deck.truncate(99);

    Some(DeckConfig {
        commander: commander.card_id.clone(),
        main_deck,
    })
}

/// Get basic land CardIds for a set of colors.
fn basics_for_colors(colors: &[Color]) -> Vec<CardId> {
    let mut basics = Vec::new();
    for color in colors {
        match color {
            Color::White => basics.push(CardId("plains".to_string())),
            Color::Blue => basics.push(CardId("island".to_string())),
            Color::Black => basics.push(CardId("swamp".to_string())),
            Color::Red => basics.push(CardId("mountain".to_string())),
            Color::Green => basics.push(CardId("forest".to_string())),
        }
    }
    if basics.is_empty() {
        // Colorless — fall back to forest
        basics.push(CardId("forest".to_string()));
    }
    basics
}

/// Build a CardRegistry containing all known cards plus enough basic lands.
pub fn build_registry() -> Arc<CardRegistry> {
    CardRegistry::new(all_cards())
}
