// Borne Upon a Wind — {1}{U}, Instant
// You may cast spells this turn as though they had flash.
// Draw a card.
//
// TODO: "Cast spells as though they had flash" — continuous effect on casting not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("borne-upon-a-wind"),
        name: "Borne Upon a Wind".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may cast spells this turn as though they had flash.\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "Cast as though flash" not expressible. Draw only.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
