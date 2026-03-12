// Gruul Turf
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gruul-turf"),
        name: "Gruul Turf".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\nWhen this land enters, return a land you control to its owner's hand.\n{T}: Add {R}{G}.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped.
            // TODO: Triggered — When this land enters, return a land you control to its owner's hand.
            // TODO: Activated — {T}: Add {R}{G}.
        ],
        ..Default::default()
    }
}
