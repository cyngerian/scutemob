// Halimar Depths
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("halimar-depths"),
        name: "Halimar Depths".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\nWhen this land enters, look at the top three cards of your library, then put them back in any order.\n{T}: Add {U}.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped.
            // TODO: Triggered — When this land enters, look at the top three cards of your library, then put the
            // TODO: Activated — {T}: Add {U}.
        ],
        ..Default::default()
    }
}
