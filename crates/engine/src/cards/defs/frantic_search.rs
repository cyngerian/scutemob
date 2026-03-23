// Frantic Search — {2}{U}, Instant
// Draw two cards, then discard two cards. Untap up to three lands.
//
// TODO: "Then discard two cards" + "untap up to three lands" not expressible.
//   Implementing draw only.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("frantic-search"),
        name: "Frantic Search".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw two cards, then discard two cards. Untap up to three lands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            // TODO: Discard + untap lands not expressible.
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
