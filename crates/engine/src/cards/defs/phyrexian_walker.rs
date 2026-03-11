// Phyrexian Walker
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phyrexian-walker"),
        name: "Phyrexian Walker".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Phyrexian", "Construct"]),
        oracle_text: "".to_string(),
        power: Some(0),
        toughness: Some(3),
        abilities: vec![],
        ..Default::default()
    }
}
