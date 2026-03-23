// Urban Evolution — {3}{G}{U}, Sorcery
// Draw three cards. You may play an additional land this turn.
//
// TODO: "You may play an additional land this turn" — one-shot additional land play
//   not in DSL (only permanent static exists).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("urban-evolution"),
        name: "Urban Evolution".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw three cards. You may play an additional land this turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(3),
            },
            // TODO: Additional land play effect.
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
