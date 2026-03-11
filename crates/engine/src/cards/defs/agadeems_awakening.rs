// Agadeem's Awakening // Agadeem, the Undercrypt — Return from your graveyard to the battlefield any number of target cre
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("agadeems-awakening"),
        name: "Agadeem's Awakening // Agadeem, the Undercrypt".to_string(),
        mana_cost: Some(ManaCost { black: 3, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return from your graveyard to the battlefield any number of target creature cards that each have a different mana value X or less.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
