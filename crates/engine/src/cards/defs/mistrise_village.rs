// Mistrise Village
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mistrise-village"),
        name: "Mistrise Village".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Mountain or a Forest.\n{T}: Add {U}.\n{U}, {T}: The next spell you cast this turn can't be countered.".to_string(),
        abilities: vec![
            // TODO: Activated — {T}: Add {U}.
            // TODO: Activated — {U}, {T}: The next spell you cast this turn can't be countered.
        ],
        ..Default::default()
    }
}
