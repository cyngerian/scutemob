// Spinerock Knoll
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spinerock-knoll"),
        name: "Spinerock Knoll".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Hideaway 4 (When this land enters, look at the top four cards of your library, exile one face down, then put the rest on the bottom in a random order.)\nThis land enters tapped.\n{T}: Add {R}.\n{R}, {T}: You may play the exiled card without paying its mana cost if an opponent was dealt 7 or more damage this turn.".to_string(),
        abilities: vec![
            // TODO: Keyword — Hideaway 4 (When this land enters, look at the top four cards of your library, e
            // TODO: This land enters tapped.
            // TODO: Activated — {T}: Add {R}.
            // TODO: Activated — {R}, {T}: You may play the exiled card without paying its mana cost if an oppone
        ],
        ..Default::default()
    }
}
