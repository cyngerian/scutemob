// Arena of Glory
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arena-of-glory"),
        name: "Arena of Glory".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Mountain.\n{T}: Add {R}.\n{R}, {T}, Exert this land: Add {R}{R}. If that mana is spent on a creature spell, it gains haste until end of turn. (An exerted permanent won't untap during your next untap step.)".to_string(),
        abilities: vec![
            // TODO: This land enters tapped unless you control a Mountain.
            // TODO: Activated — {T}: Add {R}.
            // TODO: Activated — {R}, {T}, Exert this land: Add {R}{R}. If that mana is spent on a creature spell
        ],
        ..Default::default()
    }
}
