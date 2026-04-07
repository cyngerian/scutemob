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
            // CR 400.7, 603.6a: Return permanent cards with different names from the
            // controller's graveyard to the battlefield.
            // Deterministic fallback: return ALL qualifying unique-name permanent cards
            // (maximum greed — lowest ObjectId per name wins). Interactive "any number"
            // player choice deferred to M10+.
            // TODO(M10+): Add interactive selection so player can choose a subset.
            AbilityDefinition::Spell {
                effect: Effect::ReturnAllFromGraveyardToBattlefield {
                    graveyards: PlayerTarget::Controller,
                    filter: TargetFilter::default(),
                    tapped: false,
                    controller_override: None,
                    unique_names: true,
                    permanent_cards_only: true,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
