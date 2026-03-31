// Splendid Reclamation — {3}{G} Sorcery
// Return all land cards from your graveyard to the battlefield tapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("splendid-reclamation"),
        name: "Splendid Reclamation".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return all land cards from your graveyard to the battlefield tapped.".to_string(),
        abilities: vec![
            // TODO: "Return all land cards from your graveyard to the battlefield tapped"
            // requires a ForEach that iterates over all cards in the controller's graveyard
            // filtered by CardType::Land, moving each to the battlefield tapped. The DSL
            // lacks a ForEachTarget::CardsInYourGraveyardWithFilter variant. Empty per W5 policy.
        ],
        ..Default::default()
    }
}
