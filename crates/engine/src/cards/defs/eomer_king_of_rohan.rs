// Éomer, King of Rohan — {3}{R}{W}, Legendary Creature — Human Noble 2/2
// Double strike; ETB: +1/+1 counter per other Human you control; ETB: monarch + deal damage equal to power
//
// ETB trigger (BecomeMonarch + DealDamage) is implemented.
// TODO: ETB counter placement (X +1/+1 counters where X = other Humans you control) requires
// PermanentCount with subtype filter as the counter amount — DSL gap (EffectAmount::PermanentCount
// exists but placing that many counters requires AddCounters with a PermanentCount amount variant).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eomer-king-of-rohan"),
        name: "Éomer, King of Rohan".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            white: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Noble"],
        ),
        oracle_text: "Double strike\nÉomer enters with a +1/+1 counter on it for each other Human you control.\nWhen Éomer enters, target player becomes the monarch. Éomer deals damage equal to its power to any target.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::DoubleStrike),
            // TODO: ETB with X +1/+1 counters where X = number of other Humans you control.
            // Requires AddCounters with EffectAmount::PermanentCount (subtype Human filter,
            // exclude_self). DSL gap — EffectAmount::PermanentCount works for damage/drain/draw
            // but AddCounters does not yet support dynamic EffectAmount (only Fixed).
            // CR 701.6a: Counter placement at ETB time.
            AbilityDefinition::Triggered {
                // CR 603.5: "When ~ enters" — ETB trigger.
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    // "target player becomes the monarch" — CR 724.1.
                    Effect::BecomeMonarch {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                    },
                    // "Éomer deals damage equal to its power to any target" — CR 120.1.
                    Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        amount: EffectAmount::PowerOf(EffectTarget::Source),
                    },
                ]),
                intervening_if: None,
                targets: vec![
                    // target 0: a player that becomes the monarch
                    TargetRequirement::TargetPlayer,
                    // target 1: any target (creature, planeswalker, or player) for damage
                    TargetRequirement::TargetAny,
                ],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
