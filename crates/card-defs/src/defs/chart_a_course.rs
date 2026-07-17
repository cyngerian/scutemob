// Chart a Course — {1}{U}, Sorcery
// Draw two cards. Then discard a card unless you attacked this turn.
//
// PB-AC6 added Condition::YouAttackedThisTurn (CR 508.1). The discard is mandatory
// unless the raid condition is met, so a false branch discards and a true branch is
// a no-op (Effect::Nothing).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chart-a-course"),
        name: "Chart a Course".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw two cards. Then discard a card unless you attacked this turn."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                Effect::Conditional {
                    condition: Condition::YouAttackedThisTurn,
                    if_true: Box::new(Effect::Nothing),
                    if_false: Box::new(Effect::DiscardCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    }),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
