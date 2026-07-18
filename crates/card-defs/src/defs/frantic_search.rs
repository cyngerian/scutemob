// Frantic Search — {2}{U}, Instant
// Draw two cards, then discard two cards. Untap up to three lands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("frantic-search"),
        name: "Frantic Search".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw two cards, then discard two cards. Untap up to three lands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                Effect::DiscardCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                // Only requirement here is UpToN{3}, so declared lands occupy indices
                // 0..3 (card_definition.rs:2799-2822); an undeclared slot no-ops.
                Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 1 },
                },
                Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 2 },
                },
            ]),
            targets: vec![TargetRequirement::UpToN {
                count: 3,
                inner: Box::new(TargetRequirement::TargetLand),
            }],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
