// Burst Lightning {R}
// Instant — Kicker {4}
// Burst Lightning deals 2 damage to any target. If this spell was kicked,
// it deals 4 damage instead.
// CR 702.33a: Kicker [cost] — optional additional cost for enhanced effect.
// CR 702.33d: "kicked" means the player paid the kicker cost at cast time.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("burst-lightning"),
        name: "Burst Lightning".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Kicker {4}\nBurst Lightning deals 2 damage to any target. If this spell was kicked, it deals 4 damage instead.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Kicker),
            AbilityDefinition::Kicker {
                cost: ManaCost { generic: 4, ..Default::default() },
                is_multikicker: false,
            },
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasKicked,
                    if_true: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(4),
                    }),
                    if_false: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    }),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
