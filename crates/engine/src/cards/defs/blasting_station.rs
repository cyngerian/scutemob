// Blasting Station — {3}, Artifact
// "{T}, Sacrifice a creature: This artifact deals 1 damage to any target."
// "Whenever a creature enters, you may untap this artifact."
// The first ability is fully expressible. The second (auto-untap on creature ETB) is a
// TODO: DSL gap — there is no effect variant for untapping a specific permanent (self), and
// WheneverCreatureEntersBattlefield trigger with Effect::Untap { target: Source } is not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blasting-station"),
        name: "Blasting Station".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}, Sacrifice a creature: This artifact deals 1 damage to any target.\nWhenever a creature enters, you may untap this artifact.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![TargetRequirement::TargetAny],
                timing_restriction: None,
                activation_condition: None,
            },
            // TODO: DSL gap — "Whenever a creature enters, you may untap this artifact."
            // Requires Effect::Untap { target: EffectTarget::Source } which does not exist.
            // The untap-on-creature-ETB loop is a key mechanic of this card.
        ],
        ..Default::default()
    }
}
