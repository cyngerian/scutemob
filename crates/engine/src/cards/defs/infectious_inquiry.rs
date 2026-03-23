// Infectious Inquiry — {2}{B}, Sorcery
// You draw two cards and you lose 2 life. Each opponent gets a poison counter.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("infectious-inquiry"),
        name: "Infectious Inquiry".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "You draw two cards and you lose 2 life. Each opponent gets a poison counter.".to_string(),
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
                Effect::AddCounter {
                    target: EffectTarget::EachOpponent,
                    counter: CounterType::Poison,
                    count: 1,
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
