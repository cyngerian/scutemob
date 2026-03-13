// Temple of the False God — Land, {T}: Add {C}{C} only if you control 5+ lands (conditional — TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("temple-of-the-false-god"),
        name: "Temple of the False God".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}{C}. Activate only if you control five or more lands.".to_string(),
        abilities: vec![
            // TODO: {T}: Add {C}{C}. Activate only if you control five or more lands.
            // DSL gap: conditional activation ("five or more lands" count threshold)
            // and producing two colorless from a single tap are not expressible together.
            // A wrong unconditional {C}{C} implementation would corrupt game state.
        ],
        ..Default::default()
    }
}
