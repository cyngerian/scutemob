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
    // Architecture Invariant 9 (SR-12): the fuzzer must only build games out of
    // faithfully-implemented cards. A non-`Complete` def corrupts the replay
    // history, and `start_game` now refuses any game that contains one — so a
    // deck drawn here that included an inert / partial / knowingly-wrong card
    // would simply abort at `run_game` with `IncompleteCardsInGame`. Filtering
    // to `Complete` up front keeps the fuzzer exercising real play instead.
    // Find all legendary creatures (potential commanders). Colorless identities are
    // INCLUDED — see the CR 903.5c padding arm below, which is what makes them legal
    // (OOS-M11-6, closed by PB-DX4).
    let commanders: Vec<&CardDefinition> = cards
        .iter()
        .filter(|c| {
            c.completeness.is_complete()
                && c.types.supertypes.contains(&SuperType::Legendary)
                && c.types.card_types.contains(&CardType::Creature)
        })
        .collect();

    if commanders.is_empty() {
        return None;
    }

    let commander = commanders[rng.random_range(0..commanders.len())];
    let color_identity = compute_color_identity(commander);

    // Gather cards fitting within the commander's color identity.
    // Exclude lands (we'll add basics), exclude the commander itself.
    let eligible: Vec<&CardDefinition> = cards
        .iter()
        .filter(|c| {
            if c.card_id == commander.card_id {
                return false;
            }
            // Architecture Invariant 9 (SR-12): only faithfully-implemented cards.
            if !c.completeness.is_complete() {
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

    // Pad to 99 with basic lands matching the color identity.
    let basics = basics_for_colors(&color_identity);
    if basics.is_empty() {
        // CR 903.5c — the colorless-commander case. **OOS-M11-6, closed here by PB-DX4
        // (2026-08-01, `scutemob-168`).**
        //
        // This arm used to push `CardId("forest")` under the comment "Colorless commander —
        // use Wastes (or just any basic)". It did not use Wastes (there is no `wastes.rs` in
        // the corpus) and "any basic" is not legal: Forest's color identity is {Green} and a
        // colorless commander's is {}, so a deck padded this way carried ~34 illegal Forests
        // and `validate_deck` — which PB-M11-S2 routed `build_initial_state` through —
        // refused the whole table. Worse, `bin/fuzzer.rs` feeds `random_deck` straight into
        // `GameStateBuilder` with NO validation, so the fuzzer silently PLAYED those illegal
        // decks rather than refusing them.
        //
        // The fix is the one OOS-M11-6 named as preferable: pad from the identity-legal
        // colorless cards already in `eligible`, needing no new card def and no `Complete`
        // flip. Viability was measured, not assumed — the `Complete` pool holds 40 colorless
        // nonbasic lands and 82 colorless nonlands, 122 distinct singletons against the 99 a
        // deck needs. Basics are exempt from the CR 903.5b singleton rule and these are not,
        // so each is taken at most once; `None` if the pool ever cannot fill 99, which is a
        // refusal rather than the silent illegal deck this replaces.
        let mut filler: Vec<&CardDefinition> = eligible
            .iter()
            .filter(|c| !main_deck.contains(&c.card_id))
            .copied()
            .collect();
        filler.shuffle(rng);
        for card in filler {
            if main_deck.len() >= 99 {
                break;
            }
            main_deck.push(card.card_id.clone());
        }
        if main_deck.len() < 99 {
            return None;
        }
    } else {
        while main_deck.len() < 99 {
            let basic = &basics[rng.random_range(0..basics.len())];
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
    // PB-DX4 (2026-08-01, `scutemob-168`, OOS-M11-6): the "Colorless — fall back to forest"
    // arm that used to live here is DELETED, not repaired. It was the reason the caller's own
    // `basics.is_empty()` branch was dead code, so the caller "handled" the colorless case
    // with a second Forest push that could never run -- the code named the right fix in a
    // comment twice and did it neither time. An empty return is now meaningful: it says "this
    // identity has no legal basic land", which is exactly true of a colorless commander (there
    // is no `wastes.rs` in the corpus), and the caller pads from identity-legal colorless
    // cards instead. Do not restore a fallback here; a Forest is a CR 903.5c violation on
    // every copy.
    basics
}

/// Build a CardRegistry containing all known cards plus enough basic lands.
pub fn build_registry() -> Arc<CardRegistry> {
    CardRegistry::new(all_cards())
}
