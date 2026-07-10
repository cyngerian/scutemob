// Crop Rotation — {G} Instant
// As an additional cost to cast this spell, sacrifice a land.
// Search your library for a land card, put it onto the battlefield, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crop-rotation"),
        name: "Crop Rotation".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a land.\nSearch your library for a land card, put that card onto the battlefield, then shuffle.".to_string(),
        // CR 118.8: Mandatory sacrifice of a land as additional cost.
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeLand],
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.23: Oracle says "put that card onto the battlefield, then shuffle."
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle { player: PlayerTarget::Controller },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
