// 36. Night's Whisper — {1B}, Sorcery; you draw 2 cards and lose 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nights-whisper"),
        name: "Night's Whisper".to_string(),
        mana_cost: Some(ManaCost { black: 1, generic: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "You draw two cards and you lose 2 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                Effect::LoseLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
