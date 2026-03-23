// Chart a Course — {1}{U}, Sorcery
// Draw two cards. Then discard a card unless you attacked this turn.
//
// TODO: "Unless you attacked this turn" conditional discard — needs attack-tracking
//   condition. Implementing draw only (no discard).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chart-a-course"),
        name: "Chart a Course".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw two cards. Then discard a card unless you attacked this turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "Then discard unless you attacked" not expressible.
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
