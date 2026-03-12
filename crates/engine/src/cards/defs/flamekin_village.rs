// Flamekin Village
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flamekin-village"),
        name: "Flamekin Village".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, you may reveal an Elemental card from your hand. If you don't, this land enters tapped.\n{T}: Add {R}.\n{R}, {T}: Target creature gains haste until end of turn.".to_string(),
        abilities: vec![
            // TODO: Static — As this land enters, you may reveal an Elemental card from your hand. If you don
            // TODO: Activated — {T}: Add {R}.
            // TODO: Activated — {R}, {T}: Target creature gains haste until end of turn.
        ],
        ..Default::default()
    }
}
