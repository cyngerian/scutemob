// Gaea's Cradle — Legendary Land
// {T}: Add {G} for each creature you control — scales with board state,
// not expressible in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gaeas-cradle"),
        name: "Gaea's Cradle".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {G} for each creature you control.".to_string(),
        abilities: vec![
            // TODO: {T}: Add {G} for each creature you control — variable mana
            // production based on battlefield count is not expressible in the DSL
            // (EffectAmount has no CountCreaturesYouControl variant)
        ],
        ..Default::default()
    }
}
