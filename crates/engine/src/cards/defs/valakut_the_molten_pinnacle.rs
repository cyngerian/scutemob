// Valakut, the Molten Pinnacle
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("valakut-the-molten-pinnacle"),
        name: "Valakut, the Molten Pinnacle".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\nWhenever a Mountain you control enters, if you control at least five other Mountains, you may have this land deal 3 damage to any target.\n{T}: Add {R}.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped.
            // TODO: Triggered — Whenever a Mountain you control enters, if you control at least five other Mount
            // TODO: Activated — {T}: Add {R}.
        ],
        ..Default::default()
    }
}
