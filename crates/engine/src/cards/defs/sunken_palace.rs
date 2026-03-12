// Sunken Palace
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sunken-palace"),
        name: "Sunken Palace".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Cave"]),
        oracle_text: "This land enters tapped.\n{T}: Add {U}.\n{1}{U}, {T}, Exile seven cards from your graveyard: Add {U}. When you spend this mana to cast a spell or activate an ability, copy that spell or ability. You may choose new targets for the copy. (Mana abilities can't be copied.)".to_string(),
        abilities: vec![
            // TODO: This land enters tapped.
            // TODO: Activated — {T}: Add {U}.
            // TODO: Activated — {1}{U}, {T}, Exile seven cards from your graveyard: Add {U}. When you spend this
        ],
        ..Default::default()
    }
}
