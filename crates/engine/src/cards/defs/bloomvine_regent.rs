// Bloomvine Regent // Claim Territory — Flying\nWhenever this creature or another Dragon you control enters, y
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloomvine-regent"),
        name: "Bloomvine Regent // Claim Territory".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever this creature or another Dragon you control enters, you gain 3 life.".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![],
        ..Default::default()
    }
}
