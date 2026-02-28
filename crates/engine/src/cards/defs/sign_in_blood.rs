// 36. Sign in Blood — {BB}, Sorcery; target player draws 2 cards and loses 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sign-in-blood"),
        name: "Sign in Blood".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Target player draws two cards and loses 2 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::DeclaredTarget { index: 0 },
                    count: EffectAmount::Fixed(2),
                },
                Effect::LoseLife {
                    player: PlayerTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                },
            ]),
            targets: vec![TargetRequirement::TargetPlayer],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
