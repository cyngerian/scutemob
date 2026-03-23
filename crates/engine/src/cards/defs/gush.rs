// Gush — {4}{U}, Instant
// You may return two Islands you control to their owner's hand rather than pay
// this spell's mana cost.
// Draw two cards.
//
// TODO: Alt cost "return two Islands" — AltCostKind lacks bounce-lands variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gush"),
        name: "Gush".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may return two Islands you control to their owner's hand rather than pay this spell's mana cost.\nDraw two cards.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Alt cost (bounce 2 Islands) not expressible.
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
