// Castle Vantress
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("castle-vantress"),
        name: "Castle Vantress".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control an Island.\n{T}: Add {U}.\n{2}{U}{U}, {T}: Scry 2.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped unless you control an Island.
            // TODO: Activated — {T}: Add {U}.
            // TODO: Activated — {2}{U}{U}, {T}: Scry 2.
        ],
        ..Default::default()
    }
}
