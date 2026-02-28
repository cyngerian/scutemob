// 34. Divination — {2U}, Sorcery; draw 2 cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("divination"),
        name: "Divination".to_string(),
        mana_cost: Some(ManaCost { blue: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw two cards.".to_string(),
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
