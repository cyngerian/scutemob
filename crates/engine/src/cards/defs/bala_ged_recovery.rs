// Bala Ged Recovery // Bala Ged Sanctuary — MDFC Sorcery // Land
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bala-ged-recovery"),
        name: "Bala Ged Recovery // Bala Ged Sanctuary".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return target card from your graveyard to your hand.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
