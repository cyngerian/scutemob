// Castle Ardenvale
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("castle-ardenvale"),
        name: "Castle Ardenvale".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Plains.\n{T}: Add {W}.\n{2}{W}{W}, {T}: Create a 1/1 white Human creature token.".to_string(),
        abilities: vec![
            // TODO: Activated — {T}: Add {W}.
            // TODO: Activated — {2}{W}{W}, {T}: Create a 1/1 white Human creature token.
        ],
        ..Default::default()
    }
}
