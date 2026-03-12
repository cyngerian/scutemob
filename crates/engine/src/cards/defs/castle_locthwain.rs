// Castle Locthwain
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("castle-locthwain"),
        name: "Castle Locthwain".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Swamp.\n{T}: Add {B}.\n{1}{B}{B}, {T}: Draw a card, then you lose life equal to the number of cards in your hand.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped unless you control a Swamp.
            // TODO: Activated — {T}: Add {B}.
            // TODO: Activated — {1}{B}{B}, {T}: Draw a card, then you lose life equal to the number of cards in 
        ],
        ..Default::default()
    }
}
