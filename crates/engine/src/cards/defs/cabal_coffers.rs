// Cabal Coffers — Land, {2},{T}: Add {B} for each Swamp you control (count-based, TODO)
// TODO: {2}, {T}: Add {B} for each Swamp you control
// — count-based mana scaling (count controlled Swamps) not expressible in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cabal-coffers"),
        name: "Cabal Coffers".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{2}, {T}: Add {B} for each Swamp you control.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
