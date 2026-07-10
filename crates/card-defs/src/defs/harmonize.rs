// 33. Harmonize — {2GG}, Sorcery; draw 3 cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("harmonize"),
        name: "Harmonize".to_string(),
        mana_cost: Some(ManaCost { green: 2, generic: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw three cards.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(3),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
