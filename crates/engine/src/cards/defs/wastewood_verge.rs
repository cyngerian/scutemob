// Wastewood Verge — dual verge land, {T}: Add {G}. {T}: Add {B} (only if you control a Swamp or Forest, TODO restriction).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wastewood-verge"),
        name: "Wastewood Verge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {G} or {B}. Activate only if you control a Forest, a Swamp, or a combination of both.".to_string(),
        abilities: vec![
            // TODO: {T}: Add {G} or {B}. Activate only if you control a Forest, a Swamp, or
            // a combination of both. DSL gap: no activation condition on Activated abilities.
        ],
        ..Default::default()
    }
}
