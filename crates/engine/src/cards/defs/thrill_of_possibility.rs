// Thrill of Possibility — {1}{R}, Instant
// As an additional cost to cast this spell, discard a card.
// Draw two cards.
//
// TODO: Discard additional cost not expressible on CardDefinition. Draw only.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thrill-of-possibility"),
        name: "Thrill of Possibility".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, discard a card.\nDraw two cards.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
