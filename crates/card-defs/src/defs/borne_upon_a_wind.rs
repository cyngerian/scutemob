// Borne Upon a Wind — {1}{U}, Instant
// You may cast spells this turn as though they had flash.
// Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("borne-upon-a-wind"),
        name: "Borne Upon a Wind".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may cast spells this turn as though they had flash.\nDraw a card."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 601.3b: Grant flash for all spells until end of turn, then draw a card.
            effect: Effect::Sequence(vec![
                Effect::GrantFlash {
                    filter: FlashGrantFilter::AllSpells,
                    duration: EffectDuration::UntilEndOfTurn,
                },
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
