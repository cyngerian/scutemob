// Blazemire Verge — dual verge land, {T}: Add {B}. {T}: Add {R} (only if you control a Swamp or Mountain, TODO restriction).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blazemire-verge"),
        name: "Blazemire Verge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {B} or {R}. Activate only if you control a Swamp, a Mountain, or a combination of both.".to_string(),
        abilities: vec![
            // TODO: {T}: Add {B} or {R}. Activate only if you control a Swamp, a Mountain, or
            // a combination of both. DSL gap: no activation condition on Activated abilities.
        ],
        ..Default::default()
    }
}
