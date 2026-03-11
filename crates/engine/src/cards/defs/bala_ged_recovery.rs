// Bala Ged Recovery — Return target card from your graveyard to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bala-ged-recovery"),
        name: "Bala Ged Recovery".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery, CardType::Land]),
        oracle_text: "Return target card from your graveyard to your hand.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
