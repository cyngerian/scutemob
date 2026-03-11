// Malakir Rebirth // Malakir Mire — Choose target creature. You lose 2 life. Until end of turn, that creat
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("malakir-rebirth"),
        name: "Malakir Rebirth // Malakir Mire".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose target creature. You lose 2 life. Until end of turn, that creature gains \"When this creature dies, return it to the battlefield tapped under its owner's control.\"".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
