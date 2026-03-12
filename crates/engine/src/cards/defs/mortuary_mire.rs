// Mortuary Mire
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mortuary-mire"),
        name: "Mortuary Mire".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\nWhen this land enters, you may put target creature card from your graveyard on top of your library.\n{T}: Add {B}.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped.
            // TODO: Triggered — When this land enters, you may put target creature card from your graveyard on t
            // TODO: Activated — {T}: Add {B}.
        ],
        ..Default::default()
    }
}
