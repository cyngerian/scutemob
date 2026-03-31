// Gamble — {R}, Sorcery; search your library for a card, put it into your hand,
// discard a card at random, then shuffle.
// TODO: Effect::DiscardAtRandom does not exist in the DSL. The "discard a card at random"
// effect cannot be distinguished from a player-chosen discard. Implementing the search
// without the random discard would produce wrong game state (free tutor). Using empty
// abilities per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gamble"),
        name: "Gamble".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "Search your library for a card, put that card into your hand, discard a card at random, then shuffle."
                .to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
