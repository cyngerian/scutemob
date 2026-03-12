// Den of the Bugbear
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("den-of-the-bugbear"),
        name: "Den of the Bugbear".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "If you control two or more other lands, this land enters tapped.\n{T}: Add {R}.\n{3}{R}: Until end of turn, this land becomes a 3/2 red Goblin creature with \"Whenever this creature attacks, create a 1/1 red Goblin creature token that's tapped and attacking.\" It's still a land.".to_string(),
        abilities: vec![
            // TODO: If you control two or more other lands, this land enters tapped.
            // TODO: Activated — {T}: Add {R}.
            // TODO: Activated — {3}{R}: Until end of turn, this land becomes a 3/2 red Goblin creature with "Whe
        ],
        ..Default::default()
    }
}
