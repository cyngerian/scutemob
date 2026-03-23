// Keep Watch — {2}{U}, Instant
// Draw a card for each attacking creature.
//
// TODO: "Each attacking creature" count — EffectAmount lacks AttackingCreatureCount.
//   Using Fixed(3) as rough approximation.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("keep-watch"),
        name: "Keep Watch".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw a card for each attacking creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: AttackingCreatureCount not in DSL. Using Fixed(1) placeholder.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
