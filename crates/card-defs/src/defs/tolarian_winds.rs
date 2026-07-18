// Tolarian Winds — {1}{U}, Instant
// Discard all the cards in your hand, then draw that many cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tolarian-winds"),
        name: "Tolarian Winds".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Discard all the cards in your hand, then draw that many cards.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.9 / 121.1: discard the whole hand, then draw that many cards.
            effect: Effect::WheelHand {
                player: PlayerTarget::Controller,
                disposal: WheelDisposal::Discard,
                draw: WheelDraw::ThatMany,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
