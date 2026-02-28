// 37. Read the Bones — {2B}, Sorcery; scry 2, draw 2 cards, lose 2 life.
// CR 701.18: Scry 2 implemented via Effect::Scry { count: Fixed(2) }.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("read-the-bones"),
        name: "Read the Bones".to_string(),
        mana_cost: Some(ManaCost { black: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Scry 2, then draw two cards. You lose 2 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 701.18: Scry 2 before drawing.
                Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
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
