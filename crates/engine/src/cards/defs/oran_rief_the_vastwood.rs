// Oran-Rief, the Vastwood
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oran-rief-the-vastwood"),
        name: "Oran-Rief, the Vastwood".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {G}.\n{T}: Put a +1/+1 counter on each green creature that entered this turn.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped.
            // TODO: Activated — {T}: Add {G}.
            // TODO: Activated — {T}: Put a +1/+1 counter on each green creature that entered this turn.
        ],
        ..Default::default()
    }
}
