// 32. Arcane Denial — {1U}, Instant; counter target spell. Its controller may
// draw up to two cards. You draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arcane-denial"),
        name: "Arcane Denial".to_string(),
        mana_cost: Some(ManaCost { blue: 1, generic: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. Its controller may draw up to two cards at the beginning of the next turn's upkeep.\nYou draw a card at the beginning of the next turn's upkeep.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // Simplified: counter the spell and draw a card immediately.
            effect: Effect::Sequence(vec![
                Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
            ]),
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
