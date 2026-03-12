// Creeping Tar Pit
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("creeping-tar-pit"),
        name: "Creeping Tar Pit".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {U} or {B}.\n{1}{U}{B}: Until end of turn, this land becomes a 3/2 blue and black Elemental creature. It's still a land. It can't be blocked this turn.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped.
            // TODO: Activated — {T}: Add {U} or {B}.
            // TODO: Activated — {1}{U}{B}: Until end of turn, this land becomes a 3/2 blue and black Elemental c
        ],
        ..Default::default()
    }
}
