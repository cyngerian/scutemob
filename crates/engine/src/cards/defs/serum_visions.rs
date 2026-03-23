// Serum Visions — {U}, Sorcery
// Draw a card. Scry 2.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("serum-visions"),
        name: "Serum Visions".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw a card. Scry 2.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
