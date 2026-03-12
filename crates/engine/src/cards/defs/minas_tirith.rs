// Minas Tirith
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("minas-tirith"),
        name: "Minas Tirith".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "Minas Tirith enters tapped unless you control a legendary creature.\n{T}: Add {W}.\n{1}{W}, {T}: Draw a card. Activate only if you attacked with two or more creatures this turn.".to_string(),
        abilities: vec![
            // TODO: Minas Tirith enters tapped unless you control a legendary creature.
            // TODO: Activated — {T}: Add {W}.
            // TODO: Activated — {1}{W}, {T}: Draw a card. Activate only if you attacked with two or more creatur
        ],
        ..Default::default()
    }
}
