// Plumb the Forbidden — {1}{B} Instant
// As an additional cost to cast this spell, you may sacrifice one or more creatures.
// When you do, copy this spell for each creature sacrificed this way.
// You draw a card and you lose 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("plumb-the-forbidden"),
        name: "Plumb the Forbidden".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, you may sacrifice one or more creatures. When you do, copy this spell for each creature sacrificed this way.\nYou draw a card and you lose 1 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "Sacrifice creatures" additional cost + "copy for each sacrificed" not in DSL.
            // Implementing the base effect only (1 copy — draw + lose life).
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                Effect::LoseLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
