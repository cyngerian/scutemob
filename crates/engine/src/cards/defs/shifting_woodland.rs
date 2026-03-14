// Shifting Woodland
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shifting-woodland"),
        name: "Shifting Woodland".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Forest.\n{T}: Add {G}.\nDelirium — {2}{G}{G}: This land becomes a copy of target permanent card in your graveyard until end of turn. Activate only if there are four or more card types among cards in your graveyard.".to_string(),
        abilities: vec![
            // TODO: Activated — {T}: Add {G}.
            // TODO: Keyword — Delirium — {2}{G}{G}: This land becomes a copy of target permanent card in your 
        ],
        ..Default::default()
    }
}
