// Crop Rotation — {G} Instant; as an additional cost, sacrifice a land.
// Search your library for a land card, put it onto the battlefield, then shuffle.
// TODO: "Sacrifice a land" as spell additional cost — not activated ability cost (PB-4)
// Needs required_additional_cost field on CardDef or Spell. SearchLibrary portion works.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crop-rotation"),
        name: "Crop Rotation".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a land.\nSearch your library for a land card, put that card onto the battlefield, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::SearchLibrary {
                player: PlayerTarget::Controller,
                filter: TargetFilter {
                    has_card_type: Some(CardType::Land),
                    ..Default::default()
                },
                reveal: false,
                destination: ZoneTarget::Battlefield { tapped: false },
            },
            // TODO: sacrifice a land (spell additional cost, not PB-4 activated cost)
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
