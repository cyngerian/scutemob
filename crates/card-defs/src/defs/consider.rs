// Consider — {U}, Instant; surveil 1, then draw a card.
// CR 701.25: Surveil 1 before drawing.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("consider"),
        name: "Consider".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Surveil 1. (Look at the top card of your library. You may put it into your graveyard.)\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::Surveil {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
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
