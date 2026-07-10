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
            // CR 400.7, 603.6a: Return all land cards from the controller's graveyard
            // to the battlefield simultaneously, each entering tapped.
            AbilityDefinition::Spell {
                effect: Effect::ReturnAllFromGraveyardToBattlefield {
                    graveyards: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    tapped: true,
                    controller_override: None,
                    unique_names: false,
                    permanent_cards_only: false,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
