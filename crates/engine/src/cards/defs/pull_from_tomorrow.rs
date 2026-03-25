// 38. Pull from Tomorrow — {X}{U}{U}, Instant; draw X cards, discard a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pull-from-tomorrow"),
        name: "Pull from Tomorrow".to_string(),
        mana_cost: Some(ManaCost { blue: 2, x_count: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw X cards, then discard a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::XValue,
                },
                Effect::DiscardCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
