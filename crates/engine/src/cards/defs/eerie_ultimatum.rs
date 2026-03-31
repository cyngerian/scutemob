// Eerie Ultimatum — {W}{W}{B}{B}{B}{G}{G} Sorcery
// Return any number of permanent cards with different names from your graveyard
// to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eerie-ultimatum"),
        name: "Eerie Ultimatum".to_string(),
        mana_cost: Some(ManaCost { white: 2, black: 3, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return any number of permanent cards with different names from your graveyard to the battlefield.".to_string(),
        abilities: vec![
            // TODO: "Return any number of permanent cards with different names from your
            // graveyard to the battlefield" requires:
            // 1. Variable number of targets ("any number of") — DSL targets are a fixed vec.
            // 2. "different names" constraint across selected cards — no cross-target name
            //    uniqueness validation exists.
            // 3. Filtering for permanent cards (non-instant, non-sorcery) in graveyard.
            // Empty per W5 policy.
        ],
        ..Default::default()
    }
}
