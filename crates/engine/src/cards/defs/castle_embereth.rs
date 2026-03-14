// Castle Embereth
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("castle-embereth"),
        name: "Castle Embereth".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Mountain.\n{T}: Add {R}.\n{1}{R}{R}, {T}: Creatures you control get +1/+0 until end of turn.".to_string(),
        abilities: vec![
            // TODO: Activated — {T}: Add {R}.
            // TODO: Activated — {1}{R}{R}, {T}: Creatures you control get +1/+0 until end of turn.
        ],
        ..Default::default()
    }
}
