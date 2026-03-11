// Memnite
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("memnite"),
        name: "Memnite".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: creature_types(&["Construct"]),
        oracle_text: "".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}
