// Preordain — {U}, Sorcery
// Scry 2, then draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("preordain"),
        name: "Preordain".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Scry 2, then draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
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
